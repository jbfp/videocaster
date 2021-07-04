use super::{Subtitle, DEFAULT_LANG};
use crate::opensubs;
use anyhow::Error;
use percent_encoding::NON_ALPHANUMERIC;
use rocket::{response::Debug, serde::json::Json};

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
    let mut url = format!(
        "https://rest.opensubtitles.org/search/query-{}/sublanguageid-{}",
        encode(title),
        DEFAULT_LANG
    );

    if let Some(season) = season {
        url.push_str(&format!("/season-{}", encode(season)));
    }

    if let Some(episode) = episode {
        url.push_str(&format!("/episode-{}", encode(episode)));
    }

    url
}

fn encode(input: &str) -> String {
    const INVALID_CHARS: [char; 2] = ['.', '/'];
    let input = input.replace(&INVALID_CHARS[..], " ");
    let encoded = percent_encoding::utf8_percent_encode(&input, NON_ALPHANUMERIC);
    encoded.to_string()
}
