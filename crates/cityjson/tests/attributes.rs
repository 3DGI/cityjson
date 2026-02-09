//! Tests for inline attribute functionality

use cityjson::backend::default::attributes::{OwnedAttributeValue, OwnedAttributes};
use cityjson::prelude::*;
use cityjson::v2_0::*;
use std::collections::HashMap;

const FLOAT_EPSILON: f64 = 1.0e-12;

fn assert_f64_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= FLOAT_EPSILON,
        "expected {expected}, got {actual} (epsilon {FLOAT_EPSILON})"
    );
}

#[test]
fn test_basic_attribute_operations() {
    let mut attrs = OwnedAttributes::new();

    // Insert various types
    attrs.insert("height".to_string(), OwnedAttributeValue::Float(42.5));
    attrs.insert(
        "name".to_string(),
        OwnedAttributeValue::String("Tower".to_string()),
    );
    attrs.insert("floors".to_string(), OwnedAttributeValue::Integer(10));
    attrs.insert("active".to_string(), OwnedAttributeValue::Bool(true));

    // Verify retrieval
    assert_eq!(attrs.len(), 4);

    if let Some(OwnedAttributeValue::Float(h)) = attrs.get("height") {
        assert_f64_eq(*h, 42.5);
    } else {
        panic!("Expected float value");
    }

    if let Some(OwnedAttributeValue::String(n)) = attrs.get("name") {
        assert_eq!(n, "Tower");
    } else {
        panic!("Expected string value");
    }
}

#[test]
fn test_cityobject_attributes() {
    let mut city_model: CityModel = CityModel::new(CityModelType::CityJSON);

    // Create building with attributes
    let mut building = CityObject::new(
        CityObjectIdentifier::new("building-001".to_string()),
        CityObjectType::Building,
    );

    // Add attributes directly - no pool needed!
    building.attributes_mut().insert(
        "measuredHeight".to_string(),
        OwnedAttributeValue::Float(25.5),
    );
    building.attributes_mut().insert(
        "buildingName".to_string(),
        OwnedAttributeValue::String("City Hall".to_string()),
    );
    building.attributes_mut().insert(
        "yearOfConstruction".to_string(),
        OwnedAttributeValue::Integer(1985),
    );

    // Add to model
    let building_ref = city_model.cityobjects_mut().add(building).unwrap();

    // Retrieve and verify
    let retrieved = city_model.cityobjects().get(building_ref).unwrap();
    let attrs = retrieved.attributes().unwrap();

    assert!(attrs.contains_key("measuredHeight"));
    assert!(attrs.contains_key("buildingName"));
    assert!(attrs.contains_key("yearOfConstruction"));

    if let Some(OwnedAttributeValue::Float(h)) = attrs.get("measuredHeight") {
        assert_f64_eq(*h, 25.5);
    }
}

#[test]
fn test_semantic_attributes() {
    // Create semantic with attributes
    let mut roof: Semantic<OwnedStringStorage> = Semantic::new(SemanticType::RoofSurface);

    roof.attributes_mut().insert(
        "material".to_string(),
        OwnedAttributeValue::String("tile".to_string()),
    );
    roof.attributes_mut().insert(
        "color".to_string(),
        OwnedAttributeValue::String("red".to_string()),
    );

    // Verify
    let attrs = roof.attributes().unwrap();
    assert_eq!(attrs.len(), 2);

    if let Some(OwnedAttributeValue::String(m)) = attrs.get("material") {
        assert_eq!(m.as_str(), "tile");
    }
}

#[test]
fn test_nested_map_attributes() {
    let mut attrs = OwnedAttributes::new();

    // Create nested map (like address)
    let mut address_map = HashMap::new();
    address_map.insert(
        "street".to_string(),
        Box::new(OwnedAttributeValue::String("Main St".to_string())),
    );
    address_map.insert(
        "number".to_string(),
        Box::new(OwnedAttributeValue::Integer(123)),
    );

    attrs.insert("address".to_string(), OwnedAttributeValue::Map(address_map));

    // Verify nested access
    if let Some(OwnedAttributeValue::Map(addr)) = attrs.get("address") {
        assert!(addr.contains_key("street"));
        assert!(addr.contains_key("number"));

        if let Some(street_val) = addr.get("street")
            && let OwnedAttributeValue::String(s) = street_val.as_ref()
        {
            assert_eq!(s, "Main St");
        }
    }
}

