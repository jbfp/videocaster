use crate::{app_result::AppResult, HOME};
use anyhow::Error;
use serde::Serialize;
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};
use tokio::fs::{self, DirEntry};

const VALID_EXTENSIONS: [&str; 3] = [".avi", ".mkv", ".mp4"];
const PARENT: &str = "..";

#[derive(Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
struct Item {
    is_dir: bool,
    name: String,
    path: PathBuf,
}

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub(crate) struct Directory {
    items: Vec<Item>,
    real_path: String,
}

#[get("/fs?<path>")]
pub(crate) async fn handler(path: Option<String>) -> AppResult<Directory> {
    dir(path.as_deref()).await.into()
}

async fn dir(path: Option<&str>) -> Result<Directory, Error> {
    let path = default_path(path);
    trace!("path or default: {}", path.display());
    let path = dunce::canonicalize(path)?;
    let real_path = path.display().to_string();
    trace!("canonical path: {}", real_path);
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

    info!("found {} files in {}", items.len(), real_path);

    items.sort_unstable_by(sorting);

    // insert parent ".." if applicable
    get_parent(&path)
        .into_iter()
        .for_each(|parent| items.insert(0, parent));

    Ok(Directory { items, real_path })
}

fn default_path(path: Option<&str>) -> PathBuf {
    path.map_or_else(|| HOME.clone(), PathBuf::from)
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

fn sorting(a: &Item, b: &Item) -> Ordering {
    // directories first, then files, both sorted by case-insensitive name
    b.is_dir
        .cmp(&a.is_dir)
        .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
}

fn get_parent(path: &Path) -> Option<Item> {
    path.parent().map(|path| Item {
        is_dir: true,
        name: PARENT.to_string(),
        path: path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    mod default_path {
        use super::default_path;
        use crate::HOME;
        use std::path::Path;

        #[test]
        fn uses_home_if_no_path_provided() {
            assert_eq!(*HOME, default_path(None));
        }

        #[test]
        fn uses_path_provided() {
            let expected = Path::new("../");
            let actual = default_path(Some("../"));
            assert_eq!(expected, actual);
        }
    }

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
        fn works(name: &str, is_file: bool) -> bool {
            ignore(name, is_file)
        }
    }

    mod sorting {
        use super::{sorting, Item};
        use std::{cmp::Ordering, path::PathBuf};

        fn create_item(name: &str, is_dir: bool) -> Item {
            Item {
                is_dir,
                name: name.to_string(),
                path: PathBuf::default(),
            }
        }

        #[test]
        fn dir_before_file() {
            let a = create_item("a", true);
            let b = create_item("a", false);
            let actual = sorting(&a, &b);
            assert_eq!(actual, Ordering::Less);
        }

        #[test]
        fn dirs_a_before_b() {
            let a = create_item("a", true);
            let b = create_item("b", true);
            let actual = sorting(&a, &b);
            assert_eq!(actual, Ordering::Less);
        }

        #[test]
        fn files_a_before_b() {
            let a = create_item("b", false);
            let b = create_item("a", false);
            let actual = sorting(&a, &b);
            assert_eq!(actual, Ordering::Greater);
        }

        #[test]
        fn a_eq_b() {
            let a = create_item("a", false);
            let b = create_item("a", false);
            let actual = sorting(&a, &b);
            assert_eq!(actual, Ordering::Equal);
        }
    }
}
