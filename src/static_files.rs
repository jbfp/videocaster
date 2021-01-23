use packer::Packer;
use rocket::{
    http::{ContentType, Status},
    response::{status, Content},
};
use status::Custom;
use std::path::PathBuf;

#[derive(Packer)]
#[packer(source = "www/public")]
struct StaticFiles;

#[get("/", rank = 10)]
pub(crate) fn index() -> Content<&'static [u8]> {
    get_file("index.html".into(), Some(ContentType::HTML)).expect("index.html must exist")
}

#[get("/<path..>", rank = 10)]
pub(crate) fn file(path: PathBuf) -> Option<Content<&'static [u8]>> {
    get_file(path, None)
}

#[catch(404)]
pub(crate) fn fallback() -> Custom<Content<&'static [u8]>> {
    Custom(Status::Ok, index())
}

fn get_file(path: PathBuf, content_type: Option<ContentType>) -> Option<Content<&'static [u8]>> {
    let full_path = format!("www/public/{}", path.display());

    debug!("static file path: {}", full_path);

    StaticFiles::get(&full_path).map(|bytes| {
        let content_type = content_type.unwrap_or_else(|| {
            path.extension()
                .map(|ext| ext.to_string_lossy())
                .and_then(|ext| ContentType::from_extension(&ext))
                .unwrap_or_default()
        });

        Content(content_type, bytes)
    })
}
