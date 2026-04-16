#![allow(dead_code)]

use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::OnceLock;

use cityjson_lib::CityModel;
use serde::Deserialize;
use serde_json::Value;

const DEFAULT_BENCHMARK_INDEX: &str = "artifacts/benchmark-index.json";
const PREPARE_INSTRUCTION: &str = "benchmark data is missing; set \
    CITYJSON_LIB_BENCH_SHARED_CORPUS_ROOT to your cityjson-corpus checkout \
    and ensure the corpus artifacts are present";

static BENCHMARK_CASES: OnceLock<Vec<BenchmarkCase>> = OnceLock::new();

#[derive(Debug, Clone)]
pub(crate) struct BenchmarkCase {
    pub(crate) id: String,
    pub(crate) description: String,
    pub(crate) json_path: PathBuf,
    pub(crate) input_bytes: u64,
    pub(crate) cityarrow_path: PathBuf,
    pub(crate) cityarrow_bytes: u64,
}

#[derive(Debug)]
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
    ArrowWrite {
        model: CityModel,
        output_path: PathBuf,
        _output_dir: tempfile::TempDir,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Workload {
    JsonSerdeValueRead,
    JsonCityjsonRead,
    JsonCityjsonLibRead,
    JsonSerdeValueWrite,
    JsonCityjsonWrite,
    JsonCityjsonLibWrite,
    ArrowRead,
    ArrowWrite,
}

pub(crate) const READ_WORKLOADS: [Workload; 4] = [
    Workload::JsonSerdeValueRead,
    Workload::JsonCityjsonRead,
    Workload::JsonCityjsonLibRead,
    Workload::ArrowRead,
];

pub(crate) const WRITE_WORKLOADS: [Workload; 4] = [
    Workload::JsonSerdeValueWrite,
    Workload::JsonCityjsonWrite,
    Workload::JsonCityjsonLibWrite,
    Workload::ArrowWrite,
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
        | Workload::JsonCityjsonRead
        | Workload::JsonCityjsonLibRead => PreparedWorkload::JsonRead {
            workload,
            input_json: read_text(&case.json_path),
        },
        Workload::JsonSerdeValueWrite => PreparedWorkload::JsonValueWrite {
            value: read_json_value(&case.json_path),
        },
        Workload::JsonCityjsonWrite | Workload::JsonCityjsonLibWrite => {
            PreparedWorkload::ModelWrite {
                workload,
                model: read_model(&case.json_path),
            }
        }
        Workload::ArrowRead => PreparedWorkload::ArrowRead {
            cityarrow_path: case.cityarrow_path.clone(),
        },
        Workload::ArrowWrite => {
            let output_dir = tempfile::tempdir().expect("benchmark tempdir should be creatable");
            PreparedWorkload::ArrowWrite {
                model: read_model(&case.json_path),
                output_path: output_dir.path().join("model.cjarrow"),
                _output_dir: output_dir,
            }
        }
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
            workload: Workload::JsonCityjsonRead,
            input_json,
        } => {
            let model = cityjson_json::read_model(
                black_box(input_json.as_bytes()),
                &cityjson_json::ReadOptions::default(),
            )
            .unwrap();
            black_box(model);
        }
        PreparedWorkload::JsonRead {
            workload: Workload::JsonCityjsonLibRead,
            input_json,
        } => {
            let model = cityjson_lib::json::from_slice(black_box(input_json.as_bytes())).unwrap();
            black_box(model);
        }
        PreparedWorkload::JsonValueWrite { value } => {
            let output = serde_json::to_vec(black_box(value)).unwrap();
            black_box(output);
        }
        PreparedWorkload::ModelWrite {
            workload: Workload::JsonCityjsonWrite,
            model,
        } => {
            let output =
                cityjson_json::to_vec(black_box(model), &cityjson_json::WriteOptions::default())
                    .unwrap();
            black_box(output);
        }
        PreparedWorkload::ModelWrite {
            workload: Workload::JsonCityjsonLibWrite,
            model,
        } => {
            let output = cityjson_lib::json::to_vec(black_box(model)).unwrap();
            black_box(output);
        }
        PreparedWorkload::ArrowRead { cityarrow_path } => {
            let model =
                cityjson_lib::arrow::from_file(black_box(cityarrow_path.as_path())).unwrap();
            black_box(model);
        }
        PreparedWorkload::ArrowWrite {
            model, output_path, ..
        } => {
            cityjson_lib::arrow::to_file(black_box(output_path.as_path()), black_box(model))
                .unwrap();
            black_box(output_path);
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
        | Workload::JsonCityjsonRead
        | Workload::JsonCityjsonLibRead => case.input_bytes,
        Workload::JsonSerdeValueWrite => serde_json::to_vec(&read_json_value(&case.json_path))
            .unwrap()
            .len() as u64,
        Workload::JsonCityjsonWrite => cityjson_json::to_vec(
            &read_model(&case.json_path),
            &cityjson_json::WriteOptions::default(),
        )
        .unwrap()
        .len() as u64,
        Workload::JsonCityjsonLibWrite => cityjson_lib::json::to_vec(&read_model(&case.json_path))
            .unwrap()
            .len() as u64,
        Workload::ArrowRead | Workload::ArrowWrite => case.cityarrow_bytes,
    }
}

