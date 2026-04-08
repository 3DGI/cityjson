#[path = "support/shared_corpus.rs"]
mod shared_corpus;

use cityarrow::{ModelDecoder, ModelEncoder};
use cityjson::CityModelType;

#[test]
fn arrow_roundtrip_preserves_cityjsonfeature_root_id() {
    let case =
        shared_corpus::load_named_normative_conformance_case("cityjsonfeature_root_id_resolves");
    let expected_root_id = case
        .model
        .id()
        .and_then(|handle| case.model.cityobjects().get(handle))
        .map(|cityobject| cityobject.id().to_string());
    let mut bytes = Vec::new();

    ModelEncoder.encode(&case.model, &mut bytes).unwrap();
    let decoded = ModelDecoder.decode(bytes.as_slice()).unwrap();

    assert_eq!(decoded.type_citymodel(), CityModelType::CityJSONFeature);
    assert_eq!(
        decoded
            .id()
            .and_then(|handle| decoded.cityobjects().get(handle))
            .map(|cityobject| cityobject.id().to_string()),
        expected_root_id
    );
    assert!(decoded.extra().and_then(|extra| extra.get("id")).is_none());
    assert_eq!(
        shared_corpus::normalized_json(&case.model),
        shared_corpus::normalized_json(&decoded)
    );
}

#[test]
fn arrow_roundtrip_preserves_cityjson_root_id_as_extra() {
    let case =
        shared_corpus::load_named_normative_conformance_case("cityjson_root_id_extra_property");
    let mut bytes = Vec::new();

    ModelEncoder.encode(&case.model, &mut bytes).unwrap();
    let decoded = ModelDecoder.decode(bytes.as_slice()).unwrap();

    assert_eq!(decoded.type_citymodel(), CityModelType::CityJSON);
    assert_eq!(decoded.id(), None);
    assert_eq!(
        decoded.extra().and_then(|extra| extra.get("id")),
        case.model.extra().and_then(|extra| extra.get("id"))
    );
    assert_eq!(
        shared_corpus::normalized_json(&case.model),
        shared_corpus::normalized_json(&decoded)
    );
}
