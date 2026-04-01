#![allow(dead_code)]

use std::fs;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use arrow::record_batch::RecordBatch;
use cityarrow::schema::{CityModelArrowParts, PackageTableEncoding};
use cityarrow::{from_parts, read_package_ipc_dir, to_parts, write_package_ipc_dir};
use cityjson::v2_0::OwnedCityModel;
use cityparquet::{read_package_dir, write_package_dir};
use serde::Deserialize;
use serde_cityjson::{from_str_owned, to_string_validated};
use serde_json::Value as JsonValue;
use tempfile::{Builder, NamedTempFile};

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
    roundtrip_via_cityarrow_with_encoding(model, PackageTableEncoding::Parquet)
}

#[must_use]
pub fn normalized_json(model: &OwnedCityModel) -> JsonValue {
    serde_json::from_str(
        &to_string_validated(model).expect("CityJSON serialization should succeed"),
    )
    .expect("serialized CityJSON should parse as JSON")
}

fn memory_trace_enabled() -> bool {
    std::env::var_os("CITYARROW_REAL_DATA_MEMORY_TRACE").is_some()
}

fn log_memory_phase(label: &str) {
    if !memory_trace_enabled() {
        return;
    }

    let Ok(status) = fs::read_to_string("/proc/self/status") else {
        eprintln!("memory[{label}] unavailable");
        return;
    };

    let vmrss = status
        .lines()
        .find(|line| line.starts_with("VmRSS:"))
        .unwrap_or("VmRSS:\tunknown");
    let vmhwm = status
        .lines()
        .find(|line| line.starts_with("VmHWM:"))
        .unwrap_or("VmHWM:\tunknown");
    eprintln!("memory[{label}] {vmrss} {vmhwm}");
}

fn write_validated_model_to_tempfile(model: &OwnedCityModel, prefix: &str) -> NamedTempFile {
    let output_json = to_string_validated(model)
        .unwrap_or_else(|error| panic!("serde_cityjson validation failed: {error}"));

    let mut temp = Builder::new()
        .prefix(prefix)
        .suffix(".city.json")
        .tempfile()
        .unwrap_or_else(|error| panic!("failed to create temp output: {error}"));
    temp.write_all(output_json.as_bytes())
        .unwrap_or_else(|error| panic!("failed to write temp output: {error}"));
    temp.flush()
        .unwrap_or_else(|error| panic!("failed to flush temp output: {error}"));
    temp
}

fn files_have_equal_bytes(left: &Path, right: &Path) -> bool {
    let left_file = fs::File::open(left)
        .unwrap_or_else(|error| panic!("failed to open {}: {error}", left.display()));
    let right_file = fs::File::open(right)
        .unwrap_or_else(|error| panic!("failed to open {}: {error}", right.display()));

    let mut left_reader = BufReader::new(left_file);
    let mut right_reader = BufReader::new(right_file);
    let mut left_buf = vec![0_u8; 64 * 1024];
    let mut right_buf = vec![0_u8; 64 * 1024];

    loop {
        let left_read = left_reader
            .read(&mut left_buf)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", left.display()));
        let right_read = right_reader
            .read(&mut right_buf)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", right.display()));

        if left_read != right_read {
            return false;
        }
        if left_read == 0 {
            return true;
        }
        if left_buf[..left_read] != right_buf[..right_read] {
            return false;
        }
    }
}

fn assert_json_files_eq(expected: &Path, actual: &Path) {
    if files_have_equal_bytes(expected, actual) {
        return;
    }

    let expected_file = fs::File::open(expected)
        .unwrap_or_else(|error| panic!("failed to open {}: {error}", expected.display()));
    let actual_file = fs::File::open(actual)
        .unwrap_or_else(|error| panic!("failed to open {}: {error}", actual.display()));
    let expected_json: JsonValue = serde_json::from_reader(expected_file)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", expected.display()));
    let actual_json: JsonValue = serde_json::from_reader(actual_file)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", actual.display()));

    assert_eq!(
        expected_json, actual_json,
        "normalized CityJSON changed during package roundtrip"
    );
}

#[must_use]
pub fn roundtrip_via_cityarrow_with_encoding(
    model: OwnedCityModel,
    encoding: PackageTableEncoding,
) -> OwnedCityModel {
    log_memory_phase("roundtrip_start");
    let parts = to_parts(&model).expect("cityarrow to_parts should succeed");
    log_memory_phase("after_to_parts");
    let parts = roundtrip_parts_via_package(&parts, encoding);
    log_memory_phase("after_package_roundtrip");
    drop(model);
    let reconstructed = from_parts(&parts).expect("cityarrow from_parts should succeed");
    log_memory_phase("after_from_parts");
    reconstructed
}

