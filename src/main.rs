#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate futures;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate rocket;

mod chromecast;
mod fs;
mod ip;
mod opensubs;
mod subtitles;

use anyhow::Result;
use futures::future;
use packer::Packer;
use rocket::{
    http::{ContentType, Method, Status},
    Response,
};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::{env::var, io::Cursor, path::PathBuf};
use tokio::process::Command;

lazy_static! {
    /// The user's $HOME dir
    pub(crate) static ref HOME: PathBuf = dirs::home_dir().unwrap_or_else(|| "/".into());
}

#[rocket::main]
async fn main() -> Result<()> {
    pretty_env_logger::try_init()?;
    let rocket = start_rocket();
    let chrome = start_google_chrome();

    if cfg!(target_os = "windows") {
        future::join(rocket, chrome).await;
    } else {
        pin_mut!(rocket, chrome);
        future::select(rocket, chrome).await;
    };

    Ok(())
}

async fn start_rocket() {
    let routes = routes![
        chromecast::subtitles::handler,
        chromecast::video::handler,
        fs::handler,
        ip::handler,
        subtitles::by_metadata::handler,
        subtitles::by_path::handler,
        index,
        static_files
    ];

    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::some_exact(&["https://www.gstatic.com"]),
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Accept-Encoding", "Content-Type", "Range"]),
        ..Default::default()
    }
    .to_cors()
    .expect("CORS options are invalid");

    let fut = rocket::ignite().mount("/", routes).attach(cors).launch();

    if let Err(err) = fut.await {
        error!("Rocket failed to launch: {}", err);
    }
}

async fn start_google_chrome() {
    let port = var("ROCKET_PORT").unwrap_or_else(|_| "8000".into());
    let url = format!("http://localhost:{}", port);
    let cmd = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", "start", "chrome", &url])
            .status()
    } else {
        Command::new("google-chrome").arg(&url).status()
    };

    match cmd.await {
        Ok(exit) => info!("google chrome stopped with code {}", exit),
        Err(err) => error!("failed to open google chrome: {}", err),
    }
}

#[derive(Packer)]
#[packer(source = "www/public")]
struct StaticFiles;

#[get("/", rank = 10)]
fn index() -> Response<'static> {
    get_file("index.html".into(), Some(ContentType::HTML))
}

#[get("/<path..>", rank = 10)]
fn static_files(path: PathBuf) -> Response<'static> {
    get_file(path, None)
}

fn get_file(path: PathBuf, content_type: Option<ContentType>) -> Response<'static> {
    let full_path = format!("www/public/{}", path.display());
    let file: Option<&'static [u8]> = StaticFiles::get(&full_path);

    let mut response = Response::build();

    if let Some(file) = file {
        response.status(Status::Ok);

        let size = file.len();
        let cursor = Cursor::new(file);
        response.sized_body(size, cursor);

        if let Some(content_type) = content_type {
            response.header(content_type);
        } else if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();

            if let Some(content_type) = ContentType::from_extension(&ext_str) {
                response.header(content_type);
            }
        }
    } else {
        response.status(Status::NotFound);
    }

    response.finalize()
}
