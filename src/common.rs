use serde::Deserialize;
use std::path::PathBuf;

lazy_static! {
    // the user's $HOME dir
    pub(crate) static ref HOME: PathBuf = dirs::home_dir().unwrap_or_else(|| "/".into());
}

// chromecast does not pass query parameters from the client to the server
// so we have to pass it as path parameters in escaped format
#[derive(Deserialize)]
pub(crate) struct VideoRef {
    escaped_path: String,
}

impl VideoRef {
    pub(crate) fn unescape(&self) -> String {
        self.escaped_path.replace("%2E", ".").replace("%2F", "/")
    }
}
