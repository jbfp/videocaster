#![feature(decl_macro, proc_macro_hygiene)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate futures;

#[macro_use]
extern crate lazy_static;

extern crate log;

#[macro_use]
extern crate rocket;

mod app_result;
mod bindings;
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
use log::LevelFilter;
use rocket::{Build, Config, Ignite, Rocket, Shutdown, figment::{
        providers::{Env, Format, Toml},
        Figment,
    }, http::Method};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::path::{Path, PathBuf};
use tokio::{io::AsyncWriteExt, process::Command};

const CONFIG_PATH: &str = "Videocaster.toml";
const ENV_PREFIX: &str = "VIDEOCASTER_";

#[rocket::main]
async fn main() -> Result<()> {
    let config_path = create_config_file().await?;
    let _ = configure_logging();
    let rocket = create_rocket(&config_path).ignite().await?;
    let config = rocket.config().to_owned();
    let chrome = start_google_chrome(&config);
    let server = start_rocket(rocket);

    if cfg!(target_os = "windows") {
        // Chrome on Windows returns immediately after launch. This means we can't rely
        // on closing Chrome to notify us to stop via futures. To fix this, we register
        // a handler to signal for shutdown. (If the server is stopped via other means,
        // Chrome won't be closed automatically.)
        future::join(server, chrome).await;
    } else {
        // Chrome on not-Windows does not return until the last window is closed. If the
        // server is closed via ctrl+c, we also close Chrome automatically.
        pin_mut!(server, chrome);
        future::select(server, chrome).await;
    };

    Ok(())
}

#[post("/shutdown")]
pub(crate) async fn shutdown(shutdown: Shutdown) {
    shutdown.notify()
}

#[cfg(not(debug_assertions))]
fn configure_logging() -> Result<()> {
    use std::{env, time};
    let timestamp = time::UNIX_EPOCH.elapsed().unwrap_or_default().as_secs();
    let file_name = format!("videocaster_{:#?}", timestamp);
    let mut path = env::temp_dir();
    path.push(file_name);
    path.set_extension("log");
    Ok(simple_logging::log_to_file(&path, LevelFilter::Debug)?)
}

#[cfg(debug_assertions)]
fn configure_logging() -> Result<()> {
    simple_logging::log_to_stderr(LevelFilter::Debug);
    Ok(())
}

async fn create_config_file() -> Result<PathBuf> {
    let dirs = open_project_dirs().ok_or_else(|| anyhow!("failed to open project dirs"))?;
    let mut path = dirs.config_dir().to_path_buf();
    path.push(CONFIG_PATH);

    debug!("config file path: {}", path.display());

    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&path)
        .await;

    if let Ok(mut file) = file {
        let default_config = include_bytes!("../Release.toml");
        file.write_all(default_config).await?;
        info!("re-created default config file");
    } else {
        debug!("config file exists, won't overwrite");
    }

    Ok(path)
}

fn create_rocket<P: AsRef<Path>>(config_path: P) -> Rocket<Build> {
    let figment = Figment::from(Config::default())
        .merge(Toml::file(config_path).nested())
        .merge(Env::prefixed(ENV_PREFIX).global());

    let routes = routes![
        chromecast::subtitles::handler,
        chromecast::video::handler,
        frame::handler,
        fs::fallback,
        fs::handler,
        ip::handler,
        shutdown,
        static_files::file,
        subtitles::by_metadata::handler,
        subtitles::by_path::handler,
    ];

    let catchers = catchers![static_files::fallback];

    let config = figment.extract::<Config>().expect("config");
    let rocket = rocket::custom(figment);
    let port = config.port;
    let host = format!("http://localhost:{}", port);
    let cors = CorsOptions {
        allowed_headers: AllowedHeaders::some(&["Accept-Encoding", "Content-Type", "Range"]),
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_origins: AllowedOrigins::some_exact(&["https://www.gstatic.com", &host]),
        ..Default::default()
    }
    .to_cors()
    .expect("CORS options are invalid");

    rocket
        .mount("/", routes)
        .register("/", catchers)
        .attach(cors)
}

async fn start_rocket(rocket: Rocket<Ignite>) {
    if let Err(e) = rocket.launch().await {
        error!("Rocket failed to launch: {}", e);
    }
}

async fn start_google_chrome(config: &Config) {
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

    let app = {
        let port = config.port;
        let url = format!("http://localhost:{}", port);
        format!("--app={}", url)
    };

    let user_data_dir = {
        if let Some(dirs) = open_project_dirs() {
            let path = dirs.config_dir().display();
            info!("project config dir: {}", path);
            format!("--user-data-dir={}", path)
        } else {
            warn!("no project dirs found, using chrome's default data dir");
            "".to_owned()
        }
    };

    let args = [&app, &user_data_dir, "--no-default-browser-check"];

    debug!("chrome args: {:#?}", args);

    match create_command().args(&args).status().await {
        Ok(exit) => info!("google chrome stopped with code {}", exit),
        Err(err) => error!("failed to open google chrome: {}", err),
    }
}

fn open_project_dirs() -> Option<ProjectDirs> {
    const QUALIFIER: &str = "dk";
    const ORGANIZATION: &str = "jbfp";
    const APPLICATION: &str = "Videocaster";
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
}