#[must_use]
pub fn roundtrip_parts_via_package(
    parts: &CityModelArrowParts,
    encoding: PackageTableEncoding,
) -> CityModelArrowParts {
    let dir = tempfile::tempdir().expect("cityarrow tempdir should be created");
    match encoding {
        PackageTableEncoding::Parquet => {
            write_package_dir(dir.path(), parts)
                .expect("cityparquet Parquet package write should succeed");
            read_package_dir(dir.path()).expect("cityparquet Parquet package read should succeed")
        }
        PackageTableEncoding::ArrowIpcFile => {
            write_package_ipc_dir(dir.path(), parts)
                .expect("cityarrow IPC package write should succeed");
            read_package_ipc_dir(dir.path()).expect("cityarrow IPC package read should succeed")
        }
    }
}

pub fn assert_parts_eq(expected: &CityModelArrowParts, actual: &CityModelArrowParts) {
    assert_eq!(expected.header, actual.header, "package header changed");
    assert_eq!(
        expected.projection, actual.projection,
        "projection layout changed"
    );

    assert_core_parts_eq(expected, actual);
    assert_geometry_parts_eq(expected, actual);
    assert_semantic_parts_eq(expected, actual);
    assert_appearance_parts_eq(expected, actual);
}

pub fn assert_model_roundtrip_integrity(model: OwnedCityModel, encoding: PackageTableEncoding) {
    log_memory_phase("model_roundtrip_start");
    let parts = to_parts(&model).expect("cityarrow to_parts should succeed");
    log_memory_phase("after_to_parts");
    let package_parts = roundtrip_parts_via_package(&parts, encoding);
    log_memory_phase("after_package_roundtrip");
    assert_parts_eq(&parts, &package_parts);
    drop(parts);

    let reconstructed = from_parts(&package_parts).expect("cityarrow from_parts should succeed");
    log_memory_phase("after_from_parts");
    drop(package_parts);
    let expected_file = write_validated_model_to_tempfile(&model, "cityarrow-expected");
    log_memory_phase("after_expected_serialize");
    drop(model);
    let actual_file = write_validated_model_to_tempfile(&reconstructed, "cityarrow-actual");
    log_memory_phase("after_actual_serialize");
    drop(reconstructed);
    assert_json_files_eq(expected_file.path(), actual_file.path());
    log_memory_phase("after_json_compare");

    cjval_validate(actual_file.path());
    log_memory_phase("after_cjval");
}

pub fn assert_package_roundtrip_parts_integrity(
    model: OwnedCityModel,
    encoding: PackageTableEncoding,
) {
    log_memory_phase("package_roundtrip_start");
    let parts = to_parts(&model).expect("cityarrow to_parts should succeed");
    log_memory_phase("after_to_parts");
    let package_parts = roundtrip_parts_via_package(&parts, encoding);
    log_memory_phase("after_package_roundtrip");
    assert_parts_eq(&parts, &package_parts);
    drop(parts);
    drop(model);

    let reconstructed = from_parts(&package_parts).expect("cityarrow from_parts should succeed");
    log_memory_phase("after_from_parts");
    drop(package_parts);
    validate_model_with_cjval(&reconstructed, "cityarrow-roundtrip");
    log_memory_phase("after_cjval");
}

