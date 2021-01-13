#[macro_use]
extern crate futures;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

mod chromecast;
mod fs;
mod ip;
mod subtitles;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{middleware::Logger, App, HttpServer};
use futures::future;
use std::io::Result as IoResult;
use std::path::PathBuf;

#[actix_web::main]
async fn main() -> IoResult<()> {
    pretty_env_logger::init();

    let serve = HttpServer::new(move || {
        let logger = Logger::default();

        // these CORS options are required for subtitles to work
        let cors = Cors::default()
            .allowed_headers(vec!["Accept-Encoding", "Content-Type", "Range"])
            .allowed_methods(vec!["GET"])
            .allowed_origin("https://www.gstatic.com");

        App::new()
            .wrap(cors)
            .wrap(logger)
            .service(ip::handler)
            .service(fs::handler)
            .service(chromecast::subtitles::default_subs)
            .service(chromecast::subtitles::get_subtitle)
            .service(chromecast::video::handler)
            .service(subtitles::search_by_metadata)
            .service(subtitles::search_by_path)
            .default_service(Files::new("/", "./ui/public").index_file("index.html"))
    })
    .workers(1)
    .bind(("0.0.0.0", 8080))?
    .run();

    let browser = start_google_chrome();

    // futures::future::select requires the futures to be pinned to the stack
    pin_mut!(serve, browser);
    let _ = future::select(serve, browser).await;

    Ok(())
}

async fn start_google_chrome() {
    let chrome = if std::env::consts::OS == "windows" {
        "chrome"
    } else {
        "google-chrome"
    };

    match open::with("http://localhost:8080", chrome) {
        Ok(exit) => info!("google chrome stopped with code {}", exit),
        Err(err) => error!("failed to open google chrome: {}", err),
    }
}

const OPENSUBTITLES_USER_AGENT: &str = "videocaster 1.0.0";

lazy_static! {
    /// The user's $HOME dir
    pub(crate) static ref HOME: PathBuf = dirs::home_dir().unwrap_or_else(|| "/".into());
}
