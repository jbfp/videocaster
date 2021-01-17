use super::unescape;
use crate::opensubs;
use anyhow::Error;
use rocket::http::{ContentType, RawStr};
use rocket::response::{Content, Debug};

lazy_static! {
    static ref VTT_CONTENT_TYPE: ContentType = ContentType::new("text", "vtt");
}

const NO_SUBTITLES_FOUND: &str = r"WEBVTT

00:00:00.000 --> 00:00:15.000
No subtitles found";

#[get("/subtitles/download/<url>")]
pub(crate) async fn get_subtitle(url: &RawStr) -> Result<Content<String>, Debug<Error>> {
    let url = unescape(url);
    let vtt = opensubs::download_subtitle(&url).await?;
    Ok(Content(VTT_CONTENT_TYPE.clone(), vtt))
}

#[get("/subtitles/default")]
pub(crate) async fn default_subs() -> Content<&'static str> {
    Content(VTT_CONTENT_TYPE.clone(), NO_SUBTITLES_FOUND)
}
