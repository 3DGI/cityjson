//! Public API contract for future non-JSON format boundaries.

use cjlib::{CityModel, arrow, parquet};

#[test]
fn arrow_boundary_writes_a_non_empty_transport_file() -> cjlib::Result<()> {
    let path = "tests/output/minimal.cjarrow";
    let _ = std::fs::remove_file(path);
    let model = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;

    arrow::to_file(path, &model)?;

    let metadata = std::fs::metadata(path)?;
    assert!(metadata.len() > 0);

    Ok(())
}

#[test]
fn parquet_boundary_writes_a_non_empty_transport_file() -> cjlib::Result<()> {
    let path = "tests/output/minimal.cjparquet";
    let _ = std::fs::remove_file(path);
    let model = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;

    parquet::to_file(path, &model)?;

    let metadata = std::fs::metadata(path)?;
    assert!(metadata.len() > 0);

    Ok(())
}
