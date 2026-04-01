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
const STRESS_CASE_FILE: &str = "cluster_4x.city.json";

const PREPARE_INSTRUCTION: &str =
    "benchmark data is missing; run `just bench-prepare` to download and merge the pinned 3DBAG tiles";

#[derive(Debug, Clone)]
pub(crate) struct PreparedCase {
    pub(crate) id: String,
    pub(crate) description: String,
    pub(crate) input_json: String,
    pub(crate) input_bytes: u64,
    pub(crate) model: CityModel,
    pub(crate) json_value: Value,
    pub(crate) serde_json_output_bytes: u64,
    pub(crate) serde_cityjson_output_bytes: u64,
    pub(crate) cjlib_json_output_bytes: u64,
    pub(crate) arrow_dir: PathBuf,
    pub(crate) arrow_bytes: u64,
    pub(crate) parquet_dir: PathBuf,
    pub(crate) parquet_bytes: u64,
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

pub(crate) fn prepared_cases() -> Vec<PreparedCase> {
    case_specs().into_iter().map(prepare_case).collect()
}

pub(crate) fn find_case(cases: &[PreparedCase], case_id: &str) -> PreparedCase {
    cases.iter()
        .find(|case| case.id == case_id)
        .cloned()
        .unwrap_or_else(|| panic!("unknown benchmark case '{case_id}'"))
}

pub(crate) fn run_workload(case: &PreparedCase, workload: Workload) {
    match workload {
        Workload::JsonSerdeValueRead => {
            let value =
                serde_json::from_slice::<Value>(black_box(case.input_json.as_bytes())).unwrap();
            black_box(value);
        }
        Workload::JsonSerdeCityjsonRead => {
            let model = serde_cityjson::from_str_owned(black_box(&case.input_json)).unwrap();
            black_box(model);
        }
        Workload::JsonCjlibRead => {
            let model = cjlib::json::from_slice(black_box(case.input_json.as_bytes())).unwrap();
            black_box(model);
        }
        Workload::JsonSerdeValueWrite => {
            let output = serde_json::to_vec(black_box(&case.json_value)).unwrap();
            black_box(output);
        }
        Workload::JsonSerdeCityjsonWrite => {
            let output = serde_cityjson::to_string_validated(black_box(case.model.as_inner()))
                .unwrap();
            black_box(output);
        }
        Workload::JsonCjlibWrite => {
            let output = cjlib::json::to_vec(black_box(&case.model)).unwrap();
            black_box(output);
        }
        Workload::ArrowRead => {
            let parts = cityarrow::read_package_ipc_dir(black_box(case.arrow_dir.as_path())).unwrap();
            let model = cityarrow::from_parts(&parts).unwrap();
            black_box(model);
        }
        Workload::ArrowWrite => {
            let dir = tempfile::tempdir().unwrap();
            let parts = cityarrow::to_parts(black_box(case.model.as_inner())).unwrap();
            cityarrow::write_package_ipc_dir(dir.path(), &parts).unwrap();
            black_box(dir);
        }
        Workload::ParquetRead => {
            let parts = cityparquet::read_package_dir(black_box(case.parquet_dir.as_path())).unwrap();
            let model = cityparquet::from_parts(&parts).unwrap();
            black_box(model);
        }
        Workload::ParquetWrite => {
            let dir = tempfile::tempdir().unwrap();
            let parts = cityparquet::to_parts(black_box(case.model.as_inner())).unwrap();
            cityparquet::write_package_dir(dir.path(), &parts).unwrap();
            black_box(dir);
        }
    }
}

pub(crate) fn throughput_bytes(case: &PreparedCase, workload: Workload) -> u64 {
    match workload {
        Workload::JsonSerdeValueRead
        | Workload::JsonSerdeCityjsonRead
        | Workload::JsonCjlibRead => case.input_bytes,
        Workload::JsonSerdeValueWrite => case.serde_json_output_bytes,
        Workload::JsonSerdeCityjsonWrite => case.serde_cityjson_output_bytes,
        Workload::JsonCjlibWrite => case.cjlib_json_output_bytes,
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
}

fn case_specs() -> Vec<CaseSpec> {
    let root = bench_data_root().join(RELEASE_PATH);
    vec![
        CaseSpec {
            id: BASE_CASE_ID,
            description: BASE_CASE_DESCRIPTION,
            json_path: root.join(BASE_CASE_FILE),
        },
        CaseSpec {
            id: STRESS_CASE_ID,
            description: STRESS_CASE_DESCRIPTION,
            json_path: root.join(STRESS_CASE_FILE),
        },
    ]
}

fn prepare_case(spec: CaseSpec) -> PreparedCase {
    ensure_file(&spec.json_path);

    let input_json = fs::read_to_string(&spec.json_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", spec.json_path.display()));
    let input_bytes = input_json.len() as u64;
    let model = cjlib::json::from_slice(input_json.as_bytes()).unwrap_or_else(|error| {
        panic!(
            "failed to parse benchmark input {}: {error}",
            spec.json_path.display()
        )
    });
    let json_value =
        serde_json::from_str::<Value>(&input_json).unwrap_or_else(|error| {
            panic!(
                "failed to parse JSON value benchmark input {}: {error}",
                spec.json_path.display()
            )
        });

    let derived_root = bench_data_root().join("prepared").join(&spec.id);
    let arrow_dir = derived_root.join("arrow_ipc");
    let parquet_dir = derived_root.join("parquet");

    ensure_arrow_package(&arrow_dir, &model);
    ensure_parquet_package(&parquet_dir, &model);

    let serde_json_output_bytes = serde_json::to_vec(&json_value).unwrap().len() as u64;
    let serde_cityjson_output_bytes =
        serde_cityjson::to_string_validated(model.as_inner()).unwrap().len() as u64;
    let cjlib_json_output_bytes = cjlib::json::to_vec(&model).unwrap().len() as u64;

    PreparedCase {
        id: spec.id.to_string(),
        description: spec.description.to_string(),
        input_json,
        input_bytes,
        model,
        json_value,
        serde_json_output_bytes,
        serde_cityjson_output_bytes,
        cjlib_json_output_bytes,
        arrow_dir: arrow_dir.clone(),
        arrow_bytes: dir_size_bytes(&arrow_dir),
        parquet_dir: parquet_dir.clone(),
        parquet_bytes: dir_size_bytes(&parquet_dir),
    }
}

fn ensure_arrow_package(path: &Path, model: &CityModel) {
    if path.join("manifest.json").is_file() {
        return;
    }
    if path.exists() {
        fs::remove_dir_all(path)
            .unwrap_or_else(|error| panic!("failed to remove {}: {error}", path.display()));
    }
    fs::create_dir_all(path.parent().unwrap())
        .unwrap_or_else(|error| panic!("failed to create {}: {error}", path.display()));
    let parts = cityarrow::to_parts(model.as_inner())
        .unwrap_or_else(|error| panic!("failed to derive Arrow parts: {error}"));
    cityarrow::write_package_ipc_dir(path, &parts)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
}

fn ensure_parquet_package(path: &Path, model: &CityModel) {
    if path.join("manifest.json").is_file() {
        return;
    }
    if path.exists() {
        fs::remove_dir_all(path)
            .unwrap_or_else(|error| panic!("failed to remove {}: {error}", path.display()));
    }
    fs::create_dir_all(path.parent().unwrap())
        .unwrap_or_else(|error| panic!("failed to create {}: {error}", path.display()));
    let parts = cityparquet::to_parts(model.as_inner())
        .unwrap_or_else(|error| panic!("failed to derive Parquet parts: {error}"));
    cityparquet::write_package_dir(path, &parts)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
}

fn ensure_file(path: &Path) {
    assert!(
        path.is_file(),
        "{} ({})",
        PREPARE_INSTRUCTION,
        path.display()
    );
}

fn bench_data_root() -> PathBuf {
    std::env::var_os("CJLIB_BENCH_DATA_ROOT").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_BENCH_DATA_ROOT),
        PathBuf::from,
    )
}

fn dir_size_bytes(path: &Path) -> u64 {
    let mut total = 0_u64;
    for entry in fs::read_dir(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
    {
        let entry =
            entry.unwrap_or_else(|error| panic!("failed to read dir entry for {}: {error}", path.display()));
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
