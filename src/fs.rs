use crate::HOME;
use actix_web::{get, web::Query, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub(crate) struct FsQuery {
    path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
struct Item {
    is_dir: bool,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
struct FsResult {
    items: Vec<Item>,
    real_path: String,
}

#[get("/fs")]
pub(crate) async fn handler(query: Query<FsQuery>) -> ActixResult<HttpResponse> {
    let mut path = PathBuf::new();

    match query.into_inner().path {
        None => path.push(HOME.as_os_str()),
        Some(inner) => path.push(inner),
    }

    let mut items = Vec::new();

    for entry in std::fs::read_dir(&path)? {
        if let Ok(entry) = entry {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy().to_string();

            if name.starts_with('.') {
                continue; // ignore hidden files
            }

            let file_type = entry.file_type()?;
            let is_dir = file_type.is_dir();

            if !is_dir {
                let path = Path::new(&file_name);
                let ext = path
                    .extension()
                    .map(|s| s.to_string_lossy().to_ascii_lowercase())
                    .unwrap_or_default();

                if !["avi", "mkv", "mp4"].contains(&ext.as_str()) {
                    continue;
                }
            }

            items.push(Item { is_dir, name });
        }
    }

    // directories first, then files, both sorted by case-insensitive name
    items.sort_unstable_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    // if we can go up...
    if path.parent().is_some() {
        // ... insert parent directory
        items.insert(
            0,
            Item {
                name: "..".to_string(),
                is_dir: true,
            },
        );
    }

    let real_path = path.canonicalize()?.display().to_string();

    info!("{}", real_path);

    Ok(HttpResponse::Ok().json(FsResult { items, real_path }))
}
