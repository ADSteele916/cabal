use std::hash::{BuildHasher, RandomState};

use ppm_table::{PpmTable, PpmTableBuilder};
use regex::Regex;
use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum LoadAllpairsError {
    #[error("A line in the file was not a valid allpairs entry.")]
    InvalidLine(String),
    #[error("The PPM in the file was missing or invalid.")]
    PpmCaptureFail(String),
    #[error("The provided allpairs file does not correspond to a complete similarity graph.")]
    IncompleteGraph,
}

const ALLPAIRS_LINE_REGEX: &str = r"^ *(?<ppm>\d+) +(?<edit_distance>\d+) +(?<l_len>\d+) +(?<r_len>\d+) +(?<l_path>.+) +(?<r_path>.+)$";

pub fn load(file_contents: String) -> Result<PpmTable<RandomState>, LoadAllpairsError> {
    load_with_hasher::<RandomState>(file_contents)
}

pub fn load_with_hasher<S: BuildHasher + Default>(
    file_contents: String,
) -> Result<PpmTable<S>, LoadAllpairsError> {
    let allpairs_regex = Regex::new(ALLPAIRS_LINE_REGEX).unwrap();
    let mut ppm_table_builder = PpmTableBuilder::<S>::new();

    // `captures_iter` would skip malformed lines.
    for line in file_contents.lines() {
        let (ppm, l, r) = parse_line(line, &allpairs_regex)?;
        ppm_table_builder.add_ppm(l, r, ppm);
    }

    ppm_table_builder
        .build()
        .map_err(|_| LoadAllpairsError::IncompleteGraph)
}

fn parse_line(
    line: &str,
    allpairs_regex: &Regex,
) -> Result<(u32, String, String), LoadAllpairsError> {
    let Some(captures) = allpairs_regex.captures(line) else {
        return Err(LoadAllpairsError::InvalidLine(line.to_string()));
    };

    let ppm = parse_ppm(&captures["ppm"])?;
    let l = parse_id(&captures["l_path"])?;
    let r = parse_id(&captures["r_path"])?;

    Ok((ppm, l, r))
}

fn parse_ppm(ppm_capture: &str) -> Result<u32, LoadAllpairsError> {
    ppm_capture
        .parse()
        .map_err(|_| LoadAllpairsError::PpmCaptureFail(ppm_capture.to_string()))
}

fn parse_id(path_capture: &str) -> Result<String, LoadAllpairsError> {
    Ok(path_capture.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_allpairs_one_pair() {
        let ppm_table = load(
            "  2191     23   5260   5236 a2-anonymous/001/a2.py a2-anonymous/002/a2.py\n"
                .to_string(),
        )
        .expect("File should be valid.");
        assert_eq!(
            ppm_table[("a2-anonymous/001/a2.py", "a2-anonymous/002/a2.py")],
            2191
        );
        assert_eq!(
            ppm_table[("a2-anonymous/002/a2.py", "a2-anonymous/001/a2.py")],
            2191
        );
    }

    #[test]
    fn test_load_allpairs_three_pairs() {
        let file_contents = concat!(
            "  2191     23   5260   5236 a2-anonymous/001/a2.py a2-anonymous/002/a2.py\n",
            "  2155     49   5260   5000 a2-anonymous/001/a2.py a2-anonymous/003/a2.py\n",
            "  2232     12   5236   5000 a2-anonymous/002/a2.py a2-anonymous/003/a2.py\n",
        )
        .to_string();
        let ppm_table = load(file_contents).expect("File should be valid.");
        assert_eq!(
            ppm_table[("a2-anonymous/001/a2.py", "a2-anonymous/002/a2.py")],
            2191
        );
        assert_eq!(
            ppm_table[("a2-anonymous/001/a2.py", "a2-anonymous/003/a2.py")],
            2155
        );
        assert_eq!(
            ppm_table[("a2-anonymous/002/a2.py", "a2-anonymous/003/a2.py")],
            2232
        );
    }

    #[test]
    fn test_load_allpairs_invalid_line() {
        let file_contents = concat!(
            "  2191     23   5260   5236 a2-anonymous/001/a2.py a2-anonymous/002/a2.py\n",
            "  2191     23   5260   abcd a2-anonymous/003/a2.py a2-anonymous/002/a2.py\n",
        )
        .to_string();
        let err = load(file_contents).expect_err("Line 2 should be malformed.");
        assert_eq!(
            err,
            LoadAllpairsError::InvalidLine(
                "  2191     23   5260   abcd a2-anonymous/003/a2.py a2-anonymous/002/a2.py"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_load_allpairs_ppm_parse_error() {
        let usize_max_plus_one = "18446744073709551616";
        let file_contents = format!(
            "{}     23   5260   5236 a2-anonymous/001/a2.py a2-anonymous/002/a2.py\n",
            usize_max_plus_one,
        );
        let err = load(file_contents).expect_err("Parsing of overly long usize should fail.");
        assert_eq!(
            err,
            LoadAllpairsError::PpmCaptureFail(usize_max_plus_one.to_string())
        );
    }

    #[test]
    fn test_load_allpairs_incomplete_graph() {
        let file_contents = concat!(
            "  2191     23   5260   5236 a2-anonymous/001/a2.py a2-anonymous/002/a2.py\n",
            "  2191     23   5260   5236 a2-anonymous/003/a2.py a2-anonymous/002/a2.py\n",
        )
        .to_string();
        let err = load(file_contents).expect_err("Parsing of incomplete graph should fail.");
        assert_eq!(err, LoadAllpairsError::IncompleteGraph);
    }
}
