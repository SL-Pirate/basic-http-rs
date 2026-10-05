use crate::compression::brotli::BrotliCompressor;
use crate::compression::gzip::GzipCompressor;
use crate::compression::{AVAILABLE_ALGORITHMS, BROTLI_ALGO, Compress, Compression, GZIP_ALGO};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, DuplexStream};
use tokio::sync::Mutex;

impl Compression {
    pub fn new() -> Compression {
        let mut algorithms: HashMap<String, Arc<dyn Compress>> = HashMap::new();
        algorithms.insert(String::from(GZIP_ALGO), Arc::new(GzipCompressor));
        algorithms.insert(String::from(BROTLI_ALGO), Arc::new(BrotliCompressor));

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

    pub async fn compress_stream(
        &self,
        content: DuplexStream,
        out: DuplexStream,
        supported_algorithms: Vec<&str>,
    ) -> Option<String> {
        let content_arc = Arc::new(content);
        let out_arc = Arc::new(Mutex::new(out));

        for algo in AVAILABLE_ALGORITHMS {
            let algorithm = String::from(algo);
            if supported_algorithms.contains(&algorithm.as_str()) {
                let compressor = self.algorithms.get(&algorithm).unwrap().clone();
                let content_clone = Arc::clone(&content_arc);
                let out_clone = Arc::clone(&out_arc);

                tokio::spawn(async move {
                    compressor
                        .compress_stream(content_clone, out_clone)
                        .await
                        .err();
                    if let Ok(mut e) = Arc::clone(&out_arc).try_lock() {
                        if let Err(err) = e.shutdown().await {
                            eprintln!("Error compressing stream: {err}");
                        }
                    };
                });

                return Some(algorithm);
            }
        }

        None
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
