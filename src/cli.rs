use argh::FromArgs;

const DEFAULT_ADDRESS: &str = "0.0.0.0";
const DEFAULT_SERVE_PATH: &str = ".";

#[derive(FromArgs, Debug)]
/// Configurations for basic http server
pub struct CliArgs {
    /// address to bind
    #[argh(option, short = 'a', default = "DEFAULT_ADDRESS.to_string()")]
    address: String,

    /// port to bind
    #[argh(option, short = 'p', default = "8080")]
    port: u32,

    /// enable brotli and gzip compression
    #[argh(switch, short = 'c')]
    enable_compression: bool,

    /// path to serve
    #[argh(positional, default = "DEFAULT_SERVE_PATH.to_string()")]
    path: String,
}

impl CliArgs {
    pub fn get_path(&self) -> String {
        if self.path.ends_with("/") {
            let len = self.path.len() - 1;
            self.path[..len].to_string()
        } else {
            self.path.clone()
        }
    }

    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn port(&self) -> u32 {
        self.port
    }

    pub fn is_compression_enabled(&self) -> bool {
        self.enable_compression
    }
}
