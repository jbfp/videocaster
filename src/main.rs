#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::{
    client::Client,
    dev::Body,
    get,
    middleware::Logger,
    web::{Path as ActixPath, Query},
    App, HttpResponse, HttpServer, Result as ActixResult,
};
use bytes::Buf;
use flate2::bufread::GzDecoder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, Error as IoError, Read, Result as IoResult, Seek, SeekFrom},
    path::{Path, PathBuf},
};

#[actix_web::main]
async fn main() -> IoResult<()> {
    pretty_env_logger::init();

    HttpServer::new(|| {
        let logger = Logger::default();

        let cors = Cors::default()
            .allowed_headers(vec!["Accept-Encoding", "Content-Type", "Range"])
            .allowed_methods(vec!["GET"])
            .allowed_origin("https://www.gstatic.com");

        App::new()
            .wrap(cors)
            .wrap(logger)
            .service(fs)
            .service(video)
            .service(subtitles)
            .default_service(static_files())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

fn static_files() -> Files {
    Files::new("/", "./ui").index_file("index.html")
}

lazy_static! {
    static ref HOME: PathBuf = dirs::home_dir().unwrap_or_else(|| "/".into());
}

#[derive(Deserialize)]
struct FsQuery {
    path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
struct FsResult {
    items: Vec<Item>,
    real_path: String,
}

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
struct Item {
    is_dir: bool,
    name: String,
}

#[get("/fs")]
async fn fs(query: Query<FsQuery>) -> ActixResult<HttpResponse> {
    let mut path = PathBuf::new();

    match query.into_inner().path {
        None => path.push(HOME.as_os_str()),
        Some(inner) => path.push(inner),
    }

    let mut items = Vec::new();

    for entry in std::fs::read_dir(&path)? {
        if let Ok(entry) = entry {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy().to_string();

            if name.starts_with('.') {
                continue; // ignore hidden files
            }

            let file_type = entry.file_type()?;
            let is_dir = file_type.is_dir();

            if !is_dir {
                let path = Path::new(&file_name);
                let ext = path
                    .extension()
                    .map(|s| s.to_string_lossy().to_ascii_lowercase())
                    .unwrap_or_default();

                if !["avi", "mkv", "mp4"].contains(&ext.as_str()) {
                    continue;
                }
            }

            items.push(Item { is_dir, name });
        }
    }

    // directories first, then files, both sorted by case-insensitive name
    items.sort_unstable_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    // if we can go up...
    if path.parent().is_some() {
        // ... insert parent directory
        items.insert(
            0,
            Item {
                name: "..".to_string(),
                is_dir: true,
            },
        );
    }

    let real_path = path.canonicalize()?.display().to_string();

    info!("{}", real_path);

    Ok(HttpResponse::Ok().json(FsResult { items, real_path }))
}

#[derive(Deserialize)]
struct VideoRef {
    escaped_path: String,
}

impl VideoRef {
    fn unescape(&self) -> String {
        self.escaped_path.replace("%2E", ".").replace("%2F", "/")
    }
}

#[get("/video/{escaped_path}")]
async fn video(path: ActixPath<VideoRef>) -> ActixResult<NamedFile> {
    let mut root = HOME.clone();
    root.push(path.into_inner().unescape());
    let root = root.canonicalize()?;
    let path = root.as_path();
    info!("loading video at {:#?}", path);
    let file = NamedFile::open(path)?;
    Ok(file)
}

#[derive(Deserialize)]
struct Subtitle {
    #[serde(alias = "SubDownloadLink")]
    sub_download_link: String,
}

#[get("/subtitles/{escaped_path}")]
async fn subtitles(path: ActixPath<VideoRef>) -> ActixResult<HttpResponse> {
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

    const USER_AGENT: &str = "VLsub 0.10.2"; // TODO: awaiting opensubtitles user agent

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

const NO_SUBTITLES_FOUND: &str = r"WEBVTT

00:00:00.000 --> 00:00:15.000
No subtitles found";

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
