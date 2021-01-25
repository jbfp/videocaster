use packer::Packer;
use rocket::{
    http::{ContentType, Status},
    response::{status, Content},
};
use status::Custom;
use std::path::PathBuf;

#[derive(Packer)]
#[packer(source = "www/public", prefixed = false)]
pub(crate) struct StaticFiles;

#[get("/", rank = 10)]
pub(crate) fn index() -> Option<Content<&'static [u8]>> {
    get_file("index.html", Some(ContentType::HTML))
}

#[get("/<path..>", rank = 10)]
pub(crate) fn file(path: PathBuf) -> Option<Content<&'static [u8]>> {
    get_file(&path.to_string_lossy(), None)
}

#[catch(404)]
pub(crate) fn fallback() -> Option<Custom<Content<&'static [u8]>>> {
    index().map(|content| Custom(Status::Ok, content))
}

fn get_file(file_name: &str, content_type: Option<ContentType>) -> Option<Content<&'static [u8]>> {
    StaticFiles::get(&file_name).map(|bytes| {
        let content_type = content_type.unwrap_or_else(|| {
            PathBuf::from(file_name)
                .extension()
                .map(|ext| ext.to_string_lossy())
                .and_then(|ext| ContentType::from_extension(&ext))
                .unwrap_or_default()
        });

        Content(content_type, bytes)
    })
}
