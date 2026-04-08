#![allow(dead_code)]

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use cityjson::v2_0::OwnedCityModel;
use serde::Deserialize;
use serde_cityjson::{as_json, from_str_owned};
use serde_json::Value as JsonValue;

const DEFAULT_CORRECTNESS_INDEX_PATH: &str = "artifacts/correctness-index.json";
const SHARED_CORPUS_DIRNAME: &str = "cityjson-benchmarks";

static CORRECTNESS_CASES: LazyLock<BTreeMap<String, CorrectnessCase>> =
    LazyLock::new(load_correctness_cases);

pub(crate) struct PreparedCorrectnessCase {
    pub(crate) id: String,
    pub(crate) model: OwnedCityModel,
}

#[derive(Deserialize)]
struct CorrectnessIndex {
    cases: Vec<CorrectnessCase>,
}

#[derive(Clone, Deserialize)]
struct CorrectnessCase {
    id: String,
    layer: String,
    representation: String,
    #[serde(default = "default_correctness_class")]
    correctness_class: String,
    #[serde(default)]
    cityjson_version: Option<String>,
    artifact_paths: CorrectnessArtifactPaths,
}

#[derive(Clone, Default, Deserialize)]
#[serde(default)]
struct CorrectnessArtifactPaths {
    source: Option<PathBuf>,
}

fn default_correctness_class() -> String {
    "normative".to_string()
}

impl CorrectnessCase {
    fn is_normative_conformance(&self) -> bool {
        self.layer == "conformance"
            && self.correctness_class == "normative"
            && self.cityjson_version.as_deref() == Some("2.0")
            && matches!(self.representation.as_str(), "cityjson" | "cityjsonfeature")
    }

    fn source_path(&self) -> PathBuf {
        let Some(source) = self.artifact_paths.source.clone() else {
            panic!(
                "correctness case '{}' does not define artifact_paths.source",
                self.id
            );
        };
        resolve_shared_path(source)
    }

    fn prepare(&self) -> PreparedCorrectnessCase {
        let source = self.source_path();
        let input = read_to_string(&source);
        let model = from_str_owned(&input)
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", source.display()));

        PreparedCorrectnessCase {
            id: self.id.clone(),
            model,
        }
    }
}

pub(crate) fn load_named_normative_conformance_case(case_id: &str) -> PreparedCorrectnessCase {
    let case = correctness_case(case_id);
    assert!(
        case.is_normative_conformance(),
        "correctness case '{}' is not a normative CityJSON 2.0 conformance fixture",
        case_id
    );
    case.prepare()
}

pub(crate) fn load_normative_conformance_cases() -> Vec<PreparedCorrectnessCase> {
    CORRECTNESS_CASES
        .values()
        .filter(|case| case.is_normative_conformance())
        .map(CorrectnessCase::prepare)
        .collect()
}

pub(crate) fn normalized_json(model: &OwnedCityModel) -> JsonValue {
    serde_json::to_value(as_json(model)).unwrap()
}

fn correctness_case(case_id: &str) -> &'static CorrectnessCase {
    CORRECTNESS_CASES.get(case_id).unwrap_or_else(|| {
        panic!(
            "missing correctness case '{}' in {}",
            case_id,
            correctness_index_path().display()
        )
    })
}

fn load_correctness_cases() -> BTreeMap<String, CorrectnessCase> {
    let path = correctness_index_path();
    let manifest = read_to_string(&path);
    let index = serde_json::from_str::<CorrectnessIndex>(&manifest).unwrap_or_else(|err| {
        panic!(
            "failed to parse correctness index {}: {err}",
            path.display()
        )
    });
    index
        .cases
        .into_iter()
        .map(|case| (case.id.clone(), case))
        .collect()
}

fn correctness_index_path() -> PathBuf {
    let path = env::var_os("CITYARROW_CORRECTNESS_INDEX")
        .or_else(|| env::var_os("CITYPARQUET_CORRECTNESS_INDEX"))
        .or_else(|| env::var_os("SERDE_CITYJSON_CORRECTNESS_INDEX"))
        .map_or_else(
            || shared_corpus_root().join(DEFAULT_CORRECTNESS_INDEX_PATH),
            PathBuf::from,
        );

    if path.is_absolute() {
        path
    } else {
        shared_corpus_root().join(path)
    }
}

fn shared_corpus_root() -> PathBuf {
    let path = env::var_os("CITYARROW_SHARED_CORPUS_ROOT")
        .or_else(|| env::var_os("CITYPARQUET_SHARED_CORPUS_ROOT"))
        .or_else(|| env::var_os("SERDE_CITYJSON_SHARED_CORPUS_ROOT"))
        .map_or_else(discover_shared_corpus_root, PathBuf::from);

    if path.is_absolute() {
        path
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
    }
}

fn discover_shared_corpus_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for ancestor in manifest_dir.ancestors() {
        let candidate = ancestor.join(SHARED_CORPUS_DIRNAME);
        if candidate.join(DEFAULT_CORRECTNESS_INDEX_PATH).is_file() {
            return candidate;
        }
    }

    panic!(
        "failed to locate '{}' relative to {}",
        SHARED_CORPUS_DIRNAME,
        manifest_dir.display()
    );
}

fn resolve_shared_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        shared_corpus_root().join(path)
    }
}

fn read_to_string(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}
