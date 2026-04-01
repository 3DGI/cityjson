#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use serde_cityjson::{as_json, from_str_owned, to_string, to_string_validated, OwnedCityModel};

const DEFAULT_SHARED_CORPUS_ROOT: &str = "../cityjson-benchmarks";
const DEFAULT_BENCHMARK_INDEX_PATH: &str = "artifacts/benchmark-index.json";

pub(crate) const READ_BENCH_SERDE_CITYJSON_OWNED: &str = "serde_cityjson/owned";
pub(crate) const READ_BENCH_SERDE_CITYJSON_BORROWED: &str = "serde_cityjson/borrowed";
pub(crate) const READ_BENCH_SERDE_JSON_VALUE: &str = "serde_json::Value";

pub(crate) const WRITE_BENCH_SERDE_CITYJSON_AS_JSON_TO_VALUE: &str =
    "serde_cityjson/as_json_to_value";
pub(crate) const WRITE_BENCH_SERDE_CITYJSON_TO_STRING: &str = "serde_cityjson/to_string";
pub(crate) const WRITE_BENCH_SERDE_CITYJSON_TO_STRING_VALIDATED: &str =
    "serde_cityjson/to_string_validated";
pub(crate) const WRITE_BENCH_SERDE_JSON_TO_STRING: &str = "serde_json::to_string";

#[derive(Clone)]
pub(crate) struct CaseSpec {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) source: PathBuf,
}

pub(crate) struct PreparedReadCase {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) input_json: String,
    pub(crate) input_bytes: u64,
}

pub(crate) struct PreparedWriteCase {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) model: OwnedCityModel,
    pub(crate) canonical_value: Value,
    pub(crate) benchmark_bytes: BTreeMap<String, u64>,
}

#[derive(Serialize)]
struct SuiteMetadata {
    suite: String,
    cases: Vec<CaseMetadata>,
}

#[derive(Serialize)]
struct CaseMetadata {
    id: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    benchmark_bytes: BTreeMap<String, u64>,
}

#[derive(Deserialize)]
struct BenchmarkIndex {
    #[serde(default)]
    generated_cases: Vec<BenchmarkCase>,
    #[serde(default)]
    other_cases: Vec<BenchmarkCase>,
}

#[derive(Clone, Deserialize)]
struct BenchmarkCase {
    id: String,
    layer: String,
    representation: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    output: Option<PathBuf>,
    #[serde(default)]
    artifact_paths: BenchmarkArtifactPaths,
}

#[derive(Clone, Default, Deserialize)]
#[serde(default)]
struct BenchmarkArtifactPaths {
    #[serde(default)]
    source: Option<PathBuf>,
}

pub(crate) fn read_cases() -> Vec<CaseSpec> {
    load_cases()
        .into_iter()
        .filter(|case| case.layer != "invalid")
        .filter(|case| case.representation == "cityjson")
        .filter_map(BenchmarkCase::into_case_spec)
        .collect()
}

pub(crate) fn write_cases() -> Vec<CaseSpec> {
    load_cases()
        .into_iter()
        .filter(|case| case.layer != "invalid")
        .filter(|case| case.representation == "cityjson")
        .filter_map(BenchmarkCase::into_case_spec)
        .collect()
}

pub(crate) fn write_read_suite_metadata(prepared: &[PreparedReadCase]) {
    let metadata = SuiteMetadata {
        suite: "read".to_owned(),
        cases: prepared
            .iter()
            .map(|case| CaseMetadata {
                id: case.name.clone(),
                description: case.description.clone(),
                input_bytes: Some(case.input_bytes),
                benchmark_bytes: BTreeMap::new(),
            })
            .collect(),
    };
    write_suite_metadata("read", &metadata);
}

pub(crate) fn write_write_suite_metadata(prepared: &[PreparedWriteCase]) {
    let metadata = SuiteMetadata {
        suite: "write".to_owned(),
        cases: prepared
            .iter()
            .map(|case| CaseMetadata {
                id: case.name.clone(),
                description: case.description.clone(),
                input_bytes: None,
                benchmark_bytes: case.benchmark_bytes.clone(),
            })
            .collect(),
    };
    write_suite_metadata("write", &metadata);
}

