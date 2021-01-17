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
mod subtitles;

use anyhow::Result;
use futures::future;
use rocket::http::Method;
use rocket_contrib::{crate_relative, serve::StaticFiles};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::path::PathBuf;
use tokio::task::{self, JoinHandle};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let server = start_rocket();
    let browser = start_google_chrome();

    // futures::future::select requires the futures to be pinned to the stack
    pin_mut!(server, browser);
    let _ = future::select(server, browser).await;

    Ok(())
}

async fn start_rocket() {
    let static_files = StaticFiles::from(crate_relative!("/ui/public"));

    let routes = routes![
        chromecast::subtitles::default_subs,
        chromecast::subtitles::get_subtitle,
        chromecast::video::handler,
        fs::handler,
        ip::handler,
        subtitles::search_by_metadata,
        subtitles::search_by_path
    ];

    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::some_exact(&["https://www.gstatic.com"]),
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Accept-Encoding", "Content-Type", "Range"]),
        ..Default::default()
    }
    .to_cors()
    .expect("CORS options are invalid");

    let fut = rocket::ignite()
        .mount("/", routes)
        .mount("/", static_files)
        .attach(cors)
        .launch();

    if let Err(err) = fut.await {
        error!("Rocket failed to launch: {}", err);
    }
}

fn start_google_chrome() -> JoinHandle<()> {
    task::spawn_blocking(|| {
        let chrome = if std::env::consts::OS == "windows" {
            "chrome"
        } else {
            "google-chrome"
        };

        match open::with("http://localhost:8000", chrome) {
            Ok(exit) => info!("google chrome stopped with code {}", exit),
            Err(err) => error!("failed to open google chrome: {}", err),
        }
    })
}

const OPENSUBTITLES_USER_AGENT: &str = "videocaster 1.0.0";

lazy_static! {
    /// The user's $HOME dir
    pub(crate) static ref HOME: PathBuf = dirs::home_dir().unwrap_or_else(|| "/".into());
}
