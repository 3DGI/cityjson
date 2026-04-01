use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args = parse_args(env::args().skip(1).collect());
    let model = cjlib::CityModel::from_file(&args.input)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", args.input.display()));

    if let Some(path) = args.arrow_dir.as_ref() {
        reset_output_dir(path);
        cjlib::arrow::write_package_dir(path, &model)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
        println!("wrote {}", path.display());
    }

    if let Some(path) = args.parquet_dir.as_ref() {
        reset_output_dir(path);
        cjlib::parquet::write_package_dir(path, &model)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
        println!("wrote {}", path.display());
    }
}

#[derive(Debug)]
struct Args {
    input: PathBuf,
    arrow_dir: Option<PathBuf>,
    parquet_dir: Option<PathBuf>,
}

fn parse_args(args: Vec<String>) -> Args {
    let mut input = None;
    let mut arrow_dir = None;
    let mut parquet_dir = None;

    let mut index = 0_usize;
    while index < args.len() {
        match args[index].as_str() {
            "--input" => {
                index += 1;
                input = Some(PathBuf::from(value(&args, index, "--input")));
            }
            "--arrow-dir" => {
                index += 1;
                arrow_dir = Some(PathBuf::from(value(&args, index, "--arrow-dir")));
            }
            "--parquet-dir" => {
                index += 1;
                parquet_dir = Some(PathBuf::from(value(&args, index, "--parquet-dir")));
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
        arrow_dir.is_some() || parquet_dir.is_some(),
        "at least one of --arrow-dir or --parquet-dir is required"
    );

    Args {
        input,
        arrow_dir,
        parquet_dir,
    }
}

fn value<'a>(args: &'a [String], index: usize, flag: &str) -> &'a str {
    args.get(index)
        .map(String::as_str)
        .unwrap_or_else(|| panic!("missing value for {flag}"))
}

fn print_usage() {
    println!("Usage:");
    println!(
        "  cargo run --bin bench_export_formats -- --input <cityjson> [--arrow-dir <dir>] [--parquet-dir <dir>]"
    );
}

fn reset_output_dir(path: &PathBuf) {
    if path.exists() {
        fs::remove_dir_all(path)
            .unwrap_or_else(|error| panic!("failed to remove {}: {error}", path.display()));
    }
    fs::create_dir_all(path.parent().unwrap_or_else(|| std::path::Path::new(".")))
        .unwrap_or_else(|error| panic!("failed to create parent for {}: {error}", path.display()));
}
