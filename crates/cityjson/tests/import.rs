use cityjson::import::{detect_version, import_cityjson};
use cityjson::prelude::*;
use cityjson::CityJSONVersion;

const V1_0_MINIMAL: &str = r#"{
  "type": "CityJSON",
  "version": "1.0",
  "CityObjects": {},
  "vertices": []
}"#;
const V2_0_MINIMAL: &str = r#"{
  "type": "CityJSON",
  "version": "2.0",
  "CityObjects": {},
  "vertices": []
}"#;
const V1_1_FIXTURE: &str = include_str!("data/v1_1/cityjson_fake_complete.city.json");

#[test]
fn detect_fixture_versions() {
    assert_eq!(detect_version(V1_0_MINIMAL).unwrap(), CityJSONVersion::V1_0);
    assert_eq!(detect_version(V1_1_FIXTURE).unwrap(), CityJSONVersion::V1_1);
    assert_eq!(detect_version(V2_0_MINIMAL).unwrap(), CityJSONVersion::V2_0);
}

#[test]
fn import_v1_0_minimal_to_v2() {
    let model = import_cityjson::<OwnedStringStorage>(V1_0_MINIMAL).unwrap();
    assert!(model.cityobjects().is_empty());
    assert!(model.vertices().is_empty());
}

#[test]
fn import_v1_1_fixture_to_v2() {
    let model = import_cityjson::<OwnedStringStorage>(V1_1_FIXTURE).unwrap();
    assert!(!model.cityobjects().is_empty());
    assert!(!model.vertices().is_empty());
}

#[test]
fn import_v2_0_fixture_natively() {
    let model = import_cityjson::<OwnedStringStorage>(V2_0_MINIMAL).unwrap();
    assert!(model.cityobjects().is_empty());
    assert!(model.vertices().is_empty());
}
