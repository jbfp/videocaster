#[macro_use]
extern crate futures;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

mod common;
mod fs;
mod ip;
mod subtitles;
mod video;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{middleware::Logger, App, HttpServer};
use futures::future;
use std::io::Result as IoResult;

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
            .service(video::handler)
            .service(subtitles::handler)
            .default_service(Files::new("/", "./ui/public").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run();

    let browser = start_google_chrome();

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