impl Workload {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::JsonSerdeValueRead => "serde_json::Value/read",
            Self::JsonCityjsonRead => "cityjson_lib/read",
            Self::JsonCityjsonLibRead => "cityjson_lib::json/read",
            Self::JsonSerdeValueWrite => "serde_json::Value/write",
            Self::JsonCityjsonWrite => "cityjson_lib/write",
            Self::JsonCityjsonLibWrite => "cityjson_lib::json/write",
            Self::ArrowRead => "cityarrow/read",
            Self::ArrowWrite => "cityarrow/write",
        }
    }
}

impl FromStr for Workload {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "serde_json-read" => Ok(Self::JsonSerdeValueRead),
            "cityjson_lib-read" | "serde_cityjson-read" => Ok(Self::JsonCityjsonRead),
            "cityjson-lib-json-read" => Ok(Self::JsonCityjsonLibRead),
            "serde_json-write" => Ok(Self::JsonSerdeValueWrite),
            "cityjson_lib-write" | "serde_cityjson-write" => Ok(Self::JsonCityjsonWrite),
            "cityjson-lib-json-write" => Ok(Self::JsonCityjsonLibWrite),
            "cityarrow-read" => Ok(Self::ArrowRead),
            "cityarrow-write" => Ok(Self::ArrowWrite),
            other => Err(format!("unknown workload '{other}'")),
        }
    }
}

#[derive(Deserialize)]
struct BenchmarkIndex {
    #[serde(default)]
    generated_cases: Vec<IndexCase>,
    #[serde(default)]
    other_cases: Vec<IndexCase>,
}

#[derive(Deserialize)]
struct IndexCase {
    id: String,
    #[serde(default)]
    description: String,
    layer: String,
    #[serde(default)]
    artifacts: Vec<IndexArtifact>,
}

#[derive(Deserialize)]
struct IndexArtifact {
    representation: String,
    path: PathBuf,
    byte_size: Option<u64>,
}

fn cached_benchmark_cases() -> &'static [BenchmarkCase] {
    BENCHMARK_CASES.get_or_init(load_benchmark_cases)
}

fn load_benchmark_cases() -> Vec<BenchmarkCase> {
    let index_path = benchmark_index_path();
    assert!(
        index_path.is_file(),
        "{} ({})",
        PREPARE_INSTRUCTION,
        index_path.display()
    );
    let corpus_root = shared_corpus_root();
    let bytes = fs::read(&index_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", index_path.display()));
    let index: BenchmarkIndex = serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", index_path.display()));

    index
        .generated_cases
        .into_iter()
        .chain(index.other_cases)
        .filter(|case| case.layer != "invalid")
        .filter_map(|case| try_build_case(&corpus_root, case))
        .collect()
}

fn try_build_case(corpus_root: &Path, case: IndexCase) -> Option<BenchmarkCase> {
    let find = |rep: &str| case.artifacts.iter().find(|a| a.representation == rep);
    let cityjson = find("cityjson")?;
    let cityarrow = find("cityjson-arrow")?;
    let json_path = resolve_path(corpus_root, &cityjson.path);
    let cityarrow_path = resolve_path(corpus_root, &cityarrow.path);

    if !json_path.is_file() || !cityarrow_path.is_file() {
        return None;
    }

    Some(BenchmarkCase {
        id: case.id,
        description: case.description,
        input_bytes: cityjson.byte_size.unwrap_or_else(|| file_size(&json_path)),
        cityarrow_bytes: cityarrow
            .byte_size
            .unwrap_or_else(|| file_size(&cityarrow_path)),
        json_path,
        cityarrow_path,
    })
}

fn benchmark_index_path() -> PathBuf {
    shared_corpus_root().join(DEFAULT_BENCHMARK_INDEX)
}

fn shared_corpus_root() -> PathBuf {
    std::env::var_os("CITYJSON_LIB_BENCH_SHARED_CORPUS_ROOT").map_or_else(
        || {
            panic!(
                "set CITYJSON_LIB_BENCH_SHARED_CORPUS_ROOT to your cityjson-corpus checkout, \
                or set it in a .env file at the repo root"
            )
        },
        PathBuf::from,
    )
}

fn resolve_path(corpus_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        corpus_root.join(path)
    }
}

fn file_size(path: &Path) -> u64 {
    fs::metadata(path)
        .unwrap_or_else(|error| panic!("failed to stat {}: {error}", path.display()))
        .len()
}

fn read_text(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn read_model(path: &Path) -> CityModel {
    cityjson_lib::json::from_slice(read_text(path).as_bytes()).unwrap_or_else(|error| {
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
