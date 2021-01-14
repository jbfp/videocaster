use crate::HOME;
use actix_web::{get, web::Query, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fs::DirEntry, path::PathBuf};

const VALID_EXTENSIONS: [&str; 3] = [".avi", ".mkv", ".mp4"];
const PARENT: &str = "..";

lazy_static! {
    static ref PARENT_ITEM: Item = Item {
        name: PARENT.to_string(),
        is_dir: true,
    };
}

#[derive(Deserialize)]
pub(crate) struct FsQuery {
    path: Option<String>,
}

#[derive(Clone, Serialize)]
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
    let path = default_path(query.path.as_deref());

    let mut items = std::fs::read_dir(&path)?
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(entry_to_item)
        .collect::<Vec<_>>();

    items.sort_unstable_by(sorting);

    // if we can go up...
    if path.parent().is_some() {
        // ... insert parent directory
        items.insert(0, PARENT_ITEM.clone());
    }

    let real_path = path.canonicalize()?.display().to_string();

    info!("{}", real_path);

    Ok(HttpResponse::Ok().json(FsResult { items, real_path }))
}

fn default_path(path: Option<&str>) -> PathBuf {
    let mut buf = PathBuf::new();

    match path {
        None => buf.push(HOME.as_os_str()),
        Some(inner) => buf.push(inner),
    }

    buf
}

fn entry_to_item(entry: DirEntry) -> Option<Item> {
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
    is_hidden(name) || (is_file && !has_correct_ext(name))
}

fn is_hidden(s: &str) -> bool {
    s.starts_with('.')
}

fn has_correct_ext(s: &str) -> bool {
    VALID_EXTENSIONS.iter().any(|ext| s.ends_with(ext))
}

fn sorting(a: &Item, b: &Item) -> Ordering {
    // directories first, then files, both sorted by case-insensitive name
    b.is_dir
        .cmp(&a.is_dir)
        .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
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

    mod is_hidden {
        use super::is_hidden;

        #[test]
        fn starts_with_dot() {
            assert!(is_hidden(".test"));
        }

        #[test]
        fn does_not_start_with_dot() {
            assert!(!is_hidden("test.mp4"));
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
