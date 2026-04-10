#[path = "support/shared_corpus.rs"]
mod shared_corpus;

use cityarrow::{ModelDecoder, ModelEncoder};
use cityjson::CityModelType;

#[test]
fn arrow_roundtrip_preserves_cityjsonfeature_minimal() {
    let case = shared_corpus::load_named_conformance_case("cityjsonfeature_minimal");
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
