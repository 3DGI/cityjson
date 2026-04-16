//! Public API contract for explicit cityarrow boundaries.

#[cfg(feature = "arrow")]
use cityjson_lib::CityModel;
#[cfg(feature = "arrow")]
use cityjson_lib::arrow;
#[cfg(feature = "arrow")]
use serde_json::Value;
#[cfg(feature = "arrow")]
use std::path::Path;

#[cfg(feature = "arrow")]
fn normalized_json(model: &CityModel) -> Value {
    let mut value: Value = serde_json::from_str(
        &String::from_utf8(
            cityjson_json::to_vec(model.as_inner(), &cityjson_json::WriteOptions::default())
                .expect("model should serialize"),
        )
        .expect("serialized model should be utf-8"),
    )
    .expect("serialized model should be valid JSON");
    strip_null_object_members(&mut value);
    value
}

#[cfg(feature = "arrow")]
fn strip_null_object_members(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, member| !member.is_null());
            for member in map.values_mut() {
                strip_null_object_members(member);
            }
        }
        Value::Array(items) => {
            for item in items {
                strip_null_object_members(item);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

#[cfg(feature = "arrow")]
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

#[cfg(feature = "arrow")]
#[test]
fn arrow_boundary_explicitly_exports_and_imports_batches() {
    let model =
        CityModel::from_file("tests/data/v2_0/minimal.city.json").expect("fixture should parse");

    let batches = arrow::export_batches(&model).expect("canonical batches should be exported");
    let roundtrip = arrow::import_batches(&batches).expect("canonical batches should be imported");

    assert_eq!(normalized_json(&model), normalized_json(&roundtrip));
}

#[cfg(feature = "arrow")]
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

#[cfg(feature = "arrow")]
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
