extern crate core;

pub mod cli;
pub mod http_parser;
pub mod server;
pub mod compression;

use crate::server::HttpServer;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let server = HttpServer::new().await;
    server.serve().await
}
