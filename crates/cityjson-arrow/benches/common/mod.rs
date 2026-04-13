use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use cityjson::v2_0::OwnedCityModel;
use serde::Deserialize;
use serde_cityjson::from_str_owned;

const DEFAULT_SHARED_CORPUS_ROOT: &str = "../cityjson-benchmarks";
const DEFAULT_BENCHMARK_INDEX_PATH: &str = "artifacts/benchmark-index.json";

pub(crate) struct SharedWriteCase {
    pub(crate) name: String,
    pub(crate) input_bytes: u64,
    pub(crate) model: OwnedCityModel,
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
    output: Option<PathBuf>,
}

pub(crate) fn load_named_write_cases(case_ids: &[&str]) -> Vec<SharedWriteCase> {
    let cases_by_id = load_cases()
        .into_iter()
        .filter(|case| case.layer == "workload")
        .filter(|case| case.representation == "cityjson")
        .map(|case| (case.id.clone(), case))
        .collect::<HashMap<_, _>>();

    case_ids
        .iter()
        .map(|case_id| {
            let case = cases_by_id.get(*case_id).unwrap_or_else(|| {
                panic!(
                    "shared corpus case '{}' not found in {}",
                    case_id,
                    benchmark_index_path().display()
                )
            });
            case.prepare_write()
        })
        .collect()
}

impl BenchmarkCase {
    fn prepare_write(&self) -> SharedWriteCase {
        let source = self.output.clone().map_or_else(
            || {
                panic!("case '{}' does not have an output path", self.id);
            },
            resolve_shared_path,
        );
        let input_json = read_file(&source);
        let input_bytes = u64::try_from(input_json.len()).expect("input size fits into u64");
        let model = from_str_owned(&input_json)
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", source.display()));

        SharedWriteCase {
            name: self.id.clone(),
            input_bytes,
            model,
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
    let path = std::env::var_os("CITYARROW_BENCHMARK_INDEX").map_or_else(
        || shared_corpus_root().join(DEFAULT_BENCHMARK_INDEX_PATH),
        PathBuf::from,
    );

    if path.is_absolute() {
        path
    } else {
        shared_corpus_root().join(path)
    }
}

fn shared_corpus_root() -> PathBuf {
    std::env::var_os("CITYARROW_SHARED_CORPUS_ROOT").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_SHARED_CORPUS_ROOT),
        PathBuf::from,
    )
}

fn resolve_shared_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        shared_corpus_root().join(path)
    }
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}
