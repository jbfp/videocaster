use super::unescape;
use crate::HOME;
use anyhow::Error;
use rocket::{http::RawStr, response::Debug};
use tokio::fs::File;

#[get("/video/<path>")]
pub(crate) async fn handler(path: &RawStr) -> Result<File, Debug<Error>> {
    let path = unescape(path);
    let file = load_file(&path).await?;
    Ok(file)
}

async fn load_file(path: &str) -> Result<File, Error> {
    // todo: interpret the path as the full path?
    trace!("path is {}", path);
    let mut root = HOME.clone();
    root.push(path);
    let root = root.canonicalize()?;
    let path = root.as_path();
    info!("loading video at {:#?}", path);
    Ok(File::open(path).await?)
}
