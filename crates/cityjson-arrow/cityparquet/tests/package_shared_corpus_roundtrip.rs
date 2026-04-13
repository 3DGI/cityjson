#[path = "../../tests/support/shared_corpus.rs"]
mod shared_corpus;

use cityparquet::{PackageReader, PackageWriter};
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

fn assert_package_roundtrip(case_id: &str) {
    let case = shared_corpus::load_named_conformance_case(case_id);
    let dir = tempdir().unwrap();
    let path = dir.path().join(format!("{case_id}.cityarrow"));

    PackageWriter
        .write_file(&path, &case.model)
        .unwrap_or_else(|err| panic!("{case_id}: write failed: {err}"));
    let decoded = PackageReader
        .read_file(&path)
        .unwrap_or_else(|err| panic!("{case_id}: read failed: {err}"));

    let expected = shared_corpus::normalized_json(&case.model);
    let actual = shared_corpus::normalized_json(&decoded);
    assert_eq!(actual, expected, "{case_id}: roundtrip JSON mismatch");
}

conformance_roundtrip_tests!(
    assert_package_roundtrip;
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
