use crate::{HOME, OPENSUBTITLES_USER_AGENT};
use anyhow::Error;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::ClientBuilder;
use rocket::response::Debug;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::{
    fs::Metadata,
    io::SeekFrom,
    path::{Path, PathBuf},
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
};

const DEFAULT_LANG: &str = "eng";

#[derive(Deserialize)]
struct Subtitle {
    #[serde(alias = "SubFileName")]
    sub_file_name: String,

    #[serde(alias = "SubDownloadLink")]
    sub_download_link: String,
}

#[derive(Serialize)]
pub(crate) struct SubtitleResult {
    name: String,
    url: String,
}

#[get("/subtitles/by-metadata?<title>&<season>&<episode>")]
pub(crate) async fn search_by_metadata(
    title: String,
    season: Option<usize>,
    episode: Option<usize>,
) -> Result<Json<Vec<SubtitleResult>>, Debug<Error>> {
    let title = utf8_percent_encode(&title, NON_ALPHANUMERIC).to_string();

    let mut url = format!(
        "https://rest.opensubtitles.org/search/query-{}/sublanguageid-{}",
        title, DEFAULT_LANG
    );

    if let Some(season) = season {
        url.push_str(&format!("/season-{}", season));
    }

    if let Some(episode) = episode {
        url.push_str(&format!("/episode-{}", episode));
    }

    let subtitles = download_subtitles(&url).await?;

    Ok(Json(subtitles))
}

#[get("/subtitles/by-path?<path>")]
pub(crate) async fn search_by_path(
    path: String,
) -> Result<Json<Vec<SubtitleResult>>, Debug<Error>> {
    let path = build_path(&path)?;

    info!("loading subtitles for {:#?}", path);

    let mut file = open_file(&path).await?;
    let metadata = metadata(&file).await?;
    let size = metadata.len();
    let hash = create_hash(&mut file, size).await?;
    let lang = "eng";
    let url = format!(
        "https://rest.opensubtitles.org/search/moviebytesize-{}/moviehash-{}/sublanguageid-{}",
        size, hash, lang
    );

    let subtitles = download_subtitles(&url).await?;

    Ok(Json(subtitles))
}

fn build_path(path: &str) -> Result<PathBuf, Error> {
    let mut root = HOME.clone();
    root.push(&path);
    Ok(root.canonicalize()?)
}

async fn open_file<P: AsRef<Path>>(path: &P) -> Result<File, Error> {
    Ok(File::open(path).await?)
}

async fn metadata(file: &File) -> Result<Metadata, Error> {
    Ok(file.metadata().await?)
}

// https://trac.opensubtitles.org/projects/opensubtitles/wiki/HashSourceCodes#RUST
async fn create_hash(file: &mut File, size: u64) -> Result<String, Error> {
    const HASH_BLK_SIZE: u64 = 65536;
    const ITERATIONS: u64 = HASH_BLK_SIZE / 8;

    let mut buf = [0u8; 8];
    let mut word: u64;
    let mut hash: u64 = size; // seed hash with size

    for _ in 0..ITERATIONS {
        file.read_exact(&mut buf).await?;
        word = u64::from_ne_bytes(buf);
        hash = hash.wrapping_add(word);
    }

    file.seek(SeekFrom::Start(size - HASH_BLK_SIZE)).await?;

    for _ in 0..ITERATIONS {
        file.read_exact(&mut buf).await?;
        word = u64::from_ne_bytes(buf);
        hash = hash.wrapping_add(word);
    }

    Ok(format!("{:01$x}", hash, 16))
}

async fn download_subtitles(url: &str) -> Result<Vec<SubtitleResult>, Error> {
    info!("subtitles search url: {}", url);

    let subtitles = ClientBuilder::new()
        .user_agent(OPENSUBTITLES_USER_AGENT)
        .build()?
        .get(url)
        .send()
        .await?
        .json::<Vec<Subtitle>>()
        .await?
        .into_iter()
        .map(|subtitle| SubtitleResult {
            name: subtitle.sub_file_name,
            url: subtitle.sub_download_link,
        })
        .collect::<Vec<_>>();

    info!("found {} subtitles", subtitles.len());

    Ok(subtitles)
}
