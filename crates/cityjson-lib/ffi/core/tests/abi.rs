use std::ptr;
use std::slice;

use cityjson_lib_ffi_core::exports::*;
use cityjson_lib_ffi_core::{
    AbiError, cj_bytes_list_t, cj_bytes_t, cj_cityobject_id_t, cj_error_kind_t,
    cj_geometry_boundary_t, cj_geometry_id_t, cj_geometry_type_t, cj_geometry_types_t,
    cj_indices_t, cj_json_write_options_t, cj_model_capacities_t, cj_model_summary_t, cj_model_t,
    cj_model_type_t, cj_probe_t, cj_root_kind_t, cj_status_t, cj_string_view_t, cj_transform_t,
    cj_uv_t, cj_uvs_t, cj_version_t, cj_vertex_t, cj_vertices_t, run_ffi,
};

fn v2_document() -> &'static [u8] {
    include_bytes!("../../../tests/data/v2_0/minimal.city.json")
}

fn v1_document() -> &'static [u8] {
    include_bytes!("../../../tests/data/v1_1/cityjson_minimal_complete.city.json")
}

fn feature_payload() -> &'static [u8] {
    br#"{"type":"CityJSONFeature","id":"feature-1","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}"#
}

fn subset_fixture() -> &'static [u8] {
    include_bytes!("../../../tests/data/v2_0/ops/subset_source.city.json")
}

fn merge_left_fixture() -> &'static [u8] {
    include_bytes!("../../../tests/data/v2_0/ops/merge_left.city.json")
}

fn merge_right_fixture() -> &'static [u8] {
    include_bytes!("../../../tests/data/v2_0/ops/merge_right.city.json")
}

fn bytes_to_string(bytes: cj_bytes_t) -> String {
    if bytes.len == 0 {
        let _ = cj_bytes_free(bytes);
        return String::new();
    }

    // SAFETY: the ABI returned `len` readable bytes.
    let value = unsafe { slice::from_raw_parts(bytes.data.cast_const(), bytes.len) };
    let string = std::str::from_utf8(value)
        .expect("ffi bytes should be valid utf-8 in this test")
        .to_owned();
    assert_eq!(cj_bytes_free(bytes), cj_status_t::CJ_STATUS_SUCCESS);
    string
}

fn bytes_to_vec(bytes: cj_bytes_t) -> Vec<u8> {
    if bytes.len == 0 {
        let _ = cj_bytes_free(bytes);
        return Vec::new();
    }

    // SAFETY: the ABI returned `len` readable bytes.
    let value = unsafe { slice::from_raw_parts(bytes.data.cast_const(), bytes.len) }.to_vec();
    assert_eq!(cj_bytes_free(bytes), cj_status_t::CJ_STATUS_SUCCESS);
    value
}

fn bytes_list_to_vec(bytes: cj_bytes_list_t) -> Vec<String> {
    if bytes.len == 0 {
        let _ = cj_bytes_list_free(bytes);
        return Vec::new();
    }

    // SAFETY: the ABI returned `len` readable owned strings.
    let values = unsafe { slice::from_raw_parts(bytes.data.cast_const(), bytes.len) }
        .iter()
        .map(|item| {
            if item.len == 0 {
                String::new()
            } else {
                // SAFETY: the ABI returned `len` readable UTF-8 bytes.
                let value = unsafe { slice::from_raw_parts(item.data.cast_const(), item.len) };
                std::str::from_utf8(value)
                    .expect("ffi bytes should be valid utf-8 in this test")
                    .to_owned()
            }
        })
        .collect();
    assert_eq!(cj_bytes_list_free(bytes), cj_status_t::CJ_STATUS_SUCCESS);
    values
}

