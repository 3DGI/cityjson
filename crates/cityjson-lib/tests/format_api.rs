//! Public API contract for explicit cityarrow and cityparquet boundaries.

use cjlib::{CityModel, arrow, parquet};
use serde_cityjson::to_string_validated;
use serde_json::Value;
use std::path::Path;

fn normalized_json(model: &CityModel) -> Value {
    serde_json::from_str(&to_string_validated(model.as_inner()).expect("model should serialize"))
        .expect("serialized model should be valid JSON")
}

#[test]
fn arrow_boundary_roundtrips_through_a_live_stream_buffer() {
    let model =
        CityModel::from_file("tests/data/v2_0/minimal.city.json").expect("fixture should parse");
    let mut bytes = Vec::new();

    arrow::to_writer(&mut bytes, &model).expect("arrow stream should be written");
    assert!(!bytes.is_empty());

    let roundtrip = arrow::from_reader(bytes.as_slice()).expect("arrow stream should be readable");
    assert_eq!(normalized_json(&model), normalized_json(&roundtrip));
}

#[test]
fn arrow_boundary_writes_a_stream_file_and_roundtrips() {
    let path = Path::new("tests/output/minimal.cjarrow");
    reset_output_path(path);
    let model =
        CityModel::from_file("tests/data/v2_0/minimal.city.json").expect("fixture should parse");

    arrow::to_file(path, &model).expect("arrow stream file should be written");

    assert!(path.is_file());

    let roundtrip = arrow::from_file(path).expect("arrow stream file should be readable");
    assert_eq!(normalized_json(&model), normalized_json(&roundtrip));
}

#[test]
fn parquet_boundary_writes_a_package_file_and_roundtrips() {
    let path = Path::new("tests/output/minimal.cjparquet");
    reset_output_path(path);
    let model =
        CityModel::from_file("tests/data/v2_0/minimal.city.json").expect("fixture should parse");

    parquet::to_file(path, &model).expect("parquet package file should be written");

    assert!(path.is_file());

    let roundtrip = parquet::from_file(path).expect("parquet package file should be readable");
    assert_eq!(normalized_json(&model), normalized_json(&roundtrip));
}

fn reset_output_path(path: &Path) {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => {
            std::fs::remove_dir_all(path).expect("test output directory should be removable");
        }
        Ok(_) => {
            std::fs::remove_file(path).expect("test output file should be removable");
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to inspect {}: {error}", path.display()),
    }
}
