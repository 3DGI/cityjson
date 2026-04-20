use std::env;
use std::path::PathBuf;

use cityjson_lib::Result;

#[path = "../../tests/common/data_prep.rs"]
#[allow(dead_code)]
mod data_prep;

use data_prep::{DEFAULT_OUTPUT_ROOT, PrepConfig, prepare_3dbag_benchmark_datasets};

fn main() -> Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if !args.is_empty() && args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_usage();
        return Ok(());
    }

    let options = PrepOptions::from_args(&args);
    let config = PrepConfig {
        output_root: options.output,
        validate_cjval: options.validate_cjval,
        max_tiles: options.max_tiles,
    };
    prepare_3dbag_benchmark_datasets(&config)?;

    Ok(())
}

struct PrepOptions {
    output: PathBuf,
    validate_cjval: bool,
    max_tiles: Option<usize>,
}

impl PrepOptions {
    fn from_args(args: &[std::ffi::OsString]) -> Self {
        let mut output = PathBuf::from(DEFAULT_OUTPUT_ROOT);
        let mut validate_cjval = true;
        let mut max_tiles = None;
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            match arg.to_string_lossy().as_ref() {
                "--output" => {
                    if let Some(value) = iter.next() {
                        output = PathBuf::from(value);
                    }
                }
                "--skip-cjval" => {
                    validate_cjval = false;
                }
                "--max-tiles" => {
                    if let Some(value) = iter.next() {
                        max_tiles = value.to_string_lossy().parse::<usize>().ok();
                    }
                }
                _ => {}
            }
        }

        Self {
            output,
            validate_cjval,
            max_tiles,
        }
    }
}

fn print_usage() {
    eprintln!("usage: prep-test-data [--output PATH] [--skip-cjval] [--max-tiles N]");
}
