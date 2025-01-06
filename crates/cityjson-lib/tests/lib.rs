use cjlib::{CityJSONVersion, CityModel, Transform};
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
fn test_get_version() {}

#[test]
fn test_version_get_set() {
    let mut cm = CityModel::default();
    assert_eq!(cm.version(), &Some(CityJSONVersion::default()));

    let new_version = CityJSONVersion::V1_0;
    cm.set_version(new_version);
    assert_eq!(cm.version(), &Some(new_version));
}

#[test]
fn test_transform_get_set() {
    let mut cm = CityModel::default();
    assert!(cm.transform().is_none());

    let transform = Transform::new([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
    cm.set_transform(&transform);
    assert_eq!(cm.transform(), Some(&transform));
}

#[test]
fn test_transform_new() {
    let scale = [1.0, 1.0, 1.0];
    let translate = [0.0, 0.0, 0.0];
    let transform = Transform::new(scale, translate);

    assert_eq!(transform.scale(), &scale);
    assert_eq!(transform.translate(), &translate);
}

#[test]
fn test_transform_set_scale() {
    let scale = [1.0, 1.0, 1.0];
    let translate = [0.0, 0.0, 0.0];
    let mut transform = Transform::new(scale, translate);

    let new_scale = [2.0, 2.0, 2.0];
    transform.set_scale(new_scale);

    assert_eq!(transform.scale(), &new_scale);
}

#[test]
fn test_transform_set_translate() {
    let scale = [1.0, 1.0, 1.0];
    let translate = [0.0, 0.0, 0.0];
    let mut transform = Transform::new(scale, translate);

    let new_translate = [5.0, 5.0, 5.0];
    transform.set_translate(new_translate);

    assert_eq!(transform.translate(), &new_translate);
}

#[test]
fn test_transform_display() {
    let scale = [1.0, 1.0, 1.0];
    let translate = [0.0, 0.0, 0.0];
    let transform = Transform::new(scale, translate);

    let display_output = format!("{transform}");
    assert_eq!(
        display_output,
        "Transform(scale: [1.0, 1.0, 1.0], translate: [0.0, 0.0, 0.0])"
    );
}
