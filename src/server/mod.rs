pub mod http_server;
mod request_handler;
pub mod response_handler;

use std::sync::Arc;
use crate::cli::CliArgs;
use crate::compression::Compression;
use tokio::net::{TcpListener, TcpStream};

pub struct HttpServer {
    args: CliArgs,
    listener: TcpListener,
}

#[derive(Debug)]
pub struct RequestHandler {
    stream: TcpStream,
    base_path: String,
    enable_compression: bool,
    compressor: Arc<Compression>,
}
