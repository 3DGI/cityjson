#![allow(dead_code)]

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use cityjson::v2_0::OwnedCityModel;
use cityjson_json::{as_json, from_str_owned};
use serde::Deserialize;
use serde_json::Value as JsonValue;

const DEFAULT_CORRECTNESS_INDEX_PATH: &str = "artifacts/correctness-index.json";
const SHARED_CORPUS_DIRNAME: &str = "cityjson-benchmarks";

static CORRECTNESS_CASES: LazyLock<BTreeMap<String, CorrectnessCase>> =
    LazyLock::new(load_correctness_cases);

pub(crate) struct PreparedCorrectnessCase {
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
    #[serde(default)]
    cityjson_version: Option<String>,
    artifact_paths: CorrectnessArtifactPaths,
}

#[derive(Clone, Default, Deserialize)]
#[serde(default)]
struct CorrectnessArtifactPaths {
    source: Option<PathBuf>,
    generated: Option<PathBuf>,
    profile: Option<PathBuf>,
}

impl CorrectnessCase {
    fn is_conformance_case(&self) -> bool {
        self.layer == "conformance" && self.cityjson_version.as_deref() == Some("2.0")
    }

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

    fn prepare(&self) -> PreparedCorrectnessCase {
        let source = self.input_path();
        let input = read_to_string(&source);
        let model = from_str_owned(&input)
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", source.display()));

        PreparedCorrectnessCase { model }
    }
}

pub(crate) fn load_named_conformance_case(case_id: &str) -> PreparedCorrectnessCase {
    let case = correctness_case(case_id);
    assert!(
        case.is_conformance_case(),
        "correctness case '{case_id}' is not a CityJSON 2.0 conformance fixture",
    );
    case.prepare()
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
    let path = env::var_os("CITYJSON_ARROW_CORRECTNESS_INDEX")
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
    let path = env::var_os("CITYJSON_ARROW_SHARED_CORPUS_ROOT")
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

fn generated_temp_path(case_id: &str) -> PathBuf {
    let mut path = env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|err| panic!("system clock error: {err}"))
        .as_nanos();
    path.push(format!(
        "cityjson-benchmarks-{case_id}-{pid}-{stamp}.city.json",
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
    env::var_os("CITYJSON_FAKE_CARGO_MANIFEST")
        .or_else(|| env::var_os("CJFAKE_CARGO_MANIFEST"))
        .map_or_else(default_cityjson_fake_cargo_manifest, PathBuf::from)
}

fn cityjson_fake_manifest_schema() -> PathBuf {
    env::var_os("CITYJSON_FAKE_MANIFEST_SCHEMA")
        .or_else(|| env::var_os("CJFAKE_MANIFEST_SCHEMA"))
        .map_or_else(default_cityjson_fake_manifest_schema, PathBuf::from)
}

fn default_cityjson_fake_cargo_manifest() -> PathBuf {
    let shared_corpus_root = shared_corpus_root();
    let workspace_root = shared_corpus_root.parent().unwrap();
    let cityjson_fake = workspace_root.join("cityjson-fake/Cargo.toml");
    if cityjson_fake.is_file() {
        cityjson_fake
    } else {
        workspace_root.join("cjfake/Cargo.toml")
    }
}

fn default_cityjson_fake_manifest_schema() -> PathBuf {
    let shared_corpus_root = shared_corpus_root();
    let workspace_root = shared_corpus_root.parent().unwrap();
    let cityjson_fake =
        workspace_root.join("cityjson-fake/src/data/cityjson-fake-manifest.schema.json");
    if cityjson_fake.is_file() {
        cityjson_fake
    } else {
        workspace_root.join("cjfake/src/data/cjfake-manifest.schema.json")
    }
}
