//! Example demonstrating the global attribute pool pattern

use cityjson::cityjson::core::attributes::AttributeOwnerType;
use cityjson::prelude::*;
use cityjson::v1_1::*;

fn main() {
    // Create a new city model with global attribute pool
    let mut city_model: CityModel = CityModel::new(CityModelType::CityJSON);

    println!("=== Creating Building with Attributes ===");

    // Add attributes to the global pool first
    let height_id = city_model.attributes_mut().add_float(
        "measuredHeight".to_string(),
        true,
        25.5,
        AttributeOwnerType::CityObject,
        None,
    );

    let name_id = city_model.attributes_mut().add_string(
        "buildingName".to_string(),
        true,
        "City Hall".to_string(),
        AttributeOwnerType::CityObject,
        None,
    );

    let year_id = city_model.attributes_mut().add_integer(
        "yearOfConstruction".to_string(),
        true,
        1985,
        AttributeOwnerType::CityObject,
        None,
    );

    // Create city object and attach attributes
    let mut building = CityObject::new("building-001".to_string(), CityObjectType::Building);

    let attrs = building.attributes_mut();
    attrs.insert("measuredHeight".to_string(), height_id);
    attrs.insert("buildingName".to_string(), name_id);
    attrs.insert("yearOfConstruction".to_string(), year_id);

    let building_ref = city_model.cityobjects_mut().add(building);

    println!(
        "Created building with {} attributes",
        city_model.attribute_count()
    );

    // Retrieve and display
    let building = city_model.cityobjects().get(building_ref).unwrap();
    if let Some(attrs) = building.attributes() {
        for (key, attr_id) in attrs.iter() {
            print!("  {}: ", key);

            if let Some(f) = city_model.attributes().get_float(attr_id) {
                println!("{}", f);
            } else if let Some(s) = city_model.attributes().get_string(attr_id) {
                println!("{}", s);
            } else if let Some(i) = city_model.attributes().get_integer(attr_id) {
                println!("{}", i);
            }
        }
    }

    println!("\n=== Creating Semantic with Attributes ===");

    // Add semantic attributes to pool
    let material_id = city_model.attributes_mut().add_string(
        "roofMaterial".to_string(),
        true,
        "tile".to_string(),
        AttributeOwnerType::Semantic,
        None,
    );

    let color_id = city_model.attributes_mut().add_string(
        "roofColor".to_string(),
        true,
        "red".to_string(),
        AttributeOwnerType::Semantic,
        None,
    );

    // Create semantic with attributes
    let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
    roof_semantic
        .attributes_mut()
        .insert("roofMaterial".to_string(), material_id);
    roof_semantic
        .attributes_mut()
        .insert("roofColor".to_string(), color_id);

    // Add semantic to pool
    let semantic_ref = city_model.add_semantic(roof_semantic);

    println!("Created semantic with attributes");

    // Retrieve and display
    let semantic = city_model.get_semantic(semantic_ref).unwrap();
    if let Some(attrs) = semantic.attributes() {
        for (key, attr_id) in attrs.iter() {
            print!("  {}: ", key);
            if let Some(s) = city_model.attributes().get_string(attr_id) {
                println!("{}", s);
            }
        }
    }

    println!(
        "\nTotal attributes in model: {}",
        city_model.attribute_count()
    );
}
