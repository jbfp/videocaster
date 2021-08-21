use crate::opensubs;
use anyhow::Error;
use rocket::{
    get,
    http::ContentType,
    response::{content::Custom, Debug},
};

#[get("/subtitles/download/<url>")]
pub(crate) async fn handler(url: &str) -> Result<Custom<String>, Debug<Error>> {
    let vtt = opensubs::download_subtitle(url).await?;
    Ok(Custom(ContentType::new("text", "vtt"), vtt))
}
