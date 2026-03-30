use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;

use cjindex::{
    BBox, CityIndex, DatasetInspection, StorageLayout, ValidationReport, resolve_dataset,
};
use cjlib::{CityModel, Error, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::{Map, Value};

#[derive(Debug, Parser)]
#[command(
    name = "cjindex",
    version,
    about = "Query CityJSON datasets through a persistent index"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Index(IndexCommand),
    Reindex(IndexCommand),
    Get(FeatureCommand),
    Query(QueryCommand),
    Metadata(IndexCommand),
    Inspect(StatusCommand),
    Validate(StatusCommand),
}

#[derive(Debug, Args)]
struct IndexCommand {
    #[command(flatten)]
    input: DatasetInputArgs,
}

#[derive(Debug, Args)]
struct FeatureCommand {
    #[command(flatten)]
    input: DatasetInputArgs,

    #[arg(long)]
    id: String,

    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct QueryCommand {
    #[command(flatten)]
    input: DatasetInputArgs,

    #[arg(long)]
    min_x: f64,

    #[arg(long)]
    max_x: f64,

    #[arg(long)]
    min_y: f64,

    #[arg(long)]
    max_y: f64,

    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct StatusCommand {
    #[arg(value_name = "DATASET_DIR")]
    dataset_dir: PathBuf,

    #[arg(long)]
    index: Option<PathBuf>,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args, Clone)]
struct DatasetInputArgs {
    #[arg(value_name = "DATASET_DIR")]
    dataset_dir: Option<PathBuf>,

    #[command(flatten)]
    storage: StorageArgs,

    #[arg(long)]
    index: Option<PathBuf>,
}

#[derive(Debug, Args, Clone)]
struct StorageArgs {
    #[arg(long, value_enum)]
    layout: Option<LayoutKind>,

    #[arg(long, value_name = "PATH", num_args = 1..)]
    paths: Vec<PathBuf>,

    #[arg(long)]
    root: Option<PathBuf>,

    #[arg(long, default_value = "**/metadata.json")]
    metadata_glob: String,

