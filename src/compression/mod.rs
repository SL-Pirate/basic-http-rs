use std::collections::HashMap;
use std::fmt::Debug;
use std::io;

pub mod brotli;
mod compression;
pub mod gzip;

const BROTLI_ALGO: &str = "br";
const GZIP_ALGO: &str = "gzip";

/// supported algorithms in order of preference
pub const AVAILABLE_ALGORITHMS: [&'static str; 2] = [BROTLI_ALGO, GZIP_ALGO];

#[derive(Debug)]
pub struct Compression {
    algorithms: HashMap<String, Box<dyn Compress>>,
}

pub trait Compress: Debug + Send + Sync {
    fn compress(&self, content: &Vec<u8>) -> io::Result<Vec<u8>>;
}
