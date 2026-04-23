use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use std::collections::BTreeMap;

use pretty_assertions::assert_eq;
use serde::Deserialize;
use serde_json::Value;

use cityjson_json::{OwnedCityModel, ReadOptions, WriteOptions, read_feature, read_model, to_vec};

const DEFAULT_CORRECTNESS_INDEX_PATH: &str = "artifacts/correctness-index.json";
const CONFORMANCE_SCHEMA_VERSION: &str = "2.0";

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
    generated: Option<PathBuf>,
    profile: Option<PathBuf>,
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
    fn input_path(&self) -> PathBuf {
        let Some(source) = self.artifact_paths.source.clone() else {
            return self.generated_input_path();
        };
        resolve_shared_path(source)
    }

    fn generated_input_path(&self) -> PathBuf {
        if let Some(generated) = self.artifact_paths.generated.clone() {
            let generated_path = resolve_shared_path(generated);
            if generated_path.is_file() {
                return generated_path;
            }
        }

        let Some(profile) = self.artifact_paths.profile.clone() else {
            panic!(
                "correctness case '{}' does not define artifact_paths.source or artifact_paths.profile",
                self.id
            );
        };
        let profile_path = resolve_shared_path(profile);
        let output_path = generated_temp_path(&self.id);
        generate_profile_artifact(&profile_path, &output_path);
        output_path
    }
}

#[must_use]
pub fn conformance_case_input(case_id: &str) -> String {
    let case = correctness_case(case_id);
    assert_eq!(
        case.layer, "conformance",
        "correctness case '{case_id}' is not a conformance fixture"
    );
    assert_eq!(
        case.cityjson_version.as_deref(),
        Some(CONFORMANCE_SCHEMA_VERSION),
        "correctness case '{case_id}' is not a CityJSON 2.0 fixture"
    );
    read_to_string(case.input_path())
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
    if let Some(path) = env::var_os("CITYJSON_JSON_CORRECTNESS_INDEX").map(PathBuf::from) {
        if path.is_absolute() {
            return path;
        }

        return shared_corpus_root().join(path);
    }

    shared_corpus_root().join(DEFAULT_CORRECTNESS_INDEX_PATH)
}

fn shared_corpus_root() -> PathBuf {
    env::var_os("CITYJSON_SHARED_CORPUS_ROOT").map_or_else(workspace_corpus_root, PathBuf::from)
}

fn resolve_shared_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        shared_corpus_root().join(path)
    }
}

fn workspace_corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../cityjson-corpus")
}

fn generated_temp_path(case_id: &str) -> PathBuf {
    let mut path = env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|err| panic!("system clock error: {err}"))
        .as_nanos();
    path.push(format!(
        "cityjson-corpus-{case_id}-{pid}-{stamp}.city.json",
        pid = std::process::id()
    ));
    path
}

fn generate_profile_artifact(profile: &Path, output: &Path) {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
    }

    let cargo_manifest = cityjson_fake_cargo_manifest();
    let schema_path = cityjson_fake_manifest_schema();
    let status = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(&cargo_manifest)
        .arg("--")
        .arg("--manifest")
        .arg(profile)
        .arg("--schema")
        .arg(&schema_path)
        .arg("--output")
        .arg(output)
        .status()
        .unwrap_or_else(|err| panic!("failed to run cityjson-fake via cargo: {err}"));

    assert!(
        status.success(),
        "cityjson-fake failed to generate {} using {}",
        output.display(),
        profile.display()
    );
}

fn cityjson_fake_cargo_manifest() -> PathBuf {
    env::var_os("CJFAKE_CARGO_MANIFEST").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../cityjson-fake/Cargo.toml"),
        PathBuf::from,
    )
}

fn cityjson_fake_manifest_schema() -> PathBuf {
    env::var_os("CJFAKE_MANIFEST_SCHEMA").map_or_else(
        || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../cityjson-fake/src/data/cityjson-fake-manifest.schema.json")
        },
        PathBuf::from,
    )
}

/// # Panics
///
/// Panics if serialization or deserialization fails.
#[must_use]
pub fn roundtrip_value(input: &Value) -> Value {
    let input_bytes = serde_json::to_vec(input).unwrap();
    let model = match input.get("type").and_then(Value::as_str) {
        Some("CityJSONFeature") => read_feature(&input_bytes, &ReadOptions::default()).unwrap(),
        _ => read_model_bytes(&input_bytes),
    };
    write_value(&model)
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

#[must_use]
pub fn read_model_str(input: &str) -> OwnedCityModel {
    read_model_bytes(input.as_bytes())
}

/// # Panics
///
/// Panics if the payload does not parse as a valid `CityJSON` document.
#[must_use]
pub fn read_model_bytes(input: &[u8]) -> OwnedCityModel {
    read_model(input, &ReadOptions::default()).unwrap()
}

/// # Panics
///
/// Panics if the model cannot be serialized.
#[must_use]
pub fn write_bytes(model: &OwnedCityModel) -> Vec<u8> {
    to_vec(model, &WriteOptions::default()).unwrap()
}

/// # Panics
///
/// Panics if serialization or JSON decoding fails.
#[must_use]
pub fn write_value(model: &OwnedCityModel) -> Value {
    serde_json::from_slice(&write_bytes(model)).unwrap()
}
