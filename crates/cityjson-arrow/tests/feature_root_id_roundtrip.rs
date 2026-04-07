use cityarrow::{ModelDecoder, ModelEncoder};
use cityjson::CityModelType;
use cityjson::v2_0::{
    AttributeValue, CityObject, CityObjectIdentifier, CityObjectType, OwnedCityModel,
};
use serde_cityjson::to_string_validated;
use serde_json::Value as JsonValue;

fn normalized_json(model: &OwnedCityModel) -> JsonValue {
    serde_json::from_str(&to_string_validated(model).unwrap()).unwrap()
}

fn sample_feature_model() -> OwnedCityModel {
    let mut model = OwnedCityModel::new(CityModelType::CityJSONFeature);
    let handle = model
        .cityobjects_mut()
        .add(CityObject::new(
            CityObjectIdentifier::new("building-1".to_string()),
            CityObjectType::Building,
        ))
        .unwrap();
    model.set_id(Some(handle));
    model
}

fn sample_cityjson_with_root_extra_id() -> OwnedCityModel {
    let mut model = OwnedCityModel::new(CityModelType::CityJSON);
    model.extra_mut().insert(
        "id".to_string(),
        AttributeValue::String("document-external-id".to_string()),
    );
    model
}

#[test]
fn arrow_roundtrip_preserves_cityjsonfeature_root_id() {
    let model = sample_feature_model();
    let mut bytes = Vec::new();

    ModelEncoder.encode(&model, &mut bytes).unwrap();
    let decoded = ModelDecoder.decode(bytes.as_slice()).unwrap();

    assert_eq!(decoded.type_citymodel(), CityModelType::CityJSONFeature);
    assert_eq!(
        decoded
            .id()
            .and_then(|handle| decoded.cityobjects().get(handle))
            .map(|cityobject| cityobject.id().to_string()),
        Some("building-1".to_string())
    );
    assert_eq!(normalized_json(&model), normalized_json(&decoded));
}

#[test]
fn arrow_roundtrip_preserves_cityjson_root_id_as_extra() {
    let model = sample_cityjson_with_root_extra_id();
    let mut bytes = Vec::new();

    ModelEncoder.encode(&model, &mut bytes).unwrap();
    let decoded = ModelDecoder.decode(bytes.as_slice()).unwrap();

    assert_eq!(decoded.type_citymodel(), CityModelType::CityJSON);
    assert_eq!(decoded.id(), None);
    assert_eq!(normalized_json(&model), normalized_json(&decoded));
}
