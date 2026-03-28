use std::path::PathBuf;

use cjindex::{CityIndex, StorageLayout};
use cjlib::{Error, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};

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
}

#[derive(Debug, Args, Clone)]
struct StorageArgs {
    #[arg(long, value_enum)]
    layout: LayoutKind,

    #[arg(long, value_name = "PATH", num_args = 1..)]
    paths: Vec<PathBuf>,

    #[arg(long)]
    root: Option<PathBuf>,

    #[arg(long, default_value = "**/metadata.city.json")]
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
        Command::Reindex(args) => run_reindex(args),
        Command::Get(args) => run_get(args),
        Command::Query(args) => run_query(args),
        Command::Metadata(args) => run_metadata(args),
    }
}

fn run_reindex(args: IndexCommand) -> Result<()> {
    let storage = args.storage.into_layout()?;
    let _index = CityIndex::open(storage, &args.index)?;
    Err(not_implemented("reindex"))
}

fn run_get(args: FeatureCommand) -> Result<()> {
    let storage = args.storage.into_layout()?;
    let _index = CityIndex::open(storage, &args.index)?;
    let _ = args.id;
    Err(not_implemented("get"))
}

fn run_query(args: QueryCommand) -> Result<()> {
    let storage = args.storage.into_layout()?;
    let _index = CityIndex::open(storage, &args.index)?;
    let _ = (args.min_x, args.max_x, args.min_y, args.max_y);
    Err(not_implemented("query"))
}

fn run_metadata(args: IndexCommand) -> Result<()> {
    let storage = args.storage.into_layout()?;
    let _index = CityIndex::open(storage, &args.index)?;
    Err(not_implemented("metadata"))
}

fn not_implemented(command: &str) -> Error {
    Error::Import(format!(
        "cjindex {command} is scaffolded but not implemented yet"
    ))
}
