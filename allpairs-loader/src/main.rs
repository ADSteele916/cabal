use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

/// Parses an allpairs file into a PPM table and save the table to disk.
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Path to the allpairs file.
    in_file: PathBuf,
    /// Path for the outputted PPM table file.
    out_file: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let contents = fs::read_to_string(args.in_file)?;

    let ppm_table = allpairs::load(contents)?;

    let out = postcard::to_stdvec(&ppm_table)?;

    let mut file = BufWriter::new(File::create(args.out_file.clone())?);
    file.write_all(&out)?;

    Ok(())
}
