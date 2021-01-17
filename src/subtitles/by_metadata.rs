use super::{Subtitle, DEFAULT_LANG};
use crate::opensubs;
use anyhow::Error;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use rocket::response::Debug;
use rocket_contrib::json::Json;

#[get("/subtitles/by-metadata?<title>&<season>&<episode>")]
pub(crate) async fn handler(
    title: String,
    season: Option<usize>,
    episode: Option<usize>,
) -> Result<Json<Vec<Subtitle>>, Debug<Error>> {
    let title = utf8_percent_encode(&title, NON_ALPHANUMERIC).to_string();

    let mut url = format!(
        "https://rest.opensubtitles.org/search/query-{}/sublanguageid-{}",
        title, DEFAULT_LANG
    );

    if let Some(season) = season {
        url.push_str(&format!("/season-{}", season));
    }

    if let Some(episode) = episode {
        url.push_str(&format!("/episode-{}", episode));
    }

    let subtitles = opensubs::download_subtitles(&url).await?;
    info!("found {} subtitles", subtitles.len());
    Ok(Json(subtitles))
}
