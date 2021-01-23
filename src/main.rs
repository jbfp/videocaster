#![feature(decl_macro, proc_macro_hygiene, str_split_once)]

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
use futures::future;
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::env::var;
use tokio::process::Command;

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
    let run = |cmd: &mut Command| {
        let app = format!("--app={}", whoami());

        let user_data_dir = {
            let mut data_dir = dirs::config_dir().expect("no config dir");
            data_dir.push("videocaster");
            format!("--user-data-dir={}", data_dir.display())
        };

        cmd.args(&[
            &app,
            &user_data_dir,
            "--start-maximized",
            "--no-default-browser-check",
        ]);

        debug!("chrome: {:#?}", cmd);

        cmd.status()
    };

    let fut = if cfg!(target_os = "windows") {
        run(Command::new("cmd").args(&["/C", "start", "chrome"]))
    } else {
        run(&mut Command::new("google-chrome"))
    };

    match fut.await {
        Ok(exit) => info!("google chrome stopped with code {}", exit),
        Err(err) => error!("failed to open google chrome: {}", err),
    }
}

fn whoami() -> String {
    let port = var("ROCKET_PORT").unwrap_or_else(|_| "8000".into());
    format!("http://localhost:{}", port)
}
