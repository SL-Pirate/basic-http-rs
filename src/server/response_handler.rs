use crate::compression::Compression;
use crate::http_parser::{
    CONNECTION_HEADER, CONTENT_ENCODING, CONTENT_LENGTH_HEADER, HttpResponse,
};
use std::io;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
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

    fn get_content_length(&self) -> usize;

    fn this(&self) -> &HttpResponse<T>;

    fn encode_metadata(&self) -> String {
        let status_code = self.this().status_code();
        let content_len = self.get_content_length();
        let status = self.this().status();
        let mut res = format!("HTTP/1.1 {status_code} {status}\r\n");
        for (header, value) in self.this().headers().clone() {
            res.push_str(format!("{header}: {value}\r\n").as_str())
        }
        if !self.this().headers().contains_key(CONTENT_LENGTH_HEADER) {
            res.push_str(format!("{CONTENT_LENGTH_HEADER}: {content_len}\r\n").as_str())
        }
        if !self.this().headers().contains_key(CONNECTION_HEADER) {
            res.push_str(format!("{CONNECTION_HEADER}: close\r\n").as_str())
        }
        res.push_str("\r\n");
        res
    }
}

#[async_trait::async_trait]
impl GenericHttpResponse<Vec<u8>> for HttpResponse<Vec<u8>> {
    async fn send(
        &mut self,
        compress: bool,
        acceptable_encodings: Vec<&str>,
        compressor: Arc<Compression>,
        stream: &mut TcpStream,
    ) {
        if let Some(b) = self.body()
            && compress
        {
            if let Some(result) = compressor.compress(b, acceptable_encodings) {
                let mut new_headers = self.headers().clone();
                new_headers.insert(String::from(CONTENT_ENCODING), result.0);
                let mut new_res = HttpResponse::builder()
                    .headers(new_headers)
                    .body(result.1)
                    .status(self.status())
                    .status_code(self.status_code())
                    .build();

                if let Err(e) = new_res.write_res(stream).await {
                    eprintln!("{e}")
                }
                return;
            }
        }

        if let Err(e) = self.write_res(stream).await {
            eprintln!("{e}")
        }
    }

    fn get_content_length(&self) -> usize {
        self.body().as_ref().map_or(0, |b| b.len())
    }

    fn this(&self) -> &HttpResponse<Vec<u8>> {
        self
    }
}

impl HttpResponse<Vec<u8>> {
    async fn write_res(&mut self, stream: &mut TcpStream) -> io::Result<()> {
        stream
            .write_all(self.encode_metadata().into_bytes().as_slice())
            .await?;
        if let Some(body) = self.body_mut() {
            stream.write_all(body).await?;
        }

        Ok(())
    }
}
