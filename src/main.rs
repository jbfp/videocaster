#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

mod fs;
mod ip;
mod subtitles;
mod video;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{middleware::Logger, App, HttpServer};
use serde::Deserialize;
use std::{io::Result as IoResult, path::PathBuf};

#[actix_web::main]
async fn main() -> IoResult<()> {
    pretty_env_logger::init();

    HttpServer::new(move || {
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
            .default_service(static_files())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

fn static_files() -> Files {
    Files::new("/", "./ui/public").index_file("index.html")
}

lazy_static! {
    // the user's $HOME dir
    static ref HOME: PathBuf = dirs::home_dir().unwrap_or_else(|| "/".into());
}

// chromecast does not pass query parameters from the client to the server
// so we have to pass it as path parameters in escaped format
#[derive(Deserialize)]
struct VideoRef {
    escaped_path: String,
}

impl VideoRef {
    fn unescape(&self) -> String {
        self.escaped_path.replace("%2E", ".").replace("%2F", "/")
    }
}
