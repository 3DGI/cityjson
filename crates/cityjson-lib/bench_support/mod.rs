#![allow(dead_code)]

use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::OnceLock;

use cjlib::CityModel;
use serde::Deserialize;
use serde_json::Value;

const DEFAULT_BENCH_DATA_ROOT: &str = "target/bench-data";
const RELEASE_PATH: &str = "3dbag/v20250903";
const PREPARE_INSTRUCTION: &str = "benchmark data is missing; run `just bench-prepare` to materialize the pinned 3DBAG CityJSON, cityarrow, and cityparquet artifacts";

static BENCHMARK_CASES: OnceLock<Vec<BenchmarkCase>> = OnceLock::new();

#[derive(Debug, Clone)]
pub(crate) struct BenchmarkCase {
    pub(crate) id: String,
    pub(crate) description: String,
    pub(crate) json_path: PathBuf,
    pub(crate) input_bytes: u64,
    pub(crate) cityarrow_path: PathBuf,
    pub(crate) cityarrow_bytes: u64,
    pub(crate) cityparquet_path: PathBuf,
    pub(crate) cityparquet_bytes: u64,
}

#[derive(Debug, Clone)]
pub(crate) enum PreparedWorkload {
    JsonRead {
        workload: Workload,
        input_json: String,
    },
    JsonValueWrite {
        value: Value,
    },
    ModelWrite {
        workload: Workload,
        model: CityModel,
    },
    ArrowRead {
        cityarrow_path: PathBuf,
    },
    ParquetRead {
        cityparquet_path: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Workload {
    JsonSerdeValueRead,
    JsonSerdeCityjsonRead,
    JsonCjlibRead,
    JsonSerdeValueWrite,
    JsonSerdeCityjsonWrite,
    JsonCjlibWrite,
    ArrowRead,
    ArrowWrite,
    ParquetRead,
    ParquetWrite,
}

pub(crate) const READ_WORKLOADS: [Workload; 5] = [
    Workload::JsonSerdeValueRead,
    Workload::JsonSerdeCityjsonRead,
    Workload::JsonCjlibRead,
    Workload::ArrowRead,
    Workload::ParquetRead,
];

pub(crate) const WRITE_WORKLOADS: [Workload; 5] = [
    Workload::JsonSerdeValueWrite,
    Workload::JsonSerdeCityjsonWrite,
    Workload::JsonCjlibWrite,
    Workload::ArrowWrite,
    Workload::ParquetWrite,
];

pub(crate) fn benchmark_cases() -> Vec<BenchmarkCase> {
    cached_benchmark_cases().to_vec()
}

pub(crate) fn load_case(case_id: &str) -> BenchmarkCase {
    cached_benchmark_cases()
        .iter()
        .find(|case| case.id == case_id)
        .cloned()
        .unwrap_or_else(|| panic!("unknown benchmark case '{case_id}'"))
}

pub(crate) fn prepare_workload(case: &BenchmarkCase, workload: Workload) -> PreparedWorkload {
    match workload {
        Workload::JsonSerdeValueRead
        | Workload::JsonSerdeCityjsonRead
        | Workload::JsonCjlibRead => PreparedWorkload::JsonRead {
            workload,
            input_json: read_text(&case.json_path),
        },
        Workload::JsonSerdeValueWrite => PreparedWorkload::JsonValueWrite {
            value: read_json_value(&case.json_path),
        },
        Workload::JsonSerdeCityjsonWrite
        | Workload::JsonCjlibWrite
        | Workload::ArrowWrite
        | Workload::ParquetWrite => PreparedWorkload::ModelWrite {
            workload,
            model: read_model(&case.json_path),
        },
        Workload::ArrowRead => PreparedWorkload::ArrowRead {
            cityarrow_path: case.cityarrow_path.clone(),
        },
        Workload::ParquetRead => PreparedWorkload::ParquetRead {
            cityparquet_path: case.cityparquet_path.clone(),
        },
    }
}

pub(crate) fn run_workload(workload: &PreparedWorkload) {
    match workload {
        PreparedWorkload::JsonRead {
            workload: Workload::JsonSerdeValueRead,
            input_json,
        } => {
            let value = serde_json::from_slice::<Value>(black_box(input_json.as_bytes())).unwrap();
            black_box(value);
        }
        PreparedWorkload::JsonRead {
            workload: Workload::JsonSerdeCityjsonRead,
            input_json,
        } => {
            let model = serde_cityjson::from_str_owned(black_box(input_json)).unwrap();
            black_box(model);
        }
        PreparedWorkload::JsonRead {
            workload: Workload::JsonCjlibRead,
            input_json,
        } => {
            let model = cjlib::json::from_slice(black_box(input_json.as_bytes())).unwrap();
            black_box(model);
        }
        PreparedWorkload::JsonValueWrite { value } => {
            let output = serde_json::to_vec(black_box(value)).unwrap();
            black_box(output);
        }
        PreparedWorkload::ModelWrite {
            workload: Workload::JsonSerdeCityjsonWrite,
            model,
        } => {
            let output = serde_cityjson::to_string_validated(black_box(model.as_inner())).unwrap();
            black_box(output);
        }
        PreparedWorkload::ModelWrite {
            workload: Workload::JsonCjlibWrite,
            model,
        } => {
            let output = cjlib::json::to_vec(black_box(model)).unwrap();
            black_box(output);
        }
        PreparedWorkload::ArrowRead { cityarrow_path } => {
            let model = cjlib::arrow::from_file(black_box(cityarrow_path.as_path())).unwrap();
            black_box(model);
        }
        PreparedWorkload::ModelWrite {
            workload: Workload::ArrowWrite,
            model,
        } => {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("model.cjarrow");
            cjlib::arrow::to_file(&path, black_box(model)).unwrap();
            black_box(dir);
        }
        PreparedWorkload::ParquetRead { cityparquet_path } => {
            let model = cjlib::parquet::from_file(black_box(cityparquet_path.as_path())).unwrap();
            black_box(model);
        }
        PreparedWorkload::ModelWrite {
            workload: Workload::ParquetWrite,
            model,
        } => {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("model.cjparquet");
            cjlib::parquet::to_file(&path, black_box(model)).unwrap();
            black_box(dir);
        }
        PreparedWorkload::JsonRead { workload, .. }
        | PreparedWorkload::ModelWrite { workload, .. } => {
            panic!(
                "unsupported prepared workload state for '{}'",
                workload.label()
            );
        }
    }
}

pub(crate) fn throughput_bytes(case: &BenchmarkCase, workload: Workload) -> u64 {
    match workload {
        Workload::JsonSerdeValueRead
        | Workload::JsonSerdeCityjsonRead
        | Workload::JsonCjlibRead => case.input_bytes,
        Workload::JsonSerdeValueWrite => serde_json::to_vec(&read_json_value(&case.json_path))
            .unwrap()
            .len() as u64,
        Workload::JsonSerdeCityjsonWrite => {
            serde_cityjson::to_string_validated(read_model(&case.json_path).as_inner())
                .unwrap()
                .len() as u64
        }
        Workload::JsonCjlibWrite => cjlib::json::to_vec(&read_model(&case.json_path))
            .unwrap()
            .len() as u64,
        Workload::ArrowRead | Workload::ArrowWrite => case.cityarrow_bytes,
        Workload::ParquetRead | Workload::ParquetWrite => case.cityparquet_bytes,
    }
}

impl Workload {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::JsonSerdeValueRead => "serde_json::Value/read",
            Self::JsonSerdeCityjsonRead => "serde_cityjson/read",
            Self::JsonCjlibRead => "cjlib::json/read",
            Self::JsonSerdeValueWrite => "serde_json::Value/write",
            Self::JsonSerdeCityjsonWrite => "serde_cityjson/write",
            Self::JsonCjlibWrite => "cjlib::json/write",
            Self::ArrowRead => "cityarrow/read",
            Self::ArrowWrite => "cityarrow/write",
            Self::ParquetRead => "cityparquet/read",
            Self::ParquetWrite => "cityparquet/write",
        }
    }
}

