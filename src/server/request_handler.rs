use crate::compression::Compression;
use crate::http_parser::{CONTENT_TYPE, HttpRequest, HttpResponse};
use crate::server::RequestHandler;
use crate::server::response_handler::GenericHttpResponse;
use std::sync::Arc;
use std::{fs, io};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

const DIRECTORY_HTML: &str = include_str!("../../directory.html");
const BUFFER_SIZE: usize = 128;

impl RequestHandler {
    pub fn new(
        stream: TcpStream,
        base_path: String,
        enable_compression: bool,
        compressor: Arc<Compression>,
    ) -> RequestHandler {
        RequestHandler {
            stream,
            base_path,
            enable_compression,
            compressor,
        }
    }

    pub(crate) async fn get_request(&mut self) -> Option<HttpRequest> {
        let mut req_raw: Vec<u8> = Vec::new();

        loop {
            let mut buffer = [0u8; BUFFER_SIZE];
            match self.stream.read(&mut buffer).await {
                Ok(read) => {
                    if read <= 0 {
                        break;
                    } else if read < BUFFER_SIZE {
                        req_raw.append(&mut buffer[..read].to_vec());
                        break;
                    } else {
                        req_raw.append(&mut buffer.to_vec());
                        buffer.fill(0);
                    }
                }
                Err(e) => {
                    eprintln!("error while reading the request: {e}");
                    break;
                }
            }
        }

        HttpRequest::from_vec(req_raw).ok()
    }

    pub(crate) async fn handle_response(&mut self, req_opt: Option<HttpRequest>) {
        if let Some(req) = req_opt {
            let path = req.path();
            if path.contains("..") {
                self.send(&req, &mut HttpResponse::bad_request(None)).await;
                return;
            };

            let file_path = match path.as_str() {
                "/" => format!("{}/index.html", self.base_path),
                _ => format!("{}{path}", self.base_path),
            };
            if let Ok(file) = fs::read(&file_path) {
                self.serve_file(&req, file_path, file).await;
            } else if let Ok(file) = fs::read(format!("{}/index.html", self.base_path)) {
                self.serve_file(&req, format!("{}/index.html", self.base_path), file)
                    .await;
            } else {
                let template_res = self.render_directory_template(file_path.as_str()).await;

                match template_res {
                    Ok(template) => {
                        // if path doesn't end in a /, we redirect
                        if !path.ends_with('/') {
                            self.send(
                                &req,
                                &mut HttpResponse::redirect(format!("{path}/").as_str()),
                            )
                            .await;
                            return;
                        }

                        self.send(
                            &req,
                            &mut HttpResponse::builder()
                                .body(template.into_bytes())
                                .add_header(CONTENT_TYPE, "text/html; charset=utf-8")
                                .build(),
                        )
                        .await;
                        return;
                    }
                    Err(e) => {
                        eprintln!("File not found!: {e}");
                        self.send(&req, &mut HttpResponse::not_found(Some(format!("{e}"))))
                            .await;
                    }
                }
            }
        };
    }

    async fn serve_file(&mut self, req: &HttpRequest, file_path: String, file: Vec<u8>) {
        let mime_guess = mime_guess::from_path(file_path);
        self.send(
            &req,
            &mut HttpResponse::builder()
                .body(file)
                .add_header(
                    "Content-Type",
                    format!(
                        "{}; charset=utf-8",
                        match mime_guess.first() {
                            None => {
                                "text/html".to_string()
                            }
                            Some(mime) => {
                                mime.to_string()
                            }
                        }
                    )
                    .as_str(),
                )
                .build(),
        )
        .await;
    }

    async fn render_directory_template(&self, pre_processed_dir: &str) -> io::Result<String> {
        let mut template = DIRECTORY_HTML.to_string();
        let mut dir = pre_processed_dir.to_string();

        // handling back button
        if pre_processed_dir.contains("index.html") {
            template = template.replace("{{back}}", "");
            dir = pre_processed_dir.replace("index.html", "");
        } else {
            template = template.replace("{{back}}", "<a class='back' href='..'>&larr; Back</a>");
        };

        // replacing title
        template = template.replace(
            "{{title}}",
            dir.replace(self.base_path.as_str(), "").as_str(),
        );

        // handling listing
        let mut items = tokio::fs::read_dir(&dir).await?;
        let mut dirs: Vec<String> = Vec::new();
        let mut files: Vec<String> = Vec::new();

        loop {
            if let Some(item) = items.next_entry().await? {
                let file_type = item.file_type().await?;
                if file_type.is_dir() {
                    if let Ok(path) = item.path().into_string() {
                        dirs.push(path);
                    }
                } else if file_type.is_file() {
                    if let Ok(path) = item.path().into_string() {
                        files.push(path);
                    }
                }
            } else {
                break;
            }
        }

        let mut item_list = String::new();
        for mut item in dirs {
            item = item.replace(&dir, "");
            if item.starts_with("/") {
                item.remove(0);
            }
            item_list.push_str(&format!(
                "<li class='dir'><a href='{item}/'>{item}</a></li>\n",
            ));
        }
        for mut item in files {
            item = item.replace(&dir, "");
            if item.starts_with("/") {
                item.remove(0);
            }
            item_list.push_str(&format!(
                "<li class='file'><a href='{item}'>{item}</a></li>\n"
            ));
        }

        template = template.replace("{{dirs}}", item_list.as_str());

        Ok(template)
    }

    async fn send<T: Clone>(&mut self, req: &HttpRequest, res: &mut dyn GenericHttpResponse<T>) {
        res.send(
            self.should_compress(req, res),
            req.get_acceptable_encodings(),
            self.compressor.clone(),
            &mut self.stream,
        )
        .await;
    }

    fn should_compress<T: Clone>(
        &self,
        req: &HttpRequest,
        res: &dyn GenericHttpResponse<T>,
    ) -> bool {
        if !self.enable_compression {
            return false;
        }
        if res.get_content_length() <= 1024 {
            return false;
        };
        if !Compression::can_encode(req.get_acceptable_encodings()) {
            return false;
        }
        match res.this().headers().get(CONTENT_TYPE) {
            None => false,
            Some(content_type) => {
                let mime = content_type
                    .split(';')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_ascii_lowercase();

                let can_encode = mime.starts_with("text/")
                    || mime.ends_with("+json")   // application/ld+json, application/problem+json
                    || mime.ends_with("+xml")    // image/svg+xml, application/atom+xml
                    || matches!(
                mime.as_str(),
                "application/json"
                    | "application/javascript"
                    | "application/xml"
                    | "application/wasm"
                    | "font/ttf"
                    | "font/otf"
                );
                can_encode
            }
        }
    }
}
