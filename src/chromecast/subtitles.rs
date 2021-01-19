use crate::opensubs;
use anyhow::Error;
use rocket::http::{ContentType, RawStr};
use rocket::response::{Content, Debug};

#[get("/subtitles/download/<url>")]
pub(crate) async fn handler(url: &RawStr) -> Result<Content<String>, Debug<Error>> {
    let url = url.url_decode_lossy();
    let vtt = opensubs::download_subtitle(&url).await?;
    Ok(Content(ContentType::new("text", "vtt"), vtt))
}