impl BenchmarkCase {
    fn into_case_spec(self) -> Option<CaseSpec> {
        let source = self
            .output
            .or(self.artifact_paths.source)
            .map(|path| resolve_shared_path(path));

        source.map(|source| CaseSpec {
            name: self.id,
            description: self.description,
            source,
        })
    }
}

impl CaseSpec {
    pub(crate) fn prepare_read(&self) -> PreparedReadCase {
        let input_json = read_file(&self.source);
        PreparedReadCase {
            name: self.name.clone(),
            description: self.description.clone(),
            input_bytes: input_json.len() as u64,
            input_json,
        }
    }

    pub(crate) fn prepare_write(&self) -> PreparedWriteCase {
        let input_json = read_file(&self.source);
        let model = from_str_owned(&input_json).unwrap();
        prepare_write_case(self, model)
    }
}

pub(crate) fn real_data_dir() -> PathBuf {
    shared_corpus_root().join("tests").join("data").join("downloaded")
}

fn load_cases() -> Vec<BenchmarkCase> {
    let index = load_benchmark_index();
    index
        .generated_cases
        .into_iter()
        .chain(index.other_cases)
        .collect()
}

fn load_benchmark_index() -> BenchmarkIndex {
    let path = benchmark_index_path();
    let manifest = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read benchmark index {}: {err}", path.display()));
    serde_json::from_str(&manifest)
        .unwrap_or_else(|err| panic!("failed to parse benchmark index {}: {err}", path.display()))
}

fn benchmark_index_path() -> PathBuf {
    let path = std::env::var_os("SERDE_CITYJSON_BENCHMARK_INDEX")
        .map(PathBuf::from)
        .unwrap_or_else(|| shared_corpus_root().join(DEFAULT_BENCHMARK_INDEX_PATH));

    if path.is_absolute() {
        path
    } else {
        shared_corpus_root().join(path)
    }
}

fn shared_corpus_root() -> PathBuf {
    std::env::var_os("SERDE_CITYJSON_SHARED_CORPUS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_SHARED_CORPUS_ROOT))
}

fn resolve_shared_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        shared_corpus_root().join(path)
    }
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn write_suite_metadata(suite: &str, metadata: &SuiteMetadata) {
    let output = serde_json::to_string_pretty(metadata).unwrap();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("results")
        .join(format!("suite_metadata_{suite}.json"));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, output).unwrap();
}

fn prepare_write_case(case: &CaseSpec, model: OwnedCityModel) -> PreparedWriteCase {
    let canonical_value = serde_json::to_value(as_json(&model)).unwrap();
    let serde_json_output = serde_json::to_string(&canonical_value).unwrap();
    let serde_cityjson_output = to_string(&model).unwrap();
    let serde_cityjson_validated_output = to_string_validated(&model).unwrap();

    let benchmark_bytes = BTreeMap::from([
        (
            WRITE_BENCH_SERDE_CITYJSON_AS_JSON_TO_VALUE.to_owned(),
            serde_json_output.len() as u64,
        ),
        (
            WRITE_BENCH_SERDE_CITYJSON_TO_STRING.to_owned(),
            serde_cityjson_output.len() as u64,
        ),
        (
            WRITE_BENCH_SERDE_CITYJSON_TO_STRING_VALIDATED.to_owned(),
            serde_cityjson_validated_output.len() as u64,
        ),
        (
            WRITE_BENCH_SERDE_JSON_TO_STRING.to_owned(),
            serde_json_output.len() as u64,
        ),
    ]);

    PreparedWriteCase {
        name: case.name.clone(),
        description: case.description.clone(),
        model,
        canonical_value,
        benchmark_bytes,
    }
}

impl PreparedWriteCase {
    pub(crate) fn benchmark_bytes(&self, bench_id: &str) -> u64 {
        *self
            .benchmark_bytes
            .get(bench_id)
            .unwrap_or_else(|| panic!("missing benchmark byte count for '{bench_id}'"))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn benchmark_index_loads_cases_for_both_suites() {
        assert!(!super::read_cases().is_empty());
        assert_eq!(super::read_cases().len(), super::write_cases().len());
    }
}
