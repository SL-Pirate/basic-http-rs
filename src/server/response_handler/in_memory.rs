use crate::compression::Compression;
use crate::http_parser::{CONTENT_ENCODING, HttpResponse};
use crate::server::response_handler::GenericHttpResponse;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

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
                let mut new_res = HttpResponse::builder()
                    .headers(new_headers)
                    .body(compressed.clone())
                    .status(self.status())
                    .status_code(self.status_code())
                    .build();

                if let Err(e) = stream.write_all(&new_res.as_bytes()).await {
                    eprintln!("{e}")
                }
                return;
            }
        }

        if let Err(e) = stream.write_all(&self.as_bytes()).await {
            eprintln!("{e}")
        }
    }

    fn get_content_length(&mut self) -> Option<usize> {
        Some(self.body().clone().map_or(0, |b| b.len()))
    }

    fn this(&self) -> &HttpResponse<Vec<u8>> {
        self
    }
}

impl HttpResponse<Vec<u8>> {
    fn as_bytes(&mut self) -> Vec<u8> {
        let res = self.encode_metadata();
        let mut res_raw = res.into_bytes();
        if let Some(body) = self.body().clone().as_mut() {
            res_raw.append(body);
        }

        res_raw
    }
}
