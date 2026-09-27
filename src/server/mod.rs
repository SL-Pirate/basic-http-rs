mod server_handler_impl;
pub mod http_server_impl;

use crate::cli::CliArgs;
use tokio::net::{TcpListener, TcpStream};

pub struct HttpServer {
    args: CliArgs,
    listener: TcpListener
}

pub struct ServerHandler {
    stream: TcpStream,
    base_path: String,
}
