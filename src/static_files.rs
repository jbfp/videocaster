use packer::Packer;
use rocket::{
    http::{ContentType, Status},
    response::content::Custom,
};
use std::path::PathBuf;

#[derive(Packer)]
#[packer(source = "www/public", prefixed = false)]
pub(crate) struct StaticFiles;

#[get("/<path..>", rank = 10)]
pub(crate) fn file(path: Option<PathBuf>) -> Option<Custom<&'static [u8]>> {
    match path {
        None => index(),
        Some(path) => get_file(&path.to_string_lossy(), None),
    }
}

#[catch(404)]
pub(crate) fn fallback() -> Option<(Status, Custom<&'static [u8]>)> {
    index().map(|content| (Status::Ok, content))
}

fn index() -> Option<Custom<&'static [u8]>> {
    get_file("index.html", Some(ContentType::HTML))
}

fn get_file(file_name: &str, content_type: Option<ContentType>) -> Option<Custom<&'static [u8]>> {
    StaticFiles::get(file_name).map(|bytes| {
        let content_type = content_type.unwrap_or_else(|| {
            PathBuf::from(file_name)
                .extension()
                .map(|ext| ext.to_string_lossy())
                .and_then(|ext| ContentType::from_extension(&ext))
                .unwrap_or_default()
        });

        Custom(content_type, bytes)
    })
}
