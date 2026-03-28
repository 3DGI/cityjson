use std::env;
use std::path::PathBuf;

use cjlib::{Error, Result};

#[path = "../../tests/common/data_prep.rs"]
mod data_prep;

use data_prep::{
    DEFAULT_INPUT_ROOT, DEFAULT_OUTPUT_ROOT, prepare_cityjson_only, prepare_feature_files_only,
    prepare_ndjson_only, prepare_test_sets,
};

fn main() -> Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        print_usage();
        return Ok(());
    }

    let options = PrepOptions::from_args(&args);
    match options.layout.as_deref() {
        None | Some("all") => {
            prepare_test_sets(&options.input, &options.output)?;
        }
        Some("feature-files") => {
            prepare_feature_files_only(&options.input, &options.output.join("feature-files"))?;
        }
        Some("cityjson") => {
            prepare_cityjson_only(&options.input, &options.output.join("cityjson"))?;
        }
        Some("ndjson") => {
            prepare_ndjson_only(&options.input, &options.output)?;
        }
        Some(layout) => {
            return Err(Error::Import(format!(
                "unknown layout '{layout}' (expected: all|feature-files|cityjson|ndjson)"
            )));
        }
    }

    Ok(())
}

struct PrepOptions {
    input: PathBuf,
    output: PathBuf,
    layout: Option<String>,
}

impl PrepOptions {
    fn from_args(args: &[std::ffi::OsString]) -> Self {
        let mut input = PathBuf::from(DEFAULT_INPUT_ROOT);
        let mut output = PathBuf::from(DEFAULT_OUTPUT_ROOT);
        let mut layout = None;
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            match arg.to_string_lossy().as_ref() {
                "--input" => {
                    if let Some(value) = iter.next() {
                        input = PathBuf::from(value);
                    }
                }
                "--output" => {
                    if let Some(value) = iter.next() {
                        output = PathBuf::from(value);
                    }
                }
                "--layout" => {
                    if let Some(value) = iter.next() {
                        layout = Some(value.to_string_lossy().to_string());
                    }
                }
                _ => {}
            }
        }

        Self {
            input,
            output,
            layout,
        }
    }
}

fn print_usage() {
    eprintln!(
        "usage: prep-test-data [--input PATH] [--output PATH] [--layout all|feature-files|cityjson|ndjson]"
    );
}