pub fn assert_case_roundtrip_with_encoding(case: &Case, encoding: PackageTableEncoding) {
    let input_path = resolve_case_path(case);
    let model = {
        let input_json = fs::read_to_string(&input_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", input_path.display()));

        from_str_owned(&input_json)
            .unwrap_or_else(|error| panic!("serde_cityjson failed for {}: {error}", case.id))
    };
    log_memory_phase("after_parse");
    assert_model_roundtrip_integrity(model, encoding);
}

pub fn assert_case_roundtrip(case: &Case) {
    assert_case_roundtrip_with_encoding(case, PackageTableEncoding::Parquet);
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

fn assert_batch_eq(label: &str, expected: &RecordBatch, actual: &RecordBatch) {
    assert_eq!(
        expected.schema_ref(),
        actual.schema_ref(),
        "{label} schema changed"
    );
    assert_eq!(
        expected.num_rows(),
        actual.num_rows(),
        "{label} row count changed"
    );
    assert_eq!(
        expected.num_columns(),
        actual.num_columns(),
        "{label} column count changed"
    );
    assert!(
        expected == actual,
        "{label} batch changed during package roundtrip"
    );
}

fn assert_core_parts_eq(expected: &CityModelArrowParts, actual: &CityModelArrowParts) {
    assert_batch_eq("metadata", &expected.metadata, &actual.metadata);
    assert_optional_batch_eq(
        "transform",
        expected.transform.as_ref(),
        actual.transform.as_ref(),
    );
    assert_optional_batch_eq(
        "extensions",
        expected.extensions.as_ref(),
        actual.extensions.as_ref(),
    );
    assert_batch_eq("vertices", &expected.vertices, &actual.vertices);
    assert_batch_eq("cityobjects", &expected.cityobjects, &actual.cityobjects);
    assert_optional_batch_eq(
        "cityobject_children",
        expected.cityobject_children.as_ref(),
        actual.cityobject_children.as_ref(),
    );
}

fn assert_geometry_parts_eq(expected: &CityModelArrowParts, actual: &CityModelArrowParts) {
    assert_batch_eq("geometries", &expected.geometries, &actual.geometries);
    assert_batch_eq(
        "geometry_boundaries",
        &expected.geometry_boundaries,
        &actual.geometry_boundaries,
    );
    assert_optional_batch_eq(
        "geometry_instances",
        expected.geometry_instances.as_ref(),
        actual.geometry_instances.as_ref(),
    );
    assert_optional_batch_eq(
        "template_vertices",
        expected.template_vertices.as_ref(),
        actual.template_vertices.as_ref(),
    );
    assert_optional_batch_eq(
        "template_geometries",
        expected.template_geometries.as_ref(),
        actual.template_geometries.as_ref(),
    );
    assert_optional_batch_eq(
        "template_geometry_boundaries",
        expected.template_geometry_boundaries.as_ref(),
        actual.template_geometry_boundaries.as_ref(),
    );
}

fn assert_semantic_parts_eq(expected: &CityModelArrowParts, actual: &CityModelArrowParts) {
    assert_optional_batch_eq(
        "semantics",
        expected.semantics.as_ref(),
        actual.semantics.as_ref(),
    );
    assert_optional_batch_eq(
        "semantic_children",
        expected.semantic_children.as_ref(),
        actual.semantic_children.as_ref(),
    );
    assert_optional_batch_eq(
        "geometry_surface_semantics",
        expected.geometry_surface_semantics.as_ref(),
        actual.geometry_surface_semantics.as_ref(),
    );
    assert_optional_batch_eq(
        "geometry_point_semantics",
        expected.geometry_point_semantics.as_ref(),
        actual.geometry_point_semantics.as_ref(),
    );
    assert_optional_batch_eq(
        "geometry_linestring_semantics",
        expected.geometry_linestring_semantics.as_ref(),
        actual.geometry_linestring_semantics.as_ref(),
    );
    assert_optional_batch_eq(
        "template_geometry_semantics",
        expected.template_geometry_semantics.as_ref(),
        actual.template_geometry_semantics.as_ref(),
    );
}

fn assert_appearance_parts_eq(expected: &CityModelArrowParts, actual: &CityModelArrowParts) {
    assert_optional_batch_eq(
        "materials",
        expected.materials.as_ref(),
        actual.materials.as_ref(),
    );
    assert_optional_batch_eq(
        "geometry_surface_materials",
        expected.geometry_surface_materials.as_ref(),
        actual.geometry_surface_materials.as_ref(),
    );
    assert_optional_batch_eq(
        "geometry_point_materials",
        expected.geometry_point_materials.as_ref(),
        actual.geometry_point_materials.as_ref(),
    );
    assert_optional_batch_eq(
        "geometry_linestring_materials",
        expected.geometry_linestring_materials.as_ref(),
        actual.geometry_linestring_materials.as_ref(),
    );
    assert_optional_batch_eq(
        "template_geometry_materials",
        expected.template_geometry_materials.as_ref(),
        actual.template_geometry_materials.as_ref(),
    );
    assert_optional_batch_eq(
        "textures",
        expected.textures.as_ref(),
        actual.textures.as_ref(),
    );
    assert_optional_batch_eq(
        "texture_vertices",
        expected.texture_vertices.as_ref(),
        actual.texture_vertices.as_ref(),
    );
    assert_optional_batch_eq(
        "geometry_ring_textures",
        expected.geometry_ring_textures.as_ref(),
        actual.geometry_ring_textures.as_ref(),
    );
    assert_optional_batch_eq(
        "template_geometry_ring_textures",
        expected.template_geometry_ring_textures.as_ref(),
        actual.template_geometry_ring_textures.as_ref(),
    );
}

fn assert_optional_batch_eq(
    label: &str,
    expected: Option<&RecordBatch>,
    actual: Option<&RecordBatch>,
) {
    assert_eq!(
        expected.is_some(),
        actual.is_some(),
        "{label} presence changed"
    );
    if let (Some(expected), Some(actual)) = (expected, actual) {
        assert_batch_eq(label, expected, actual);
    }
}

fn validate_model_with_cjval(model: &OwnedCityModel, prefix: &str) {
    let temp = write_validated_model_to_tempfile(model, prefix);
    cjval_validate(temp.path());
}
