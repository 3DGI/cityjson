#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use cityarrow::{from_parts, read_package_dir, to_parts, write_package_dir};
use cityjson::v2_0::OwnedCityModel;
use serde::Deserialize;
use serde_cityjson::{from_str_owned, to_string_validated};
use tempfile::Builder;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub purpose: String,
    pub cases: Vec<Case>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CaseKind {
    Real,
    Synthetic,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Case {
    pub id: String,
    pub kind: CaseKind,
    pub suites: Vec<String>,
    pub borrowed: bool,
    pub description: String,
    pub source: Option<Source>,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub profile_path: Option<PathBuf>,
    #[serde(default)]
    pub intent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Source {
    pub path: PathBuf,
}

#[must_use]
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[must_use]
pub fn sibling_serde_cityjson_root() -> PathBuf {
    workspace_root()
        .parent()
        .expect("cityarrow lives inside Development/")
        .join("serde_cityjson")
}

#[must_use]
pub fn manifest_path() -> PathBuf {
    workspace_root().join("tests/data/generated/manifest.json")
}

#[must_use]
pub fn load_manifest() -> Manifest {
    let manifest_json =
        fs::read_to_string(manifest_path()).expect("failed to read acceptance manifest");
    serde_json::from_str(&manifest_json).expect("failed to parse acceptance manifest")
}

#[must_use]
pub fn resolve_case_path(case: &Case) -> PathBuf {
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

pub fn cjval_validate(path: &Path) {
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

#[must_use]
pub fn roundtrip_via_cityarrow(model: OwnedCityModel) -> OwnedCityModel {
    let parts = to_parts(&model).expect("cityarrow to_parts should succeed");
    let dir = tempfile::tempdir().expect("cityarrow tempdir should be created");
    write_package_dir(dir.path(), &parts).expect("cityarrow package write should succeed");
    let parts = read_package_dir(dir.path()).expect("cityarrow package read should succeed");
    from_parts(&parts).expect("cityarrow from_parts should succeed")
}

pub fn assert_case_roundtrip(case: &Case) {
    let input_path = resolve_case_path(case);
    let input_json = fs::read_to_string(&input_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", input_path.display()));

    let model = from_str_owned(&input_json)
        .unwrap_or_else(|error| panic!("serde_cityjson failed for {}: {error}", case.id));
    let model = roundtrip_via_cityarrow(model);

    let output_json = to_string_validated(&model).unwrap_or_else(|error| {
        panic!("serde_cityjson validation failed for {}: {error}", case.id)
    });

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

#[must_use]
pub fn acceptance_cases() -> Vec<Case> {
    let manifest = load_manifest();

    assert_eq!(
        manifest.version, 2,
        "unexpected acceptance manifest version"
    );
    assert!(
        manifest.purpose.starts_with("Benchmark profile catalog"),
        "acceptance manifest purpose should match the serde_cityjson catalog"
    );

    manifest
        .cases
        .into_iter()
        .filter(|case| case.kind == CaseKind::Real)
        .filter(|case| case.suites.iter().any(|suite| suite == "write"))
        .collect()
}
