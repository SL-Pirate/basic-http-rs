use crate::compression::Compress;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io;
use std::io::Write;

#[derive(Debug)]
pub struct GzipCompressor;

impl Compress for GzipCompressor {
    fn compress(&self, content: &Vec<u8>) -> io::Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&content)?;
        encoder.finish()
    }
}
