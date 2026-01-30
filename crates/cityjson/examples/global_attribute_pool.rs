//! Example demonstrating inline attribute usage (formerly: global attribute pool pattern)

use cityjson::prelude::*;
use cityjson::v1_1::*;

fn main() {
    // Create a new city model
    let mut city_model: CityModel = CityModel::new(CityModelType::CityJSON);

    println!("=== Creating Building with Attributes ===");

    // Create city object with inline attributes
    let mut building = CityObject::new("building-001".to_string(), CityObjectType::Building);

    // Add attributes directly - no pool needed!
    let attrs = building.attributes_mut();
    attrs.insert(
        "measuredHeight".to_string(),
        AttributeValue::Float(25.5)
    );
    attrs.insert(
        "buildingName".to_string(),
        AttributeValue::String("City Hall".to_string())
    );
    attrs.insert(
        "yearOfConstruction".to_string(),
        AttributeValue::Integer(1985)
    );

    let building_ref = city_model.cityobjects_mut().add(building);

    println!(
        "Created building with {} attributes",
        city_model.cityobjects().get(building_ref).unwrap().attributes().unwrap().len()
    );

    // Retrieve and display
    let building = city_model.cityobjects().get(building_ref).unwrap();
    if let Some(attrs) = building.attributes() {
        for (key, attr_value) in attrs.iter() {
            print!("  {}: ", key);

            match attr_value {
                AttributeValue::Float(f) => println!("{}", f),
                AttributeValue::String(s) => println!("{}", s),
                AttributeValue::Integer(i) => println!("{}", i),
                _ => println!("(other type)"),
            }
        }
    }

    println!("\n=== Creating Semantic with Attributes ===");

    // Create semantic with inline attributes
    let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
    roof_semantic.attributes_mut().insert(
        "roofMaterial".to_string(),
        AttributeValue::String("tile".to_string())
    );
    roof_semantic.attributes_mut().insert(
        "roofColor".to_string(),
        AttributeValue::String("red".to_string())
    );

    // Add semantic to model
    let semantic_ref = city_model.add_semantic(roof_semantic);

    println!("Created semantic with attributes");

    // Retrieve and display
    let semantic = city_model.get_semantic(semantic_ref).unwrap();
    if let Some(attrs) = semantic.attributes() {
        for (key, attr_value) in attrs.iter() {
            print!("  {}: ", key);
            if let AttributeValue::String(s) = attr_value {
                println!("{}", s);
            }
        }
    }

    println!(
        "\nBuilding attributes: {}",
        city_model.cityobjects().get(building_ref).unwrap().attributes().unwrap().len()
    );
}
