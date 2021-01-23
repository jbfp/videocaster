#![feature(decl_macro, proc_macro_hygiene, str_split_once)]
#![cfg_attr(profile = "release", windows_subsystem = "windows")]

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate futures;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate rocket;

mod app_result;
mod chromecast;
mod frame;
mod fs;
mod ip;
mod opensubs;
mod static_files;
mod subtitles;

use anyhow::Result;
use directories_next::ProjectDirs;
use futures::future;
use rocket::{http::Method, Shutdown};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::env::var;
use tokio::process::Command;

const QUALIFIER: &str = "dk";
const ORGANIZATION: &str = "jbfp";
const APPLICATION: &str = "Videocaster";

#[rocket::main]
async fn main() -> Result<()> {
    pretty_env_logger::try_init()?;
    let rocket = start_rocket();
    let chrome = start_google_chrome();

    if cfg!(target_os = "windows") {
        // Chrome on Windows returns immediately after launch. This means we can't rely
        // on closing Chrome to notify us to stop via futures. To fix this, we register
        // a handler to signal for shutdown. (If the server is stopped via other means,
        // Chrome won't be closed automatically.)
        future::join(rocket, chrome).await;
    } else {
        // Chrome on not-Windows does not return until the last window is closed. If the
        // server is closed via ctrl+c, we also close Chrome automatically.
        pin_mut!(rocket, chrome);
        future::select(rocket, chrome).await;
    };

    Ok(())
}

#[post("/shutdown")]
pub(crate) async fn shutdown(shutdown: Shutdown) {
    shutdown.shutdown()
}

async fn start_rocket() {
    let routes = routes![
        chromecast::subtitles::handler,
        chromecast::video::handler,
        frame::handler,
        fs::fallback,
        fs::handler,
        ip::handler,
        shutdown,
        static_files::index,
        static_files::file,
        subtitles::by_metadata::handler,
        subtitles::by_path::handler,
    ];

    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::some_exact(&["https://www.gstatic.com", &whoami()]),
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Accept-Encoding", "Content-Type", "Range"]),
        ..Default::default()
    }
    .to_cors()
    .expect("CORS options are invalid");

    let fut = rocket::ignite()
        .mount("/", routes)
        .register(catchers![static_files::fallback])
        .attach(cors)
        .launch();

    if let Err(err) = fut.await {
        error!("Rocket failed to launch: {}", err);
    }
}

async fn start_google_chrome() {
    #[cfg(target_os = "windows")]
    fn create_command() -> Command {
        const DETACHED_PROCESS: u32 = 0x00000008;
        let mut command = Command::new("cmd");
        command.args(&["/C", "start", "chrome"]);
        command.creation_flags(DETACHED_PROCESS);
        command
    }

    #[cfg(not(target_os = "windows"))]
    fn create_command() -> Command {
        Command::new("google-chrome")
    }

    let app = format!("--app={}", whoami());

    let user_data_dir = {
        if let Some(dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
            let path = dirs.config_dir().display();
            info!("project config dir: {}", path);
            format!("--user-data-dir={}", path)
        } else {
            warn!("no project dirs found, using chrome's default data dir");
            "".to_string()
        }
    };

    let args = [
        &app,
        &user_data_dir,
        "--start-maximized",
        "--no-default-browser-check",
    ];

    debug!("chrome args: {:#?}", args);

    match create_command().args(&args).status().await {
        Ok(exit) => info!("google chrome stopped with code {}", exit),
        Err(err) => error!("failed to open google chrome: {}", err),
    }
}

fn whoami() -> String {
    let port = var("ROCKET_PORT").unwrap_or_else(|_| "8000".into());
    format!("http://localhost:{}", port)
}
