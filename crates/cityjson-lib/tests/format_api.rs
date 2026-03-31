//! Public API contract for explicit Arrow IPC and Parquet package boundaries.

use cjlib::{CityModel, arrow, parquet};
use serde_cityjson::to_string_validated;
use serde_json::Value;
use std::path::Path;

fn normalized_json(model: &CityModel) -> Value {
    serde_json::from_str(&to_string_validated(model.as_inner()).expect("model should serialize"))
        .expect("serialized model should be valid JSON")
}

#[test]
fn arrow_boundary_writes_a_package_directory_and_roundtrips() {
    let path = Path::new("tests/output/minimal.cjarrow");
    let _ = std::fs::remove_dir_all(path);
    let model =
        CityModel::from_file("tests/data/v2_0/minimal.city.json").expect("fixture should parse");

    arrow::to_file(path, &model).expect("arrow package should be written");

    assert!(path.join("manifest.json").is_file());

    let roundtrip = arrow::from_file(path).expect("arrow package should be readable");
    assert_eq!(normalized_json(&model), normalized_json(&roundtrip));
}

#[test]
fn parquet_boundary_writes_a_package_directory_and_roundtrips() {
    let path = Path::new("tests/output/minimal.cjparquet");
    let _ = std::fs::remove_dir_all(path);
    let model =
        CityModel::from_file("tests/data/v2_0/minimal.city.json").expect("fixture should parse");

    parquet::to_file(path, &model).expect("parquet package should be written");

    assert!(path.join("manifest.json").is_file());

    let roundtrip = parquet::from_file(path).expect("parquet package should be readable");
    assert_eq!(normalized_json(&model), normalized_json(&roundtrip));
}
