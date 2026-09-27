use crate::cli::CliArgs;
use crate::server::{HttpServer, ServerHandler};
use std::io;
use std::process::exit;
use std::sync::Arc;
use tokio::net::TcpListener;

impl HttpServer {
    pub async fn new() -> HttpServer {
        let args: CliArgs = argh::from_env();
        let addr = args.address.clone();
        let port = args.port.clone();
        HttpServer {
            args,
            listener: Self::create_listener(addr, port).await,
        }
    }

    pub async fn serve(self) -> io::Result<()> {
        let arc = Arc::new(self);

        loop {
            let server_arc = Arc::clone(&arc);
            let (stream, _) = server_arc.listener.accept().await?;
            tokio::spawn(async move {
                let server = server_arc.as_ref();
                let mut handler = ServerHandler::new(stream, server.args.get_path());
                let path = handler.get_path_from_req().await;
                handler.handle_response(path).await
            });
        }
    }

    async fn create_listener(addr: String, port: u32) -> TcpListener {
        match TcpListener::bind(format!("{addr}:{port}")).await {
            Ok(listener) => {
                println!("Server listening on {addr}:{port}");
                listener
            }
            Err(e) => {
                eprintln!("{e}");
                exit(-1)
            }
        }
    }
}
