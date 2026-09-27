pub mod cli;
pub mod directory;
pub mod parse_http;

use crate::cli::CliArgs;
use crate::directory::render_directory_template;
use crate::parse_http::{HttpParser, HttpResponse};
use std::collections::HashMap;
use std::sync::Arc;
use std::{fs, io};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    let argh: CliArgs = argh::from_env();
    let base_path = Arc::new(argh.get_path());
    let addr = argh.address;
    let port = argh.port;

    let listener = TcpListener::bind(format!("{addr}:{port}")).await?;

    println!("Server listening on {addr}:{port}");

    loop {
        let (mut socket, _) = listener.accept().await?;
        let base_path = Arc::clone(&base_path);
        tokio::spawn(async move {
            let path = get_path_from_req(&mut socket).await;
            handle_response(path, socket, base_path.as_ref()).await
        });
    }
}

async fn get_path_from_req(stream: &mut TcpStream) -> Option<String> {
    let mut req_raw: Vec<u8> = Vec::new();

    loop {
        const BUFFER_SIZE: usize = 128;
        let mut buffer = [0u8; BUFFER_SIZE];
        match stream.read(&mut buffer).await {
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

async fn handle_response(path_opt: Option<String>, mut socket: TcpStream, base_path: &String) {
    let bad_request_response = HttpParser::new().parse_response(HttpResponse::bad_request(None));

    if let Some(path) = path_opt {
        if path.contains("..") {
            if let Err(e) = socket.write_all(&*bad_request_response).await {
                eprintln!("{e}")
            }
            return;
        };

        let file_path = match path.as_str() {
            "/" => format!("{base_path}/index.html"),
            _ => format!("{base_path}{path}"),
        };
        if let Ok(file) = fs::read(&file_path) {
            serve_file(&mut socket, file_path, file).await;
        } else if let Ok(file) = fs::read(format!("{base_path}/index.html")) {
            serve_file(&mut socket, format!("{base_path}/index.html"), file).await;
        } else {
            let template_res = render_directory_template(file_path.as_str()).await;
            let mut headers: HashMap<String, String> = HashMap::new();
            headers.insert(
                "Content-Type".to_string(),
                "text/html; charset=utf-8".to_string(),
            );

            match template_res {
                Ok(template) => {
                    let res = HttpResponse::ok(headers, Some(template.into_bytes()));
                    if let Err(e) = socket
                        .write_all(&*HttpParser::new().parse_response(res))
                        .await
                    {
                        eprintln!("{e}")
                    }
                    return;
                }
                Err(e) => {
                    eprintln!("{e}");
                    let res = HttpResponse::not_found(Some(format!("{e}")));
                    if let Err(e) = socket
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

async fn serve_file(socket: &mut TcpStream, file_path: String, file: Vec<u8>) {
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
    if let Err(e) = socket
        .write_all(&*HttpParser::new().parse_response(res))
        .await
    {
        eprintln!("{e}")
    }
}
