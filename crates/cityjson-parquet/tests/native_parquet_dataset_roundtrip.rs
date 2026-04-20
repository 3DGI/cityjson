#[path = "../../cityjson-arrow/tests/support/shared_corpus.rs"]
mod shared_corpus;

use cityjson::CityModelType;
use cityjson_parquet::{ParquetDatasetReader, ParquetDatasetWriter};
use tempfile::tempdir;

macro_rules! conformance_roundtrip_tests {
    ($assert_fn:ident; $($case_id:ident),+ $(,)?) => {
        $(
            #[test]
            fn $case_id() {
                $assert_fn(stringify!($case_id));
            }
        )+
    };
}

fn assert_dataset_roundtrip(case_id: &str) {
    let case = shared_corpus::load_named_conformance_case(case_id);
    let expected_root_id = (case_id == "cityjsonfeature_minimal")
        .then(|| {
            case.model
                .id()
                .and_then(|handle| case.model.cityobjects().get(handle))
                .map(|cityobject| cityobject.id().to_string())
        })
        .flatten();
    let dir = tempdir().unwrap();
    let path = dir.path().join(case_id);

    ParquetDatasetWriter
        .write_dir(&path, &case.model)
        .unwrap_or_else(|err| panic!("{case_id}: write failed: {err}"));
    let decoded = ParquetDatasetReader
        .read_dir(&path)
        .unwrap_or_else(|err| panic!("{case_id}: read failed: {err}"));

    let expected = shared_corpus::transport_roundtrip_json(&case.model, &case.model);
    let actual = shared_corpus::transport_roundtrip_json(&decoded, &case.model);
    assert_eq!(actual, expected, "{case_id}: roundtrip JSON mismatch");

    if case_id == "cityjsonfeature_minimal" {
        assert_eq!(decoded.type_citymodel(), CityModelType::CityJSONFeature);
        assert_eq!(
            decoded
                .id()
                .and_then(|handle| decoded.cityobjects().get(handle))
                .map(|cityobject| cityobject.id().to_string()),
            expected_root_id
        );
        assert!(decoded.extra().and_then(|extra| extra.get("id")).is_none());
    }
}

conformance_roundtrip_tests!(
    assert_dataset_roundtrip;
    appearance_complete,
    cityobject_building_address,
    cityobject_complete,
    cityobject_extended,
    cityobject_all_types,
    coordinates_precision_ecef,
    coordinates_precision_local,
    coordinates_precision_stateplane,
    coordinates_precision_utm,
    coordinates_precision_wgs84,
    coordinates_precision_worst,
    geometry_instance,
    geometry_material_solid,
    geometry_material_multisolid,
    geometry_material_multisurface,
    geometry_texture_solid,
    geometry_texture_multisolid,
    geometry_texture_multisurface,
    geometry_semantics_solid,
    geometry_semantics_multisolid,
    geometry_semantics_multisurface,
    geometry_semantics_multilinestring,
    geometry_semantics_multipoint,
    cityjson_extended,
    cityjsonfeature_minimal,
    cityjson_fake_complete,
    cityjson_minimal,
    metadata_complete,
    metadata_extra_properties,
    semantic_all_types,
    semantic_complete,
    semantic_extended,
    vertices,
    extension,
    spec_geometry_matrix,
);
