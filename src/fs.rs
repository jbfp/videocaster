use crate::HOME;
use anyhow::Error;
use rocket::response::Debug;
use rocket_contrib::json::Json;
use serde::Serialize;
use std::{
    cmp::Ordering,
    fs::DirEntry,
    path::{Path, PathBuf},
};

const VALID_EXTENSIONS: [&str; 3] = [".avi", ".mkv", ".mp4"];
const PARENT: &str = "..";

lazy_static! {
    static ref PARENT_ITEM: Item = Item {
        name: PARENT.to_string(),
        is_dir: true,
    };
}

#[derive(Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
struct Item {
    is_dir: bool,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub(crate) struct FsResult {
    items: Vec<Item>,
    real_path: String,
}

#[get("/fs?<path>")]
pub(crate) async fn handler(path: Option<String>) -> Result<Json<FsResult>, Debug<Error>> {
    Ok(Json(run(path.as_deref())?))
}

fn run(path: Option<&str>) -> Result<FsResult, Error> {
    let path = default_path(path);
    let path = dunce::canonicalize(path)?;

    trace!("canonical path: {:#?}", path);

    let mut items = std::fs::read_dir(&path)?
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(entry_to_item)
        .collect::<Vec<_>>();

    items.sort_unstable_by(sorting);

    // insert parent ".." if applicable
    get_parent(&path)
        .into_iter()
        .for_each(|parent| items.insert(0, parent));

    let real_path = path.display().to_string();

    Ok(FsResult { items, real_path })
}

fn default_path(path: Option<&str>) -> PathBuf {
    path.map_or_else(|| HOME.clone(), PathBuf::from)
}

fn entry_to_item(entry: DirEntry) -> Option<Item> {
    trace!("entry to item for {:#?}", entry);

    match entry.file_type() {
        Err(err) => {
            info!("failed to get file type: {}", err);
            None
        }

        Ok(file_type) => {
            let name = entry.file_name().to_string_lossy().to_string();

            if ignore(&name, file_type.is_file()) {
                None
            } else {
                let item = Item {
                    is_dir: file_type.is_dir(),
                    name,
                };

                Some(item)
            }
        }
    }
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
    path.parent().map(|_| PARENT_ITEM.clone())
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
        use std::cmp::Ordering;

        #[test]
        fn dir_before_file() {
            let a = Item {
                name: "a".to_string(),
                is_dir: true,
            };
            let b = Item {
                name: "a".to_string(),
                is_dir: false,
            };
            let actual = sorting(&a, &b);
            assert_eq!(actual, Ordering::Less);
        }

        #[test]
        fn dirs_a_before_b() {
            let a = Item {
                name: "a".to_string(),
                is_dir: true,
            };
            let b = Item {
                name: "b".to_string(),
                is_dir: true,
            };
            let actual = sorting(&a, &b);
            assert_eq!(actual, Ordering::Less);
        }

        #[test]
        fn files_a_before_b() {
            let a = Item {
                name: "b".to_string(),
                is_dir: false,
            };
            let b = Item {
                name: "a".to_string(),
                is_dir: false,
            };
            let actual = sorting(&a, &b);
            assert_eq!(actual, Ordering::Greater);
        }

        #[test]
        fn a_eq_b() {
            let a = Item {
                name: "a".to_string(),
                is_dir: false,
            };
            let b = Item {
                name: "a".to_string(),
                is_dir: false,
            };
            let actual = sorting(&a, &b);
            assert_eq!(actual, Ordering::Equal);
        }
    }
}
