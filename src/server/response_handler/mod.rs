mod in_memory;
pub mod streaming;

use crate::compression::Compression;
use crate::http_parser::{CONNECTION_HEADER, CONTENT_LENGTH_HEADER, HttpResponse};
use std::sync::Arc;
use tokio::net::TcpStream;

#[async_trait::async_trait]
pub trait GenericHttpResponse<T>: Send {
    async fn send(
        &mut self,
        compress: bool,
        acceptable_encodings: Vec<&str>,
        compressor: Arc<Compression>,
        stream: &mut TcpStream,
    );

    fn get_content_length(&mut self) -> Option<usize>;

    fn this(&self) -> &HttpResponse<T>;

    fn encode_metadata(&mut self) -> String {
        let status_code = self.this().status_code();
        let content_len_opt = self.get_content_length();
        let status = self.this().status();
        let mut res = format!("HTTP/1.1 {status_code} {status}\r\n");
        for (header, value) in self.this().headers().clone() {
            res.push_str(format!("{header}: {value}\r\n").as_str())
        }
        if let Some(content_len) = content_len_opt {
            res.push_str(format!("{CONTENT_LENGTH_HEADER}: {content_len}\r\n").as_str())
        }
        if !self.this().headers().contains_key(CONNECTION_HEADER) {
            res.push_str(format!("{CONNECTION_HEADER}: close\r\n").as_str())
        }
        res.push_str("\r\n");
        res
    }
}

pub const STREAM_BODY_BUFFER_SIZE: usize = 1024;
