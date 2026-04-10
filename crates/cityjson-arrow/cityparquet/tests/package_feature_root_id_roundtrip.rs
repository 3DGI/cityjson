#[path = "../../tests/support/shared_corpus.rs"]
mod shared_corpus;

use cityjson::CityModelType;
use cityparquet::{PackageReader, PackageWriter};
use tempfile::tempdir;

#[test]
fn package_roundtrip_preserves_cityjsonfeature_minimal() {
    let case = shared_corpus::load_named_conformance_case("cityjsonfeature_minimal");
    let expected_root_id = case
        .model
        .id()
        .and_then(|handle| case.model.cityobjects().get(handle))
        .map(|cityobject| cityobject.id().to_string());
    let dir = tempdir().unwrap();
    let path = dir.path().join("feature.cityarrow");

    PackageWriter.write_file(&path, &case.model).unwrap();
    let decoded = PackageReader.read_file(&path).unwrap();

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
