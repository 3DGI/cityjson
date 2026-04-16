use std::env;
use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    let args = parse_args(&raw_args);
    let input = fs::read(&args.input)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", args.input.display()));
    let model =
        cityjson_lib::json::read_model(&input, &cityjson_lib::json::JsonReadOptions::default())
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", args.input.display()));

    if let Some(path) = args.arrow_file.as_ref() {
        reset_output_path(path);
        let file = fs::File::create(path)
            .unwrap_or_else(|error| panic!("failed to create {}: {error}", path.display()));
        let mut writer = BufWriter::new(file);
        cityjson_lib::arrow::write_stream(
            &mut writer,
            &model,
            &cityjson_lib::arrow::ExportOptions::default(),
        )
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
        println!("wrote {}", path.display());
    }
}

#[derive(Debug)]
struct Args {
    input: PathBuf,
    arrow_file: Option<PathBuf>,
}

fn parse_args(args: &[String]) -> Args {
    let mut input = None;
    let mut arrow_file = None;

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
        arrow_file.is_some(),
        "missing required --arrow-file argument"
    );

    Args { input, arrow_file }
}

fn value<'a>(args: &'a [String], index: usize, flag: &str) -> &'a str {
    args.get(index)
        .map_or_else(|| panic!("missing value for {flag}"), String::as_str)
}

fn print_usage() {
    println!("Usage:");
    println!("  cargo run --bin cjexport -- --input <cityjson> --arrow-file <path>");
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
