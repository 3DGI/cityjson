//! Public API contract for future non-JSON format boundaries.

use cjlib::{CityModel, arrow, parquet};

#[test]
#[should_panic(expected = "implement the Arrow format boundary in a dedicated backend crate")]
fn arrow_boundary_writes_a_non_empty_transport_file() {
    let path = "tests/output/minimal.cjarrow";
    let _ = std::fs::remove_file(path);
    let model =
        CityModel::from_file("tests/data/v2_0/minimal.city.json").expect("fixture should parse");

    arrow::to_file(path, &model).expect("arrow::to_file is intentionally unimplemented");

    let metadata = std::fs::metadata(path).expect("arrow output should be inspectable");
    assert!(metadata.len() > 0);
}

#[test]
#[should_panic(expected = "implement the Parquet format boundary in a dedicated backend crate")]
fn parquet_boundary_writes_a_non_empty_transport_file() {
    let path = "tests/output/minimal.cjparquet";
    let _ = std::fs::remove_file(path);
    let model =
        CityModel::from_file("tests/data/v2_0/minimal.city.json").expect("fixture should parse");

    parquet::to_file(path, &model).expect("parquet::to_file is intentionally unimplemented");

    let metadata = std::fs::metadata(path).expect("parquet output should be inspectable");
    assert!(metadata.len() > 0);
}
