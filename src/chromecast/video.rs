// https://docs.rs/crate/actix-files/0.5.0/source/src/named.rs with modifications
use super::range::HttpRange;
use log::{debug, error, info, warn};
use rocket::{
    async_trait, get,
    http::{ContentType, Header, Status},
    request::{FromRequest, Outcome},
    response::{Responder, Result as RocketResult},
    Response,
};
use std::{
    fs::File,
    io::{Result as IoResult, Seek, SeekFrom},
    ops::{Deref, DerefMut},
    path::PathBuf,
    pin::Pin,
    task::{Context, Poll},
};
use thiserror::Error;
use tokio::{
    fs::File as AsyncFile,
    io::{AsyncRead, AsyncSeek, ReadBuf},
};

#[get("/video/<path>")]
pub(crate) async fn handler<'r>(path: &str, range: Range) -> VideoResponder {
    let path: PathBuf = path.into();
    VideoResponder { path, range }
}

pub(crate) struct VideoResponder {
    path: PathBuf,
    range: Range,
}

impl<'r> Responder<'r, 'r> for VideoResponder {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> RocketResult<'r> {
        let path = self.path;
        let mut response = Response::build();
        response.header(Header::new("Accept-Ranges", "bytes"));

        if let Ok(mut file) = File::open(&path) {
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

            let size = match file.metadata() {
                Ok(metadata) => metadata.len(),
                Err(err) => {
                    let path = path.display();
                    error!("failed to get metadata for file {}: {}", path, err);
                    response.status(Status::InternalServerError);
                    return Ok(response.finalize());
                }
            };

            let mut length = size;
            let mut offset = 0;

            info!("range: {}", *self.range);

            match HttpRange::parse(&*self.range, length) {
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
                if let Err(err) = file.seek(SeekFrom::Start(offset)) {
                    error!("failed to seek in file {}: {}", path.display(), err);
                    response.status(Status::InternalServerError);
                    return Ok(response.finalize());
                }
            }

            if offset > 0 || length < size {
                response.status(Status::PartialContent);
            } else {
                response.status(Status::Ok);
            }

            debug!("size {} len {} offset {}", size, length, offset);

            response.sized_body(Some(length as usize), FileWrapper::from(file));
        } else {
            response.status(Status::NotFound);
        }

        Ok(response.finalize())
    }
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
impl<'a, 'r> FromRequest<'r> for Range {
    type Error = MissingRangeHeaderError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
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

// Only for resetting system idle timer on Drop
// when the request has streamed what it needs to from the file.
struct FileWrapper {
    file: Pin<Box<AsyncFile>>,
}

impl From<File> for FileWrapper {
    fn from(file: File) -> Self {
        stop_system_idle_timer();

        Self {
            file: Box::pin(AsyncFile::from_std(file)),
        }
    }
}

impl AsyncRead for FileWrapper {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        self.get_mut().file.as_mut().poll_read(cx, buf)
    }
}

impl AsyncSeek for FileWrapper {
    fn start_seek(self: Pin<&mut Self>, position: SeekFrom) -> IoResult<()> {
        self.get_mut().file.as_mut().start_seek(position)
    }

    fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<u64>> {
        self.get_mut().file.as_mut().poll_complete(cx)
    }
}

impl Drop for FileWrapper {
    fn drop(&mut self) {
        start_system_idle_timer();
    }
}

#[cfg(target_os = "windows")]
fn stop_system_idle_timer() {
    use windows::Win32::System::Power::{
        SetThreadExecutionState, ES_AWAYMODE_REQUIRED, ES_CONTINUOUS, ES_DISPLAY_REQUIRED,
        ES_SYSTEM_REQUIRED,
    };

    debug!("stopping system idle timer");

    unsafe {
        SetThreadExecutionState(
            ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED,
        );
    }
}

#[cfg(target_os = "windows")]
fn start_system_idle_timer() {
    use windows::Win32::System::Power::{SetThreadExecutionState, ES_CONTINUOUS};

    debug!("starting system idle timer");

    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS);
    }
}

#[cfg(not(target_os = "windows"))]
fn stop_system_idle_timer() {}

#[cfg(not(target_os = "windows"))]
fn start_system_idle_timer() {}
