use crate::cli::CliArgs;
use crate::compression::Compression;
use crate::server::{HttpServer, RequestHandler};
use std::io;
use std::process::exit;
use std::sync::Arc;
use tokio::net::TcpListener;

impl HttpServer {
    pub async fn new() -> HttpServer {
        let args: CliArgs = argh::from_env();
        let addr = args.address().clone();
        let port = args.port().clone();
        HttpServer {
            args,
            listener: Self::create_listener(addr, port).await,
        }
    }

    pub async fn serve(self) -> io::Result<()> {
        let arc = Arc::new(self);
        let compressor_acr = Arc::new(Compression::new());

        loop {
            let server_arc = arc.clone();
            let compressor_arc = compressor_acr.clone();
            let (stream, _) = server_arc.listener.accept().await?;
            tokio::spawn(async move {
                let server = server_arc.as_ref();

                let mut handler = RequestHandler::new(
                    stream,
                    server.args.get_path(),
                    server.args.is_compression_enabled(),
                    compressor_arc,
                );
                let req = handler.get_request().await;
                handler.handle_response(req).await
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