impl FromStr for Workload {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "serde_json-read" => Ok(Self::JsonSerdeValueRead),
            "serde_cityjson-read" => Ok(Self::JsonSerdeCityjsonRead),
            "cjlib-json-read" => Ok(Self::JsonCjlibRead),
            "serde_json-write" => Ok(Self::JsonSerdeValueWrite),
            "serde_cityjson-write" => Ok(Self::JsonSerdeCityjsonWrite),
            "cjlib-json-write" => Ok(Self::JsonCjlibWrite),
            "cityarrow-read" => Ok(Self::ArrowRead),
            "cityarrow-write" => Ok(Self::ArrowWrite),
            "cityparquet-read" => Ok(Self::ParquetRead),
            "cityparquet-write" => Ok(Self::ParquetWrite),
            other => Err(format!("unknown workload '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestRoot {
    cases: Vec<ManifestCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestCase {
    id: String,
    description: String,
    artifacts: ManifestArtifacts,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestArtifacts {
    cityjson: ManifestArtifact,
    cityarrow: ManifestArtifact,
    cityparquet: ManifestArtifact,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestArtifact {
    path: PathBuf,
    byte_size: u64,
}

fn cached_benchmark_cases() -> &'static [BenchmarkCase] {
    BENCHMARK_CASES.get_or_init(load_benchmark_cases)
}

fn load_benchmark_cases() -> Vec<BenchmarkCase> {
    let manifest_path = benchmark_manifest_path();
    ensure_file(&manifest_path);

    let manifest_dir = manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let manifest_bytes = fs::read(&manifest_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", manifest_path.display()));
    let manifest: ManifestRoot = serde_json::from_slice(&manifest_bytes)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", manifest_path.display()));

    manifest
        .cases
        .into_iter()
        .map(|case| load_manifest_case(&manifest_dir, case))
        .collect()
}

fn load_manifest_case(manifest_dir: &Path, case: ManifestCase) -> BenchmarkCase {
    let json_path = resolve_artifact_path(manifest_dir, case.artifacts.cityjson.path);
    let cityarrow_path = resolve_artifact_path(manifest_dir, case.artifacts.cityarrow.path);
    let cityparquet_path = resolve_artifact_path(manifest_dir, case.artifacts.cityparquet.path);

    ensure_file(&json_path);
    ensure_file(&cityarrow_path);
    ensure_file(&cityparquet_path);

    BenchmarkCase {
        id: case.id,
        description: case.description,
        json_path,
        input_bytes: case.artifacts.cityjson.byte_size,
        cityarrow_path,
        cityarrow_bytes: case.artifacts.cityarrow.byte_size,
        cityparquet_path,
        cityparquet_bytes: case.artifacts.cityparquet.byte_size,
    }
}

fn resolve_artifact_path(manifest_dir: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        manifest_dir.join(path)
    }
}

fn benchmark_manifest_path() -> PathBuf {
    bench_data_root().join(RELEASE_PATH).join("manifest.json")
}

fn bench_data_root() -> PathBuf {
    std::env::var_os("CJLIB_BENCH_DATA_ROOT").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_BENCH_DATA_ROOT),
        PathBuf::from,
    )
}

fn read_text(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn read_model(path: &Path) -> CityModel {
    cjlib::json::from_slice(read_text(path).as_bytes()).unwrap_or_else(|error| {
        panic!(
            "failed to parse benchmark input {}: {error}",
            path.display()
        )
    })
}

fn read_json_value(path: &Path) -> Value {
    serde_json::from_str(&read_text(path)).unwrap_or_else(|error| {
        panic!(
            "failed to parse JSON value benchmark input {}: {error}",
            path.display()
        )
    })
}

fn ensure_file(path: &Path) {
    assert!(
        path.is_file(),
        "{} ({})",
        PREPARE_INSTRUCTION,
        path.display()
    );
}
