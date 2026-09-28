use crate::compression::brotli::BrotliCompressor;
use crate::compression::gzip::GzipCompressor;
use crate::compression::{AVAILABLE_ALGORITHMS, BROTLI_ALGO, Compress, Compression, GZIP_ALGO};
use std::collections::HashMap;

impl Compression {
    pub fn new() -> Compression {
        let mut algorithms: HashMap<String, Box<dyn Compress>> = HashMap::new();
        algorithms.insert(String::from(GZIP_ALGO), Box::new(GzipCompressor));
        algorithms.insert(String::from(BROTLI_ALGO), Box::new(BrotliCompressor));

        Compression { algorithms }
    }

    /// returns the used algorithm, the compressed bytes
    pub fn compress(
        &self,
        content: Vec<u8>,
        supported_algorithms: Vec<&str>,
    ) -> (Option<String>, Vec<u8>) {
        for algo in AVAILABLE_ALGORITHMS {
            let algorithm = String::from(algo);
            if supported_algorithms.contains(&algorithm.as_str()) {
                let compressor = self.algorithms.get(&algorithm).unwrap();

                match compressor.compress(&content) {
                    Ok(compressed) => {
                        return (Some(algorithm), compressed);
                    }
                    Err(e) => {
                        eprintln!("error compressing with {algo}: {e}");
                    }
                }
            }
        }

        (None, content)
    }

    pub fn can_encode(encodings: Vec<&str>) -> bool {
        for en in encodings {
            if AVAILABLE_ALGORITHMS.contains(&en) {
                return true;
            }
        }

        false
    }
}
