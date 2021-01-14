use super::VideoRef;
use crate::HOME;
use actix_files::NamedFile;
use actix_web::{get, web::Path, Result as ActixResult};

#[get("/video/{escaped_path}")]
pub(crate) async fn handler(path: Path<VideoRef>) -> ActixResult<NamedFile> {
    let mut root = HOME.clone();
    root.push(path.into_inner().unescape());
    let root = root.canonicalize()?;
    let path = root.as_path();
    info!("loading video at {:#?}", path);
    let file = NamedFile::open(path)?;
    Ok(file)
}
