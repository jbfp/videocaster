use super::unescape;
use crate::OPENSUBTITLES_USER_AGENT;
use anyhow::Error;
use bytes::Buf;
use flate2::bufread::GzDecoder;
use regex::Regex;
use reqwest::ClientBuilder;
use rocket::http::{ContentType, RawStr};
use rocket::response::{Content, Debug};
use std::io::Read;

lazy_static! {
    static ref VTT_CONTENT_TYPE: ContentType = ContentType::new("text", "vtt");
}

const NO_SUBTITLES_FOUND: &str = r"WEBVTT

00:00:00.000 --> 00:00:15.000
No subtitles found";

#[get("/subtitles/download/<url>")]
pub(crate) async fn get_subtitle(url: &RawStr) -> Result<Content<String>, Debug<Error>> {
    let url = unescape(url);
    let vtt = run (&url).await?;
    Ok(Content(VTT_CONTENT_TYPE.clone(), vtt))
}

async fn run(url: &str) -> Result<String, Error> {
    info!("downloading subs from {}", url);

    let buf = ClientBuilder::new()
        .user_agent(OPENSUBTITLES_USER_AGENT)
        .build()?
        .get(url)
        .send()
        .await?
        .bytes()
        .await?;

    let size = buf.len();
    trace!("unzipping {} bytes", size);
    let mut gz = GzDecoder::new(buf.reader());
    let mut buf = String::with_capacity(size * 2);
    gz.read_to_string(&mut buf)?;

    trace!("converting srt ({} bytes) to vtt", size);
    Ok(srt_to_vtt(&buf))
}

#[get("/subtitles/default")]
pub(crate) async fn default_subs() -> Content<&'static str> {
    Content(VTT_CONTENT_TYPE.clone(), NO_SUBTITLES_FOUND)
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
