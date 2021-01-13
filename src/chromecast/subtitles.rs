use std::io::Read;

use crate::OPENSUBTITLES_USER_AGENT;
use actix_web::{client::Client, dev::Body, get, web::Path, HttpResponse, Result as ActixResult};
use bytes::Buf;
use flate2::bufread::GzDecoder;
use regex::Regex;
use serde::Deserialize;

const NO_SUBTITLES_FOUND: &str = r"WEBVTT

00:00:00.000 --> 00:00:15.000
No subtitles found";

#[derive(Deserialize)]
pub(crate) struct SubtitleRef {
    url: String,
}

#[get("/subtitles/download/{escaped_path}")]
pub(crate) async fn get_subtitle(path: Path<SubtitleRef>) -> ActixResult<HttpResponse> {
    info!("downloading subs from {}", path.url);

    let buf = Client::new()
        .get(&path.url)
        .header("User-Agent", OPENSUBTITLES_USER_AGENT)
        .send()
        .await?
        .body()
        .await?;

    trace!("unzipping {} bytes", buf.len());
    let mut gz = GzDecoder::new(buf.reader());
    let mut buf = String::with_capacity(buf.len() * 2);
    gz.read_to_string(&mut buf)?;

    trace!("converting srt ({} bytes) to vtt", buf.len());
    let vtt = srt_to_vtt(&buf);

    Ok(vtt_to_http_response(vtt))
}

#[get("/subtitles/default")]
pub(crate) async fn default_subs() -> HttpResponse {
    vtt_to_http_response(NO_SUBTITLES_FOUND)
}

// https://raw.githubusercontent.com/nwoltman/srt-to-vtt-converter/master/SRT%20to%20VTT%20Converter/SubtitleConverter.cs
fn srt_to_vtt(srt: &str) -> String {
    lazy_static! {
        static ref CUE_ID_REGEX: Regex =
            Regex::new(r"^\d+$").expect("cue id regex compilation failed");
        static ref TIME_FRAME_REGEX: Regex =
            Regex::new(r"(\d\d:\d\d:\d\d(?:[,.]\d\d\d)?) --> (\d\d:\d\d:\d\d(?:[,.]\d\d\d)?)")
                .expect("time frame regex compilation failed");
    }

    let mut vtt = String::with_capacity(srt.len());

    // vtt header
    vtt.push_str("WEBVTT\n\n");

    for line in srt.lines() {
        // ignore cue ID number lines
        if CUE_ID_REGEX.is_match(line) {
            continue;
        }

        // replace , in timestamps with .
        if TIME_FRAME_REGEX.is_match(line) {
            vtt.push_str(&line.replace(",", "."))
        } else {
            vtt.push_str(line)
        }

        vtt.push('\n');
    }

    vtt
}

fn vtt_to_http_response<T: Into<Body>>(vtt: T) -> HttpResponse {
    HttpResponse::Ok()
        .set_header("Content-Type", "text/vtt")
        .body(vtt)
}
