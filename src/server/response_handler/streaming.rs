use crate::compression::{Compression, AVAILABLE_ALGORITHMS};
use crate::http_parser::{CONNECTION_HEADER, CONTENT_ENCODING, HttpResponse, TRANSFER_ENCODING};
use crate::server::response_handler::{GenericHttpResponse, STREAM_BODY_BUFFER_SIZE};
use std::sync::Arc;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::io::{DuplexStream, duplex};
use tokio::net::TcpStream;

#[async_trait::async_trait]
impl GenericHttpResponse<DuplexStream> for HttpResponse<DuplexStream> {
    async fn send(&mut self, compress: bool, acceptable_encodings: Vec<&str>, compressor: Arc<Compression>, stream: &mut TcpStream) {
        let encoding = if compress { choose_encoding(acceptable_encodings) } else { None };
        let Some(encoding) = encoding else {
            return self.write_to_stream(stream).await;
        };
        let Some(body) = self.body().take() else {
            return self.write_to_stream(stream).await;
        };

        let headers = self.headers();

        headers.insert(CONTENT_ENCODING.into(), encoding.into());
        headers.insert(TRANSFER_ENCODING.into(), "chunked".into());
        headers.insert("vary".into(), "accept-encoding".into());

        let mut new_res = HttpResponse::builder()
            .headers(new_headers)
            .body(rx)
            .status(self.status())
            .status_code(self.status_code())
            .build();

        self.write_head(stream).await?;                   // status line + headers only
        write_chunked(encoder(body, encoding), stream).await // frames chunks, ends with 0\r\n\r\n, flushes
    }
    // async fn send(
    //     &mut self,
    //     compress: bool,
    //     accepted: &[&str],
    //     stream: &mut TcpStream,
    // ) -> io::Result<()> {
    //     // Decide first, so the body is only taken when we know we'll compress it
    // }

    fn get_content_length(&mut self) -> Option<usize> {
        None
    }

    fn this(&self) -> &HttpResponse<DuplexStream> {
        self
    }
}

fn choose_encoding(accepted: Vec<&str>) -> Option<&str> {
    for supported in AVAILABLE_ALGORITHMS {
        if accepted.contains(&supported) {
            return Some(supported);
        }
    }

    None
}

impl HttpResponse<DuplexStream> {
    async fn write_to_stream(&mut self, stream: &mut TcpStream) {
        let meta = self.encode_metadata();
        if let Err(e) = stream.write_all(meta.as_bytes()).await {
            eprintln!("error when writing to meta to stream: {}", e);
        };
        if let Err(e) = tokio::io::copy(self.body().as_mut().unwrap(), stream).await {
            eprintln!("error when writing to body to stream: {}", e);
        };
    }
}
