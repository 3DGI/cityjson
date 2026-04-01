use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use std::collections::BTreeMap;

use pretty_assertions::assert_eq;
use serde::Deserialize;
use serde_json::Value;

use serde_cityjson::{from_str_borrowed, from_str_owned, to_string};

const DEFAULT_SHARED_CORPUS_ROOT: &str = "../cityjson-benchmarks";
const DEFAULT_CORRECTNESS_INDEX_PATH: &str = "artifacts/correctness-index.json";

static CORRECTNESS_CASES: LazyLock<BTreeMap<String, CorrectnessCase>> =
    LazyLock::new(load_correctness_cases);

#[derive(Deserialize)]
struct CorrectnessIndex {
    cases: Vec<CorrectnessCase>,
}

#[derive(Deserialize)]
struct CorrectnessCase {
    id: String,
    layer: String,
    #[serde(default)]
    cityjson_version: Option<String>,
    artifact_paths: CorrectnessArtifactPaths,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct CorrectnessArtifactPaths {
    source: Option<PathBuf>,
}

/// # Panics
///
/// Panics if the file cannot be opened or read.
#[must_use]
pub fn read_to_string(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

impl CorrectnessCase {
    fn source_path(&self) -> PathBuf {
        let Some(source) = self.artifact_paths.source.clone() else {
            panic!(
                "correctness case '{}' does not define artifact_paths.source",
                self.id
            );
        };
        resolve_shared_path(source)
    }
}

pub fn conformance_case_input(case_id: &str) -> String {
    let case = correctness_case(case_id);
    assert_eq!(
        case.layer, "conformance",
        "correctness case '{case_id}' is not a conformance fixture"
    );
    assert_eq!(
        case.cityjson_version.as_deref(),
        Some("2.0"),
        "correctness case '{case_id}' is not a CityJSON 2.0 fixture"
    );
    read_to_string(case.source_path())
}

fn correctness_case(case_id: &str) -> &'static CorrectnessCase {
    CORRECTNESS_CASES.get(case_id).unwrap_or_else(|| {
        panic!(
            "missing correctness case '{case_id}' in {}",
            correctness_index_path().display()
        )
    })
}

fn load_correctness_cases() -> BTreeMap<String, CorrectnessCase> {
    let path = correctness_index_path();
    let manifest = read_to_string(&path);
    let index: CorrectnessIndex = serde_json::from_str(&manifest).unwrap_or_else(|err| {
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
    let path = env::var_os("SERDE_CITYJSON_CORRECTNESS_INDEX")
        .map(PathBuf::from)
        .unwrap_or_else(|| shared_corpus_root().join(DEFAULT_CORRECTNESS_INDEX_PATH));

    if path.is_absolute() {
        path
    } else {
        shared_corpus_root().join(path)
    }
}

fn shared_corpus_root() -> PathBuf {
    env::var_os("SERDE_CITYJSON_SHARED_CORPUS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_SHARED_CORPUS_ROOT)
        })
}

fn resolve_shared_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        shared_corpus_root().join(path)
    }
}

/// # Panics
///
/// Panics if serialization or deserialization fails.
#[must_use]
pub fn roundtrip_value(input: &Value) -> Value {
    let model = from_str_owned(&serde_json::to_string(input).unwrap()).unwrap();
    serde_json::from_str(&to_string(&model).unwrap()).unwrap()
}

/// # Panics
///
/// Panics if serialization or deserialization fails.
#[must_use]
pub fn roundtrip_value_borrowed(input: &Value) -> Value {
    let s = serde_json::to_string(input).unwrap();
    let model = from_str_borrowed(&s).unwrap();
    serde_json::from_str(&to_string(&model).unwrap()).unwrap()
}

/// Assert that the data retains the same content after an adapter deserialize-serialize roundtrip.
///
/// # Panics
///
/// Panics if the roundtrip fails or the result does not match the input.
pub fn assert_eq_roundtrip(json_input: &str) {
    let expected: Value = serde_json::from_str(json_input).unwrap();
    let result = roundtrip_value(&expected);
    assert_eq!(result, expected);
}

/// Assert that the data retains the same content after a borrowed-mode roundtrip.
///
/// # Panics
///
/// Panics if the roundtrip fails or the result does not match the input.
pub fn assert_eq_roundtrip_borrowed(json_input: &str) {
    let expected: Value = serde_json::from_str(json_input).unwrap();
    let result = roundtrip_value_borrowed(&expected);
    assert_eq!(result, expected);
}

/// Assert that owned and borrowed modes both roundtrip the input correctly and produce
/// identical output.
///
/// # Panics
///
/// Panics if the roundtrip fails or the results do not match.
pub fn assert_eq_roundtrip_parity(json_input: &str) {
    let expected: Value = serde_json::from_str(json_input).unwrap();
    let owned = roundtrip_value(&expected);
    let borrowed = roundtrip_value_borrowed(&expected);
    assert_eq!(owned, expected, "owned roundtrip mismatch");
    assert_eq!(borrowed, expected, "borrowed roundtrip mismatch");
    assert_eq!(owned, borrowed, "owned/borrowed output mismatch");
}
