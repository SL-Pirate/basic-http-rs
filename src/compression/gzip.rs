use crate::compression::Compress;
use async_compression::Level::Precise;
use async_compression::tokio::write::GzipEncoder;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io;
use std::io::Write;
use std::sync::Arc;
use tokio::io::DuplexStream;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct GzipCompressor;

#[async_trait::async_trait]
impl Compress for GzipCompressor {
    fn compress(&self, content: &Vec<u8>) -> io::Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&content)?;
        encoder.finish()
    }

    async fn compress_stream(
        &self,
        input: Arc<&mut DuplexStream>,
        out: Arc<Mutex<DuplexStream>>,
    ) -> io::Result<()> {
        let mut enc = GzipEncoder::with_quality(input, Precise(5));
        tokio::io::copy(&mut enc, out).await.map(|_| ())
    }
}
