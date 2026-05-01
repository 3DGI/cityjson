//! Public API contract for the `cityjson_lib::CityModel` boundary.

use cityjson_lib::cityjson_types::v2_0::{
    BBox, CityObject, CityObjectIdentifier, CityObjectType, GeometryDraft, PointDraft,
    RealWorldCoordinate,
};
use cityjson_lib::json;
use cityjson_lib::{CityJSONVersion, CityModel};

#[test]
fn citymodel_is_the_default_owned_model_type() -> cityjson_lib::Result<()> {
    let bytes = br#"{"type":"CityJSON","version":"2.0","transform":{"scale":[1.0,1.0,1.0],"translate":[0.0,0.0,0.0]},"CityObjects":{},"vertices":[]}"#;

    let model = json::from_slice(bytes)?;
    let _ = json::from_file("tests/data/v2_0/minimal.city.json")?;
    let _: CityModel = model;

    Ok(())
}

#[test]
fn citymodel_is_a_direct_alias_over_cityjson_rs() {
    let model: CityModel = cityjson_lib::cityjson_types::v2_0::OwnedCityModel::new(
        cityjson_lib::cityjson_types::CityModelType::CityJSON,
    );

    let _: &cityjson_lib::cityjson_types::v2_0::OwnedCityModel = &model;
    let _: &mut cityjson_lib::cityjson_types::v2_0::OwnedCityModel = &mut model.clone();
    let _: cityjson_lib::cityjson_types::v2_0::OwnedCityModel = model.clone();
}

#[test]
fn advanced_model_access_is_directly_available_on_the_alias() {
    let model = cityjson_lib::cityjson_types::v2_0::OwnedCityModel::new(
        cityjson_lib::cityjson_types::CityModelType::CityJSON,
    );

    let owned: cityjson_lib::cityjson_types::v2_0::OwnedCityModel = model;
    let _ = owned;
}

#[test]
fn cityjson_version_stays_in_the_public_boundary() {
    assert_eq!(
        CityJSONVersion::try_from("1.0.3").unwrap(),
        CityJSONVersion::V1_0
    );
    assert_eq!(
        CityJSONVersion::try_from("1.1.2").unwrap(),
        CityJSONVersion::V1_1
    );
    assert_eq!(
        CityJSONVersion::try_from("2.0.1").unwrap(),
        CityJSONVersion::V2_0
    );
}

#[test]
fn cityjson_lib_citymodel_exposes_geographical_extent_calculation_api() -> cityjson_lib::Result<()>
{
    let mut model: CityModel = cityjson_lib::cityjson_types::v2_0::OwnedCityModel::new(
        cityjson_lib::cityjson_types::CityModelType::CityJSON,
    );
    let geometry = GeometryDraft::multi_point(
        None,
        [
            PointDraft::new(RealWorldCoordinate::new(1.0, 2.0, 3.0)),
            PointDraft::new(RealWorldCoordinate::new(4.0, 5.0, 6.0)),
        ],
    )
    .insert_into(&mut model)?;
    let mut cityobject = CityObject::new(
        CityObjectIdentifier::new("building-1".to_string()),
        CityObjectType::Building,
    );
    cityobject.add_geometry(geometry);
    let handle = model.cityobjects_mut().add(cityobject)?;

    assert_eq!(
        model.calculate_cityobject_geographical_extent(handle)?,
        Some(BBox::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0))
    );
    assert_eq!(
        model.calculate_geographical_extent()?,
        Some(BBox::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0))
    );

    Ok(())
}
