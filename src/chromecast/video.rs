use super::unescape;
use crate::HOME;
use anyhow::Error;
use rocket::{http::RawStr, response::Debug};
use tokio::fs::File;

#[get("/video/<path>")]
pub(crate) async fn handler(path: &RawStr) -> Result<File, Debug<Error>> {
    let path = unescape(path);
    Ok(run(&path).await?)
}

async fn run(path: &str) -> Result<File, Error> {
    let mut root = HOME.clone();
    root.push(path);
    let root = root.canonicalize()?;
    let path = root.as_path();

    info!("loading video at {:#?}", path);

    Ok(File::open(path).await?)
}
