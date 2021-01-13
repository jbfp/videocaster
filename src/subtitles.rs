use crate::{HOME, OPENSUBTITLES_USER_AGENT};
use actix_web::{client::Client, get, web::Query, HttpResponse, Result as ActixResult};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, Error as IoError, Read, Seek, SeekFrom},
};

const DEFAULT_LANG: &str = "eng";

#[derive(Deserialize)]
struct Subtitle {
    #[serde(alias = "SubFileName")]
    sub_file_name: String,

    #[serde(alias = "SubDownloadLink")]
    sub_download_link: String,
}

#[derive(Deserialize)]
pub(crate) struct SearchByPathParams {
    path: String,
}

#[derive(Deserialize)]
pub(crate) struct SearchByMetadataParams {
    title: String,
    season: Option<usize>,
    episode: Option<usize>,
}

#[derive(Serialize)]
struct SubtitleResult {
    name: String,
    url: String,
}

#[get("/subtitles/by-metadata")]
pub(crate) async fn search_by_metadata(
    query: Query<SearchByMetadataParams>,
) -> ActixResult<HttpResponse> {
    let title = utf8_percent_encode(&query.title, NON_ALPHANUMERIC).to_string();

    let mut url = format!(
        "https://rest.opensubtitles.org/search/query-{}/sublanguageid-{}",
        title, DEFAULT_LANG
    );

    if let Some(season) = query.season {
        url.push_str(&format!("/season-{}", season));
    }

    if let Some(episode) = query.episode {
        url.push_str(&format!("/episode-{}", episode));
    }

    search_response(&url).await
}

#[get("/subtitles/by-path")]
pub(crate) async fn search_by_path(query: Query<SearchByPathParams>) -> ActixResult<HttpResponse> {
    let mut root = HOME.clone();
    root.push(&query.path);
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

    search_response(&url).await
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

async fn download_subtitles(mut url: &str) -> ActixResult<Vec<Subtitle>> {
    info!("subtitles search url: {}", url);

    let client = Client::new();
    let mut response;

    // todo: remove loop when awc supports redirection for real
    loop {
        response = client
            .get(url)
            .header("User-Agent", OPENSUBTITLES_USER_AGENT)
            .send()
            .await?;

        if response.status().is_redirection() {
            url = response
                .headers()
                .get("Location")
                .expect("a redirect must have a Location header")
                .to_str()
                .expect("Location header value must be a valid string");
        } else {
            break;
        }
    }

    let subtitles = response.json::<Vec<Subtitle>>().await?;

    info!("found {} subtitles", subtitles.len());

    Ok(subtitles)
}

async fn search_response(url: &str) -> ActixResult<HttpResponse> {
    let subtitles = download_subtitles(&url)
        .await?
        .into_iter()
        .map(|subtitle| SubtitleResult {
            name: subtitle.sub_file_name,
            url: subtitle.sub_download_link,
        })
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(subtitles))
}
