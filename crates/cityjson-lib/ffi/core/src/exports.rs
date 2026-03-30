#![allow(non_camel_case_types)]

use std::ffi::c_char;
use std::ptr::{self, NonNull};
use std::slice;

use cjlib::{CityJSONVersion, CityModel, Error, cityjson::CityModelType, json::RootKind};

use crate::abi::{
    cj_bytes_t, cj_error_kind_t, cj_geometry_type_t, cj_model_capacities_t, cj_model_summary_t,
    cj_model_t, cj_model_type_t, cj_probe_t, cj_status_t, cj_uv_t, cj_uvs_t, cj_vertex_t,
    cj_vertices_t,
};
use crate::error::{
    AbiError, clear_last_error, copy_last_error_message, last_error_kind, last_error_message_len,
    run_ffi,
};
use crate::handle::{
    bytes_free as free_bytes, bytes_from_vec, model_as_mut, model_as_ref, model_free,
    model_into_handle, uvs_free as free_uvs, uvs_from_vec, vertices_free as free_vertices,
    vertices_from_vec,
};

fn invalid_argument(message: impl Into<String>) -> AbiError {
    AbiError::invalid_argument(message)
}

fn ffi_status(result: Result<(), cj_status_t>) -> cj_status_t {
    match result {
        Ok(()) => cj_status_t::CJ_STATUS_SUCCESS,
        Err(status) => status,
    }
}

fn required_bytes<'a>(
    data: *const u8,
    len: usize,
    name: &'static str,
) -> Result<&'a [u8], AbiError> {
    if len == 0 {
        return Ok(&[]);
    }

    let ptr = NonNull::new(data.cast_mut())
        .ok_or_else(|| invalid_argument(format!("{name} must not be null when len is non-zero")))?;

    // SAFETY: the caller promises `len` readable bytes when the pointer is non-null.
    Ok(unsafe { slice::from_raw_parts(ptr.as_ptr().cast_const(), len) })
}

fn required_model_ref<'a>(model: *const cj_model_t) -> Result<&'a CityModel, AbiError> {
    // SAFETY: null is rejected here; valid handles originate from Rust.
    unsafe { model_as_ref(model) }.ok_or_else(|| invalid_argument("model must not be null"))
}

fn required_model_mut<'a>(model: *mut cj_model_t) -> Result<&'a mut CityModel, AbiError> {
    // SAFETY: null is rejected here; valid handles originate from Rust.
    unsafe { model_as_mut(model) }.ok_or_else(|| invalid_argument("model must not be null"))
}

fn write_value<T>(out: *mut T, name: &'static str, value: T) -> Result<(), AbiError> {
    let out =
        NonNull::new(out).ok_or_else(|| invalid_argument(format!("{name} must not be null")))?;

    // SAFETY: `out` is validated to be non-null and points to writable storage.
    unsafe {
        ptr::write(out.as_ptr(), value);
    }

    Ok(())
}

fn write_model_handle(out_model: *mut *mut cj_model_t, model: CityModel) -> Result<(), AbiError> {
    write_value(out_model, "out_model", model_into_handle(model))
}

fn write_bytes(out_bytes: *mut cj_bytes_t, bytes: Vec<u8>) -> Result<(), AbiError> {
    write_value(out_bytes, "out_bytes", bytes_from_vec(bytes))
}

fn write_vertices(
    out_vertices: *mut cj_vertices_t,
    vertices: Vec<cj_vertex_t>,
) -> Result<(), AbiError> {
    write_value(out_vertices, "out_vertices", vertices_from_vec(vertices))
}

fn write_uvs(out_uvs: *mut cj_uvs_t, uvs: Vec<cj_uv_t>) -> Result<(), AbiError> {
    write_value(out_uvs, "out_uvs", uvs_from_vec(uvs))
}

fn copy_string_bytes(value: Option<&str>) -> Vec<u8> {
    value.unwrap_or_default().as_bytes().to_vec()
}

fn reject_unsupported_document_version(version: Option<CityJSONVersion>) -> Result<(), AbiError> {
    match version {
        Some(CityJSONVersion::V2_0) => Ok(()),
        Some(found) => Err(AbiError::from(Error::UnsupportedVersion {
            found: found.to_string(),
            supported: CityJSONVersion::V2_0.to_string(),
        })),
        None => Err(AbiError::from(Error::MissingVersion)),
    }
}

fn reject_unsupported_feature_version(version: Option<CityJSONVersion>) -> Result<(), AbiError> {
    match version {
        Some(found) => Err(AbiError::from(Error::UnsupportedVersion {
            found: found.to_string(),
            supported: CityJSONVersion::V2_0.to_string(),
        })),
        None => Ok(()),
    }
}

