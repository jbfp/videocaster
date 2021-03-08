use crate::opensubs;
use anyhow::Error;
use rocket::http::ContentType;
use rocket::response::{Content, Debug};

#[get("/subtitles/download/<url>")]
pub(crate) async fn handler(url: &str) -> Result<Content<String>, Debug<Error>> {
    let vtt = opensubs::download_subtitle(&url).await?;
    Ok(Content(ContentType::new("text", "vtt"), vtt))
}
