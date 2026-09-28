use crate::compression::Compress;
use brotli::CompressorWriter;
use brotli::enc::BrotliEncoderParams;
use std::io::Write;

#[derive(Debug)]
pub struct BrotliCompressor;

impl Compress for BrotliCompressor {
    fn compress(&self, content: &Vec<u8>) -> std::io::Result<Vec<u8>> {
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
}
