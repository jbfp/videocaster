use packer::Packer;
use rocket::{
    http::{ContentType, Status},
    Response,
};
use std::{io::Cursor, path::PathBuf};

#[derive(Packer)]
#[packer(source = "www/public")]
struct StaticFiles;

#[get("/", rank = 10)]
pub(crate) fn index() -> Response<'static> {
    get_file("index.html".into(), Some(ContentType::HTML))
}

#[get("/<path..>", rank = 10)]
pub(crate) fn file(path: PathBuf) -> Response<'static> {
    get_file(path, None)
}

fn get_file(path: PathBuf, content_type: Option<ContentType>) -> Response<'static> {
    let full_path = format!("www/public/{}", path.display());
    let file: Option<&'static [u8]> = StaticFiles::get(&full_path);

    let mut response = Response::build();

    if let Some(file) = file {
        response.status(Status::Ok);

        let size = file.len();
        let cursor = Cursor::new(file);
        response.sized_body(size, cursor);

        if let Some(content_type) = content_type {
            response.header(content_type);
        } else if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();

            if let Some(content_type) = ContentType::from_extension(&ext_str) {
                response.header(content_type);
            }
        }
    } else {
        response.status(Status::NotFound);
    }

    response.finalize()
}
