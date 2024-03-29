use super::{Subtitle, DEFAULT_LANG};
use crate::opensubs;
use anyhow::{Context, Error};
use log::{debug, info};
use rocket::{get, response::Debug, serde::json::Json};
use std::{
    io::SeekFrom,
    path::{Path, PathBuf},
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
};

#[get("/subtitles/by-path?<path>")]
pub(crate) async fn handler(path: String) -> Result<Json<Vec<Subtitle>>, Debug<Error>> {
    let path = canonicalize(&path)?;
    info!("loading subtitles for {}", path.display());
    let mut file = open_file(&path).await?;
    let size = file_size(&file).await?;
    let hash = create_hash(&mut file, size).await?;
    let url = format_url(size, &hash);
    debug!("file size: {}, hash: {}", size, hash);
    let subtitles = opensubs::download_subtitles(&url).await?;
    info!("found {} subtitles", subtitles.len());
    Ok(Json(subtitles))
}

fn canonicalize<P: AsRef<Path>>(path: &P) -> Result<PathBuf, Error> {
    dunce::canonicalize(path)
        .with_context(|| format!("failed to canonicalize path: {:#?}", path.as_ref()))
}

async fn open_file<P: AsRef<Path>>(path: &P) -> Result<File, Error> {
    File::open(path)
        .await
        .with_context(|| format!("failed to open file: {:#?}", path.as_ref()))
}

async fn file_size(file: &File) -> Result<u64, Error> {
    file.metadata()
        .await
        .with_context(|| "failed to load metadata for file")
        .map(|md| md.len())
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

fn format_url(size: u64, hash: &str) -> String {
    format!(
        "https://rest.opensubtitles.org/search/moviebytesize-{}/moviehash-{}/sublanguageid-{}",
        size, hash, DEFAULT_LANG
    )
}
