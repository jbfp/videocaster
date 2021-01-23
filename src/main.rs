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
use rocket::http::Method;
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
        frame::handler,
        fs::fallback,
        fs::handler,
        ip::handler,
        subtitles::by_metadata::handler,
        subtitles::by_path::handler,
        static_files::index,
        static_files::file,
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
        use std::os::windows::process::CommandExt;
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

    let mut cmd = create_command();
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

    cmd.args(&[
        &app,
        &user_data_dir,
        "--start-maximized",
        "--no-default-browser-check",
    ]);

    debug!("chrome: {:#?}", cmd);

    match cmd.status().await {
        Ok(exit) => info!("google chrome stopped with code {}", exit),
        Err(err) => error!("failed to open google chrome: {}", err),
    }
}

fn whoami() -> String {
    let port = var("ROCKET_PORT").unwrap_or_else(|_| "8000".into());
    format!("http://localhost:{}", port)
}
