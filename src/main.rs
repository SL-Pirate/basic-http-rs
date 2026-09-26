pub mod cli;
pub mod parse_http;

use crate::cli::CliArgs;
use crate::parse_http::{HttpParser, HttpResponse};
use std::collections::HashMap;
use std::{fs, io};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    let argh: CliArgs = argh::from_env();
    let base_path: String = argh.get_path();
    let addr = argh.address;
    let port = argh.port;

    let listener = TcpListener::bind(format!("{addr}:{port}")).await?;

    println!("Server listening on {addr}:{port}");

    loop {
        let (socket, _) = listener.accept().await?;
        let (path, socket) = get_path_from_req(socket).await;
        handle_response(path, socket, &base_path).await
    }
}

async fn get_path_from_req(mut stream: TcpStream) -> (Option<String>, TcpStream) {
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

    (HttpParser::new().get_path(req_raw), stream)
}

async fn handle_response(path_opt: Option<String>, mut socket: TcpStream, base_path: &String) {
    if let Some(path) = path_opt {
        if path.contains("..") {
            socket
                .write("Invalid Request".as_bytes())
                .await
                .expect("Stream closed");
        };

        let file_path = match path.as_str() {
            "/" => format!("{base_path}/index.html"),
            _ => format!("{base_path}{path}"),
        };
        println!("looking for file: {file_path}");
        if let Ok(file) = fs::read_to_string(&file_path) {
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
            socket
                .write(&*HttpParser::new().parse_response(res))
                .await
                .expect("Stream closed");
        } else {
            let res = HttpResponse::new(404, "Not Found".to_string(), HashMap::new(), None);
            socket
                .write(&*HttpParser::new().parse_response(res))
                .await
                .expect("Stream Closed");
        }
    } else {
        let res = HttpResponse::new(401, "Invalid Request".to_string(), HashMap::new(), None);
        socket
            .write(&*HttpParser::new().parse_response(res))
            .await
            .expect("Stream Closed");
    }
}
