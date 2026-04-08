#[path = "../../tests/support/shared_corpus.rs"]
mod shared_corpus;

use cityjson::CityModelType;
use cityparquet::{PackageReader, PackageWriter};
use tempfile::tempdir;

#[test]
fn package_roundtrip_preserves_cityjsonfeature_root_id() {
    let case =
        shared_corpus::load_named_normative_conformance_case("cityjsonfeature_root_id_resolves");
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

#[test]
fn package_roundtrip_preserves_cityjson_root_id_as_extra() {
    let case =
        shared_corpus::load_named_normative_conformance_case("cityjson_root_id_extra_property");
    let dir = tempdir().unwrap();
    let path = dir.path().join("root-extra.cityarrow");

    PackageWriter.write_file(&path, &case.model).unwrap();
    let decoded = PackageReader.read_file(&path).unwrap();

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