fn summarize_model(model: &CityModel) -> cj_model_summary_t {
    let inner = model.as_inner();
    let extension_count = inner.extensions().map_or(0, |extensions| extensions.len());
    let material_count = inner.material_count();
    let texture_count = inner.texture_count();
    let uv_coordinate_count = inner.vertices_texture().len();
    let geometry_template_count = inner.geometry_template_count();
    let template_vertex_count = inner.template_vertices().len();

    cj_model_summary_t {
        model_type: inner.type_citymodel().into(),
        version: if inner.version().is_some() {
            crate::abi::cj_version_t::CJ_VERSION_V2_0
        } else {
            crate::abi::cj_version_t::CJ_VERSION_UNKNOWN
        },
        cityobject_count: inner.cityobjects().len(),
        geometry_count: inner.geometry_count(),
        geometry_template_count,
        vertex_count: inner.vertices().len(),
        template_vertex_count,
        uv_coordinate_count,
        semantic_count: inner.semantic_count(),
        material_count,
        texture_count,
        extension_count,
        has_metadata: inner.metadata().is_some(),
        has_transform: inner.transform().is_some(),
        has_templates: geometry_template_count > 0 || template_vertex_count > 0,
        has_appearance: material_count > 0 || texture_count > 0 || uv_coordinate_count > 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_free(handle: *mut cj_model_t) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        if handle.is_null() {
            return Ok(());
        }

        // SAFETY: the ABI only frees handles that it allocated.
        unsafe {
            model_free(handle);
        }

        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_bytes_free(bytes: cj_bytes_t) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        if bytes.data.is_null() {
            if bytes.len == 0 {
                return Ok(());
            }

            return Err(invalid_argument(
                "bytes data must not be null when len is non-zero",
            ));
        }

        // SAFETY: the ABI only frees buffers allocated by `bytes_from_vec`.
        unsafe {
            free_bytes(bytes);
        }

        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_vertices_free(vertices: cj_vertices_t) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        if vertices.data.is_null() {
            if vertices.len == 0 {
                return Ok(());
            }

            return Err(invalid_argument(
                "vertices data must not be null when len is non-zero",
            ));
        }

        // SAFETY: the ABI only frees buffers allocated by `vertices_from_vec`.
        unsafe {
            free_vertices(vertices);
        }

        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_uvs_free(uvs: cj_uvs_t) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        if uvs.data.is_null() {
            if uvs.len == 0 {
                return Ok(());
            }

            return Err(invalid_argument(
                "uvs data must not be null when len is non-zero",
            ));
        }

        // SAFETY: the ABI only frees buffers allocated by `uvs_from_vec`.
        unsafe {
            free_uvs(uvs);
        }

        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_last_error_kind() -> cj_error_kind_t {
    last_error_kind()
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_last_error_message_len() -> usize {
    last_error_message_len()
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_last_error_message_copy(
    buffer: *mut u8,
    capacity: usize,
    out_len: *mut usize,
) -> cj_status_t {
    // SAFETY: this helper validates the out-pointer and buffer/capacity pairing.
    unsafe { copy_last_error_message(buffer.cast::<c_char>(), capacity, out_len) }
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_clear_error() -> cj_status_t {
    clear_last_error();
    cj_status_t::CJ_STATUS_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_probe_bytes(
    data: *const u8,
    len: usize,
    out_probe: *mut cj_probe_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let input = required_bytes(data, len, "data")?;
        let probe = cjlib::json::probe(input)?;
        write_value(out_probe, "out_probe", cj_probe_t::from_probe(&probe))
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_parse_document_bytes(
    data: *const u8,
    len: usize,
    out_model: *mut *mut cj_model_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let input = required_bytes(data, len, "data")?;
        let probe = cjlib::json::probe(input)?;
        if probe.kind() != RootKind::CityJSON {
            return Err(AbiError::from(Error::ExpectedCityJSON(
                probe.kind().to_string(),
            )));
        }

        reject_unsupported_document_version(probe.version())?;
        let model = cjlib::json::from_slice(input)?;
        write_model_handle(out_model, model)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_parse_feature_bytes(
    data: *const u8,
    len: usize,
    out_model: *mut *mut cj_model_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let input = required_bytes(data, len, "data")?;
        let probe = cjlib::json::probe(input)?;
        if probe.kind() != RootKind::CityJSONFeature {
            return Err(AbiError::from(Error::ExpectedCityJSONFeature(
                probe.kind().to_string(),
            )));
        }

        reject_unsupported_feature_version(probe.version())?;
        let model = cjlib::json::from_feature_slice(input)?;
        write_model_handle(out_model, model)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_parse_feature_with_base_bytes(
    feature_data: *const u8,
    feature_len: usize,
    base_data: *const u8,
    base_len: usize,
    out_model: *mut *mut cj_model_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let feature = required_bytes(feature_data, feature_len, "feature_data")?;
        let base = required_bytes(base_data, base_len, "base_data")?;

        let feature_probe = cjlib::json::probe(feature)?;
        if feature_probe.kind() != RootKind::CityJSONFeature {
            return Err(AbiError::from(Error::ExpectedCityJSONFeature(
                feature_probe.kind().to_string(),
            )));
        }

        reject_unsupported_feature_version(feature_probe.version())?;

        let base_probe = cjlib::json::probe(base)?;
        if base_probe.kind() != RootKind::CityJSON {
            return Err(AbiError::from(Error::ExpectedCityJSON(
                base_probe.kind().to_string(),
            )));
        }

        reject_unsupported_document_version(base_probe.version())?;
        let model = cjlib::json::from_feature_slice_with_base(feature, base)?;
        write_model_handle(out_model, model)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_serialize_document(
    model: *const cj_model_t,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model = required_model_ref(model)?;
        let bytes = cjlib::json::to_vec(model)?;
        write_bytes(out_bytes, bytes)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_serialize_feature(
    model: *const cj_model_t,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model = required_model_ref(model)?;
        let bytes = cjlib::json::to_feature_string(model)?.into_bytes();
        write_bytes(out_bytes, bytes)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_get_summary(
    model: *const cj_model_t,
    out_summary: *mut cj_model_summary_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model = required_model_ref(model)?;
        write_value(out_summary, "out_summary", summarize_model(model))
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_get_metadata_title(
    model: *const cj_model_t,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model = required_model_ref(model)?;
        let bytes = copy_string_bytes(
            model
                .as_inner()
                .metadata()
                .and_then(|metadata| metadata.title()),
        );
        write_bytes(out_bytes, bytes)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_get_metadata_identifier(
    model: *const cj_model_t,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model = required_model_ref(model)?;
        let bytes = copy_string_bytes(
            model
                .as_inner()
                .metadata()
                .and_then(|metadata| metadata.identifier())
                .map(|identifier| identifier.to_string())
                .as_deref(),
        );
        write_bytes(out_bytes, bytes)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_get_cityobject_id(
    model: *const cj_model_t,
    index: usize,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model = required_model_ref(model)?;
        let cityobject = model
            .as_inner()
            .cityobjects()
            .iter()
            .nth(index)
            .map(|(_, cityobject)| cityobject)
            .ok_or_else(|| invalid_argument(format!("cityobject index {index} is out of range")))?;
        write_bytes(out_bytes, cityobject.id().as_bytes().to_vec())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_get_geometry_type(
    model: *const cj_model_t,
    index: usize,
    out_type: *mut cj_geometry_type_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model = required_model_ref(model)?;
        let geometry_type = model
            .as_inner()
            .iter_geometries()
            .nth(index)
            .map(|(_, geometry)| *geometry.type_geometry())
            .ok_or_else(|| invalid_argument(format!("geometry index {index} is out of range")))?;
        write_value(out_type, "out_type", geometry_type.into())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_copy_vertices(
    model: *const cj_model_t,
    out_vertices: *mut cj_vertices_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model = required_model_ref(model)?;
        let vertices = model
            .as_inner()
            .vertices()
            .as_slice()
            .iter()
            .copied()
            .map(Into::into)
            .collect();
        write_vertices(out_vertices, vertices)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_copy_template_vertices(
    model: *const cj_model_t,
    out_vertices: *mut cj_vertices_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model = required_model_ref(model)?;
        let vertices = model
            .as_inner()
            .template_vertices()
            .as_slice()
            .iter()
            .copied()
            .map(Into::into)
            .collect();
        write_vertices(out_vertices, vertices)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_copy_uv_coordinates(
    model: *const cj_model_t,
    out_uvs: *mut cj_uvs_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model = required_model_ref(model)?;
        let uvs = model
            .as_inner()
            .vertices_texture()
            .as_slice()
            .iter()
            .cloned()
            .map(Into::into)
            .collect();
        write_uvs(out_uvs, uvs)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_create(
    model_type: cj_model_type_t,
    out_model: *mut *mut cj_model_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model = CityModel::new(CityModelType::from(model_type));
        write_model_handle(out_model, model)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_reserve_import(
    model: *mut cj_model_t,
    capacities: cj_model_capacities_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        required_model_mut(model)?
            .as_inner_mut()
            .reserve_import(capacities.into())
            .map_err(cjlib::Error::from)?;
        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_add_vertex(
    model: *mut cj_model_t,
    vertex: cj_vertex_t,
    out_index: *mut usize,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let index = required_model_mut(model)?
            .as_inner_mut()
            .add_vertex(vertex.into())
            .map_err(cjlib::Error::from)?
            .to_usize();
        write_value(out_index, "out_index", index)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_add_template_vertex(
    model: *mut cj_model_t,
    vertex: cj_vertex_t,
    out_index: *mut usize,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let index = required_model_mut(model)?
            .as_inner_mut()
            .add_template_vertex(vertex.into())
            .map_err(cjlib::Error::from)?
            .to_usize();
        write_value(out_index, "out_index", index)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_add_uv_coordinate(
    model: *mut cj_model_t,
    uv: cj_uv_t,
    out_index: *mut usize,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let index = required_model_mut(model)?
            .as_inner_mut()
            .add_uv_coordinate(uv.into())
            .map_err(cjlib::Error::from)?
            .to_usize();
        write_value(out_index, "out_index", index)
    }))
}