    #[arg(long, default_value = "**/*.city.jsonl")]
    feature_glob: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum LayoutKind {
    Ndjson,
    Cityjson,
    FeatureFiles,
}

impl StorageArgs {
    fn into_layout(self) -> Result<StorageLayout> {
        let layout = self
            .layout
            .ok_or_else(|| Error::Import("explicit layout mode requires --layout".to_owned()))?;
        match layout {
            LayoutKind::Ndjson => Ok(StorageLayout::Ndjson { paths: self.paths }),
            LayoutKind::Cityjson => Ok(StorageLayout::CityJson { paths: self.paths }),
            LayoutKind::FeatureFiles => {
                let root = self.root.ok_or_else(|| {
                    Error::Import("feature-files layout requires --root".to_string())
                })?;
                Ok(StorageLayout::FeatureFiles {
                    root,
                    metadata_glob: self.metadata_glob,
                    feature_glob: self.feature_glob,
                })
            }
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Index(args) | Command::Reindex(args) => run_reindex(args),
        Command::Get(args) => run_get(args),
        Command::Query(args) => run_query(args),
        Command::Metadata(args) => run_metadata(args),
        Command::Inspect(args) => run_inspect(args),
        Command::Validate(args) => run_validate(args),
    }
}

fn run_reindex(args: IndexCommand) -> Result<()> {
    let (storage, index_path) = resolve_operational_input(args.input)?;
    let mut index = CityIndex::open(storage, &index_path)?;
    index.reindex()
}

fn run_get(args: FeatureCommand) -> Result<()> {
    let (storage, index_path) = resolve_operational_input(args.input)?;
    let index = CityIndex::open(storage, &index_path)?;
    let (metadata, model) = index
        .get_with_metadata(&args.id)?
        .ok_or_else(|| Error::Import(format!("feature {} was not found", args.id)))?;
    let writer = open_writer(args.output)?;
    let mut writer = BufWriter::new(writer);
    write_model_stream(&mut writer, metadata.as_ref(), &model)?;
    writer.flush()?;
    Ok(())
}

fn run_query(args: QueryCommand) -> Result<()> {
    let (storage, index_path) = resolve_operational_input(args.input)?;
    let index = CityIndex::open(storage, &index_path)?;
    let bbox = BBox {
        min_x: args.min_x,
        max_x: args.max_x,
        min_y: args.min_y,
        max_y: args.max_y,
    };
    let writer = open_writer(args.output)?;
    let mut writer = BufWriter::new(writer);
    let mut results = index.query_iter_with_metadata(&bbox)?;
    write_query_stream(&mut writer, &mut results)?;
    writer.flush()?;
    Ok(())
}

fn run_metadata(args: IndexCommand) -> Result<()> {
    let (storage, index_path) = resolve_operational_input(args.input)?;
    let index = CityIndex::open(storage, &index_path)?;
    let metadata = index.metadata()?;
    let borrowed_metadata = metadata
        .iter()
        .map(std::convert::AsRef::as_ref)
        .collect::<Vec<_>>();
    let mut writer = BufWriter::new(io::stdout());
    serde_json::to_writer(&mut writer, &borrowed_metadata)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn run_inspect(args: StatusCommand) -> Result<()> {
    let resolved = resolve_dataset(&args.dataset_dir, args.index)?;
    let inspection = resolved.inspect()?;
    if args.json {
        print_json(&inspection)?;
    } else {
        print_dataset_inspection(&inspection)?;
    }
    Ok(())
}

fn run_validate(args: StatusCommand) -> Result<()> {
    let resolved = resolve_dataset(&args.dataset_dir, args.index)?;
    let report = resolved.validate()?;
    if args.json {
        print_json(&report)?;
    } else {
        print_validation_report(&report)?;
    }
    if report.ok {
        Ok(())
    } else {
        Err(Error::Import(report.inspection.index.issues.join("; ")))
    }
}

fn resolve_operational_input(args: DatasetInputArgs) -> Result<(StorageLayout, PathBuf)> {
    if args.dataset_dir.is_some() && args.storage.layout.is_some() {
        return Err(Error::Import(
            "specify either DATASET_DIR or explicit layout flags, not both".to_owned(),
        ));
    }

    if let Some(dataset_dir) = args.dataset_dir {
        let resolved = resolve_dataset(&dataset_dir, args.index)?;
        return Ok((resolved.storage_layout(), resolved.index_path));
    }

    let storage = args.storage.into_layout()?;
    let index_path = args
        .index
        .ok_or_else(|| Error::Import("explicit layout mode requires --index".to_owned()))?;
    Ok((storage, index_path))
}

fn print_dataset_inspection(report: &DatasetInspection) -> Result<()> {
    let mut writer = BufWriter::new(io::stdout());
    writeln!(writer, "dataset: {}", report.dataset_root.display())?;
    writeln!(writer, "layout: {}", report.layout.as_str())?;
    writeln!(writer, "index: {}", report.index.path.display())?;
    writeln!(
        writer,
        "index status: {}",
        if report.index.exists {
            "present"
        } else {
            "missing"
        }
    )?;
    if let Some(manifest) = &report.manifest {
        writeln!(writer, "manifest: {}", manifest.path.display())?;
    } else {
        writeln!(writer, "manifest: none")?;
    }
    writeln!(writer, "detected sources: {}", report.detected_source_count)?;
    if report.detected_feature_file_count > 0 {
        writeln!(
            writer,
            "detected feature files: {}",
            report.detected_feature_file_count
        )?;
    }
    if let Some(indexed_source_count) = report.index.indexed_source_count {
        writeln!(writer, "indexed sources: {indexed_source_count}")?;
    }
    if let Some(indexed_feature_count) = report.index.indexed_feature_count {
        writeln!(writer, "indexed feature packages: {indexed_feature_count}")?;
    }
    if let Some(indexed_cityobject_count) = report.index.indexed_cityobject_count {
        writeln!(writer, "indexed CityObjects: {indexed_cityobject_count}")?;
    }
    writeln!(
        writer,
        "freshness: {}",
        option_status(report.index.fresh, "fresh", "stale")
    )?;
    writeln!(
        writer,
        "coverage: {}",
        option_status(report.index.covered, "covered", "uncovered")
    )?;
    writeln!(
        writer,
        "needs reindex: {}",
        if report.index.needs_reindex {
            "yes"
        } else {
            "no"
        }
    )?;
    if !report.index.issues.is_empty() {
        writeln!(writer, "issues:")?;
        for issue in &report.index.issues {
            writeln!(writer, "- {issue}")?;
        }
    }
    writer.flush()?;
    Ok(())
}

fn print_validation_report(report: &ValidationReport) -> Result<()> {
    print_dataset_inspection(&report.inspection)?;
    let mut writer = BufWriter::new(io::stdout());
    writeln!(
        writer,
        "validation: {}",
        if report.ok { "ok" } else { "failed" }
    )?;
    writer.flush()?;
    Ok(())
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    let mut writer = BufWriter::new(io::stdout());
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn option_status<'a>(value: Option<bool>, yes: &'a str, no: &'a str) -> &'a str {
    match value {
        Some(true) => yes,
        Some(false) => no,
        None => "unknown",
    }
}

fn open_writer(output: Option<PathBuf>) -> Result<Box<dyn Write>> {
    match output {
        Some(path) => Ok(Box::new(File::create(path)?)),
        None => Ok(Box::new(io::stdout())),
    }
}

fn write_query_stream<W, I>(writer: &mut W, results: &mut I) -> Result<()>
where
    W: Write,
    I: Iterator<Item = Result<(Arc<Value>, CityModel)>>,
{
    let Some(first_result) = results.next() else {
        return Ok(());
    };
    let (current_metadata, first_model) = first_result?;
    write_model_stream_item(writer, current_metadata.as_ref(), &first_model)?;

    for result in results {
        let (metadata, model) = result?;
        if current_metadata.as_ref() != metadata.as_ref() {
            return Err(Error::Import(
                "query results span incompatible metadata roots".to_string(),
            ));
        }
        write_feature_line(writer, &model)?;
    }

    Ok(())
}

fn write_model_stream<W>(writer: &mut W, metadata: &Value, model: &CityModel) -> Result<()>
where
    W: Write,
{
    write_model_stream_item(writer, metadata, model)
}

fn write_model_stream_item<W>(writer: &mut W, metadata: &Value, model: &CityModel) -> Result<()>
where
    W: Write,
{
    write_model_stream_header(writer, metadata)?;
    write_feature_line(writer, model)
}

fn write_model_stream_header<W>(writer: &mut W, metadata: &Value) -> Result<()>
where
    W: Write,
{
    let header = stream_header(metadata)?;
    serde_json::to_writer(&mut *writer, &header)?;
    writer.write_all(b"\n")?;
    Ok(())
}

fn write_feature_line<W>(writer: &mut W, model: &CityModel) -> Result<()>
where
    W: Write,
{
    cjlib::json::to_feature_writer(writer, model)?;
    writer.write_all(b"\n")?;
    Ok(())
}

fn stream_header(metadata: &Value) -> Result<Value> {
    let mut header = metadata.clone();
    let root = header
        .as_object_mut()
        .ok_or_else(|| Error::Import("stream metadata must be a JSON object".to_string()))?;
    root.insert("type".to_string(), Value::String("CityJSON".to_string()));
    root.insert("CityObjects".to_string(), Value::Object(Map::new()));
    root.insert("vertices".to_string(), Value::Array(Vec::new()));
    Ok(header)
}