#[test]
fn test_nested_vector_attributes() {
    let mut attrs = OwnedAttributes::new();

    // Create vector (like materials list)
    let materials = vec![
        Box::new(OwnedAttributeValue::String("concrete".to_string())),
        Box::new(OwnedAttributeValue::String("glass".to_string())),
        Box::new(OwnedAttributeValue::String("steel".to_string())),
    ];

    attrs.insert("materials".to_string(), OwnedAttributeValue::Vec(materials));

    // Verify vector access
    if let Some(OwnedAttributeValue::Vec(mats)) = attrs.get("materials") {
        assert_eq!(mats.len(), 3);

        if let OwnedAttributeValue::String(first) = mats[0].as_ref() {
            assert_eq!(first, "concrete");
        }
    }
}

#[test]
fn test_attribute_modification() {
    let mut attrs = OwnedAttributes::new();

    attrs.insert("count".to_string(), OwnedAttributeValue::Integer(10));

    // Modify via get_mut
    if let Some(OwnedAttributeValue::Integer(c)) = attrs.get_mut("count") {
        *c += 5;
    }

    // Verify modification
    if let Some(OwnedAttributeValue::Integer(c)) = attrs.get("count") {
        assert_eq!(*c, 15);
    }
}

#[test]
fn test_attribute_removal() {
    let mut attrs = OwnedAttributes::new();

    attrs.insert("temp".to_string(), OwnedAttributeValue::Bool(true));
    assert_eq!(attrs.len(), 1);

    let removed = attrs.remove("temp");
    assert!(matches!(removed, Some(OwnedAttributeValue::Bool(true))));
    assert!(attrs.is_empty());
}

#[test]
fn test_attribute_iteration() {
    let mut attrs = OwnedAttributes::new();

    attrs.insert("a".to_string(), OwnedAttributeValue::Integer(1));
    attrs.insert("b".to_string(), OwnedAttributeValue::Integer(2));
    attrs.insert("c".to_string(), OwnedAttributeValue::Integer(3));

    let keys: Vec<&String> = attrs.keys().collect();
    assert_eq!(keys.len(), 3);

    let mut sum = 0i64;
    for (_, value) in attrs.iter() {
        if let OwnedAttributeValue::Integer(n) = value {
            sum += n;
        }
    }
    assert_eq!(sum, 6);
}

#[test]
fn test_cityobject_extra_attributes() {
    let mut city_model: CityModel = CityModel::new(CityModelType::CityJSON);

    // Create building with extra attributes
    let mut building = CityObject::new(
        CityObjectIdentifier::new("building-001".to_string()),
        CityObjectType::Building,
    );

    // Add extra attributes (custom/extension properties)
    building.extra_mut().insert(
        "customProperty".to_string(),
        OwnedAttributeValue::String("customValue".to_string()),
    );

    let building_ref = city_model.cityobjects_mut().add(building).unwrap();

    // Retrieve and verify
    let retrieved = city_model.cityobjects().get(building_ref).unwrap();
    let extra = retrieved.extra().unwrap();

    assert!(extra.contains_key("customProperty"));
}

#[test]
fn test_null_attribute_value() {
    let mut attrs = OwnedAttributes::new();

    // Test null values
    attrs.insert("nullable_field".to_string(), OwnedAttributeValue::Null);

    assert!(attrs.contains_key("nullable_field"));
    if let Some(OwnedAttributeValue::Null) = attrs.get("nullable_field") {
        // Expected behavior
    } else {
        panic!("Expected null value");
    }
}
