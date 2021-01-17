pub(crate) mod by_metadata;
pub(crate) mod by_path;

use serde::Serialize;

const DEFAULT_LANG: &str = "eng";

#[derive(Serialize)]
pub(crate) struct Subtitle {
    pub(crate) name: String,
    pub(crate) url: String,
}