fn geometry_types_to_vec(types: cj_geometry_types_t) -> Vec<cj_geometry_type_t> {
    if types.len == 0 {
        let _ = cj_geometry_types_free(types);
        return Vec::new();
    }

    // SAFETY: the ABI returned `len` readable geometry types.
    let values = unsafe { slice::from_raw_parts(types.data.cast_const(), types.len) }.to_vec();
    assert_eq!(
        cj_geometry_types_free(types),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    values
}

fn string_view(value: &str) -> cj_string_view_t {
    cj_string_view_t {
        data: value.as_ptr(),
        len: value.len(),
    }
}

fn vertices_to_vec(vertices: cj_vertices_t) -> Vec<cj_vertex_t> {
    if vertices.len == 0 {
        let _ = cj_vertices_free(vertices);
        return Vec::new();
    }

    // SAFETY: the ABI returned `len` readable vertices.
    let values =
        unsafe { slice::from_raw_parts(vertices.data.cast_const(), vertices.len) }.to_vec();
    assert_eq!(cj_vertices_free(vertices), cj_status_t::CJ_STATUS_SUCCESS);
    values
}

fn uvs_to_vec(uvs: cj_uvs_t) -> Vec<cj_uv_t> {
    if uvs.len == 0 {
        let _ = cj_uvs_free(uvs);
        return Vec::new();
    }

    // SAFETY: the ABI returned `len` readable UV coordinates.
    let values = unsafe { slice::from_raw_parts(uvs.data.cast_const(), uvs.len) }.to_vec();
    assert_eq!(cj_uvs_free(uvs), cj_status_t::CJ_STATUS_SUCCESS);
    values
}

fn indices_to_vec(indices: cj_indices_t) -> Vec<usize> {
    if indices.len == 0 {
        let _ = cj_indices_free(indices);
        return Vec::new();
    }

    // SAFETY: the ABI returned `len` readable indices.
    let values = unsafe { slice::from_raw_parts(indices.data.cast_const(), indices.len) }.to_vec();
    assert_eq!(cj_indices_free(indices), cj_status_t::CJ_STATUS_SUCCESS);
    values
}

#[derive(Debug, PartialEq, Eq)]
struct BoundaryPayload {
    geometry_type: cj_geometry_type_t,
    has_boundaries: bool,
    vertex_indices: Vec<usize>,
    ring_offsets: Vec<usize>,
    surface_offsets: Vec<usize>,
    shell_offsets: Vec<usize>,
    solid_offsets: Vec<usize>,
}

fn boundary_to_payload(boundary: cj_geometry_boundary_t) -> BoundaryPayload {
    BoundaryPayload {
        geometry_type: boundary.geometry_type,
        has_boundaries: boundary.has_boundaries,
        vertex_indices: indices_to_vec(boundary.vertex_indices),
        ring_offsets: indices_to_vec(boundary.ring_offsets),
        surface_offsets: indices_to_vec(boundary.surface_offsets),
        shell_offsets: indices_to_vec(boundary.shell_offsets),
        solid_offsets: indices_to_vec(boundary.solid_offsets),
    }
}

fn model_summary(handle: *const cj_model_t) -> cj_model_summary_t {
    let mut summary = cj_model_summary_t::default();
    assert_eq!(
        cj_model_get_summary(handle, &raw mut summary),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    summary
}

fn assert_same_transport_summary(actual: cj_model_summary_t, expected: cj_model_summary_t) {
    assert_eq!(actual.model_type, expected.model_type);
    assert_eq!(actual.version, expected.version);
    assert_eq!(actual.cityobject_count, expected.cityobject_count);
    assert_eq!(actual.geometry_count, expected.geometry_count);
    assert_eq!(
        actual.geometry_template_count,
        expected.geometry_template_count
    );
    assert_eq!(actual.vertex_count, expected.vertex_count);
    assert_eq!(actual.template_vertex_count, expected.template_vertex_count);
    assert_eq!(actual.uv_coordinate_count, expected.uv_coordinate_count);
    assert_eq!(actual.semantic_count, expected.semantic_count);
    assert_eq!(actual.material_count, expected.material_count);
    assert_eq!(actual.texture_count, expected.texture_count);
    assert_eq!(actual.extension_count, expected.extension_count);
    assert_eq!(actual.has_metadata, expected.has_metadata);
    assert_eq!(actual.has_templates, expected.has_templates);
    assert_eq!(actual.has_appearance, expected.has_appearance);
}

#[test]
fn free_functions_accept_null_handles_and_buffers() {
    assert_eq!(
        cj_model_free(ptr::null_mut()),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        cj_bytes_free(cj_bytes_t::default()),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        cj_bytes_list_free(cj_bytes_list_t::default()),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        cj_vertices_free(cj_vertices_t::default()),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        cj_uvs_free(cj_uvs_t::default()),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        cj_indices_free(cj_indices_t::default()),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        cj_geometry_types_free(cj_geometry_types_t::default()),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        cj_geometry_boundary_free(cj_geometry_boundary_t::default()),
        cj_status_t::CJ_STATUS_SUCCESS
    );
}

#[test]
fn probe_reports_root_kind_and_version() {
    let mut probe = cj_probe_t::default();

    let status = cj_probe_bytes(v2_document().as_ptr(), v2_document().len(), &raw mut probe);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(probe.root_kind, cj_root_kind_t::CJ_ROOT_KIND_CITY_JSON);
    assert_eq!(probe.version, cj_version_t::CJ_VERSION_V2_0);
    assert!(probe.has_version);
}

#[test]
fn parse_and_serialize_document_round_trip() {
    let mut handle = ptr::null_mut();
    let status =
        cj_model_parse_document_bytes(v2_document().as_ptr(), v2_document().len(), &raw mut handle);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert!(!handle.is_null());

    let mut serialized = cj_bytes_t::default();
    let status = cj_model_serialize_document(handle, &raw mut serialized);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert!(!serialized.data.is_null());
    assert!(serialized.len > 0);

    let mut probe = cj_probe_t::default();
    let status = cj_probe_bytes(serialized.data, serialized.len, &raw mut probe);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(probe.root_kind, cj_root_kind_t::CJ_ROOT_KIND_CITY_JSON);
    assert_eq!(probe.version, cj_version_t::CJ_VERSION_V2_0);

    let mut round_trip = ptr::null_mut();
    let status =
        cj_model_parse_document_bytes(serialized.data, serialized.len, &raw mut round_trip);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert!(!round_trip.is_null());

    assert_eq!(cj_model_free(round_trip), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_bytes_free(serialized), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_model_free(handle), cj_status_t::CJ_STATUS_SUCCESS);
}

#[test]
fn parse_feature_with_base_and_serialize_feature_round_trip() {
    let mut handle = ptr::null_mut();
    let status = cj_model_parse_feature_with_base_bytes(
        feature_payload().as_ptr(),
        feature_payload().len(),
        v2_document().as_ptr(),
        v2_document().len(),
        &raw mut handle,
    );
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert!(!handle.is_null());

    let mut serialized = cj_bytes_t::default();
    let status = cj_model_serialize_feature(handle, &raw mut serialized);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert!(!serialized.data.is_null());
    assert!(serialized.len > 0);

    let mut probe = cj_probe_t::default();
    let status = cj_probe_bytes(serialized.data, serialized.len, &raw mut probe);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(
        probe.root_kind,
        cj_root_kind_t::CJ_ROOT_KIND_CITY_JSON_FEATURE
    );

    let mut round_trip = ptr::null_mut();
    let status = cj_model_parse_feature_with_base_bytes(
        serialized.data,
        serialized.len,
        v2_document().as_ptr(),
        v2_document().len(),
        &raw mut round_trip,
    );
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert!(!round_trip.is_null());

    assert_eq!(cj_model_free(round_trip), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_bytes_free(serialized), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_model_free(handle), cj_status_t::CJ_STATUS_SUCCESS);
}

#[cfg(feature = "native-formats")]
#[test]
fn native_format_ffi_roundtrips_arrow_and_parquet() {
    let mut handle = ptr::null_mut();
    let status =
        cj_model_parse_document_bytes(v2_document().as_ptr(), v2_document().len(), &raw mut handle);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    let expected = model_summary(handle);

    let mut arrow_payload = cj_bytes_t::default();
    assert_eq!(
        cj_model_serialize_arrow_bytes(handle, &raw mut arrow_payload),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    let arrow_bytes = bytes_to_vec(arrow_payload);
    assert!(!arrow_bytes.is_empty());

    let mut arrow_handle = ptr::null_mut();
    assert_eq!(
        cj_model_parse_arrow_bytes(
            arrow_bytes.as_ptr(),
            arrow_bytes.len(),
            &raw mut arrow_handle
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_same_transport_summary(model_summary(arrow_handle), expected);

    let tempdir = tempfile::tempdir().expect("tempdir should be created");
    let package_path = tempdir.path().join("minimal.cityjson-parquet");
    let package_path = package_path
        .to_str()
        .expect("test path should be valid UTF-8")
        .to_owned();
    assert_eq!(
        cj_model_serialize_parquet_file(handle, string_view(&package_path)),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    let mut package_handle = ptr::null_mut();
    assert_eq!(
        cj_model_parse_parquet_file(string_view(&package_path), &raw mut package_handle),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_same_transport_summary(model_summary(package_handle), expected);

    let dataset_path = tempdir.path().join("minimal.dataset");
    let dataset_path = dataset_path
        .to_str()
        .expect("test path should be valid UTF-8")
        .to_owned();
    assert_eq!(
        cj_model_serialize_parquet_dataset_dir(handle, string_view(&dataset_path)),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    let mut dataset_handle = ptr::null_mut();
    assert_eq!(
        cj_model_parse_parquet_dataset_dir(string_view(&dataset_path), &raw mut dataset_handle),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_same_transport_summary(model_summary(dataset_handle), expected);

    assert_eq!(
        cj_model_free(dataset_handle),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        cj_model_free(package_handle),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(cj_model_free(arrow_handle), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_model_free(handle), cj_status_t::CJ_STATUS_SUCCESS);
}

#[test]
fn summary_and_indexed_inspection_cover_basic_model_state() {
    let mut handle = ptr::null_mut();
    let status =
        cj_model_parse_document_bytes(v2_document().as_ptr(), v2_document().len(), &raw mut handle);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);

    let mut summary = cj_model_summary_t::default();
    let status = cj_model_get_summary(handle, &raw mut summary);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(summary.model_type, cj_model_type_t::CJ_MODEL_TYPE_CITY_JSON);
    assert_eq!(summary.version, cj_version_t::CJ_VERSION_V2_0);
    assert_eq!(summary.cityobject_count, 2);
    assert_eq!(summary.geometry_count, 2);
    assert_eq!(summary.vertex_count, 5);
    assert_eq!(summary.uv_coordinate_count, 4);
    assert_eq!(summary.semantic_count, 1);
    assert_eq!(summary.material_count, 1);
    assert_eq!(summary.texture_count, 1);
    assert_eq!(summary.extension_count, 1);
    assert!(summary.has_metadata);
    assert!(summary.has_transform);
    assert!(summary.has_appearance);
    assert!(!summary.has_templates);

    let mut title = cj_bytes_t::default();
    assert_eq!(
        cj_model_get_metadata_title(handle, &raw mut title),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(bytes_to_string(title), "Facade Fixture");

    let mut identifier = cj_bytes_t::default();
    assert_eq!(
        cj_model_get_metadata_identifier(handle, &raw mut identifier),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(bytes_to_string(identifier), "fixture-1");

    let mut object0 = cj_bytes_t::default();
    assert_eq!(
        cj_model_get_cityobject_id(handle, 0, &raw mut object0),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(bytes_to_string(object0), "building-1");

    let mut object1 = cj_bytes_t::default();
    assert_eq!(
        cj_model_get_cityobject_id(handle, 1, &raw mut object1),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(bytes_to_string(object1), "building-part-1");

    let mut geometry0 = cj_geometry_type_t::default();
    assert_eq!(
        cj_model_get_geometry_type(handle, 0, &raw mut geometry0),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        geometry0,
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_SURFACE
    );

    let mut geometry1 = cj_geometry_type_t::default();
    assert_eq!(
        cj_model_get_geometry_type(handle, 1, &raw mut geometry1),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(geometry1, cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_POINT);

    let mut cityobject_ids = cj_bytes_list_t::default();
    assert_eq!(
        cj_model_copy_cityobject_ids(handle, &raw mut cityobject_ids),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        bytes_list_to_vec(cityobject_ids),
        vec!["building-1".to_owned(), "building-part-1".to_owned()]
    );

    let mut geometry_types = cj_geometry_types_t::default();
    assert_eq!(
        cj_model_copy_geometry_types(handle, &raw mut geometry_types),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        geometry_types_to_vec(geometry_types),
        vec![
            cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_SURFACE,
            cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_POINT,
        ]
    );

    assert_eq!(cj_model_free(handle), cj_status_t::CJ_STATUS_SUCCESS);
}

#[test]
fn copied_vertex_and_uv_buffers_are_owned_and_readable() {
    let mut handle = ptr::null_mut();
    let status =
        cj_model_parse_document_bytes(v2_document().as_ptr(), v2_document().len(), &raw mut handle);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);

    let mut vertices = cj_vertices_t::default();
    assert_eq!(
        cj_model_copy_vertices(handle, &raw mut vertices),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    let vertices = vertices_to_vec(vertices);
    assert_eq!(vertices.len(), 5);
    assert_eq!(
        vertices[0],
        cj_vertex_t {
            x: 10.0,
            y: 20.0,
            z: 0.0
        }
    );
    assert_eq!(
        vertices[4],
        cj_vertex_t {
            x: 12.0,
            y: 22.0,
            z: 0.0
        }
    );

    let mut template_vertices = cj_vertices_t::default();
    assert_eq!(
        cj_model_copy_template_vertices(handle, &raw mut template_vertices),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert!(vertices_to_vec(template_vertices).is_empty());

    let mut uvs = cj_uvs_t::default();
    assert_eq!(
        cj_model_copy_uv_coordinates(handle, &raw mut uvs),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    let uvs = uvs_to_vec(uvs);
    assert_eq!(uvs.len(), 4);
    assert_eq!(uvs[0], cj_uv_t { u: 0.0, v: 0.0 });
    assert_eq!(uvs[2], cj_uv_t { u: 1.0, v: 1.0 });

    assert_eq!(cj_model_free(handle), cj_status_t::CJ_STATUS_SUCCESS);
}

#[test]
fn geometry_boundary_and_coordinate_extraction_are_columnar_and_owned() {
    let mut handle = ptr::null_mut();
    let status =
        cj_model_parse_document_bytes(v2_document().as_ptr(), v2_document().len(), &raw mut handle);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);

    let mut boundary = cj_geometry_boundary_t::default();
    assert_eq!(
        cj_model_copy_geometry_boundary(handle, 0, &raw mut boundary),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        boundary_to_payload(boundary),
        BoundaryPayload {
            geometry_type: cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_SURFACE,
            has_boundaries: true,
            vertex_indices: vec![0, 1, 2, 3, 0],
            ring_offsets: vec![0],
            surface_offsets: vec![0],
            shell_offsets: Vec::new(),
            solid_offsets: Vec::new(),
        }
    );

    let mut surface_vertices = cj_vertices_t::default();
    assert_eq!(
        cj_model_copy_geometry_boundary_coordinates(handle, 0, &raw mut surface_vertices),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        vertices_to_vec(surface_vertices),
        vec![
            cj_vertex_t {
                x: 10.0,
                y: 20.0,
                z: 0.0,
            },
            cj_vertex_t {
                x: 11.0,
                y: 20.0,
                z: 0.0,
            },
            cj_vertex_t {
                x: 11.0,
                y: 21.0,
                z: 0.0,
            },
            cj_vertex_t {
                x: 10.0,
                y: 21.0,
                z: 0.0,
            },
            cj_vertex_t {
                x: 10.0,
                y: 20.0,
                z: 0.0,
            },
        ]
    );

    let mut point_boundary = cj_geometry_boundary_t::default();
    assert_eq!(
        cj_model_copy_geometry_boundary(handle, 1, &raw mut point_boundary),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        boundary_to_payload(point_boundary),
        BoundaryPayload {
            geometry_type: cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_POINT,
            has_boundaries: true,
            vertex_indices: vec![4],
            ring_offsets: Vec::new(),
            surface_offsets: Vec::new(),
            shell_offsets: Vec::new(),
            solid_offsets: Vec::new(),
        }
    );

    let mut point_vertices = cj_vertices_t::default();
    assert_eq!(
        cj_model_copy_geometry_boundary_coordinates(handle, 1, &raw mut point_vertices),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        vertices_to_vec(point_vertices),
        vec![cj_vertex_t {
            x: 12.0,
            y: 22.0,
            z: 0.0,
        }]
    );

    assert_eq!(cj_model_free(handle), cj_status_t::CJ_STATUS_SUCCESS);
}

#[test]
fn model_creation_reserve_and_vertex_insertion_work() {
    let mut handle = ptr::null_mut();
    assert_eq!(
        cj_model_create(
            cj_model_type_t::CJ_MODEL_TYPE_CITY_JSON_FEATURE,
            &raw mut handle
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert!(!handle.is_null());

    let capacities = cj_model_capacities_t {
        cityobjects: 8,
        vertices: 4,
        semantics: 2,
        materials: 2,
        textures: 2,
        geometries: 4,
        template_vertices: 2,
        template_geometries: 2,
        uv_coordinates: 3,
    };
    assert_eq!(
        cj_model_reserve_import(handle, capacities),
        cj_status_t::CJ_STATUS_SUCCESS
    );

    let mut first_vertex = 0usize;
    assert_eq!(
        cj_model_add_vertex(
            handle,
            cj_vertex_t {
                x: 1.0,
                y: 2.0,
                z: 3.0
            },
            &raw mut first_vertex,
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(first_vertex, 0);

    let mut template_vertex = 0usize;
    assert_eq!(
        cj_model_add_template_vertex(
            handle,
            cj_vertex_t {
                x: 4.0,
                y: 5.0,
                z: 6.0
            },
            &raw mut template_vertex,
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(template_vertex, 0);

    let mut uv_index = 0usize;
    assert_eq!(
        cj_model_add_uv_coordinate(handle, cj_uv_t { u: 0.25, v: 0.75 }, &raw mut uv_index),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(uv_index, 0);

    let mut summary = cj_model_summary_t::default();
    assert_eq!(
        cj_model_get_summary(handle, &raw mut summary),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        summary.model_type,
        cj_model_type_t::CJ_MODEL_TYPE_CITY_JSON_FEATURE
    );
    assert_eq!(summary.vertex_count, 1);
    assert_eq!(summary.template_vertex_count, 1);
    assert_eq!(summary.uv_coordinate_count, 1);
    assert!(summary.has_templates);

    assert_eq!(cj_model_free(handle), cj_status_t::CJ_STATUS_SUCCESS);
}

fn build_targeted_fixture() -> *mut cj_model_t {
    let mut handle = ptr::null_mut();
    assert_eq!(
        cj_model_create(cj_model_type_t::CJ_MODEL_TYPE_CITY_JSON, &raw mut handle),
        cj_status_t::CJ_STATUS_SUCCESS
    );

    assert_eq!(
        cj_model_set_metadata_title(handle, string_view("Generated Fixture")),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        cj_model_set_metadata_identifier(handle, string_view("generated-1")),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        cj_model_set_transform(
            handle,
            cj_transform_t {
                scale_x: 1.0,
                scale_y: 1.0,
                scale_z: 1.0,
                translate_x: 10.0,
                translate_y: 20.0,
                translate_z: 0.0,
            },
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );

    for vertex in [
        cj_vertex_t {
            x: 10.0,
            y: 20.0,
            z: 0.0,
        },
        cj_vertex_t {
            x: 11.0,
            y: 20.0,
            z: 0.0,
        },
        cj_vertex_t {
            x: 11.0,
            y: 21.0,
            z: 0.0,
        },
        cj_vertex_t {
            x: 10.0,
            y: 21.0,
            z: 0.0,
        },
    ] {
        let mut index = 0usize;
        assert_eq!(
            cj_model_add_vertex(handle, vertex, &raw mut index),
            cj_status_t::CJ_STATUS_SUCCESS
        );
    }

    let mut cityobject_draft = ptr::null_mut();
    assert_eq!(
        cj_cityobject_draft_new(
            string_view("building-a"),
            string_view("Building"),
            &raw mut cityobject_draft,
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    let mut cityobject_id = cj_cityobject_id_t::default();
    assert_eq!(
        cj_model_add_cityobject(handle, cityobject_draft, &raw mut cityobject_id),
        cj_status_t::CJ_STATUS_SUCCESS
    );

    let mut ring = ptr::null_mut();
    assert_eq!(
        cj_ring_draft_new(&raw mut ring),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    for index in [0_u32, 1, 2, 3] {
        assert_eq!(
            cj_ring_draft_push_vertex_index(ring, index),
            cj_status_t::CJ_STATUS_SUCCESS
        );
    }

    let mut surface = ptr::null_mut();
    assert_eq!(
        cj_surface_draft_new(ring, &raw mut surface),
        cj_status_t::CJ_STATUS_SUCCESS
    );

    let mut geometry_draft = ptr::null_mut();
    assert_eq!(
        cj_geometry_draft_new(
            cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_SURFACE,
            string_view("2.2"),
            &raw mut geometry_draft,
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        cj_geometry_draft_add_surface(geometry_draft, surface),
        cj_status_t::CJ_STATUS_SUCCESS
    );

    let mut geometry_id = cj_geometry_id_t::default();
    assert_eq!(
        cj_model_add_geometry(handle, geometry_draft, &raw mut geometry_id),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        cj_model_cityobject_add_geometry(handle, cityobject_id, geometry_id),
        cj_status_t::CJ_STATUS_SUCCESS
    );

    handle
}

#[test]
fn targeted_mutation_and_write_options_work() {
    let handle = build_targeted_fixture();

    let mut pretty = cj_bytes_t::default();
    assert_eq!(
        cj_model_serialize_document_with_options(
            handle,
            cj_json_write_options_t {
                pretty: true,
                validate_default_themes: false,
            },
            &raw mut pretty,
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    let pretty = bytes_to_string(pretty);
    assert!(pretty.contains("\"title\": \"Generated Fixture\""));
    assert!(pretty.contains("\"identifier\": \"generated-1\""));
    assert!(pretty.contains("\"transform\""));
    assert!(pretty.contains("\"type\": \"MultiSurface\""));

    assert_eq!(cj_model_free(handle), cj_status_t::CJ_STATUS_SUCCESS);
}

#[test]
fn targeted_cleanup_work() {
    let handle = build_targeted_fixture();

    assert_eq!(
        cj_model_clear_transform(handle),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    let mut compact = cj_bytes_t::default();
    assert_eq!(
        cj_model_serialize_document(handle, &raw mut compact),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    let compact = bytes_to_string(compact);
    assert!(!compact.contains("\"transform\""));

    assert_eq!(
        cj_model_remove_cityobject(handle, string_view("building-a")),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(cj_model_cleanup(handle), cj_status_t::CJ_STATUS_SUCCESS);

    let mut summary = cj_model_summary_t::default();
    assert_eq!(
        cj_model_get_summary(handle, &raw mut summary),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(summary.cityobject_count, 0);

    assert_eq!(cj_model_free(handle), cj_status_t::CJ_STATUS_SUCCESS);
}

#[test]
fn subset_merge_and_feature_stream_exports_work() {
    let mut subset_input = ptr::null_mut();
    assert_eq!(
        cj_model_parse_document_bytes(
            subset_fixture().as_ptr(),
            subset_fixture().len(),
            &raw mut subset_input,
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );

    let ids = [string_view("my-group")];
    let mut subset = ptr::null_mut();
    assert_eq!(
        cj_model_subset_cityobjects(
            subset_input,
            ids.as_ptr(),
            ids.len(),
            false,
            &raw mut subset,
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    let mut subset_summary = cj_model_summary_t::default();
    assert_eq!(
        cj_model_get_summary(subset, &raw mut subset_summary),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(subset_summary.cityobject_count, 3);

    let mut left = ptr::null_mut();
    assert_eq!(
        cj_model_parse_document_bytes(
            merge_left_fixture().as_ptr(),
            merge_left_fixture().len(),
            &raw mut left,
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );

    let mut right = ptr::null_mut();
    assert_eq!(
        cj_model_parse_document_bytes(
            merge_right_fixture().as_ptr(),
            merge_right_fixture().len(),
            &raw mut right,
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );

    let models = [left.cast_const(), right.cast_const()];
    let mut merged = ptr::null_mut();
    assert_eq!(
        cj_model_merge_models(models.as_ptr(), models.len(), &raw mut merged),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    let mut merged_summary = cj_model_summary_t::default();
    assert_eq!(
        cj_model_get_summary(merged, &raw mut merged_summary),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(merged_summary.cityobject_count, 3);
    assert_eq!(merged_summary.geometry_count, 8);
    assert_eq!(merged_summary.geometry_template_count, 2);
    assert_eq!(merged_summary.template_vertex_count, 36);
    assert_eq!(merged_summary.material_count, 5);
    assert_eq!(merged_summary.texture_count, 4);
    assert_eq!(merged_summary.vertex_count, 8);

    let mut base = ptr::null_mut();
    assert_eq!(
        cj_model_parse_document_bytes(v2_document().as_ptr(), v2_document().len(), &raw mut base),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    let mut feature = ptr::null_mut();
    assert_eq!(
        cj_model_parse_feature_bytes(
            feature_payload().as_ptr(),
            feature_payload().len(),
            &raw mut feature,
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );

    let stream_models = [base.cast_const(), feature.cast_const()];
    let mut stream = cj_bytes_t::default();
    assert_eq!(
        cj_model_serialize_feature_stream(
            stream_models.as_ptr(),
            stream_models.len(),
            cj_json_write_options_t::default(),
            &raw mut stream,
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    let stream_text = bytes_to_string(stream);
    assert!(stream_text.contains("\"type\":\"CityJSON\""));
    assert!(stream_text.contains("\"type\":\"CityJSONFeature\""));

    let mut merged_stream = ptr::null_mut();
    assert_eq!(
        cj_model_parse_feature_stream_merge_bytes(
            stream_text.as_ptr(),
            stream_text.len(),
            &raw mut merged_stream,
        ),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    let mut merged_stream_summary = cj_model_summary_t::default();
    assert_eq!(
        cj_model_get_summary(merged_stream, &raw mut merged_stream_summary),
        cj_status_t::CJ_STATUS_SUCCESS
    );
    assert_eq!(
        merged_stream_summary.model_type,
        cj_model_type_t::CJ_MODEL_TYPE_CITY_JSON
    );
    assert!(merged_stream_summary.cityobject_count >= 3);

    assert_eq!(cj_model_free(merged_stream), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_model_free(feature), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_model_free(base), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_model_free(merged), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_model_free(right), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_model_free(left), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_model_free(subset), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_model_free(subset_input), cj_status_t::CJ_STATUS_SUCCESS);
}

#[test]
fn append_rejects_self_aliasing_handles() {
    let mut handle = ptr::null_mut();
    assert_eq!(
        cj_model_parse_document_bytes(v2_document().as_ptr(), v2_document().len(), &raw mut handle),
        cj_status_t::CJ_STATUS_SUCCESS
    );

    assert_eq!(
        cj_model_append_model(handle, handle.cast_const()),
        cj_status_t::CJ_STATUS_INVALID_ARGUMENT
    );
    assert_eq!(
        cj_last_error_kind(),
        cj_error_kind_t::CJ_ERROR_KIND_INVALID_ARGUMENT
    );

    assert_eq!(cj_model_free(handle), cj_status_t::CJ_STATUS_SUCCESS);
}

#[test]
fn unsupported_version_is_reported_without_panicking() {
    let mut handle = ptr::null_mut();
    let status =
        cj_model_parse_document_bytes(v1_document().as_ptr(), v1_document().len(), &raw mut handle);
    assert_eq!(status, cj_status_t::CJ_STATUS_VERSION);
    assert!(handle.is_null());
    assert_eq!(cj_last_error_kind(), cj_error_kind_t::CJ_ERROR_KIND_VERSION);

    let mut len = 0usize;
    let status = cj_last_error_message_copy(ptr::null_mut(), 0, &raw mut len);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert!(len > 0);

    let mut buffer = vec![0u8; len + 1];
    let mut copied = 0usize;
    let status = cj_last_error_message_copy(buffer.as_mut_ptr(), buffer.len(), &raw mut copied);
    assert_eq!(status, cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(copied, len);

    let message = std::str::from_utf8(&buffer[..copied]).expect("last error should be utf-8");
    assert!(message.contains("unsupported"));
}

#[test]
fn last_error_copy_failures_do_not_clobber_the_stored_error() {
    let mut handle = ptr::null_mut();
    let status =
        cj_model_parse_document_bytes(v1_document().as_ptr(), v1_document().len(), &raw mut handle);
    assert_eq!(status, cj_status_t::CJ_STATUS_VERSION);
    assert_eq!(cj_last_error_kind(), cj_error_kind_t::CJ_ERROR_KIND_VERSION);

    let mut copied = 0usize;
    let mut buffer = [0u8; 4];
    let status = cj_last_error_message_copy(buffer.as_mut_ptr(), buffer.len(), &raw mut copied);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);
    assert_eq!(cj_last_error_kind(), cj_error_kind_t::CJ_ERROR_KIND_VERSION);

    assert_eq!(cj_clear_error(), cj_status_t::CJ_STATUS_SUCCESS);
    assert_eq!(cj_last_error_kind(), cj_error_kind_t::CJ_ERROR_KIND_NONE);
    assert_eq!(cj_last_error_message_len(), 0);
}

#[test]
fn null_arguments_are_rejected_and_reported() {
    let mut probe = cj_probe_t::default();
    let status = cj_probe_bytes(ptr::null(), 1, &raw mut probe);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);
    assert_eq!(
        cj_last_error_kind(),
        cj_error_kind_t::CJ_ERROR_KIND_INVALID_ARGUMENT
    );

    let mut document_handle = ptr::null_mut();
    let status = cj_model_parse_document_bytes(ptr::null(), 1, &raw mut document_handle);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);

    let mut feature_handle = ptr::null_mut();
    let status = cj_model_parse_feature_bytes(ptr::null(), 1, &raw mut feature_handle);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);

    let mut feature_with_base_handle = ptr::null_mut();
    let status = cj_model_parse_feature_with_base_bytes(
        ptr::null(),
        1,
        ptr::null(),
        1,
        &raw mut feature_with_base_handle,
    );
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);

    let mut bytes = cj_bytes_t::default();
    let status = cj_model_serialize_document(ptr::null(), &raw mut bytes);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);

    let mut summary = cj_model_summary_t::default();
    let status = cj_model_get_summary(ptr::null(), &raw mut summary);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);

    let mut cityobject_ids = cj_bytes_list_t::default();
    let status = cj_model_copy_cityobject_ids(ptr::null(), &raw mut cityobject_ids);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);

    let mut geometry_types = cj_geometry_types_t::default();
    let status = cj_model_copy_geometry_types(ptr::null(), &raw mut geometry_types);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);

    let mut handle = ptr::null_mut();
    assert_eq!(
        cj_model_parse_document_bytes(v2_document().as_ptr(), v2_document().len(), &raw mut handle),
        cj_status_t::CJ_STATUS_SUCCESS
    );

    let mut boundary = cj_geometry_boundary_t::default();
    let status = cj_model_copy_geometry_boundary(ptr::null(), 0, &raw mut boundary);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);

    let status = cj_model_copy_geometry_boundary(handle, 0, ptr::null_mut());
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);

    let mut vertices = cj_vertices_t::default();
    let status = cj_model_copy_geometry_boundary_coordinates(ptr::null(), 0, &raw mut vertices);
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);

    let status = cj_model_copy_geometry_boundary_coordinates(handle, 0, ptr::null_mut());
    assert_eq!(status, cj_status_t::CJ_STATUS_INVALID_ARGUMENT);

    assert_eq!(cj_model_free(handle), cj_status_t::CJ_STATUS_SUCCESS);
}

#[test]
fn panic_helper_translates_panics_to_internal_status() {
    let status = run_ffi::<(), _, _>(|| -> Result<(), AbiError> {
        panic!("ffi panic test");
    })
    .unwrap_err();

    assert_eq!(status, cj_status_t::CJ_STATUS_INTERNAL);
    assert_eq!(
        cj_last_error_kind(),
        cj_error_kind_t::CJ_ERROR_KIND_INTERNAL
    );
}
