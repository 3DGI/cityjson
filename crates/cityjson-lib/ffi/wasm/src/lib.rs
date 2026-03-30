#![allow(clippy::all, clippy::pedantic)]

//! Narrow wasm-oriented adapter over the shared `cjlib-ffi-core` substrate.
//!
//! The public surface here stays task-oriented: probe input, summarize parsed
//! models, and extract flat coordinate and boundary buffers. Deep editable
//! model handles stay internal to this crate for now.

use std::ptr;
use std::slice;

pub use cjlib_ffi_core as core;

use cjlib_ffi_core::exports::{
    cj_bytes_free, cj_geometry_boundary_free, cj_last_error_message_copy,
    cj_last_error_message_len, cj_model_add_cityobject, cj_model_add_geometry_from_boundary,
    cj_model_add_vertex, cj_model_attach_geometry_to_cityobject, cj_model_cleanup,
    cj_model_copy_geometry_boundary, cj_model_copy_geometry_boundary_coordinates,
    cj_model_copy_template_vertices, cj_model_copy_uv_coordinates, cj_model_copy_vertices,
    cj_model_create, cj_model_free, cj_model_get_cityobject_id, cj_model_get_geometry_type,
    cj_model_get_summary, cj_model_parse_document_bytes, cj_model_parse_feature_bytes,
    cj_model_parse_feature_stream_merge_bytes, cj_model_serialize_document_with_options,
    cj_model_serialize_feature_with_options, cj_model_set_metadata_identifier,
    cj_model_set_metadata_title, cj_model_set_transform, cj_probe_bytes, cj_uvs_free,
    cj_vertices_free,
};
use cjlib_ffi_core::{
    cj_bytes_t, cj_geometry_boundary_t, cj_geometry_boundary_view_t, cj_geometry_type_t,
    cj_indices_view_t, cj_json_write_options_t, cj_model_summary_t, cj_model_t, cj_model_type_t,
    cj_probe_t, cj_status_t, cj_string_view_t, cj_transform_t, cj_uv_t, cj_uvs_t, cj_vertex_t,
    cj_vertices_t,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WriteOptions {
    pub pretty: bool,
    pub validate_default_themes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmError {
    pub status: cj_status_t,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeSummary {
    pub root_kind: core::cj_root_kind_t,
    pub version: core::cj_version_t,
    pub has_version: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSummary {
    pub summary: cj_model_summary_t,
    pub cityobject_ids: Vec<String>,
    pub geometry_types: Vec<cj_geometry_type_t>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoordinateBuffers {
    pub vertices: Vec<cj_vertex_t>,
    pub template_vertices: Vec<cj_vertex_t>,
    pub uv_coordinates: Vec<cj_uv_t>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometryBoundary {
    pub geometry_type: cj_geometry_type_t,
    pub has_boundaries: bool,
    pub vertex_indices: Vec<usize>,
    pub ring_offsets: Vec<usize>,
    pub surface_offsets: Vec<usize>,
    pub shell_offsets: Vec<usize>,
    pub solid_offsets: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeometryBoundaryCoordinates {
    pub geometry_type: cj_geometry_type_t,
    pub coordinates: Vec<cj_vertex_t>,
}

fn last_error_message() -> String {
    let len = cj_last_error_message_len();
    if len == 0 {
        return String::new();
    }

    let mut buffer = vec![0u8; len + 1];
    let mut copied = 0usize;
    let status = cj_last_error_message_copy(buffer.as_mut_ptr(), buffer.len(), &raw mut copied);
    if status != cj_status_t::CJ_STATUS_SUCCESS {
        return "failed to retrieve cjlib last-error message".to_string();
    }

    String::from_utf8_lossy(&buffer[..copied]).into_owned()
}

fn status_result(status: cj_status_t) -> Result<(), WasmError> {
    if status == cj_status_t::CJ_STATUS_SUCCESS {
        return Ok(());
    }

    Err(WasmError {
        status,
        message: last_error_message(),
    })
}

fn string_view(text: &str) -> cj_string_view_t {
    if text.is_empty() {
        cj_string_view_t::null()
    } else {
        cj_string_view_t {
            data: text.as_ptr(),
            len: text.len(),
        }
    }
}

fn indices_view(values: &[usize]) -> cj_indices_view_t {
    if values.is_empty() {
        cj_indices_view_t::null()
    } else {
        cj_indices_view_t {
            data: values.as_ptr(),
            len: values.len(),
        }
    }
}

fn boundary_view(
    geometry_type: cj_geometry_type_t,
    vertex_indices: &[usize],
    ring_offsets: &[usize],
    surface_offsets: &[usize],
    shell_offsets: &[usize],
    solid_offsets: &[usize],
) -> cj_geometry_boundary_view_t {
    cj_geometry_boundary_view_t {
        geometry_type,
        vertex_indices: indices_view(vertex_indices),
        ring_offsets: indices_view(ring_offsets),
        surface_offsets: indices_view(surface_offsets),
        shell_offsets: indices_view(shell_offsets),
        solid_offsets: indices_view(solid_offsets),
    }
}

fn to_core_write_options(options: WriteOptions) -> cj_json_write_options_t {
    cj_json_write_options_t {
        pretty: options.pretty,
        validate_default_themes: options.validate_default_themes,
    }
}

struct ModelHandle(*mut cj_model_t);

impl ModelHandle {
    fn raw(&self) -> *mut cj_model_t {
        self.0
    }
}

impl Drop for ModelHandle {
    fn drop(&mut self) {
        let _ = cj_model_free(self.0);
    }
}

fn take_string(bytes: cj_bytes_t) -> Result<String, WasmError> {
    Ok(String::from_utf8_lossy(&take_bytes(bytes)?).into_owned())
}

fn take_bytes(bytes: cj_bytes_t) -> Result<Vec<u8>, WasmError> {
    let values = if bytes.len == 0 {
        Vec::new()
    } else {
        // SAFETY: the ABI returned `len` readable bytes.
        unsafe { slice::from_raw_parts(bytes.data.cast_const(), bytes.len) }.to_vec()
    };
    status_result(cj_bytes_free(bytes))?;
    Ok(values)
}

fn take_vertices(vertices: cj_vertices_t) -> Result<Vec<cj_vertex_t>, WasmError> {
    let values = if vertices.len == 0 {
        Vec::new()
    } else {
        // SAFETY: the ABI returned `len` readable vertices.
        unsafe { slice::from_raw_parts(vertices.data.cast_const(), vertices.len) }.to_vec()
    };
    status_result(cj_vertices_free(vertices))?;
    Ok(values)
}

fn take_uvs(uvs: cj_uvs_t) -> Result<Vec<cj_uv_t>, WasmError> {
    let values = if uvs.len == 0 {
        Vec::new()
    } else {
        // SAFETY: the ABI returned `len` readable UV coordinates.
        unsafe { slice::from_raw_parts(uvs.data.cast_const(), uvs.len) }.to_vec()
    };
    status_result(cj_uvs_free(uvs))?;
    Ok(values)
}

fn take_boundary(boundary: cj_geometry_boundary_t) -> Result<GeometryBoundary, WasmError> {
    let vertex_indices = if boundary.vertex_indices.len == 0 {
        Vec::new()
    } else {
        // SAFETY: the ABI returned `len` readable indices.
        unsafe {
            slice::from_raw_parts(
                boundary.vertex_indices.data.cast_const(),
                boundary.vertex_indices.len,
            )
        }
        .to_vec()
    };
    let ring_offsets = if boundary.ring_offsets.len == 0 {
        Vec::new()
    } else {
        // SAFETY: the ABI returned `len` readable indices.
        unsafe {
            slice::from_raw_parts(
                boundary.ring_offsets.data.cast_const(),
                boundary.ring_offsets.len,
            )
        }
        .to_vec()
    };
    let surface_offsets = if boundary.surface_offsets.len == 0 {
        Vec::new()
    } else {
        // SAFETY: the ABI returned `len` readable indices.
        unsafe {
            slice::from_raw_parts(
                boundary.surface_offsets.data.cast_const(),
                boundary.surface_offsets.len,
            )
        }
        .to_vec()
    };
    let shell_offsets = if boundary.shell_offsets.len == 0 {
        Vec::new()
    } else {
        // SAFETY: the ABI returned `len` readable indices.
        unsafe {
            slice::from_raw_parts(
                boundary.shell_offsets.data.cast_const(),
                boundary.shell_offsets.len,
            )
        }
        .to_vec()
    };
    let solid_offsets = if boundary.solid_offsets.len == 0 {
        Vec::new()
    } else {
        // SAFETY: the ABI returned `len` readable indices.
        unsafe {
            slice::from_raw_parts(
                boundary.solid_offsets.data.cast_const(),
                boundary.solid_offsets.len,
            )
        }
        .to_vec()
    };

    let payload = GeometryBoundary {
        geometry_type: boundary.geometry_type,
        has_boundaries: boundary.has_boundaries,
        vertex_indices,
        ring_offsets,
        surface_offsets,
        shell_offsets,
        solid_offsets,
    };

    status_result(cj_geometry_boundary_free(boundary))?;
    Ok(payload)
}

pub fn probe_bytes(bytes: &[u8]) -> Result<ProbeSummary, WasmError> {
    let mut probe = cj_probe_t::default();
    status_result(cj_probe_bytes(bytes.as_ptr(), bytes.len(), &raw mut probe))?;
    Ok(ProbeSummary {
        root_kind: probe.root_kind,
        version: probe.version,
        has_version: probe.has_version,
    })
}

fn parse_document(bytes: &[u8]) -> Result<ModelHandle, WasmError> {
    let mut handle = ptr::null_mut();
    status_result(cj_model_parse_document_bytes(
        bytes.as_ptr(),
        bytes.len(),
        &raw mut handle,
    ))?;
    Ok(ModelHandle(handle))
}

fn parse_feature(bytes: &[u8]) -> Result<ModelHandle, WasmError> {
    let mut handle = ptr::null_mut();
    status_result(cj_model_parse_feature_bytes(
        bytes.as_ptr(),
        bytes.len(),
        &raw mut handle,
    ))?;
    Ok(ModelHandle(handle))
}

pub fn parse_document_summary(bytes: &[u8]) -> Result<DocumentSummary, WasmError> {
    let model = parse_document(bytes)?;

    let mut summary = cj_model_summary_t::default();
    status_result(cj_model_get_summary(model.raw(), &raw mut summary))?;

    let mut cityobject_ids = Vec::with_capacity(summary.cityobject_count);
    for index in 0..summary.cityobject_count {
        let mut bytes = cj_bytes_t::default();
        status_result(cj_model_get_cityobject_id(
            model.raw(),
            index,
            &raw mut bytes,
        ))?;
        cityobject_ids.push(take_string(bytes)?);
    }

    let mut geometry_types = Vec::with_capacity(summary.geometry_count);
    for index in 0..summary.geometry_count {
        let mut geometry_type = cj_geometry_type_t::default();
        status_result(cj_model_get_geometry_type(
            model.raw(),
            index,
            &raw mut geometry_type,
        ))?;
        geometry_types.push(geometry_type);
    }

    Ok(DocumentSummary {
        summary,
        cityobject_ids,
        geometry_types,
    })
}

pub fn extract_coordinate_buffers(bytes: &[u8]) -> Result<CoordinateBuffers, WasmError> {
    let model = parse_document(bytes)?;

    let mut vertices = cj_vertices_t::default();
    status_result(cj_model_copy_vertices(model.raw(), &raw mut vertices))?;

    let mut template_vertices = cj_vertices_t::default();
    status_result(cj_model_copy_template_vertices(
        model.raw(),
        &raw mut template_vertices,
    ))?;

    let mut uvs = cj_uvs_t::default();
    status_result(cj_model_copy_uv_coordinates(model.raw(), &raw mut uvs))?;

    Ok(CoordinateBuffers {
        vertices: take_vertices(vertices)?,
        template_vertices: take_vertices(template_vertices)?,
        uv_coordinates: take_uvs(uvs)?,
    })
}

pub fn extract_geometry_boundary(
    bytes: &[u8],
    geometry_index: usize,
) -> Result<GeometryBoundary, WasmError> {
    let model = parse_document(bytes)?;

    let mut boundary = cj_geometry_boundary_t::default();
    status_result(cj_model_copy_geometry_boundary(
        model.raw(),
        geometry_index,
        &raw mut boundary,
    ))?;
    take_boundary(boundary)
}

pub fn extract_geometry_boundary_coordinates(
    bytes: &[u8],
    geometry_index: usize,
) -> Result<GeometryBoundaryCoordinates, WasmError> {
    let model = parse_document(bytes)?;

    let mut coordinates = cj_vertices_t::default();
    status_result(cj_model_copy_geometry_boundary_coordinates(
        model.raw(),
        geometry_index,
        &raw mut coordinates,
    ))?;
    let coordinates = take_vertices(coordinates)?;

    let mut geometry_type = cj_geometry_type_t::default();
    status_result(cj_model_get_geometry_type(
        model.raw(),
        geometry_index,
        &raw mut geometry_type,
    ))?;

    Ok(GeometryBoundaryCoordinates {
        geometry_type,
        coordinates,
    })
}

pub fn serialize_document_with_options(
    bytes: &[u8],
    options: WriteOptions,
) -> Result<Vec<u8>, WasmError> {
    let model = parse_document(bytes)?;
    let mut payload = cj_bytes_t::default();
    status_result(cj_model_serialize_document_with_options(
        model.raw(),
        to_core_write_options(options),
        &raw mut payload,
    ))?;
    take_bytes(payload)
}

pub fn serialize_feature_with_options(
    bytes: &[u8],
    options: WriteOptions,
) -> Result<Vec<u8>, WasmError> {
    let model = parse_feature(bytes)?;
    let mut payload = cj_bytes_t::default();
    status_result(cj_model_serialize_feature_with_options(
        model.raw(),
        to_core_write_options(options),
        &raw mut payload,
    ))?;
    take_bytes(payload)
}

pub fn merge_feature_stream(bytes: &[u8], options: WriteOptions) -> Result<Vec<u8>, WasmError> {
    let mut handle = ptr::null_mut();
    status_result(cj_model_parse_feature_stream_merge_bytes(
        bytes.as_ptr(),
        bytes.len(),
        &raw mut handle,
    ))?;
    let model = ModelHandle(handle);

    let mut payload = cj_bytes_t::default();
    status_result(cj_model_serialize_document_with_options(
        model.raw(),
        to_core_write_options(options),
        &raw mut payload,
    ))?;
    take_bytes(payload)
}

pub fn build_document_roundtrip() -> Result<Vec<u8>, WasmError> {
    let mut handle = ptr::null_mut();
    status_result(cj_model_create(
        cj_model_type_t::CJ_MODEL_TYPE_CITY_JSON,
        &raw mut handle,
    ))?;
    let model = ModelHandle(handle);

    status_result(cj_model_set_metadata_title(
        model.raw(),
        string_view("Wasm roundtrip"),
    ))?;
    status_result(cj_model_set_metadata_identifier(
        model.raw(),
        string_view("wasm-roundtrip"),
    ))?;
    status_result(cj_model_set_transform(
        model.raw(),
        cj_transform_t {
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
            translate_x: 0.0,
            translate_y: 0.0,
            translate_z: 0.0,
        },
    ))?;

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
        let mut vertex_index = 0usize;
        status_result(cj_model_add_vertex(
            model.raw(),
            vertex,
            &raw mut vertex_index,
        ))?;
    }

    status_result(cj_model_add_cityobject(
        model.raw(),
        string_view("building-1"),
        string_view("Building"),
    ))?;

    let mut geometry_index = 0usize;
    status_result(cj_model_add_geometry_from_boundary(
        model.raw(),
        boundary_view(
            cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_SURFACE,
            &[0, 1, 2, 3, 0],
            &[0],
            &[0],
            &[],
            &[],
        ),
        string_view("2.2"),
        &raw mut geometry_index,
    ))?;
    status_result(cj_model_attach_geometry_to_cityobject(
        model.raw(),
        string_view("building-1"),
        geometry_index,
    ))?;
    status_result(cj_model_cleanup(model.raw()))?;

    let mut payload = cj_bytes_t::default();
    status_result(cj_model_serialize_document_with_options(
        model.raw(),
        to_core_write_options(WriteOptions::default()),
        &raw mut payload,
    ))?;
    take_bytes(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_bytes() -> &'static [u8] {
        include_bytes!("../../../tests/data/v2_0/minimal.city.json")
    }

    #[test]
    fn probe_and_summary_are_available_for_browser_facing_tasks() {
        let probe = probe_bytes(fixture_bytes()).expect("probe should succeed");
        assert_eq!(
            probe.root_kind,
            core::cj_root_kind_t::CJ_ROOT_KIND_CITY_JSON
        );
        assert_eq!(probe.version, core::cj_version_t::CJ_VERSION_V2_0);
        assert!(probe.has_version);

        let summary = parse_document_summary(fixture_bytes()).expect("summary should succeed");
        assert_eq!(summary.summary.cityobject_count, 2);
        assert_eq!(summary.summary.geometry_count, 2);
        assert_eq!(
            summary.cityobject_ids,
            vec!["building-1", "building-part-1"]
        );
        assert_eq!(
            summary.geometry_types,
            vec![
                cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_SURFACE,
                cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_POINT,
            ]
        );
    }

    #[test]
    fn coordinate_extraction_and_minimal_creation_work() {
        let buffers =
            extract_coordinate_buffers(fixture_bytes()).expect("coordinate extraction should work");
        assert_eq!(buffers.vertices.len(), 5);
        assert_eq!(buffers.vertices[0].x, 10.0);
        assert_eq!(buffers.vertices[4].y, 22.0);
        assert!(buffers.template_vertices.is_empty());
        assert_eq!(buffers.uv_coordinates.len(), 4);
        assert_eq!(buffers.uv_coordinates[2].u, 1.0);

        let roundtrip = build_document_roundtrip().expect("document roundtrip should work");
        let summary = parse_document_summary(&roundtrip).expect("roundtrip should parse");
        assert_eq!(
            summary.summary.model_type,
            cj_model_type_t::CJ_MODEL_TYPE_CITY_JSON
        );
        assert_eq!(summary.summary.cityobject_count, 1);
        assert_eq!(summary.summary.geometry_count, 1);
        assert_eq!(summary.cityobject_ids, vec!["building-1"]);
        assert_eq!(
            summary.geometry_types,
            vec![cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_SURFACE]
        );
    }

    #[test]
    fn serialization_options_and_feature_stream_merge_work() {
        let compact = serialize_document_with_options(
            fixture_bytes(),
            WriteOptions {
                pretty: false,
                validate_default_themes: false,
            },
        )
        .expect("compact serialize should work");
        let pretty = serialize_document_with_options(
            fixture_bytes(),
            WriteOptions {
                pretty: true,
                validate_default_themes: false,
            },
        )
        .expect("pretty serialize should work");
        assert!(pretty.len() > compact.len());

        let feature = br#"{"type":"CityJSONFeature","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}"#;
        let feature_roundtrip = serialize_feature_with_options(
            feature,
            WriteOptions {
                pretty: false,
                validate_default_themes: false,
            },
        )
        .expect("feature serialize should work");
        let feature_probe = probe_bytes(&feature_roundtrip).expect("feature probe should work");
        assert_eq!(
            feature_probe.root_kind,
            core::cj_root_kind_t::CJ_ROOT_KIND_CITY_JSON_FEATURE
        );

        let stream = br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}
{"type":"CityJSONFeature","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}
"#;
        let merged = merge_feature_stream(
            stream,
            WriteOptions {
                pretty: false,
                validate_default_themes: false,
            },
        )
        .expect("feature stream merge should work");
        let merged_summary = parse_document_summary(&merged).expect("merged output should parse");
        assert_eq!(merged_summary.summary.cityobject_count, 1);
        assert_eq!(merged_summary.cityobject_ids, vec!["feature-1"]);
    }

    #[test]
    fn geometry_boundary_extraction_matches_fixture_topology() {
        let boundary = extract_geometry_boundary(fixture_bytes(), 0)
            .expect("geometry boundary extraction should work");
        assert_eq!(
            boundary,
            GeometryBoundary {
                geometry_type: cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_SURFACE,
                has_boundaries: true,
                vertex_indices: vec![0, 1, 2, 3, 0],
                ring_offsets: vec![0],
                surface_offsets: vec![0],
                shell_offsets: vec![],
                solid_offsets: vec![],
            }
        );

        let coordinates = extract_geometry_boundary_coordinates(fixture_bytes(), 0)
            .expect("geometry boundary coordinates should work");
        assert_eq!(
            coordinates.geometry_type,
            cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_SURFACE
        );
        assert_eq!(
            coordinates.coordinates,
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
    }
}
