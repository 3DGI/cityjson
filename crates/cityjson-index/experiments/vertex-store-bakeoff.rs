//! CLI entry point for the coordinator-owned vertex-store bake-off harness.

use std::collections::BTreeMap;
use std::path::PathBuf;

use cityjson_index::vertex_store_bakeoff::{
    BakeoffProvenance, BakeoffResult, VertexStoreStrategy, VertexStoreTelemetry,
    open_matching_read_sidecar, write_result,
};
use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "vertex-store-bakeoff",
    about = "Validate and record vertex-store bake-off runs"
)]
struct Cli {
    #[arg(long, value_enum)]
    strategy: StrategyArg,
    #[arg(long)]
    dataset_root: PathBuf,
    #[arg(long)]
    result: Option<PathBuf>,
    #[arg(long, default_value_t = 1)]
    workers: usize,
    #[arg(long, default_value_t = 1)]
    repetition: usize,
    #[arg(long, default_value = "unknown")]
    candidate_commit: String,
    #[arg(long, default_value = "unknown")]
    harness_commit: String,
    #[arg(long, default_value = "unknown")]
    corpus_identity: String,
    #[arg(long, value_enum, default_value_t = ExperimentArg::ReadLatency)]
    experiment: ExperimentArg,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum StrategyArg {
    PackedChunks,
    JsonOffsets,
    FrameOfReference,
}

impl From<StrategyArg> for VertexStoreStrategy {
    fn from(value: StrategyArg) -> Self {
        match value {
            StrategyArg::PackedChunks => Self::PackedChunks,
            StrategyArg::JsonOffsets => Self::JsonOffsets,
            StrategyArg::FrameOfReference => Self::FrameOfReference,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ExperimentArg {
    CorrectnessStorage,
    ReindexCost,
    ReadLatency,
    TylerMaterialization,
}

impl ExperimentArg {
    const fn label(self) -> &'static str {
        match self {
            Self::CorrectnessStorage => "correctness-storage",
            Self::ReindexCost => "reindex-cost",
            Self::ReadLatency => "read-latency",
            Self::TylerMaterialization => "tyler-materialization",
        }
    }
}

fn main() -> cityjson_lib::Result<()> {
    let cli = Cli::parse();
    let strategy = VertexStoreStrategy::from(cli.strategy);
    let sidecar_path = strategy.sidecar_path(&cli.dataset_root);
    // A measured process is read-only by construction: this check does not
    // create, migrate, remove, or rebuild a sidecar.
    let _connection = open_matching_read_sidecar(&sidecar_path, strategy)?;
    if let Some(path) = cli.result {
        let provenance = BakeoffProvenance {
            strategy,
            candidate_commit: cli.candidate_commit,
            harness_commit: cli.harness_commit,
            corpus_identity: cli.corpus_identity,
            sidecar_path,
            worker_count: cli.workers,
            repetition: cli.repetition,
            runtime_configuration: BTreeMap::new(),
        };
        let result = BakeoffResult::new(
            cli.experiment.label(),
            provenance,
            VertexStoreTelemetry::default(),
            serde_json::json!({"status": "validated-sidecar"}),
        );
        write_result(&path, &result)?;
    }
    Ok(())
}
