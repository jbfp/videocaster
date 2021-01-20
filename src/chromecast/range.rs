// https://docs.rs/crate/actix-files/0.5.0/source/src/range.rs with modifications
use thiserror::Error;
use std::num::ParseIntError;

/// HTTP Range header representation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct HttpRange {
    /// Start of range.
    pub(crate) start: u64,

    /// Length of range.
    pub(crate) length: u64,
}

#[derive(Debug, Error)]
pub(crate) enum HttpRangeParseError {
    #[error("Missing unit prefix")]
    MissingUnitPrefix,

    #[error("Range is invalid")]
    InvalidRange,

    #[error("Failed to parse range")]
    ParsingError(#[from] ParseIntError),

    #[error("Start of range is after end of range")]
    StartAfterEnd,

    #[error("Start of range is negative")]
    StartIsNegative,

    #[error("No overlap and no ranges found")]
    NoRanges,
}

const PREFIX: &str = "bytes=";
const PREFIX_LEN: usize = 6;

impl HttpRange {
    /// Parses Range HTTP header string as per RFC 2616.
    ///
    /// `header` is HTTP Range header (e.g. `bytes=bytes=0-9`).
    /// `size` is full size of response (file).
    pub fn parse(header: &str, size: u64) -> Result<Vec<HttpRange>, HttpRangeParseError> {
        use HttpRangeParseError::*;

        if header.is_empty() {
            return Ok(Vec::new());
        }

        if !header.starts_with(PREFIX) {
            return Err(MissingUnitPrefix)?;
        }

        let size_sig = size as i64;
        let mut no_overlap = false;

        let all_ranges: Vec<Option<HttpRange>> = header[PREFIX_LEN..]
            .split(',')
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .map(|ra| {
                let (start_str, end_str) = ra.split_once('-').ok_or(InvalidRange)?;
                let start_str = start_str.trim();
                let end_str = end_str.trim();

                if start_str.is_empty() {
                    // If no start is specified, end specifies the
                    // range start relative to the end of the file.
                    let mut length = end_str.parse::<i64>()?;

                    if length > size_sig {
                        length = size_sig;
                    }

                    Ok(Some(HttpRange {
                        start: (size_sig - length) as u64,
                        length: length as u64,
                    }))
                } else {
                    let start = start_str.parse::<i64>()?;

                    if start < 0 {
                        return Err(StartIsNegative)?;
                    }

                    if start >= size_sig {
                        no_overlap = true;
                        return Ok(None);
                    }

                    let length = if end_str.is_empty() {
                        // If no end is specified, range extends to end of the file.
                        size_sig - start
                    } else {
                        let mut end = end_str.parse::<i64>()?;

                        if start > end {
                            return Err(StartAfterEnd)?;
                        }

                        if end >= size_sig {
                            end = size_sig - 1;
                        }

                        end - start + 1
                    };

                    Ok(Some(HttpRange {
                        start: start as u64,
                        length: length as u64,
                    }))
                }
            })
            .collect::<Result<_, HttpRangeParseError>>()?;

        let ranges: Vec<HttpRange> = all_ranges.into_iter().filter_map(|x| x).collect();

        if no_overlap && ranges.is_empty() {
            return Err(NoRanges)?;
        }

        Ok(ranges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct T(&'static str, u64, Vec<HttpRange>);

    #[test]
    fn test_parse() {
        let tests = vec![
            T("", 0, vec![]),
            T("", 1000, vec![]),
            T("foo", 0, vec![]),
            T("bytes=", 0, vec![]),
            T("bytes=7", 10, vec![]),
            T("bytes= 7 ", 10, vec![]),
            T("bytes=1-", 0, vec![]),
            T("bytes=5-4", 10, vec![]),
            T("bytes=0-2,5-4", 10, vec![]),
            T("bytes=2-5,4-3", 10, vec![]),
            T("bytes=--5,4--3", 10, vec![]),
            T("bytes=A-", 10, vec![]),
            T("bytes=A- ", 10, vec![]),
            T("bytes=A-Z", 10, vec![]),
            T("bytes= -Z", 10, vec![]),
            T("bytes=5-Z", 10, vec![]),
            T("bytes=Ran-dom, garbage", 10, vec![]),
            T("bytes=0x01-0x02", 10, vec![]),
            T("bytes=         ", 10, vec![]),
            T("bytes= , , ,   ", 10, vec![]),
            T(
                "bytes=0-9",
                10,
                vec![HttpRange {
                    start: 0,
                    length: 10,
                }],
            ),
            T(
                "bytes=0-",
                10,
                vec![HttpRange {
                    start: 0,
                    length: 10,
                }],
            ),
            T(
                "bytes=5-",
                10,
                vec![HttpRange {
                    start: 5,
                    length: 5,
                }],
            ),
            T(
                "bytes=0-20",
                10,
                vec![HttpRange {
                    start: 0,
                    length: 10,
                }],
            ),
            T(
                "bytes=15-,0-5",
                10,
                vec![HttpRange {
                    start: 0,
                    length: 6,
                }],
            ),
            T(
                "bytes=1-2,5-",
                10,
                vec![
                    HttpRange {
                        start: 1,
                        length: 2,
                    },
                    HttpRange {
                        start: 5,
                        length: 5,
                    },
                ],
            ),
            T(
                "bytes=-2 , 7-",
                11,
                vec![
                    HttpRange {
                        start: 9,
                        length: 2,
                    },
                    HttpRange {
                        start: 7,
                        length: 4,
                    },
                ],
            ),
            T(
                "bytes=0-0 ,2-2, 7-",
                11,
                vec![
                    HttpRange {
                        start: 0,
                        length: 1,
                    },
                    HttpRange {
                        start: 2,
                        length: 1,
                    },
                    HttpRange {
                        start: 7,
                        length: 4,
                    },
                ],
            ),
            T(
                "bytes=-5",
                10,
                vec![HttpRange {
                    start: 5,
                    length: 5,
                }],
            ),
            T(
                "bytes=-15",
                10,
                vec![HttpRange {
                    start: 0,
                    length: 10,
                }],
            ),
            T(
                "bytes=0-499",
                10000,
                vec![HttpRange {
                    start: 0,
                    length: 500,
                }],
            ),
            T(
                "bytes=500-999",
                10000,
                vec![HttpRange {
                    start: 500,
                    length: 500,
                }],
            ),
            T(
                "bytes=-500",
                10000,
                vec![HttpRange {
                    start: 9500,
                    length: 500,
                }],
            ),
            T(
                "bytes=9500-",
                10000,
                vec![HttpRange {
                    start: 9500,
                    length: 500,
                }],
            ),
            T(
                "bytes=0-0,-1",
                10000,
                vec![
                    HttpRange {
                        start: 0,
                        length: 1,
                    },
                    HttpRange {
                        start: 9999,
                        length: 1,
                    },
                ],
            ),
            T(
                "bytes=500-600,601-999",
                10000,
                vec![
                    HttpRange {
                        start: 500,
                        length: 101,
                    },
                    HttpRange {
                        start: 601,
                        length: 399,
                    },
                ],
            ),
            T(
                "bytes=500-700,601-999",
                10000,
                vec![
                    HttpRange {
                        start: 500,
                        length: 201,
                    },
                    HttpRange {
                        start: 601,
                        length: 399,
                    },
                ],
            ),
            // Match Apache laxity:
            T(
                "bytes=   1 -2   ,  4- 5, 7 - 8 , ,,",
                11,
                vec![
                    HttpRange {
                        start: 1,
                        length: 2,
                    },
                    HttpRange {
                        start: 4,
                        length: 2,
                    },
                    HttpRange {
                        start: 7,
                        length: 2,
                    },
                ],
            ),
        ];

        for t in tests {
            let header = t.0;
            let size = t.1;
            let expected = t.2;

            let res = HttpRange::parse(header, size);

            if res.is_err() {
                if expected.is_empty() {
                    continue;
                } else {
                    assert!(
                        false,
                        "parse({}, {}) returned error {:?}",
                        header,
                        size,
                        res.unwrap_err()
                    );
                }
            }

            let got = res.unwrap();

            if got.len() != expected.len() {
                assert!(
                    false,
                    "len(parseRange({}, {})) = {}, want {}",
                    header,
                    size,
                    got.len(),
                    expected.len()
                );
                continue;
            }

            for i in 0..expected.len() {
                if got[i].start != expected[i].start {
                    assert!(
                        false,
                        "parseRange({}, {})[{}].start = {}, want {}",
                        header, size, i, got[i].start, expected[i].start
                    )
                }
                if got[i].length != expected[i].length {
                    assert!(
                        false,
                        "parseRange({}, {})[{}].length = {}, want {}",
                        header, size, i, got[i].length, expected[i].length
                    )
                }
            }
        }
    }
}
