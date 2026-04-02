use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    let args = parse_args(&raw_args);
    let model = cjlib::CityModel::from_file(&args.input)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", args.input.display()));

    if let Some(path) = args.arrow_file.as_ref() {
        reset_output_path(path);
        cjlib::arrow::to_file(path, &model)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
        println!("wrote {}", path.display());
    }

    if let Some(path) = args.parquet_file.as_ref() {
        reset_output_path(path);
        cjlib::parquet::to_file(path, &model)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
        println!("wrote {}", path.display());
    }
}

#[derive(Debug)]
struct Args {
    input: PathBuf,
    arrow_file: Option<PathBuf>,
    parquet_file: Option<PathBuf>,
}

fn parse_args(args: &[String]) -> Args {
    let mut input = None;
    let mut arrow_file = None;
    let mut parquet_file = None;

    let mut index = 0_usize;
    while index < args.len() {
        match args[index].as_str() {
            "--input" => {
                index += 1;
                input = Some(PathBuf::from(value(args, index, "--input")));
            }
            "--arrow-file" => {
                index += 1;
                arrow_file = Some(PathBuf::from(value(args, index, "--arrow-file")));
            }
            "--parquet-file" => {
                index += 1;
                parquet_file = Some(PathBuf::from(value(args, index, "--parquet-file")));
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => panic!("unknown argument '{other}'"),
        }
        index += 1;
    }

    let input = input.unwrap_or_else(|| panic!("missing required --input argument"));
    assert!(
        arrow_file.is_some() || parquet_file.is_some(),
        "at least one of --arrow-file or --parquet-file is required"
    );

    Args {
        input,
        arrow_file,
        parquet_file,
    }
}

fn value<'a>(args: &'a [String], index: usize, flag: &str) -> &'a str {
    args.get(index)
        .map_or_else(|| panic!("missing value for {flag}"), String::as_str)
}

fn print_usage() {
    println!("Usage:");
    println!(
        "  cargo run --bin bench_export_formats -- --input <cityjson> [--arrow-file <path>] [--parquet-file <path>]"
    );
}

fn reset_output_path(path: &Path) {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.is_dir() {
            fs::remove_dir_all(path)
                .unwrap_or_else(|error| panic!("failed to remove {}: {error}", path.display()));
        } else {
            fs::remove_file(path)
                .unwrap_or_else(|error| panic!("failed to remove {}: {error}", path.display()));
        }
    }

    fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new(".")))
        .unwrap_or_else(|error| panic!("failed to create parent for {}: {error}", path.display()));
}
