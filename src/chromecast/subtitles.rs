use super::VideoRef;
use crate::HOME;
use actix_web::{client::Client, dev::Body, get, web::Path, HttpResponse, Result as ActixResult};
use bytes::Buf;
use flate2::bufread::GzDecoder;
use regex::Regex;
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufReader, Error as IoError, Read, Seek, SeekFrom},
};

const USER_AGENT: &str = "videocaster 1.0.0";

const NO_SUBTITLES_FOUND: &str = r"WEBVTT

00:00:00.000 --> 00:00:15.000
No subtitles found";

#[derive(Deserialize)]
struct Subtitle {
    #[serde(alias = "SubDownloadLink")]
    sub_download_link: String,
}

#[get("/subtitles/{escaped_path}")]
pub(crate) async fn handler(path: Path<VideoRef>) -> ActixResult<HttpResponse> {
    let mut root = HOME.clone();
    root.push(path.into_inner().unescape());
    let root = root.canonicalize()?;
    let path = root.as_path();

    info!("loading subtitles for {:#?}", path);

    let file = File::open(path)?;
    let metadata = file.metadata()?;

    let size = metadata.len();
    let hash = create_hash(&file, size)?;
    let lang = "eng";
    let url = format!(
        "https://rest.opensubtitles.org/search/moviebytesize-{}/moviehash-{}/sublanguageid-{}",
        size, hash, lang
    );

    info!("url: {}", url);

    let client = Client::new();

    let subtitles = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .json::<Vec<Subtitle>>()
        .await?;

    info!("found {} subtitles", subtitles.len());

    let vtt: Body = if let Some(subtitle) = subtitles.first() {
        info!("downloading subs from {}", subtitle.sub_download_link);

        let buf = client
            .get(&subtitle.sub_download_link)
            .header("User-Agent", USER_AGENT)
            .send()
            .await?
            .body()
            .await?;

        // unzip response
        let mut gz = GzDecoder::new(buf.reader());
        let mut buf = String::with_capacity(buf.len() * 2);
        gz.read_to_string(&mut buf)?;

        // convert srt to vtt
        srt_to_vtt(&buf).into()
    } else {
        info!("couldn't find any subtitles for {}", hash);
        NO_SUBTITLES_FOUND.into()
    };

    Ok(HttpResponse::Ok()
        .set_header("Content-Type", "text/vtt")
        .body(vtt))
}

// https://trac.opensubtitles.org/projects/opensubtitles/wiki/HashSourceCodes#RUST
fn create_hash<S: Read + Seek>(stream: S, size: u64) -> Result<String, IoError> {
    const HASH_BLK_SIZE: u64 = 65536;
    const ITERATIONS: u64 = HASH_BLK_SIZE / 8;

    let mut buf = [0u8; 8];
    let mut word: u64;
    let mut hash: u64 = size; // seed hash with size
    let mut reader = BufReader::with_capacity(HASH_BLK_SIZE as usize, stream);

    for _ in 0..ITERATIONS {
        reader.read_exact(&mut buf)?;
        word = u64::from_ne_bytes(buf);
        hash = hash.wrapping_add(word);
    }

    reader.seek(SeekFrom::Start(size - HASH_BLK_SIZE))?;

    for _ in 0..ITERATIONS {
        reader.read_exact(&mut buf)?;
        word = u64::from_ne_bytes(buf);
        hash = hash.wrapping_add(word);
    }

    Ok(format!("{:01$x}", hash, 16))
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
