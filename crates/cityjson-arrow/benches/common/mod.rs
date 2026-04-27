#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use cityjson_arrow::{ExportOptions, write_stream};
use cityjson_json::v2_0::{ReadOptions, WriteOptions, read_model, to_vec};
use serde::{Deserialize, Serialize};

const DEFAULT_BENCHMARK_INDEX_PATH: &str = "artifacts/benchmark-index.json";

pub(crate) const READ_BENCH_STREAM: &str = "cityjson-arrow/stream_read";
pub(crate) const READ_BENCH_JSON: &str = "cityjson-json/owned";

pub(crate) const WRITE_BENCH_STREAM: &str = "cityjson-arrow/stream_write";
pub(crate) const WRITE_BENCH_JSON: &str = "cityjson-json/to_vec";

#[derive(Clone)]
pub(crate) struct CaseSpec {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) source: PathBuf,
}

pub(crate) struct PreparedReadCase {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) stream_bytes: Vec<u8>,
    pub(crate) json_bytes: String,
    pub(crate) stream_input_bytes: u64,
    pub(crate) json_input_bytes: u64,
}

pub(crate) struct PreparedWriteCase {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) model: cityjson::v2_0::OwnedCityModel,
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
    artifacts: Vec<BenchmarkArtifact>,
}

#[derive(Clone, Deserialize)]
struct BenchmarkArtifact {
    representation: String,
    path: PathBuf,
}

pub(crate) fn read_cases() -> Vec<CaseSpec> {
    load_cases()
        .into_iter()
        .filter(|case| case.layer != "invalid")
        .filter(|case| case.representation == "cityjson")
        .flat_map(BenchmarkCase::into_case_specs)
        .collect()
}

pub(crate) fn write_cases() -> Vec<CaseSpec> {
    load_cases()
        .into_iter()
        .filter(|case| case.layer != "invalid")
        .filter(|case| case.representation == "cityjson")
        .flat_map(BenchmarkCase::into_case_specs)
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
                input_bytes: Some(case.stream_input_bytes),
                benchmark_bytes: BTreeMap::from([
                    (READ_BENCH_STREAM.to_owned(), case.stream_input_bytes),
                    (READ_BENCH_JSON.to_owned(), case.json_input_bytes),
                ]),
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

impl PreparedWriteCase {
    pub(crate) fn benchmark_bytes(&self, bench_id: &str) -> u64 {
        self.benchmark_bytes.get(bench_id).copied().unwrap_or(0)
    }
}

impl BenchmarkCase {
    fn into_case_specs(self) -> Vec<CaseSpec> {
        let BenchmarkCase {
            id,
            description,
            artifacts,
            ..
        } = self;

        let cityjson_artifacts: Vec<_> = artifacts
            .into_iter()
            .filter(|artifact| artifact.representation == "cityjson")
            .collect();
        let use_suffix = cityjson_artifacts.len() > 1;

        cityjson_artifacts
            .into_iter()
            .map(|artifact| CaseSpec {
                name: case_spec_name(&id, &artifact, use_suffix),
                description: description.clone(),
                source: resolve_shared_path(artifact.path),
            })
            .collect()
    }
}

impl CaseSpec {
    pub(crate) fn prepare_read(&self) -> PreparedReadCase {
        let json_bytes = read_file(&self.source);
        let model = read_model(json_bytes.as_bytes(), &ReadOptions::default())
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", self.source.display()));
        let mut stream_bytes = Vec::new();
        write_stream(&mut stream_bytes, &model, &ExportOptions::default())
            .unwrap_or_else(|err| panic!("failed to encode Arrow IPC for {}: {err}", self.name));
        let stream_input_bytes = stream_bytes.len() as u64;
        let json_input_bytes = json_bytes.len() as u64;
        PreparedReadCase {
            name: self.name.clone(),
            description: self.description.clone(),
            stream_bytes,
            json_bytes,
            stream_input_bytes,
            json_input_bytes,
        }
    }

    pub(crate) fn prepare_write(&self) -> PreparedWriteCase {
        let json_bytes = read_file(&self.source);
        let model = read_model(json_bytes.as_bytes(), &ReadOptions::default())
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", self.source.display()));

        let mut stream_output = Vec::new();
        write_stream(&mut stream_output, &model, &ExportOptions::default())
            .unwrap_or_else(|err| panic!("failed to encode Arrow IPC for {}: {err}", self.name));

        let json_output = to_vec(&model, &WriteOptions::default())
            .unwrap_or_else(|err| panic!("failed to encode JSON for {}: {err}", self.name));

        let benchmark_bytes = BTreeMap::from([
            (WRITE_BENCH_STREAM.to_owned(), stream_output.len() as u64),
            (WRITE_BENCH_JSON.to_owned(), json_output.len() as u64),
        ]);

        PreparedWriteCase {
            name: self.name.clone(),
            description: self.description.clone(),
            model,
            benchmark_bytes,
        }
    }
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
    if let Some(path) = std::env::var_os("CITYJSON_ARROW_BENCHMARK_INDEX").map(PathBuf::from) {
        if path.is_absolute() {
            return path;
        }
        return shared_corpus_root().join(path);
    }
    shared_corpus_root().join(DEFAULT_BENCHMARK_INDEX_PATH)
}

fn shared_corpus_root() -> PathBuf {
    std::env::var_os("CITYJSON_SHARED_CORPUS_ROOT")
        .map_or_else(workspace_corpus_root, PathBuf::from)
}

fn resolve_shared_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        shared_corpus_root().join(path)
    }
}

fn workspace_corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../cityjson-corpus")
}

fn case_spec_name(case_id: &str, artifact: &BenchmarkArtifact, use_suffix: bool) -> String {
    if !use_suffix {
        return case_id.to_owned();
    }
    let suffix = artifact
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            name.trim_end_matches(".city.json")
                .trim_end_matches(".json")
        })
        .filter(|suffix| !suffix.is_empty())
        .unwrap_or("artifact");
    format!("{case_id}__{suffix}")
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
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
