use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;

use cjindex::{BBox, CityIndex, StorageLayout};
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
}

#[derive(Debug, Args)]
struct IndexCommand {
    #[command(flatten)]
    storage: StorageArgs,

    #[arg(long)]
    index: PathBuf,
}

#[derive(Debug, Args)]
struct FeatureCommand {
    #[command(flatten)]
    storage: StorageArgs,

    #[arg(long)]
    index: PathBuf,

    #[arg(long)]
    id: String,

    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct QueryCommand {
    #[command(flatten)]
    storage: StorageArgs,

    #[arg(long)]
    index: PathBuf,

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

#[derive(Debug, Args, Clone)]
struct StorageArgs {
    #[arg(long, value_enum)]
    layout: LayoutKind,

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
        match self.layout {
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
    }
}

fn run_reindex(args: IndexCommand) -> Result<()> {
    let storage = args.storage.into_layout()?;
    let mut index = CityIndex::open(storage, &args.index)?;
    index.reindex()
}

fn run_get(args: FeatureCommand) -> Result<()> {
    let storage = args.storage.into_layout()?;
    let index = CityIndex::open(storage, &args.index)?;
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
    let storage = args.storage.into_layout()?;
    let index = CityIndex::open(storage, &args.index)?;
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
    let storage = args.storage.into_layout()?;
    let index = CityIndex::open(storage, &args.index)?;
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
    writer.write_all(cjlib::json::to_feature_string(model)?.as_bytes())?;
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
