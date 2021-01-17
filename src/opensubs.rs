//! This module contains all functions related to OpenSubtitles.org
//! which is used for downloading subtitles.
use crate::subtitles::Subtitle;
use anyhow::Result;
use bytes::Buf;
use flate2::bufread::GzDecoder;
use regex::Regex;
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;
use std::io::Read;

const USER_AGENT: &str = "videocaster 1.0.0";

#[derive(Deserialize)]
struct OpenSubsSubtitle {
    #[serde(alias = "SubFileName")]
    sub_file_name: String,

    #[serde(alias = "SubDownloadLink")]
    sub_download_link: String,
}

pub(crate) async fn download_subtitles(url: &str) -> Result<Vec<Subtitle>> {
    info!("subtitles search url: {}", url);

    let subtitles = build_client()?
        .get(url)
        .send()
        .await?
        .json::<Vec<OpenSubsSubtitle>>()
        .await?
        .into_iter()
        .map(|subtitle| Subtitle {
            name: subtitle.sub_file_name,
            url: subtitle.sub_download_link,
        })
        .collect::<Vec<_>>();

    Ok(subtitles)
}

pub(crate) async fn download_subtitle(url: &str) -> Result<String> {
    info!("downloading subs from {}", url);
    let buf = build_client()?.get(url).send().await?.bytes().await?;
    let size = buf.len();
    trace!("unzipping {} bytes", size);
    let mut gz = GzDecoder::new(buf.reader());
    let mut buf = String::with_capacity(size * 2);
    gz.read_to_string(&mut buf)?;
    trace!("converting srt ({} bytes) to vtt", size);
    Ok(srt_to_vtt(&buf))
}

fn build_client() -> Result<Client> {
    Ok(ClientBuilder::new().user_agent(USER_AGENT).build()?)
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
