use cjlib::{CityFeature, CityFeatureStreamDeserializer, CityModel, Transform};
use serde_json::Deserializer;
use std::io::{BufRead, Cursor};
use std::path::PathBuf;
use std::str::FromStr;

mod common;

#[test]
fn init_citymodel() {
    let _cm = CityModel::new();
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
fn citymodel_from_file_minimal_complete() {
    let pb: PathBuf = common::DATA_DIR.join("cityjson_minimal_complete.city.json");
    let cm = CityModel::from_file(&pb);
    assert!(cm.is_ok());
}

#[test]
fn citymodel_from_file_dummy_complete() {
    let pb: PathBuf = common::DATA_DIR.join("cityjson_dummy_complete.city.json");
    let cm = CityModel::from_file(&pb);
    assert!(cm.is_ok());
}

#[test]
fn citymodel_to_string() {
    let tr = Transform {
        scale: [0.001, 0.001, 0.001],
        translate: [0.0, 0.0, 0.0],
    };
    let mut cm = CityModel::new();
    cm.set_transform(&tr);
    let cj = cm.to_string().unwrap();
    common::validate(cj.as_str(), "citymodel_to_string");
}

#[test]
fn citymodel_to_file() {
    let pb: PathBuf = common::OUTPUT_DIR.join("citymodel_to_file.city.json");
    assert!(CityModel::new().to_file(pb).is_ok());
}

#[test]
fn debug_citymodel() {
    let cm = CityModel::new();
    println!("{:?}", cm);
}

#[test]
fn display_citymodel() {
    let cm = CityModel::new();
    println!("{}", cm);
}
