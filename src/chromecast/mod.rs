//! These modules provide handlers for the Chromecast to connect to
//! such as loading videos and downloading subtitles.

use percent_encoding::percent_decode_str;
use rocket::http::RawStr;

pub mod subtitles;
pub mod video;

fn unescape(s: &RawStr) -> String {
    percent_decode_str(s).decode_utf8_lossy().to_string()
}
