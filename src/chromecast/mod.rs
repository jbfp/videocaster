//! These modules provide handlers for the Chromecast to connect to
//! such as loading videos and downloading subtitles.
use serde::Deserialize;

pub mod subtitles;
pub mod video;

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
