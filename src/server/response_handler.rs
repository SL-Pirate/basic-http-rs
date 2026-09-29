use crate::compression::Compression;
use crate::http_parser::{
    CONNECTION_HEADER, CONTENT_ENCODING, CONTENT_LENGTH_HEADER, HttpResponse,
};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[async_trait::async_trait]
pub trait GenericHttpResponse<T: Clone>: Send {
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
            let (algo_opt, compressed) = compressor.compress(b.clone(), acceptable_encodings);
            if let Some(algo) = algo_opt {
                let mut new_headers = self.headers().clone();
                new_headers.insert(String::from(CONTENT_ENCODING), algo);
                let new_res = HttpResponse::builder()
                    .headers(new_headers)
                    .body(compressed.clone())
                    .status(self.status())
                    .status_code(self.status_code())
                    .build();

                if let Err(e) = stream.write_all(&new_res.to_bytes()).await {
                    eprintln!("{e}")
                }
                return;
            }
        }

        if let Err(e) = stream.write_all(&self.to_bytes()).await {
            eprintln!("{e}")
        }
    }

    fn get_content_length(&self) -> usize {
        self.body().clone().map_or(0, |b| b.len())
    }

    fn this(&self) -> &HttpResponse<Vec<u8>> {
        self
    }
}

impl HttpResponse<Vec<u8>> {
    fn to_bytes(&self) -> Vec<u8> {
        let res = self.encode_metadata();
        let mut res_raw = res.into_bytes();
        if let Some(body) = self.body().clone().as_mut() {
            res_raw.append(body);
        }

        res_raw
    }
}
