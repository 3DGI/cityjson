//! Example demonstrating inline attribute usage in cityjson-rs
//!
//! This example shows how to work with attributes using the new `AoS` (Array of Structures)
//! inline API. Attributes are now stored directly on objects rather than in a global pool.
//!
//! Run with: cargo run --example attributes

use cityjson::cityjson::core::attributes::OwnedAttributeValue;
use cityjson::prelude::*;
use cityjson::resources::CityObjectRef;
use cityjson::v2_0::{CityModel, CityObject, CityObjectType, Semantic, SemanticType};
use std::collections::HashMap;

fn main() -> Result<()> {
    println!("=== CityJSON Inline Attributes Example ===\n");

    let mut city_model: CityModel<u32, OwnedStringStorage> =
        CityModel::new(CityModelType::CityJSON);
    let building_ref = add_building(&mut city_model)?;
    add_semantics(&mut city_model)?;
    add_nested_address(&mut city_model, building_ref);
    add_materials(&mut city_model, building_ref);
    print_building_data(&city_model, building_ref);
    modify_building(&mut city_model, building_ref);
    print_summary(&city_model, building_ref);

    println!("\nExample completed successfully!");
    Ok(())
}

fn add_building(model: &mut CityModel<u32, OwnedStringStorage>) -> Result<CityObjectRef> {
    println!("1. Creating building with basic attributes...");
    let mut building = CityObject::new(
        CityObjectIdentifier::new("building-001".to_string()),
        CityObjectType::Building,
    );
    building
        .attributes_mut()
        .insert("measuredHeight".to_string(), OwnedAttributeValue::Float(25.5));
    building.attributes_mut().insert(
        "buildingName".to_string(),
        OwnedAttributeValue::String("City Hall".to_string()),
    );
    building.attributes_mut().insert(
        "yearOfConstruction".to_string(),
        OwnedAttributeValue::Integer(1985),
    );
    building
        .attributes_mut()
        .insert("isHistoric".to_string(), OwnedAttributeValue::Bool(true));
    let building_ref = model.cityobjects_mut().add(building)?;
    println!("   Added building with 4 attributes\n");
    Ok(building_ref)
}

fn add_semantics(model: &mut CityModel<u32, OwnedStringStorage>) -> Result<()> {
    println!("2. Creating semantic surfaces with attributes...");
    let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
    roof_semantic.attributes_mut().insert(
        "roofMaterial".to_string(),
        OwnedAttributeValue::String("tile".to_string()),
    );
    roof_semantic.attributes_mut().insert(
        "roofColor".to_string(),
        OwnedAttributeValue::String("red".to_string()),
    );

    let mut wall_semantic = Semantic::new(SemanticType::WallSurface);
    wall_semantic.attributes_mut().insert(
        "wallMaterial".to_string(),
        OwnedAttributeValue::String("brick".to_string()),
    );

    let _roof_ref = model.add_semantic(roof_semantic)?;
    let _wall_ref = model.add_semantic(wall_semantic)?;
    println!("   Created roof and wall semantics with attributes\n");
    Ok(())
}

fn add_nested_address(model: &mut CityModel<u32, OwnedStringStorage>, building_ref: CityObjectRef) {
    println!("3. Creating nested attributes (address)...");
    let mut address_map = HashMap::new();
    address_map.insert(
        "street".to_string(),
        Box::new(OwnedAttributeValue::String("123 Main Street".to_string())),
    );
    address_map.insert(
        "city".to_string(),
        Box::new(OwnedAttributeValue::String("Amsterdam".to_string())),
    );
    address_map.insert(
        "country".to_string(),
        Box::new(OwnedAttributeValue::String("Netherlands".to_string())),
    );
    address_map.insert(
        "postalCode".to_string(),
        Box::new(OwnedAttributeValue::String("1012 AB".to_string())),
    );
    if let Some(building) = model.cityobjects_mut().get_mut(building_ref) {
        building
            .extra_mut()
            .insert("address".to_string(), OwnedAttributeValue::Map(address_map));
    }
    println!("   Added nested address to building\n");
}

fn add_materials(model: &mut CityModel<u32, OwnedStringStorage>, building_ref: CityObjectRef) {
    println!("4. Creating vector attributes (materials)...");
    let materials = vec![
        Box::new(OwnedAttributeValue::String("concrete".to_string())),
        Box::new(OwnedAttributeValue::String("glass".to_string())),
        Box::new(OwnedAttributeValue::String("steel".to_string())),
        Box::new(OwnedAttributeValue::String("wood".to_string())),
    ];
    if let Some(building) = model.cityobjects_mut().get_mut(building_ref) {
        building
            .attributes_mut()
            .insert("materials".to_string(), OwnedAttributeValue::Vec(materials));
    }
    println!("   Added materials list to building\n");
}

fn print_building_data(model: &CityModel<u32, OwnedStringStorage>, building_ref: CityObjectRef) {
    println!("5. Reading attributes back...\n");
    if let Some(building) = model.cityobjects().get(building_ref) {
        println!("   Building: {}", building.id());
        if let Some(attrs) = building.attributes() {
            println!("   Attributes:");
            for (key, value) in attrs.iter() {
                println!("     - {key}: {value}");
            }
        }
        if let Some(extra) = building.extra() {
            println!("   Extra properties:");
            for (key, value) in extra.iter() {
                match value {
                    OwnedAttributeValue::Map(_) => println!("     - {key}: <nested map>"),
                    _ => println!("     - {key}: {value}"),
                }
            }
        }
    }
}

fn modify_building(model: &mut CityModel<u32, OwnedStringStorage>, building_ref: CityObjectRef) {
    println!("\n6. Modifying attributes...");
    if let Some(building) = model.cityobjects_mut().get_mut(building_ref) {
        if let Some(OwnedAttributeValue::Integer(year)) =
            building.attributes_mut().get_mut("yearOfConstruction")
        {
            println!("   Original year: {year}");
            *year = 1986;
            println!("   Updated year: {year}");
        }
        building.attributes_mut().insert(
            "lastRenovation".to_string(),
            OwnedAttributeValue::Integer(2020),
        );
        println!("   Added lastRenovation attribute");
    }
}

fn print_summary(model: &CityModel<u32, OwnedStringStorage>, building_ref: CityObjectRef) {
    println!("\n=== Summary ===");
    println!("City objects in model: {}", model.cityobjects().len());
    if let Some(building) = model.cityobjects().get(building_ref)
        && let Some(attrs) = building.attributes()
    {
        println!("Building attributes count: {}", attrs.len());
    }
}
