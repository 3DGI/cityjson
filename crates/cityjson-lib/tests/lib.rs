use crate::common::DATA_DIR;
use cjlib::{
    Attributes, CityJSONVersion, CityModel, Extension, ExtensionName, Extensions, Transform,
};
use serde_cityjson::CityModelType;
use std::collections::HashMap;

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
fn citymodel_from_str_dummy() {
    let cityjson_str = std::fs::read_to_string(DATA_DIR.join("cityjson_dummy_complete.city.json"))
        .expect("Failed to read the file");
    let cm = CityModel::from_str(cityjson_str.as_str()).unwrap();
    println!("{}", &cm);
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
fn test_transform() {
    // Test Transform construction and getters
    let scale = [1.0, 1.0, 1.0];
    let translate = [0.0, 0.0, 0.0];
    let mut transform = Transform::new(scale, translate);
    assert_eq!(transform.scale(), &scale);
    assert_eq!(transform.translate(), &translate);

    // Test setters
    let new_scale = [2.0, 2.0, 2.0];
    let new_translate = [5.0, 5.0, 5.0];
    transform.set_scale(new_scale);
    transform.set_translate(new_translate);
    assert_eq!(transform.scale(), &new_scale);
    assert_eq!(transform.translate(), &new_translate);

    // Test Transform in CityModel
    let mut cm = CityModel::default();
    assert!(cm.transform().is_none());
    cm.set_transform(&transform);
    assert_eq!(cm.transform(), Some(&transform));
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

#[test]
fn test_citymodel_extensions() {
    let mut cm = CityModel::new(CityModelType::CityJSON);
    assert!(cm.extensions().is_none());

    let ext_name = "test";
    let ext_name_2 = ExtensionName::from("test_2");
    let ext = Extension::new("https://example.com/ext".to_string(), "1.0".to_string());
    cm.extensions_mut().insert(ext_name, ext.clone());
    cm.extensions_mut().insert(ext_name_2, ext.clone());
    assert_eq!(cm.extensions().as_ref().unwrap().get(ext_name), Some(&ext));

    let ext_removed = cm.extensions_mut().remove(ext_name).unwrap();
    assert_eq!(ext, ext_removed);
}

#[test]
fn test_extensions() {
    let mut extensions = Extensions::new();
    let ext = Extension::new("https://example.com/ext".to_string(), "1.0".to_string());
    let name = "test".to_string();

    // Insert and retrieve
    extensions.insert(&name, ext.clone());
    assert!(extensions.contains(&name));
    assert_eq!(extensions.get(&name), Some(&ext));

    // Remove
    assert_eq!(extensions.remove(&name), Some(ext));
    assert!(!extensions.contains(&name));

    // Iteration
    let mut extensions = Extensions::new();

    extensions.insert(
        &"ext1".to_string(),
        Extension::new("https://example.com/ext1".to_string(), "1.0".to_string()),
    );
    extensions.insert(
        &"ext2".to_string(),
        Extension::new("https://example.com/ext2".to_string(), "2.0".to_string()),
    );

    // Test iter()
    let mut count = 0;
    for (name, ext) in extensions.iter() {
        assert!(["ext1", "ext2"].contains(&name.as_str()));
        assert!(ext.url().starts_with("https://example.com/"));
        count += 1;
    }
    assert_eq!(count, 2);

    // Test iter_mut()
    for (_, ext) in extensions.iter_mut() {
        ext.set_version("3.0".to_string());
    }

    // Verify all versions were updated
    for (_, ext) in &extensions {
        assert_eq!(ext.version(), "3.0");
    }
}

#[test]
fn test_extension() {
    let url = "https://example.com/ext".to_string();
    let version = "1.0".to_string();
    let mut ext = Extension::new(url.clone(), version.clone());

    // Check initial values
    assert_eq!(ext.url(), url);
    assert_eq!(ext.version(), version);

    // Update values
    let new_url = "https://example.com/ext2".to_string();
    let new_version = "2.0".to_string();
    ext.set_url(new_url.clone());
    ext.set_version(new_version.clone());

    assert_eq!(ext.url(), new_url);
    assert_eq!(ext.version(), new_version);
}

#[test]
fn test_extra_root_properties() {
    let mut cm = CityModel::default();

    // Initially, extra root properties should be None
    assert!(cm.extra_root_properties().is_none());

    // Getting mutable reference should create empty Attributes
    let extra = cm.extra_root_properties_mut();
    assert!(extra.is_null()); // Default is Null

    // Create a map with various types
    let mut map = HashMap::new();
    map.insert("string".to_string(), Attributes::String("test".to_string()));
    map.insert("number".to_string(), Attributes::Integer(42));
    map.insert("boolean".to_string(), Attributes::Bool(true));
    map.insert("float".to_string(), Attributes::Float(3.14));

    *extra = Attributes::Map(map);

    // Test reading values through immutable reference
    if let Some(extra) = cm.extra_root_properties() {
        if let Some(map) = extra.as_map() {
            assert_eq!(map.get("string").unwrap().as_str(), Some("test"));
            assert_eq!(map.get("number").unwrap().as_integer(), Some(42));
            assert_eq!(map.get("boolean").unwrap().as_bool(), Some(true));
            assert_eq!(map.get("float").unwrap().as_float(), Some(3.14));
        } else {
            panic!("Expected Map variant");
        }
    } else {
        panic!("Extra root properties should exist");
    }

    // Test modification through mutable reference
    if let Some(map) = cm.extra_root_properties_mut().as_map_mut() {
        map.insert(
            "new_value".to_string(),
            Attributes::String("added later".to_string()),
        );
    }

    // Verify all values including the new one
    if let Some(map) = cm.extra_root_properties().unwrap().as_map() {
        assert_eq!(map.get("string").unwrap().as_str(), Some("test"));
        assert_eq!(map.get("number").unwrap().as_integer(), Some(42));
        assert_eq!(map.get("boolean").unwrap().as_bool(), Some(true));
        assert_eq!(map.get("float").unwrap().as_float(), Some(3.14));
        assert_eq!(map.get("new_value").unwrap().as_str(), Some("added later"));
    } else {
        panic!("Expected Map variant");
    }
}
