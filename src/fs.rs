use crate::{app_result::AppResult, HOME};
use anyhow::Error;
use rocket::response::Redirect;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::fs::{self, DirEntry};

const VALID_EXTENSIONS: [&str; 4] = [".avi", ".mkv", ".mp4", ".webm"];
const PARENT: &str = "..";

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
struct Item {
    is_dir: bool,
    name: String,
    path: PathBuf,
}

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub(crate) struct Directory {
    items: Vec<Item>,
    parent: Option<Item>,
    path: PathBuf,
}

#[get("/fs")]
pub(crate) async fn fallback() -> Redirect {
    let default_path = HOME.to_string_lossy().to_string();
    let uri = uri!(handler: default_path);
    Redirect::permanent(uri)
}

#[get("/fs?<path>")]
pub(crate) async fn handler(path: String) -> AppResult<Directory> {
    dir(&path).await.into()
}

async fn dir(path: &str) -> Result<Directory, Error> {
    info!("reading dir: {}", path);
    let path = dunce::canonicalize(path)?;
    debug!("canonical path: {}", path.display());
    let parent = get_parent(&path);
    debug!("parent: {:#?}", parent);
    let mut entries = fs::read_dir(&path).await?;
    let mut items = Vec::new();

    loop {
        let next = entries.next_entry().await?;

        if let Some(entry) = next {
            match entry_to_item(&entry).await {
                Ok(Some(item)) => items.push(item),
                Ok(None) => trace!("ignored file {}", entry.path().display()),
                Err(err) => error!("failed to convert entry to item: {}", err),
            }
        } else {
            break;
        }
    }

    info!("found {} files in {}", items.len(), path.display());

    Ok(Directory {
        items,
        parent,
        path,
    })
}

async fn entry_to_item(entry: &DirEntry) -> Result<Option<Item>, Error> {
    trace!("entry to item for {:#?}", entry);

    let file_type = entry.file_type().await?;
    let name = entry.file_name().to_string_lossy().to_string();

    let item = if ignore(&name, file_type.is_file()) {
        None
    } else {
        Some(Item {
            is_dir: file_type.is_dir(),
            name,
            path: entry.path(),
        })
    };

    Ok(item)
}

fn ignore(name: &str, is_file: bool) -> bool {
    fn is_hidden(s: &str) -> bool {
        s.starts_with('.')
    }

    fn has_correct_ext(s: &str) -> bool {
        VALID_EXTENSIONS.iter().any(|ext| s.ends_with(ext))
    }

    is_hidden(name) || (is_file && !has_correct_ext(name))
}

fn get_parent(path: &Path) -> Option<Item> {
    path.parent().map(|path| Item {
        is_dir: true,
        name: path
            .file_name()
            .and_then(|s| s.to_str())
            .or_else(|| path.to_str())
            .unwrap_or(PARENT)
            .to_string(),
        path: path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    mod ignore {
        use super::ignore;
        use test_case::test_case;

        #[test_case(".test", false => true; "when dir starts with dot")]
        #[test_case(".test", true => true; "when file starts with dot")]
        #[test_case("video", false => false; "when dir does not start with dot")]
        #[test_case("video", true => true; "when file does not start with dot but has incorrect ext")]
        #[test_case("video.avi", true => false; "when file does not start with dot and has avi ext")]
        #[test_case("video.mkv", true => false; "when file does not start with dot and has mkv ext")]
        #[test_case("video.mp4", true => false; "when file does not start with dot and has mp4 ext")]
        #[test_case("video.webm", true => false; "when file does not start with dot and has webm ext")]
        fn works(name: &str, is_file: bool) -> bool {
            ignore(name, is_file)
        }
    }
}
