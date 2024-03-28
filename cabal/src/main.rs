mod clique;
mod cliques;

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use cliques::Cliques;
use regex::Regex;

/// Parses an allpairs file and produces a list of cliques.
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Path to the allpairs file.
    file: PathBuf,
    /// Maximum percentage to display similarities at (lower is more similar).
    #[arg(short, long, default_value_t=6, value_parser=clap::value_parser!(u32).range(0..=100))]
    max_similarity: u32,
    /// File name used in the paths in the allpairs file.
    #[arg(long = "handin-name", default_value = "handin.rkt")]
    handin_file_name: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let contents = fs::read_to_string(args.file)?;
    let ppm_limit = args.max_similarity * 10000;
    let regex_string = format!(r"^[^/]+/(.+)/{}", args.handin_file_name);

    let id_from_path = Regex::new(&regex_string).unwrap();
    let ppm_table = allpairs::load_with_hasher::<ahash::RandomState>(contents, id_from_path)?;

    let mut cliques = Cliques::new(0);
    for max_ppm in (0..=ppm_limit).step_by(10000) {
        let prev_cliques = cliques;
        cliques = Cliques::new(max_ppm);

        for (l, r, ppm) in ppm_table.edges() {
            if ppm <= max_ppm {
                cliques.add(l, r, ppm);
            }
        }

        println!("At {}%", max_ppm / 10000);
        println!("{}", cliques.export(&prev_cliques));
    }

    Ok(())
}
