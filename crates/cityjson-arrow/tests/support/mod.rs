#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

use arrow::record_batch::RecordBatch;
use cityarrow::schema::{CityModelArrowParts, PackageTableEncoding};
use cityarrow::{from_parts, read_package_ipc_dir, to_parts, write_package_ipc_dir};
use cityjson::v2_0::OwnedCityModel;
use cityparquet::{read_package_dir, write_package_dir};
use serde::Deserialize;
use serde_cityjson::{from_str_owned, to_string};
use tempfile::{Builder, NamedTempFile};

const DEFAULT_SHARED_CORPUS_ROOT: &str = "../cityjson-benchmarks";
const DEFAULT_CORRECTNESS_INDEX_PATH: &str = "artifacts/correctness-index.json";

static CORRECTNESS_CASES: LazyLock<std::collections::BTreeMap<String, CorrectnessCase>> =
    LazyLock::new(load_correctness_cases);

#[derive(Debug, Deserialize)]
struct CorrectnessIndex {
    cases: Vec<CorrectnessCase>,
}

#[derive(Debug, Deserialize)]
struct CorrectnessCase {
    id: String,
    layer: String,
    #[serde(default)]
    cityjson_version: Option<String>,
    representation: String,
    artifact_paths: CorrectnessArtifactPaths,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CorrectnessArtifactPaths {
    source: Option<PathBuf>,
    generated: Option<PathBuf>,
    profile: Option<PathBuf>,
}

#[must_use]
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[must_use]
pub fn shared_corpus_root() -> PathBuf {
    std::env::var_os("CITYARROW_SHARED_CORPUS_ROOT").map_or_else(
        || workspace_root().join(DEFAULT_SHARED_CORPUS_ROOT),
        PathBuf::from,
    )
}

#[must_use]
pub fn correctness_index_path() -> PathBuf {
    let path = std::env::var_os("CITYARROW_CORRECTNESS_INDEX").map_or_else(
        || shared_corpus_root().join(DEFAULT_CORRECTNESS_INDEX_PATH),
        PathBuf::from,
    );

    if path.is_absolute() {
        path
    } else {
        shared_corpus_root().join(path)
    }
}

#[must_use]
pub fn resolve_shared_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        shared_corpus_root().join(path)
    }
}

#[must_use]
pub fn conformance_case_ids() -> Vec<&'static str> {
    CORRECTNESS_CASES
        .values()
        .filter(|case| {
            case.layer == "conformance"
                && case.cityjson_version.as_deref() == Some("2.0")
                && case.representation == "cityjson"
        })
        .map(|case| case.id.as_str())
        .collect()
}

fn load_correctness_cases() -> std::collections::BTreeMap<String, CorrectnessCase> {
    let path = correctness_index_path();
    let manifest = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read correctness index {}: {error}",
            path.display()
        )
    });
    let index: CorrectnessIndex = serde_json::from_str(&manifest).unwrap_or_else(|error| {
        panic!(
            "failed to parse correctness index {}: {error}",
            path.display()
        )
    });
    index
        .cases
        .into_iter()
        .map(|case| (case.id.clone(), case))
        .collect()
}

fn conformance_case_path(case_id: &str) -> PathBuf {
    let case = CORRECTNESS_CASES.get(case_id).unwrap_or_else(|| {
        panic!(
            "missing correctness case '{}' in {}",
            case_id,
            correctness_index_path().display()
        )
    });
    assert_eq!(case.layer, "conformance");
    assert_eq!(case.cityjson_version.as_deref(), Some("2.0"));
    assert_eq!(case.representation, "cityjson");
    if let Some(path) = case.artifact_paths.source.clone() {
        return resolve_shared_path(path);
    }
    if let Some(path) = case.artifact_paths.generated.clone() {
        let resolved = resolve_shared_path(path);
        if resolved.exists() {
            return resolved;
        }
    }
    if let Some(profile) = case.artifact_paths.profile.clone() {
        return materialize_generated_case(case_id, profile);
    }
    panic!("correctness case '{case_id}' is missing a consumable artifact path");
}

fn materialize_generated_case(case_id: &str, profile: PathBuf) -> PathBuf {
    let output_dir = std::env::temp_dir().join("cityarrow-shared-corpus");
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|error| panic!("failed to create {}: {error}", output_dir.display()));
    let output_path = output_dir.join(format!("{case_id}.city.json"));
    if output_path.exists() {
        return output_path;
    }

    let profile_path = resolve_shared_path(profile);
    let schema_path = shared_corpus_root().join("profiles/cjfake-manifest.schema.json");
    let cjfake_manifest = std::env::var_os("CITYARROW_CJFAKE_CARGO_MANIFEST").map_or_else(
        || {
            shared_corpus_root()
                .parent()
                .expect("shared corpus root should have a parent")
                .join("cjfake/Cargo.toml")
        },
        PathBuf::from,
    );

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--manifest-path",
            cjfake_manifest
                .to_str()
                .expect("non-utf8 cjfake manifest path"),
            "--",
            "--manifest",
            profile_path.to_str().expect("non-utf8 profile path"),
            "--schema",
            schema_path.to_str().expect("non-utf8 schema path"),
            "--output",
            output_path.to_str().expect("non-utf8 output path"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to execute cjfake for {case_id}: {error}"));

    assert!(
        output.status.success(),
        "cjfake failed for {}:\nstdout:\n{}\nstderr:\n{}",
        case_id,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    output_path
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

fn write_model_to_tempfile(model: &OwnedCityModel, prefix: &str) -> NamedTempFile {
    let output_json = to_string(model)
        .unwrap_or_else(|error| panic!("serde_cityjson serialization failed: {error}"));

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

pub fn assert_conformance_case_roundtrip_with_encoding(
    case_id: &str,
    encoding: PackageTableEncoding,
) {
    let input_path = conformance_case_path(case_id);
    let model = {
        let input_json = fs::read_to_string(&input_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", input_path.display()));

        from_str_owned(&input_json)
            .unwrap_or_else(|error| panic!("serde_cityjson failed for {case_id}: {error}"))
    };
    log_memory_phase("after_parse");
    assert_package_roundtrip_parts_integrity(model, encoding);
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
    let temp = write_model_to_tempfile(model, prefix);
    cjval_validate(temp.path());
}
