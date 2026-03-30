use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use serde_cityjson::{from_str_owned, to_string_validated};
use tempfile::Builder;

#[derive(Debug, Deserialize)]
struct Manifest {
    version: u32,
    purpose: String,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum CaseKind {
    Real,
    Synthetic,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    kind: CaseKind,
    suites: Vec<String>,
    borrowed: bool,
    description: String,
    source: Option<Source>,
    #[serde(default)]
    seed: Option<u64>,
    #[serde(default)]
    profile_path: Option<PathBuf>,
    #[serde(default)]
    intent: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Source {
    path: PathBuf,
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn sibling_serde_cityjson_root() -> PathBuf {
    workspace_root()
        .parent()
        .expect("cityarrow lives inside Development/")
        .join("serde_cityjson")
}

fn manifest_path() -> PathBuf {
    workspace_root().join("tests/data/generated/manifest.json")
}

fn load_manifest() -> Manifest {
    let manifest_json =
        fs::read_to_string(manifest_path()).expect("failed to read acceptance manifest");
    serde_json::from_str(&manifest_json).expect("failed to parse acceptance manifest")
}

fn resolve_case_path(case: &Case) -> PathBuf {
    let source = case
        .source
        .as_ref()
        .unwrap_or_else(|| panic!("case {} is missing a source path", case.id));

    let direct = workspace_root().join(&source.path);
    if direct.exists() {
        return direct;
    }

    let sibling = sibling_serde_cityjson_root().join(&source.path);
    if sibling.exists() {
        return sibling;
    }

    panic!(
        "could not resolve source path for case {}: {}",
        case.id,
        source.path.display()
    );
}

fn cjval_validate(path: &Path) {
    let output = Command::new("cjval")
        .args(["-q", path.to_str().expect("non-utf8 temp path")])
        .output()
        .unwrap_or_else(|error| panic!("failed to execute cjval for {}: {error}", path.display()));

    assert!(
        output.status.success(),
        "cjval rejected {}:\nstdout:\n{}\nstderr:\n{}",
        path.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_case_roundtrip_identity(case: &Case) {
    let input_path = resolve_case_path(case);
    let input_json = fs::read_to_string(&input_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", input_path.display()));

    let model = from_str_owned(&input_json)
        .unwrap_or_else(|error| panic!("serde_cityjson failed for {}: {error}", case.id));

    let output_json = to_string_validated(&model)
        .unwrap_or_else(|error| panic!("serde_cityjson validation failed for {}: {error}", case.id));

    let mut temp = Builder::new()
        .prefix(&format!("cityarrow-{}", case.id.replace(' ', "_")))
        .suffix(".city.json")
        .tempfile()
        .unwrap_or_else(|error| panic!("failed to create temp output for {}: {error}", case.id));
    temp.write_all(output_json.as_bytes())
        .unwrap_or_else(|error| panic!("failed to write output for {}: {error}", case.id));
    temp.flush()
        .unwrap_or_else(|error| panic!("failed to flush output for {}: {error}", case.id));

    cjval_validate(temp.path());
}

fn acceptance_cases() -> Vec<Case> {
    let manifest = load_manifest();

    assert_eq!(manifest.version, 2, "unexpected acceptance manifest version");
    assert!(
        manifest
            .purpose
            .starts_with("Benchmark profile catalog"),
        "acceptance manifest purpose should match the serde_cityjson catalog"
    );

    manifest
        .cases
        .into_iter()
        .filter(|case| case.kind == CaseKind::Real)
        .filter(|case| case.suites.iter().any(|suite| suite == "write"))
        .collect()
}

#[test]
fn manifest_layout_matches_serde_cityjson_real_cases() {
    let cases = acceptance_cases();
    let ids = cases.iter().map(|case| case.id.as_str()).collect::<Vec<_>>();

    assert!(ids.contains(&"3DBAG"));
    assert!(ids.contains(&"3D Basisvoorziening"));

    for case in &cases {
        assert!(
            case.source.is_some(),
            "real acceptance case {} should have a source path",
            case.id
        );
        assert!(
            case.description.len() > 10,
            "case {} should keep the serde_cityjson description",
            case.id
        );
    }
}

#[test]
#[ignore = "expensive real-data acceptance gate"]
fn real_datasets_validate_with_serde_cityjson_and_cjval() {
    for case in acceptance_cases() {
        assert_case_roundtrip_identity(&case);
    }
}
