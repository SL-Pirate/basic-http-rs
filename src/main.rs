pub mod cli;
pub mod parse_http;
pub mod server;

use crate::server::HttpServer;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let server = HttpServer::new().await;
    server.serve().await
}
