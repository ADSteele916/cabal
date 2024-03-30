mod clique;
mod cliques;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser};
use cliques::Cliques;
use ppm_table::PpmTable;
use regex::Regex;

/// Parses an allpairs file and produces a list of cliques.
#[derive(Parser, Debug)]
#[command(version)]
struct Cmd {
    /// Path to the allpairs file.
    #[command(flatten)]
    file: InputFile,
    /// Maximum percentage to display similarities at (lower is more similar).
    #[arg(short, long, default_value_t=6, value_parser=clap::value_parser!(u32).range(0..=100))]
    max_similarity: u32,
    /// File name used in the paths in the allpairs file.
    #[arg(long = "handin-name", default_value = "handin.rkt")]
    handin_file_name: String,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct InputFile {
    allpairs_file: Option<PathBuf>,
    #[arg(long = "ppm-table")]
    ppm_table_file: Option<PathBuf>,
}

impl InputFile {
    fn ppm_table(&self) -> Result<PpmTable> {
        if let Some(allpairs_file) = &self.allpairs_file {
            let contents = fs::read_to_string(allpairs_file)?;
            Ok(allpairs::load(contents)?)
        } else {
            // Clap guarantees that one of the fields will not be `None`.
            let ppm_table_file = self.ppm_table_file.clone().unwrap();
            Ok(postcard::from_bytes(&fs::read(ppm_table_file)?)?)
        }
    }
}

fn main() -> Result<()> {
    let args = Cmd::parse();

    let ppm_limit = args.max_similarity * 10000;

    let regex_string = format!(r"^[^/]+/(.+)/{}", args.handin_file_name);
    let id_from_path = Regex::new(&regex_string).unwrap();
    let mut files_to_ids = HashMap::new();

    let ppm_table = args.file.ppm_table()?;
    let sorted_ppm_table_edges = {
        let mut edges = ppm_table
            .edges()
            .filter(|e| e.2 <= ppm_limit)
            .collect::<Vec<_>>();
        edges.sort_by_key(|e| e.2);
        edges
    };

    let mut max_ppm = 0;
    let mut prev_cliques = Cliques::new(max_ppm);
    let mut cliques = Cliques::new(max_ppm);
    for (l, r, ppm) in sorted_ppm_table_edges {
        let l_id = files_to_ids
            .entry(l)
            .or_insert_with(|| id_from_path.captures(l).unwrap().get(1).unwrap())
            .as_str();
        let r_id = files_to_ids
            .entry(r)
            .or_insert_with(|| id_from_path.captures(r).unwrap().get(1).unwrap())
            .as_str();

        while ppm > max_ppm {
            println!("At {}%", max_ppm / 10000);
            println!("{}", cliques.export(&prev_cliques));
            prev_cliques = cliques.clone();
            max_ppm += 10000;
        }
        cliques.add(l_id, r_id, ppm)
    }
    println!("At {}%", max_ppm / 10000);
    println!("{}", cliques.export(&prev_cliques));

    Ok(())
}
