use std::hash::{BuildHasher, RandomState};

use ppm_table::{PpmTable, PpmTableBuilder};
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

pub fn load(file_contents: String) -> Result<PpmTable<RandomState>, LoadAllpairsError> {
    load_with_hasher::<RandomState>(file_contents)
}

pub fn load_with_hasher<S: BuildHasher + Default>(
    file_contents: String,
) -> Result<PpmTable<S>, LoadAllpairsError> {
    let mut ppm_table_builder = PpmTableBuilder::<S>::new();

    for edge in file_contents.lines().map(parse_line) {
        match edge {
            Ok((ppm, l, r)) => ppm_table_builder.add_ppm(l, r, ppm),
            Err(e) => return Err(e),
        }
    }

    ppm_table_builder
        .build()
        .map_err(|_| LoadAllpairsError::IncompleteGraph)
}

fn parse_line(line: &str) -> Result<(u32, String, String), LoadAllpairsError> {
    let generate_error = || LoadAllpairsError::InvalidLine(line.to_string());

    let mut columns = line.split_whitespace();

    let ppm_str = columns.next().ok_or_else(generate_error)?;
    let _edit_distance = columns.next().ok_or_else(generate_error)?;
    let _l_len = columns.next().ok_or_else(generate_error)?;
    let _r_len = columns.next().ok_or_else(generate_error)?;
    let l = columns.next().ok_or_else(generate_error)?;
    let r = columns.next().ok_or_else(generate_error)?;
    if columns.next().is_some() {
        return Err(LoadAllpairsError::InvalidLine(line.to_string()));
    }

    let ppm = ppm_str
        .parse()
        .map_err(|_| LoadAllpairsError::PpmCaptureFail(ppm_str.to_string()))?;

    Ok((ppm, l.to_string(), r.to_string()))
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
            "  2191     23   5260   abcda2-anonymous/003/a2.py a2-anonymous/002/a2.py\n",
        )
        .to_string();
        let err = load(file_contents).expect_err("Line 2 should be malformed.");
        assert_eq!(
            err,
            LoadAllpairsError::InvalidLine(
                "  2191     23   5260   abcda2-anonymous/003/a2.py a2-anonymous/002/a2.py"
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
