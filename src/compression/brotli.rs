use crate::compression::Compress;
use async_compression::Level::Precise;
use async_compression::tokio::write::BrotliEncoder;
use brotli::CompressorWriter;
use brotli::enc::BrotliEncoderParams;
use std::io::Write;
use std::sync::Arc;
use tokio::io;
use tokio::io::DuplexStream;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct BrotliCompressor;

#[async_trait::async_trait]
impl Compress for BrotliCompressor {
    fn compress(&self, content: &Vec<u8>) -> io::Result<Vec<u8>> {
        let mut compressed = Vec::new();

        // so that the borrower is freed
        {
            let mut writer = CompressorWriter::with_params(
                &mut compressed,
                1024,
                &BrotliEncoderParams::default(),
            );
            writer.write_all(content)?;
        }

        Ok(compressed)
    }

    async fn compress_stream(&self, input: Arc<Mutex<DuplexStream>>, out: Arc<Mutex<DuplexStream>>) -> io::Result<()> {
        if let Ok(e) = input.try_lock() {
            let mut enc = BrotliEncoder::with_quality(e, Precise(5));
            return io::copy(&mut enc, &mut *out.try_lock().unwrap()).await.map(|_| ())
        }
        
        Ok(())
    }
}
