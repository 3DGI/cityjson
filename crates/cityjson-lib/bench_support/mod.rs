#![allow(dead_code)]

use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use cjlib::CityModel;
use serde_json::Value;

const DEFAULT_BENCH_DATA_ROOT: &str = "target/bench-data";
const RELEASE_PATH: &str = "3dbag/v20250903";
const BASE_CASE_ID: &str = "io_3dbag_cityjson";
const STRESS_CASE_ID: &str = "io_3dbag_cityjson_cluster_4x";

const BASE_CASE_DESCRIPTION: &str =
    "Pinned real 3DBAG tile from the shared corpus release v20250903.";
const STRESS_CASE_DESCRIPTION: &str =
    "Merged four-tile real 3DBAG workload built from contiguous v20250903 tiles.";

const BASE_CASE_FILE: &str = "10-758-50.city.json";
const BASE_CASE_ARROW_DIR: &str = "10-758-50.arrow-ipc";
const BASE_CASE_PARQUET_DIR: &str = "10-758-50.parquet";
const STRESS_CASE_FILE: &str = "cluster_4x.city.json";
const STRESS_CASE_ARROW_DIR: &str = "cluster_4x.arrow-ipc";
const STRESS_CASE_PARQUET_DIR: &str = "cluster_4x.parquet";

const PREPARE_INSTRUCTION: &str = "benchmark data is missing; run `just bench-prepare` to materialize the pinned 3DBAG CityJSON, Arrow IPC, and Parquet artifacts";

#[derive(Debug, Clone)]
pub(crate) struct BenchmarkCase {
    pub(crate) id: String,
    pub(crate) description: String,
    pub(crate) json_path: PathBuf,
    pub(crate) input_bytes: u64,
    pub(crate) arrow_dir: PathBuf,
    pub(crate) arrow_bytes: u64,
    pub(crate) parquet_dir: PathBuf,
    pub(crate) parquet_bytes: u64,
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
        arrow_dir: PathBuf,
    },
    ParquetRead {
        parquet_dir: PathBuf,
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
    case_specs().into_iter().map(load_case_spec).collect()
}

pub(crate) fn load_case(case_id: &str) -> BenchmarkCase {
    let spec = case_specs()
        .into_iter()
        .find(|spec| spec.id == case_id)
        .unwrap_or_else(|| panic!("unknown benchmark case '{case_id}'"));
    load_case_spec(spec)
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
            arrow_dir: case.arrow_dir.clone(),
        },
        Workload::ParquetRead => PreparedWorkload::ParquetRead {
            parquet_dir: case.parquet_dir.clone(),
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
        PreparedWorkload::ArrowRead { arrow_dir } => {
            let parts = cityarrow::read_package_ipc_dir(black_box(arrow_dir.as_path())).unwrap();
            let model = cityarrow::from_parts(&parts).unwrap();
            black_box(model);
        }
        PreparedWorkload::ModelWrite {
            workload: Workload::ArrowWrite,
            model,
        } => {
            let dir = tempfile::tempdir().unwrap();
            let parts = cityarrow::to_parts(black_box(model.as_inner())).unwrap();
            cityarrow::write_package_ipc_dir(dir.path(), &parts).unwrap();
            black_box(dir);
        }
        PreparedWorkload::ParquetRead { parquet_dir } => {
            let parts = cityparquet::read_package_dir(black_box(parquet_dir.as_path())).unwrap();
            let model = cityparquet::from_parts(&parts).unwrap();
            black_box(model);
        }
        PreparedWorkload::ModelWrite {
            workload: Workload::ParquetWrite,
            model,
        } => {
            let dir = tempfile::tempdir().unwrap();
            let parts = cityparquet::to_parts(black_box(model.as_inner())).unwrap();
            cityparquet::write_package_dir(dir.path(), &parts).unwrap();
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
        Workload::ArrowRead | Workload::ArrowWrite => case.arrow_bytes,
        Workload::ParquetRead | Workload::ParquetWrite => case.parquet_bytes,
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

#[derive(Debug, Clone)]
struct CaseSpec {
    id: &'static str,
    description: &'static str,
    json_path: PathBuf,
    arrow_dir: PathBuf,
    parquet_dir: PathBuf,
}

fn case_specs() -> Vec<CaseSpec> {
    let root = bench_data_root().join(RELEASE_PATH);
    vec![
        CaseSpec {
            id: BASE_CASE_ID,
            description: BASE_CASE_DESCRIPTION,
            json_path: root.join(BASE_CASE_FILE),
            arrow_dir: root.join(BASE_CASE_ARROW_DIR),
            parquet_dir: root.join(BASE_CASE_PARQUET_DIR),
        },
        CaseSpec {
            id: STRESS_CASE_ID,
            description: STRESS_CASE_DESCRIPTION,
            json_path: root.join(STRESS_CASE_FILE),
            arrow_dir: root.join(STRESS_CASE_ARROW_DIR),
            parquet_dir: root.join(STRESS_CASE_PARQUET_DIR),
        },
    ]
}

fn load_case_spec(spec: CaseSpec) -> BenchmarkCase {
    ensure_file(&spec.json_path);
    ensure_dir(&spec.arrow_dir);
    ensure_dir(&spec.parquet_dir);

    BenchmarkCase {
        id: spec.id.to_string(),
        description: spec.description.to_string(),
        input_bytes: file_size_bytes(&spec.json_path),
        json_path: spec.json_path,
        arrow_bytes: dir_size_bytes(&spec.arrow_dir),
        arrow_dir: spec.arrow_dir,
        parquet_bytes: dir_size_bytes(&spec.parquet_dir),
        parquet_dir: spec.parquet_dir,
    }
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

fn ensure_dir(path: &Path) {
    assert!(
        path.join("manifest.json").is_file(),
        "{} ({})",
        PREPARE_INSTRUCTION,
        path.display()
    );
}

fn file_size_bytes(path: &Path) -> u64 {
    fs::metadata(path)
        .unwrap_or_else(|error| panic!("failed to stat {}: {error}", path.display()))
        .len()
}

fn dir_size_bytes(path: &Path) -> u64 {
    let mut total = 0_u64;
    for entry in fs::read_dir(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
    {
        let entry = entry.unwrap_or_else(|error| {
            panic!("failed to read dir entry for {}: {error}", path.display())
        });
        let entry_path = entry.path();
        let metadata = entry
            .metadata()
            .unwrap_or_else(|error| panic!("failed to stat {}: {error}", entry_path.display()));
        if metadata.is_dir() {
            total += dir_size_bytes(&entry_path);
        } else {
            total += metadata.len();
        }
    }
    total
}
