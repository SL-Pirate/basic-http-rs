use argh::FromArgs;

const DEFAULT_ADDRESS: &str = "0.0.0.0";
const DEFAULT_SERVE_PATH: &str = ".";

#[derive(FromArgs, Debug)]
/// Configurations for basic http server
pub struct CliArgs {
    /// address to bind
    #[argh(option, short = 'a', default = "DEFAULT_ADDRESS.to_string()")]
    pub address: String,

    /// port to bind
    #[argh(option, short = 'p', default = "8080")]
    pub port: u32,

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
}
