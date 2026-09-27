use crate::parse_http::{HttpParser, HttpResponse};
use crate::server::ServerHandler;
use std::collections::HashMap;
use std::{fs, io};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const DIRECTORY_HTML: &str = include_str!("../../directory.html");

impl ServerHandler {
    pub fn new(stream: TcpStream, base_path: String) -> ServerHandler {
        ServerHandler { stream, base_path }
    }

    pub(crate) async fn get_path_from_req(&mut self) -> Option<String> {
        let mut req_raw: Vec<u8> = Vec::new();

        loop {
            const BUFFER_SIZE: usize = 128;
            let mut buffer = [0u8; BUFFER_SIZE];
            match self.stream.read(&mut buffer).await {
                Ok(read) => {
                    if read == 0 {
                        break;
                    } else if read < 128 {
                        req_raw.append(&mut buffer[..read].to_vec());
                        buffer.fill(0);
                        break;
                    } else {
                        req_raw.append(&mut buffer.to_vec());
                        buffer.fill(0);
                    }
                }
                Err(e) => {
                    println!("error while reading the request: {e}");
                }
            }
        }

        HttpParser::new().get_path(req_raw)
    }

    pub(crate) async fn handle_response(&mut self, path_opt: Option<String>) {
        let bad_request_response =
            HttpParser::new().parse_response(HttpResponse::bad_request(None));

        if let Some(path) = path_opt {
            if path.contains("..") {
                if let Err(e) = self.stream.write_all(&*bad_request_response).await {
                    eprintln!("{e}")
                }
                return;
            };

            let file_path = match path.as_str() {
                "/" => format!("{}/index.html", self.base_path),
                _ => format!("{}{path}", self.base_path),
            };
            if let Ok(file) = fs::read(&file_path) {
                self.serve_file(file_path, file).await;
            } else if let Ok(file) = fs::read(format!("{}/index.html", self.base_path)) {
                self.serve_file(format!("{}/index.html", self.base_path), file)
                    .await;
            } else {
                let template_res = self.render_directory_template(file_path.as_str()).await;
                let mut headers: HashMap<String, String> = HashMap::new();
                headers.insert(
                    "Content-Type".to_string(),
                    "text/html; charset=utf-8".to_string(),
                );

                match template_res {
                    Ok(template) => {
                        // if path doesn't end in a /, we redirect
                        if !path.ends_with('/') {
                            let mut headers = HashMap::new();
                            headers.insert("Location".to_string(), format!("{path}/"));
                            let res = HttpResponse::new(302, "Found".to_string(), headers, None);
                            if let Err(e) = self
                                .stream
                                .write_all(&*HttpParser::new().parse_response(res))
                                .await
                            {
                                eprintln!("{e}")
                            }
                            return;
                        }

                        let res = HttpResponse::ok(headers, Some(template.into_bytes()));
                        if let Err(e) = self
                            .stream
                            .write_all(&*HttpParser::new().parse_response(res))
                            .await
                        {
                            eprintln!("{e}")
                        }
                        return;
                    }
                    Err(e) => {
                        eprintln!("File not found!: {e}");
                        let res = HttpResponse::not_found(Some(format!("{e}")));
                        if let Err(e) = self
                            .stream
                            .write_all(&*HttpParser::new().parse_response(res))
                            .await
                        {
                            eprintln!("{e}")
                        }
                    }
                }
            }
        };
    }

    async fn serve_file(&mut self, file_path: String, file: Vec<u8>) {
        let mut headers: HashMap<String, String> = HashMap::new();
        let mime_guess = mime_guess::from_path(file_path);
        headers.insert(
            "Content-Type".to_string(),
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
            ),
        );
        let res = HttpResponse::ok(headers, Some(file));
        if let Err(e) = self
            .stream
            .write_all(&*HttpParser::new().parse_response(res))
            .await
        {
            eprintln!("{e}")
        }
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
        template = template.replace("{{title}}", dir.as_str());

        // handling listing
        let mut directories = tokio::fs::read_dir(&dir).await?;
        let mut dirs: Vec<String> = Vec::new();
        let mut files: Vec<String> = Vec::new();
        loop {
            if let Some(item) = directories.next_entry().await? {
                let file_type = item.file_type().await?;
                if file_type.is_dir() {
                    if let Ok(mut path) = item.path().into_string() {
                        path.remove(0);
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
            let mut item_name = item.clone();
            item_name.insert(0, '.');
            item_name = item_name.replace(&dir, "");
            if item_name.starts_with("/") {
                item_name.remove(0);
            }
            item_list.push_str(&format!(
                "<li class='dir'><a href='{item}/'>{item_name}</a></li>\n",
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
}
