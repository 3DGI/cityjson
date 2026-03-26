//! Public API contract for the future `cjlib::CityModel` surface.
//! These tests intentionally describe the target API before the implementation is finished.

use std::io::Cursor;

use cjlib::{CityJSONVersion, CityModel};

#[test]
fn citymodel_is_the_default_entry_point_for_cityjson_json() -> cjlib::Result<()> {
    let bytes = br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#;

    let _ = CityModel::from_slice(bytes)?;
    let _ = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;

    Ok(())
}

#[test]
fn citymodel_from_stream_remains_a_compatibility_alias() -> cjlib::Result<()> {
    let bytes = br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#;

    let _ = CityModel::from_stream(Cursor::new(bytes))?;

    Ok(())
}

#[test]
fn citymodel_is_a_thin_owned_wrapper_over_cityjson_rs() {
    let inner =
        cjlib::cityjson::v2_0::OwnedCityModel::new(cjlib::cityjson::CityModelType::CityJSON);
    let mut model = CityModel::from(inner);

    let _: &cjlib::cityjson::v2_0::OwnedCityModel = model.as_inner();
    let _: &mut cjlib::cityjson::v2_0::OwnedCityModel = model.as_inner_mut();
    let _: &cjlib::cityjson::v2_0::OwnedCityModel = model.as_ref();
    let _: &mut cjlib::cityjson::v2_0::OwnedCityModel = model.as_mut();
}

#[test]
fn advanced_model_access_flows_through_the_cityjson_crate_reexport() {
    let inner =
        cjlib::cityjson::v2_0::OwnedCityModel::new(cjlib::cityjson::CityModelType::CityJSON);
    let model = CityModel::from(inner);

    let owned: cjlib::cityjson::v2_0::OwnedCityModel = model.into_inner();
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
