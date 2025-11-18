//! Tests for global attribute pool integration

use cityjson::cityjson::core::attributes::AttributeOwnerType;
use cityjson::prelude::*;
use cityjson::v1_1::*;

#[test]
fn test_attribute_pool_in_city_model() {
    let city_model: CityModel = CityModel::new(CityModelType::CityJSON);

    assert_eq!(city_model.attribute_count(), 0);
    assert!(!city_model.has_attributes());
}

#[test]
fn test_add_cityobject_attributes() {
    let mut city_model: CityModel = CityModel::new(CityModelType::CityJSON);

    // Add attributes to pool
    let height_id = city_model.attributes_mut().add_float(
        "height".to_string(),
        true,
        42.5,
        AttributeOwnerType::CityObject,
        None,
    );

    let name_id = city_model.attributes_mut().add_string(
        "name".to_string(),
        true,
        "Building A".to_string(),
        AttributeOwnerType::CityObject,
        None,
    );

    // Create city object and link attributes
    let mut city_object = CityObject::new("building-1".to_string(), CityObjectType::Building);
    city_object
        .attributes_mut()
        .insert("height".to_string(), height_id);
    city_object
        .attributes_mut()
        .insert("name".to_string(), name_id);

    let co_ref = city_model.cityobjects_mut().add(city_object);

    // Verify
    assert_eq!(city_model.attribute_count(), 2);

    let retrieved_co = city_model.cityobjects().get(co_ref).unwrap();
    let attrs = retrieved_co.attributes().unwrap();

    assert_eq!(attrs.get("height"), Some(height_id));
    assert_eq!(attrs.get("name"), Some(name_id));

    // Verify values in pool
    assert_eq!(city_model.attributes().get_float(height_id), Some(42.5));
    assert_eq!(
        city_model.attributes().get_string(name_id),
        Some(&"Building A".to_string())
    );
}

#[test]
fn test_semantic_attributes() {
    let mut city_model: CityModel = CityModel::new(CityModelType::CityJSON);

    // Add semantic attributes to pool
    let material_id = city_model.attributes_mut().add_string(
        "material".to_string(),
        true,
        "concrete".to_string(),
        AttributeOwnerType::Semantic,
        None,
    );

    let color_id = city_model.attributes_mut().add_string(
        "color".to_string(),
        true,
        "red".to_string(),
        AttributeOwnerType::Semantic,
        None,
    );

    // Create semantic with attributes
    let mut roof = Semantic::new(SemanticType::RoofSurface);
    roof.attributes_mut()
        .insert("material".to_string(), material_id);
    roof.attributes_mut().insert("color".to_string(), color_id);

    // Add semantic to model
    let semantic_ref = city_model.add_semantic(roof);

    // Verify
    assert_eq!(city_model.attribute_count(), 2);
    assert_eq!(city_model.semantic_count(), 1);

    let retrieved_semantic = city_model.get_semantic(semantic_ref).unwrap();
    let attrs = retrieved_semantic.attributes().unwrap();

    assert_eq!(attrs.get("material"), Some(material_id));
    assert_eq!(attrs.get("color"), Some(color_id));

    // Verify values in pool
    assert_eq!(
        city_model.attributes().get_string(material_id),
        Some(&"concrete".to_string())
    );
    assert_eq!(
        city_model.attributes().get_string(color_id),
        Some(&"red".to_string())
    );
}

#[test]
fn test_nested_attributes() {
    let mut city_model: CityModel = CityModel::new(CityModelType::CityJSON);

    // Create nested map structure
    let street_id = city_model.attributes_mut().add_string(
        "street".to_string(),
        true,
        "Main St".to_string(),
        AttributeOwnerType::Element,
        None,
    );

    let number_id = city_model.attributes_mut().add_integer(
        "number".to_string(),
        true,
        123,
        AttributeOwnerType::Element,
        None,
    );

    let mut address_map = std::collections::HashMap::new();
    address_map.insert("street".to_string(), street_id);
    address_map.insert("number".to_string(), number_id);

    let address_id = city_model.attributes_mut().add_map(
        "address".to_string(),
        true,
        address_map,
        AttributeOwnerType::CityObject,
        None,
    );

    // Verify nested structure
    let street_val_id = city_model
        .attributes()
        .get_map_value(address_id, "street")
        .unwrap();
    assert_eq!(
        city_model.attributes().get_string(street_val_id),
        Some(&"Main St".to_string())
    );
}

#[test]
fn test_clear_attributes() {
    let mut city_model: CityModel = CityModel::new(CityModelType::CityJSON);

    city_model.attributes_mut().add_float(
        "test".to_string(),
        true,
        1.0,
        AttributeOwnerType::CityObject,
        None,
    );

    assert_eq!(city_model.attribute_count(), 1);

    city_model.clear_attributes();

    assert_eq!(city_model.attribute_count(), 0);
    assert!(!city_model.has_attributes());
}
