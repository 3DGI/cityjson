use std::env;
use std::path::PathBuf;

use cjlib::Result;

#[path = "../../tests/common/data_prep.rs"]
#[allow(dead_code)]
mod data_prep;

use data_prep::{DEFAULT_OUTPUT_ROOT, prepare_3dbag_benchmark_datasets};

fn main() -> Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if !args.is_empty() && args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_usage();
        return Ok(());
    }

    let options = PrepOptions::from_args(&args);
    prepare_3dbag_benchmark_datasets(&options.output)?;

    Ok(())
}

struct PrepOptions {
    output: PathBuf,
}

impl PrepOptions {
    fn from_args(args: &[std::ffi::OsString]) -> Self {
        let mut output = PathBuf::from(DEFAULT_OUTPUT_ROOT);
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            if arg.to_string_lossy().as_ref() == "--output"
                && let Some(value) = iter.next()
            {
                output = PathBuf::from(value);
            }
        }

        Self { output }
    }
}

fn print_usage() {
    eprintln!("usage: prep-test-data [--output PATH]");
}
