use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc};
use tokio::io;
use tokio::io::DuplexStream;
use tokio::sync::Mutex;

pub mod brotli;
mod compression;
pub mod gzip;

const BROTLI_ALGO: &str = "br";
const GZIP_ALGO: &str = "gzip";

/// supported algorithms in order of preference
pub const AVAILABLE_ALGORITHMS: [&'static str; 2] = [BROTLI_ALGO, GZIP_ALGO];

#[derive(Debug)]
pub struct Compression {
    algorithms: HashMap<String, Arc<dyn Compress>>,
}

#[async_trait::async_trait]
pub trait Compress: Debug + Send + Sync {
    fn compress(&self, content: &Vec<u8>) -> io::Result<Vec<u8>>;

    async fn compress_stream(
        &self,
        input: Arc<DuplexStream>,
        out: Arc<Mutex<DuplexStream>>,
    ) -> io::Result<()>;
}
