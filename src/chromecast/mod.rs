//! These modules provide handlers for the Chromecast to connect to
//! such as loading videos and downloading subtitles.
mod range;
pub(crate) mod subtitles;
pub(crate) mod video;

use percent_encoding::percent_decode_str;
use rocket::http::RawStr;

fn unescape(s: &RawStr) -> String {
    percent_decode_str(s).decode_utf8_lossy().to_string()
}
