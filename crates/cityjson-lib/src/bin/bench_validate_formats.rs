use std::env;
use std::path::PathBuf;

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    let args = parse_args(&raw_args);

    for path in &args.arrow_files {
        cityjson_lib::arrow::from_file(path)
            .unwrap_or_else(|error| panic!("failed to validate {}: {error}", path.display()));
        println!("validated {}", path.display());
    }

    for path in &args.parquet_files {
        cityjson_lib::parquet::from_file(path)
            .unwrap_or_else(|error| panic!("failed to validate {}: {error}", path.display()));
        println!("validated {}", path.display());
    }
}

#[derive(Debug)]
struct Args {
    arrow_files: Vec<PathBuf>,
    parquet_files: Vec<PathBuf>,
}

fn parse_args(args: &[String]) -> Args {
    let mut arrow_files = Vec::new();
    let mut parquet_files = Vec::new();

    let mut index = 0_usize;
    while index < args.len() {
        match args[index].as_str() {
            "--arrow-file" => {
                index += 1;
                arrow_files.push(PathBuf::from(value(args, index, "--arrow-file")));
            }
            "--parquet-file" => {
                index += 1;
                parquet_files.push(PathBuf::from(value(args, index, "--parquet-file")));
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => panic!("unknown argument '{other}'"),
        }
        index += 1;
    }

    assert!(
        !arrow_files.is_empty() || !parquet_files.is_empty(),
        "at least one of --arrow-file or --parquet-file is required"
    );

    Args {
        arrow_files,
        parquet_files,
    }
}

fn value<'a>(args: &'a [String], index: usize, flag: &str) -> &'a str {
    args.get(index)
        .map_or_else(|| panic!("missing value for {flag}"), String::as_str)
}

fn print_usage() {
    println!("Usage:");
    println!(
        "  cargo run --bin bench_validate_formats -- [--arrow-file <path>] [--parquet-file <path>]"
    );
}
