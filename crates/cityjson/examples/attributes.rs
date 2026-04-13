//! Small example exercising every `OwnedAttributeValue` variant once.

use std::collections::HashMap;

use cityjson::error::Result;
use cityjson::resources::storage::OwnedStringStorage;
use cityjson::v2_0::{
    CityModel, CityModelType, CityObject, CityObjectIdentifier, CityObjectType,
    OwnedAttributeValue, Semantic, SemanticType,
};

fn main() -> Result<()> {
    let mut model = CityModel::<u32, OwnedStringStorage>::new(CityModelType::CityJSON);

    let mut building = CityObject::new(
        CityObjectIdentifier::new("building-001".to_string()),
        CityObjectType::Building,
    );

    // String
    building.attributes_mut().insert(
        "buildingName".to_string(),
        OwnedAttributeValue::String("City Hall".to_string()),
    );
    // Float
    building.attributes_mut().insert(
        "measuredHeight".to_string(),
        OwnedAttributeValue::Float(25.5),
    );
    // Integer
    building.attributes_mut().insert(
        "yearOfConstruction".to_string(),
        OwnedAttributeValue::Integer(-50),
    );
    // Unsigned
    building.attributes_mut().insert(
        "storeysAboveGround".to_string(),
        OwnedAttributeValue::Unsigned(5),
    );
    // Bool
    building.attributes_mut().insert(
        "isMonument".to_string(),
        OwnedAttributeValue::Bool(true),
    );
    // Null
    building.attributes_mut().insert(
        "demolitionDate".to_string(),
        OwnedAttributeValue::Null,
    );
    // Vec
    building.attributes_mut().insert(
        "alternateNames".to_string(),
        OwnedAttributeValue::Vec(vec![
            OwnedAttributeValue::String("Stadhuis".to_string()),
            OwnedAttributeValue::String("Town Hall".to_string()),
        ]),
    );
    // Map
    let mut address = HashMap::new();
    address.insert(
        "city".to_string(),
        OwnedAttributeValue::String("Amsterdam".to_string()),
    );
    building
        .extra_mut()
        .insert("address".to_string(), OwnedAttributeValue::Map(address));

    let building_ref = model.cityobjects_mut().add(building)?;

    let roof = Semantic::new(SemanticType::RoofSurface);
    model.add_semantic(roof)?;

    println!(
        "example done: {} cityobject(s), building ref {:?}",
        model.cityobjects().len(),
        building_ref
    );
    Ok(())
}
