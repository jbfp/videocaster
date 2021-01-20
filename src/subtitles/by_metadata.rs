use super::{Subtitle, DEFAULT_LANG};
use crate::opensubs;
use anyhow::Error;
use rocket::{http::uri::Uri, response::Debug};
use rocket_contrib::json::Json;

#[get("/subtitles/by-metadata?<title>&<season>&<episode>")]
pub(crate) async fn handler(
    title: String,
    season: Option<String>,
    episode: Option<String>,
) -> Result<Json<Vec<Subtitle>>, Debug<Error>> {
    let url = format_url(&title, season.as_deref(), episode.as_deref());
    let subtitles = opensubs::download_subtitles(&url).await?;
    info!("found {} subtitles", subtitles.len());
    Ok(Json(subtitles))
}

fn format_url(title: &str, season: Option<&str>, episode: Option<&str>) -> String {
    let title = Uri::percent_encode(&title);

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

    url
}
