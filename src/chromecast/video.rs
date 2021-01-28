// https://docs.rs/crate/actix-files/0.5.0/source/src/named.rs with modifications
use super::range::HttpRange;
use rocket::{
    http::{ContentType, Header, RawStr, Status},
    request::{FromRequest, Outcome},
    Response,
};
use std::{
    io::SeekFrom,
    ops::{Deref, DerefMut},
    path::PathBuf,
};
use thiserror::Error;
use tokio::{fs::File, io::AsyncSeekExt};

#[get("/video/<path>")]
pub(crate) async fn handler(path: &RawStr, range: Range) -> Response<'_> {
    let path: PathBuf = path.url_decode_lossy().into();

    let mut response = Response::build();
    response.header(Header::new("Accept-Ranges", "bytes"));

    if let Ok(mut file) = File::open(&path).await {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();

            if let Some(content_type) = ContentType::from_extension(&ext_str) {
                response.header(content_type);
            } else {
                warn!("no content type found for {} extension", ext_str);
            }
        } else {
            warn!("file {} does not have an extension", path.display());
        }

        let size;

        match file.metadata().await {
            Ok(metadata) => size = metadata.len(),
            Err(err) => {
                let path = path.display();
                error!("failed to get metadata for file {}: {}", path, err);
                response.status(Status::InternalServerError);
                return response.finalize();
            }
        }

        let mut length = size;
        let mut offset = 0;

        info!("range: {}", *range);

        match HttpRange::parse(&*range, length) {
            Ok(ranges) => {
                length = ranges[0].length;
                offset = ranges[0].start;

                response.header(Header::new("Content-Encoding", "identity"));
                response.header(Header::new(
                    "Content-Range",
                    format!("bytes {}-{}/{}", offset, offset + length - 1, size),
                ));
            }
            Err(err) => {
                warn!("range parsing error: {}", err);
                response.header(Header::new("Content-Range", format!("bytes */{}", length)));
                response.status(Status::RangeNotSatisfiable);
            }
        }

        if offset > 0 {
            if let Err(err) = file.seek(SeekFrom::Start(offset)).await {
                error!("failed to seek in file {}: {}", path.display(), err);
                response.status(Status::InternalServerError);
                return response.finalize();
            }
        }

        if offset > 0 || length < size {
            response.status(Status::PartialContent);
        } else {
            response.status(Status::Ok);
        }

        debug!("size {} len {} offset {}", size, length, offset);

        response.sized_body(Some(length as usize), file);
    } else {
        response.status(Status::NotFound);
    }

    response.finalize()
}

pub(crate) struct Range(String);

impl Deref for Range {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Range {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for Range {
    type Error = MissingRangeHeaderError;

    async fn from_request(
        request: &'a rocket::Request<'r>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let range = request.headers().get_one("Range").map(|s| s.to_owned());

        if let Some(range) = range {
            Outcome::Success(Range(range))
        } else {
            Outcome::Failure((Status::BadRequest, MissingRangeHeaderError))
        }
    }
}

#[derive(Debug, Error)]
#[error("Range header is missing")]
pub(crate) struct MissingRangeHeaderError;
