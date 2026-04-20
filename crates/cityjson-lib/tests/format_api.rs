//! Public API contract for the core JSON boundary.

use cityjson_lib::json;
use serde_json::Value;

fn minimal_model() -> cityjson_lib::Result<cityjson_lib::CityModel> {
    json::from_file("tests/data/v2_0/minimal.city.json")
}

fn assert_same_transport_shape(
    expected: &cityjson_lib::CityModel,
    actual: &cityjson_lib::CityModel,
) {
    let expected = cityjson_lib::query::summary(expected);
    let actual = cityjson_lib::query::summary(actual);
    assert_eq!(expected.cityobject_count, actual.cityobject_count);
    assert_eq!(expected.geometry_count, actual.geometry_count);
    assert_eq!(
        expected.geometry_template_count,
        actual.geometry_template_count
    );
    assert_eq!(expected.vertex_count, actual.vertex_count);
    assert_eq!(expected.template_vertex_count, actual.template_vertex_count);
    assert_eq!(expected.uv_vertex_count, actual.uv_vertex_count);
    assert_eq!(expected.semantic_count, actual.semantic_count);
    assert_eq!(expected.material_count, actual.material_count);
    assert_eq!(expected.texture_count, actual.texture_count);
    assert_eq!(expected.has_metadata, actual.has_metadata);
}

#[test]
fn json_boundary_roundtrips_through_the_core_module() {
    let model = minimal_model().expect("fixture should parse through the core JSON boundary");

    let bytes = json::to_vec(&model).expect("model should serialize");
    let roundtrip = json::from_slice(&bytes).expect("serialized bytes should parse");
    let model_value: Value =
        serde_json::from_slice(&bytes).expect("serialized bytes should be JSON");
    let roundtrip_bytes = json::to_vec(&roundtrip).expect("roundtrip model should serialize");
    let roundtrip_value: Value =
        serde_json::from_slice(&roundtrip_bytes).expect("roundtrip bytes should be JSON");

    assert!(!bytes.is_empty());
    assert_eq!(model_value, roundtrip_value);
}

#[cfg(feature = "arrow")]
#[test]
fn arrow_boundary_roundtrips_through_bytes_and_file() -> cityjson_lib::Result<()> {
    let model = minimal_model()?;

    let bytes = cityjson_lib::arrow::to_vec(&model)?;
    let from_bytes = cityjson_lib::arrow::from_bytes(&bytes)?;
    assert!(!bytes.is_empty());
    assert_same_transport_shape(&model, &from_bytes);

    let tempdir = tempfile::tempdir()?;
    let path = tempdir.path().join("minimal.cjarrow");
    let report = cityjson_lib::arrow::to_file(&path, &model)?;
    let from_file = cityjson_lib::arrow::from_file(&path)?;
    assert!(report.batch_count > 0);
    assert_same_transport_shape(&model, &from_file);

    Ok(())
}

#[cfg(feature = "parquet")]
#[test]
fn parquet_boundary_roundtrips_through_package_and_dataset() -> cityjson_lib::Result<()> {
    let model = minimal_model()?;

    let tempdir = tempfile::tempdir()?;
    let package_path = tempdir.path().join("minimal.cityjson-parquet");
    let package_manifest = cityjson_lib::parquet::to_file(&package_path, &model)?;
    let from_package = cityjson_lib::parquet::from_file(&package_path)?;
    assert!(!package_manifest.tables.is_empty());
    assert_same_transport_shape(&model, &from_package);

    let dataset_path = tempdir.path().join("minimal.dataset");
    let dataset_manifest = cityjson_lib::parquet::to_dir(&dataset_path, &model)?;
    let from_dataset = cityjson_lib::parquet::from_dir(&dataset_path)?;
    assert!(!dataset_manifest.tables.is_empty());
    assert_same_transport_shape(&model, &from_dataset);

    Ok(())
}
