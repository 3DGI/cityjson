//! Small example of inline attributes.

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
    building.attributes_mut().insert(
        "measuredHeight".to_string(),
        OwnedAttributeValue::Float(25.5),
    );
    building.attributes_mut().insert(
        "buildingName".to_string(),
        OwnedAttributeValue::String("City Hall".to_string()),
    );

    let mut address = HashMap::new();
    address.insert(
        "city".to_string(),
        Box::new(OwnedAttributeValue::String("Amsterdam".to_string())),
    );
    address.insert(
        "country".to_string(),
        Box::new(OwnedAttributeValue::String("Netherlands".to_string())),
    );
    building
        .extra_mut()
        .insert("address".to_string(), OwnedAttributeValue::Map(address));

    let building_ref = model.cityobjects_mut().add(building)?;

    let mut roof = Semantic::new(SemanticType::RoofSurface);
    roof.attributes_mut().insert(
        "material".to_string(),
        OwnedAttributeValue::String("tile".to_string()),
    );
    model.add_semantic(roof)?;

    println!(
        "example done: {} cityobject(s), building ref {:?}",
        model.cityobjects().len(),
        building_ref
    );
    Ok(())
}
