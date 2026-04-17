use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

use cityjson::v2_0::OwnedCityModel;
use cityjson_json::{ReadOptions, WriteOptions, read_feature, read_model, to_vec};
use serde::Deserialize;
use serde_json::Value;

const CORRECTNESS_INDEX_PATH: &str = "artifacts/correctness-index.json";

pub struct ConformanceCase {
    pub model: OwnedCityModel,
}

#[derive(Deserialize)]
struct CorrectnessIndex {
    cases: Vec<CorrectnessEntry>,
}

#[derive(Deserialize)]
struct CorrectnessEntry {
    id: String,
    layer: String,
    #[serde(default)]
    cityjson_version: Option<String>,
    #[serde(default)]
    artifact_paths: ArtifactPaths,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct ArtifactPaths {
    source: Option<PathBuf>,
    generated: Option<PathBuf>,
    profile: Option<PathBuf>,
}

static CORRECTNESS_CASES: LazyLock<BTreeMap<String, CorrectnessEntry>> =
    LazyLock::new(load_correctness_cases);

pub fn load_named_conformance_case(case_id: &str) -> ConformanceCase {
    let entry = CORRECTNESS_CASES
        .get(case_id)
        .unwrap_or_else(|| panic!("missing conformance case '{case_id}'"));
    assert_eq!(
        entry.layer, "conformance",
        "correctness case '{case_id}' is not a conformance fixture"
    );
    assert_eq!(
        entry.cityjson_version.as_deref(),
        Some("2.0"),
        "correctness case '{case_id}' is not a CityJSON 2.0 fixture"
    );

    let path = resolve_artifact_path(case_id, entry);
    let bytes =
        fs::read(&path).unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    let model = if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
        read_feature(&bytes, &ReadOptions::default())
            .unwrap_or_else(|err| panic!("failed to parse feature {}: {err}", path.display()))
    } else {
        read_model(&bytes, &ReadOptions::default())
            .unwrap_or_else(|err| panic!("failed to parse model {}: {err}", path.display()))
    };

    ConformanceCase { model }
}

pub fn normalized_json(model: &OwnedCityModel) -> Value {
    let bytes = to_vec(model, &WriteOptions::default())
        .unwrap_or_else(|err| panic!("failed to serialize model: {err}"));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|err| panic!("failed to parse serialized model: {err}"))
}

fn resolve_artifact_path(case_id: &str, entry: &CorrectnessEntry) -> PathBuf {
    if let Some(source) = &entry.artifact_paths.source {
        return corpus_root().join(source);
    }

    if let Some(generated) = &entry.artifact_paths.generated {
        let generated_path = corpus_root().join(generated);
        if generated_path.is_file() {
            return generated_path;
        }

        if let Some(profile) = &entry.artifact_paths.profile {
            let profile_path = corpus_root().join(profile);
            generate_profile_artifact(&profile_path, &generated_path);
            return generated_path;
        }
    }

    panic!(
        "conformance case '{case_id}' has no resolvable artifact: \
        no source, no generated file, and no profile to generate from"
    )
}

fn generate_profile_artifact(profile: &Path, output: &Path) {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
    }

    let cargo_manifest = env::var_os("CJFAKE_CARGO_MANIFEST").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../cityjson-fake/Cargo.toml"),
        PathBuf::from,
    );

    let schema_path = env::var_os("CJFAKE_MANIFEST_SCHEMA").map_or_else(
        || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../cityjson-fake/src/data/cityjson-fake-manifest.schema.json")
        },
        PathBuf::from,
    );

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

fn corpus_root() -> PathBuf {
    env::var_os("CITYJSON_PARQUET_SHARED_CORPUS_ROOT").map_or_else(
        || panic!("set CITYJSON_PARQUET_SHARED_CORPUS_ROOT to your cityjson-corpus checkout"),
        PathBuf::from,
    )
}

fn load_correctness_cases() -> BTreeMap<String, CorrectnessEntry> {
    let path = corpus_root().join(CORRECTNESS_INDEX_PATH);
    let manifest = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let index: CorrectnessIndex = serde_json::from_str(&manifest)
        .unwrap_or_else(|err| panic!("failed to parse correctness index: {err}"));
    index.cases.into_iter().map(|c| (c.id.clone(), c)).collect()
}
