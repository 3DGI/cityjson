use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
enum Action {
    Help,
    Run(Args),
}

#[derive(Debug, PartialEq, Eq)]
struct Args {
    input: PathBuf,
    arrow_file: PathBuf,
}

pub fn run<I>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = String>,
{
    match parse_args(args.into_iter().skip(1).collect::<Vec<_>>().as_slice())? {
        Action::Help => {
            print_usage();
            Ok(())
        }
        Action::Run(args) => run_inner(args),
    }
}

fn run_inner(args: Args) -> Result<(), String> {
    let input = fs::read(&args.input)
        .map_err(|error| format!("failed to read {}: {error}", args.input.display()))?;
    let model =
        cityjson_lib::json::read_model(&input, &cityjson_lib::json::JsonReadOptions::default())
            .map_err(|error| format!("failed to read {}: {error}", args.input.display()))?;

    reset_output_path(&args.arrow_file)?;
    let file = fs::File::create(&args.arrow_file)
        .map_err(|error| format!("failed to create {}: {error}", args.arrow_file.display()))?;
    let mut writer = BufWriter::new(file);
    cityjson_lib::arrow::write_stream(
        &mut writer,
        &model,
        &cityjson_lib::arrow::ExportOptions::default(),
    )
    .map_err(|error| format!("failed to write {}: {error}", args.arrow_file.display()))?;
    println!("wrote {}", args.arrow_file.display());
    Ok(())
}

fn parse_args(args: &[String]) -> Result<Action, String> {
    let mut input = None;
    let mut arrow_file = None;

    let mut index = 0_usize;
    while index < args.len() {
        match args[index].as_str() {
            "--input" => {
                index += 1;
                input = Some(PathBuf::from(value(args, index, "--input")?));
            }
            "--arrow-file" => {
                index += 1;
                arrow_file = Some(PathBuf::from(value(args, index, "--arrow-file")?));
            }
            "--help" | "-h" => return Ok(Action::Help),
            other => return Err(format!("unknown argument '{other}'")),
        }
        index += 1;
    }

    let input = input.ok_or_else(|| "missing required --input argument".to_string())?;
    let arrow_file =
        arrow_file.ok_or_else(|| "missing required --arrow-file argument".to_string())?;

    Ok(Action::Run(Args { input, arrow_file }))
}

fn value<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, String> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn print_usage() {
    println!("Usage:");
    println!(
        "  cargo run -p cityjson-export --bin cjexport -- --input <cityjson> --arrow-file <path>"
    );
}

fn reset_output_path(path: &Path) -> Result<(), String> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.is_dir() {
            fs::remove_dir_all(path)
                .map_err(|error| format!("failed to remove {}: {error}", path.display()))?;
        } else {
            fs::remove_file(path)
                .map_err(|error| format!("failed to remove {}: {error}", path.display()))?;
        }
    }

    fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new(".")))
        .map_err(|error| format!("failed to create parent for {}: {error}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Action, parse_args, run};

    #[test]
    fn help_is_reported_explicitly() {
        let args = vec!["--help".to_string()];
        assert_eq!(parse_args(&args).unwrap(), Action::Help);
    }

    #[test]
    fn required_arguments_are_enforced() {
        let error =
            parse_args(&["--input".to_string(), "input.city.json".to_string()]).unwrap_err();
        assert_eq!(error, "missing required --arrow-file argument");
    }

    #[test]
    fn run_writes_a_stream_file() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let input = tempdir.path().join("input.city.json");
        let output = tempdir.path().join("output.cjarrow");

        std::fs::write(
            &input,
            br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#,
        )
        .expect("input fixture");

        run(vec![
            "cjexport".to_string(),
            "--input".to_string(),
            input.display().to_string(),
            "--arrow-file".to_string(),
            output.display().to_string(),
        ])
        .expect("run should succeed");

        assert!(output.is_file());
    }
}
