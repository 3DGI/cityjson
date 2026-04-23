#[path = "support/shared_corpus.rs"]
mod shared_corpus;

use cityjson_arrow::{
    ExportOptions, ImportOptions, export_reader, import_batches, read_stream, write_stream,
};

macro_rules! conformance_roundtrip_tests {
    ($($case_id:ident),+ $(,)?) => {
        mod stream {
            use super::*;
            $(
                #[test]
                fn $case_id() {
                    assert_stream_roundtrip(stringify!($case_id));
                }
            )+
        }
        mod batch {
            use super::*;
            $(
                #[test]
                fn $case_id() {
                    assert_batch_roundtrip(stringify!($case_id));
                }
            )+
        }
    };
}

fn assert_stream_roundtrip(case_id: &str) {
    let case = shared_corpus::load_named_conformance_case(case_id);
    let mut bytes = Vec::new();
    write_stream(&mut bytes, &case.model, &ExportOptions::default())
        .unwrap_or_else(|err| panic!("{case_id}: stream write failed: {err}"));
    let decoded = read_stream(bytes.as_slice(), &ImportOptions::default())
        .unwrap_or_else(|err| panic!("{case_id}: stream read failed: {err}"));
    let expected = shared_corpus::transport_roundtrip_json(&case.model, &case.model);
    let actual = shared_corpus::transport_roundtrip_json(&decoded, &case.model);
    assert_eq!(actual, expected, "{case_id}: stream roundtrip mismatch");
}

fn assert_batch_roundtrip(case_id: &str) {
    let case = shared_corpus::load_named_conformance_case(case_id);
    let reader = export_reader(&case.model, &ExportOptions::default())
        .unwrap_or_else(|err| panic!("{case_id}: export failed: {err}"));
    let header = reader.header().clone();
    let projection = reader.projection().clone();
    let batches = reader.collect::<Vec<_>>();
    let decoded = import_batches(header, projection, batches, &ImportOptions::default())
        .unwrap_or_else(|err| panic!("{case_id}: import failed: {err}"));
    let expected = shared_corpus::transport_roundtrip_json(&case.model, &case.model);
    let actual = shared_corpus::transport_roundtrip_json(&decoded, &case.model);
    assert_eq!(actual, expected, "{case_id}: batch roundtrip mismatch");
}

#[test]
fn removed_transform_stream_tag_is_rejected() {
    let case = shared_corpus::load_named_conformance_case("cityjson_minimal");
    let mut bytes = Vec::new();
    write_stream(&mut bytes, &case.model, &ExportOptions::default()).expect("stream write");
    let first_tag = first_stream_frame_tag_offset(&bytes);
    bytes[first_tag] = 1;

    let Err(err) = read_stream(bytes.as_slice(), &ImportOptions::default()) else {
        panic!("stream tag 1 should be rejected");
    };
    assert!(
        err.to_string().contains("tag 1"),
        "unexpected error for removed transform tag: {err}"
    );
}

fn first_stream_frame_tag_offset(bytes: &[u8]) -> usize {
    let magic_len = b"CITYJSON_ARROW_STREAM_V3\0".len();
    let prelude_len_start = magic_len;
    let prelude_len_end = prelude_len_start + 8;
    let prelude_len = u64::from_le_bytes(
        bytes[prelude_len_start..prelude_len_end]
            .try_into()
            .expect("prelude length bytes"),
    );
    prelude_len_end + usize::try_from(prelude_len).expect("prelude length fits usize")
}

conformance_roundtrip_tests!(
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
