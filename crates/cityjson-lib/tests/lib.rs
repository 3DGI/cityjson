use cjlib::{CityJSONVersion, CityModel};
use serde_cityjson::CityModelType;

mod common;

#[test]
fn init_citymodel() {
    let _cm = CityModel::new(CityModelType::CityJSON);
    let _cm2 = CityModel::default();
}

#[test]
fn citymodel_from_str_minimal() {
    let cityjson_str = r#"{
      "type": "CityJSON",
      "version": "1.1",
      "extensions": {},
      "transform": {
        "scale": [ 1.0, 1.0, 1.0 ],
        "translate": [ 0.0, 0.0, 0.0 ]
      },
      "metadata": {},
      "CityObjects": {},
      "vertices": [],
      "appearance": {},
      "geometry-templates": {
        "templates": [],
        "vertices-templates": []
      }
    }"#;
    assert!(CityModel::from_str(cityjson_str).is_ok());
}

#[test]
fn debug_citymodel() {
    let cm = CityModel::new(CityModelType::CityJSON);
    println!("{:?}", cm);
}

#[test]
fn display_citymodel() {
    let cm = CityModel::new(CityModelType::CityJSON);
    println!("{}", cm);
}

#[test]
fn test_get_version() {
    let cm = CityModel::default();
    assert_eq!(cm.version(), &Some(CityJSONVersion::default()));
}

#[test]
fn test_set_version() {
    let mut cm = CityModel::default();
    let new_version = CityJSONVersion::V1_0;
    cm.set_version(new_version.clone());
    assert_eq!(cm.version(), &Some(new_version));
}
