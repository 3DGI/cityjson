//! Public API contract for the `cityjson_lib::CityModel` boundary.

use cityjson_lib::{CityJSONVersion, CityModel};

#[test]
fn citymodel_is_the_default_entry_point_for_cityjson_json() -> cityjson_lib::Result<()> {
    let bytes = br#"{"type":"CityJSON","version":"2.0","transform":{"scale":[1.0,1.0,1.0],"translate":[0.0,0.0,0.0]},"CityObjects":{},"vertices":[]}"#;

    let _ = CityModel::from_slice(bytes)?;
    let _ = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;

    Ok(())
}

#[test]
fn citymodel_is_a_thin_owned_wrapper_over_cityjson_rs() {
    let inner = cityjson_lib::cityjson::v2_0::OwnedCityModel::new(
        cityjson_lib::cityjson::CityModelType::CityJSON,
    );
    let mut model = CityModel::from(inner);

    let _: &cityjson_lib::cityjson::v2_0::OwnedCityModel = model.as_inner();
    let _: &mut cityjson_lib::cityjson::v2_0::OwnedCityModel = model.as_inner_mut();
    let _: &cityjson_lib::cityjson::v2_0::OwnedCityModel = model.as_ref();
    let _: &mut cityjson_lib::cityjson::v2_0::OwnedCityModel = model.as_mut();
}

#[test]
fn advanced_model_access_flows_through_the_cityjson_crate_reexport() {
    let inner = cityjson_lib::cityjson::v2_0::OwnedCityModel::new(
        cityjson_lib::cityjson::CityModelType::CityJSON,
    );
    let model = CityModel::from(inner);

    let owned: cityjson_lib::cityjson::v2_0::OwnedCityModel = model.into_inner();
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
