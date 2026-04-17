#![allow(non_camel_case_types)]

use std::ffi::c_char;
#[cfg(feature = "arrow")]
use std::io::Cursor;
use std::ptr::{self, NonNull};
use std::slice;
use std::str::FromStr;

use cityjson_lib::cityjson::v2_0::{
    Boundary, BoundaryNestedMultiLineString, BoundaryNestedMultiOrCompositeSolid,
    BoundaryNestedMultiOrCompositeSurface, BoundaryNestedMultiPoint, BoundaryNestedSolid,
    CityModelIdentifier, CityObject, CityObjectIdentifier, CityObjectType, Geometry, GeometryType,
    LoD, StoredGeometryParts, Transform,
};
use cityjson_lib::{CityJSONVersion, CityModel, Error, cityjson::CityModelType, json::RootKind};

use crate::abi::{
    cj_bytes_t, cj_cityjsonseq_auto_transform_options_t, cj_cityjsonseq_write_options_t,
    cj_error_kind_t, cj_geometry_boundary_t, cj_geometry_boundary_view_t, cj_geometry_type_t,
    cj_indices_t, cj_indices_view_t, cj_json_write_options_t, cj_model_capacities_t,
    cj_model_summary_t, cj_model_t, cj_model_type_t, cj_probe_t, cj_status_t, cj_string_view_t,
    cj_transform_t, cj_uv_t, cj_uvs_t, cj_vertex_t, cj_vertices_t,
};
use crate::error::{
    AbiError, clear_last_error, copy_last_error_message, last_error_kind, last_error_message_len,
    run_ffi,
};
use crate::handle::{
    bytes_free as free_bytes, bytes_from_vec, geometry_boundary_free as free_geometry_boundary,
    indices_free as free_indices, indices_from_vec, model_as_mut, model_as_ref, model_free,
    model_into_handle, uvs_free as free_uvs, uvs_from_vec, vertices_free as free_vertices,
    vertices_from_vec,
};

type OwnedGeometry = cityjson_lib::cityjson::v2_0::Geometry<
    u32,
    cityjson_lib::cityjson::resources::storage::OwnedStringStorage,
>;

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

fn optional_bytes<'a>(
    data: *const u8,
    len: usize,
    name: &'static str,
) -> Result<Option<&'a [u8]>, AbiError> {
    if len == 0 {
        return Ok(None);
    }

    required_bytes(data, len, name).map(Some)
}

fn optional_utf8(
    data: *const u8,
    len: usize,
    name: &'static str,
) -> Result<Option<String>, AbiError> {
    optional_bytes(data, len, name)?
        .map(|bytes| {
            std::str::from_utf8(bytes)
                .map(str::to_owned)
                .map_err(|error| invalid_argument(format!("{name} must be valid UTF-8: {error}")))
        })
        .transpose()
}

fn required_utf8(data: *const u8, len: usize, name: &'static str) -> Result<String, AbiError> {
    let bytes = required_bytes(data, len, name)?;
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|error| invalid_argument(format!("{name} must be valid UTF-8: {error}")))
}

fn view_utf8(view: cj_string_view_t, name: &'static str) -> Result<String, AbiError> {
    required_utf8(view.data, view.len, name)
}

fn optional_view_utf8(
    view: cj_string_view_t,
    name: &'static str,
) -> Result<Option<String>, AbiError> {
    optional_utf8(view.data, view.len, name)
}

fn required_indices_view(
    view: cj_indices_view_t,
    name: &'static str,
) -> Result<&'static [usize], AbiError> {
    if view.len == 0 {
        return Ok(&[]);
    }

    let ptr = NonNull::new(view.data.cast_mut())
        .ok_or_else(|| invalid_argument(format!("{name} must not be null when len is non-zero")))?;

    // SAFETY: the caller promises `len` readable indices when the pointer is non-null.
    Ok(unsafe { slice::from_raw_parts(ptr.as_ptr().cast_const(), view.len) })
}

fn required_string_views(
    data: *const cj_string_view_t,
    len: usize,
    name: &'static str,
) -> Result<&'static [cj_string_view_t], AbiError> {
    if len == 0 {
        return Ok(&[]);
    }

    let ptr = NonNull::new(data.cast_mut())
        .ok_or_else(|| invalid_argument(format!("{name} must not be null when len is non-zero")))?;

    // SAFETY: the caller promises `len` readable views when the pointer is non-null.
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

fn required_out<T>(out: *mut T, name: &'static str) -> Result<NonNull<T>, AbiError> {
    NonNull::new(out).ok_or_else(|| invalid_argument(format!("{name} must not be null")))
}

fn write_model_handle(out_model: *mut *mut cj_model_t, model: CityModel) -> Result<(), AbiError> {
    let out = required_out(out_model, "out_model")?;

    // SAFETY: `out` is validated to be non-null and points to writable storage.
    unsafe {
        ptr::write(out.as_ptr(), model_into_handle(model));
    }

    Ok(())
}

fn write_bytes(out_bytes: *mut cj_bytes_t, bytes: Vec<u8>) -> Result<(), AbiError> {
    let out = required_out(out_bytes, "out_bytes")?;

    // SAFETY: `out` is validated to be non-null and points to writable storage.
    unsafe {
        ptr::write(out.as_ptr(), bytes_from_vec(bytes));
    }

    Ok(())
}

fn write_vertices(
    out_vertices: *mut cj_vertices_t,
    vertices: Vec<cj_vertex_t>,
) -> Result<(), AbiError> {
    let out = required_out(out_vertices, "out_vertices")?;

    // SAFETY: `out` is validated to be non-null and points to writable storage.
    unsafe {
        ptr::write(out.as_ptr(), vertices_from_vec(vertices));
    }

    Ok(())
}

fn write_uvs(out_uvs: *mut cj_uvs_t, uvs: Vec<cj_uv_t>) -> Result<(), AbiError> {
    let out = required_out(out_uvs, "out_uvs")?;

    // SAFETY: `out` is validated to be non-null and points to writable storage.
    unsafe {
        ptr::write(out.as_ptr(), uvs_from_vec(uvs));
    }

    Ok(())
}

fn write_boundary(
    out_boundary: *mut cj_geometry_boundary_t,
    boundary: cj_geometry_boundary_t,
) -> Result<(), AbiError> {
    let out = required_out(out_boundary, "out_boundary")?;

    // SAFETY: `out` is validated to be non-null and points to writable storage.
    unsafe {
        ptr::write(out.as_ptr(), boundary);
    }

    Ok(())
}

fn required_model_refs<'a>(
    models: *const *const cj_model_t,
    model_count: usize,
    name: &'static str,
) -> Result<Vec<&'a CityModel>, AbiError> {
    if model_count == 0 {
        return Ok(Vec::new());
    }

    let models_ptr = NonNull::new(models.cast_mut()).ok_or_else(|| {
        invalid_argument(format!(
            "{name} must not be null when model_count is non-zero"
        ))
    })?;
    let models = unsafe { slice::from_raw_parts(models_ptr.as_ptr().cast_const(), model_count) };
    models
        .iter()
        .map(|handle| required_model_ref(*handle))
        .collect::<Result<Vec<_>, _>>()
}

fn transform_from_abi(transform: cj_transform_t) -> Transform {
    let mut value = Transform::new();
    value.set_scale([transform.scale_x, transform.scale_y, transform.scale_z]);
    value.set_translate([
        transform.translate_x,
        transform.translate_y,
        transform.translate_z,
    ]);
    value
}

fn copy_string_bytes(value: Option<&str>) -> Vec<u8> {
    value.unwrap_or_default().as_bytes().to_vec()
}

fn index_values(indices: &[cityjson_lib::cityjson::v2_0::VertexIndex<u32>]) -> Vec<usize> {
    indices.iter().map(|index| index.to_usize()).collect()
}

fn empty_boundary(geometry: &OwnedGeometry) -> cj_geometry_boundary_t {
    cj_geometry_boundary_t {
        geometry_type: (*geometry.type_geometry()).into(),
        has_boundaries: false,
        vertex_indices: cj_indices_t::null(),
        ring_offsets: cj_indices_t::null(),
        surface_offsets: cj_indices_t::null(),
        shell_offsets: cj_indices_t::null(),
        solid_offsets: cj_indices_t::null(),
    }
}

fn boundary_from_geometry(geometry: &OwnedGeometry) -> cj_geometry_boundary_t {
    let Some(boundary) = geometry.boundaries() else {
        return empty_boundary(geometry);
    };

    let columnar = boundary.to_columnar();
    cj_geometry_boundary_t {
        geometry_type: (*geometry.type_geometry()).into(),
        has_boundaries: true,
        vertex_indices: indices_from_vec(index_values(columnar.vertices)),
        ring_offsets: indices_from_vec(index_values(columnar.ring_offsets)),
        surface_offsets: indices_from_vec(index_values(columnar.surface_offsets)),
        shell_offsets: indices_from_vec(index_values(columnar.shell_offsets)),
        solid_offsets: indices_from_vec(index_values(columnar.solid_offsets)),
    }
}

fn geometry_at(model: &CityModel, index: usize) -> Result<&OwnedGeometry, AbiError> {
    model
        .iter_geometries()
        .nth(index)
        .map(|(_, geometry)| geometry)
        .ok_or_else(|| invalid_argument(format!("geometry index {index} is out of range")))
}

fn geometry_boundary_coordinates(
    model: &CityModel,
    index: usize,
) -> Result<Vec<cj_vertex_t>, AbiError> {
    let geometry = geometry_at(model, index)?;

    Ok(geometry
        .coordinates(model.vertices())
        .map_or_else(Vec::new, |coordinates| {
            coordinates.copied().map(Into::into).collect()
        }))
}

fn find_cityobject_mut<'a>(
    model: &'a mut CityModel,
    id: &str,
) -> Result<
    &'a mut CityObject<cityjson_lib::cityjson::resources::storage::OwnedStringStorage>,
    AbiError,
> {
    model
        .cityobjects_mut()
        .iter_mut()
        .find_map(|(_, cityobject)| (cityobject.id() == id).then_some(cityobject))
        .ok_or_else(|| invalid_argument(format!("CityObject '{id}' was not found")))
}

fn find_geometry_handle(
    model: &CityModel,
    index: usize,
) -> Result<cityjson_lib::cityjson::resources::handles::GeometryHandle, AbiError> {
    model
        .iter_geometries()
        .nth(index)
        .map(|(handle, _)| handle)
        .ok_or_else(|| invalid_argument(format!("geometry index {index} is out of range")))
}

fn parse_lod(value: Option<String>) -> Result<Option<LoD>, AbiError> {
    fn parse_one(lod: &str) -> Option<LoD> {
        Some(match lod {
            "0" => LoD::LoD0,
            "0.0" => LoD::LoD0_0,
            "0.1" => LoD::LoD0_1,
            "0.2" => LoD::LoD0_2,
            "0.3" => LoD::LoD0_3,
            "1" => LoD::LoD1,
            "1.0" => LoD::LoD1_0,
            "1.1" => LoD::LoD1_1,
            "1.2" => LoD::LoD1_2,
            "1.3" => LoD::LoD1_3,
            "2" => LoD::LoD2,
            "2.0" => LoD::LoD2_0,
            "2.1" => LoD::LoD2_1,
            "2.2" => LoD::LoD2_2,
            "2.3" => LoD::LoD2_3,
            "3" => LoD::LoD3,
            "3.0" => LoD::LoD3_0,
            "3.1" => LoD::LoD3_1,
            "3.2" => LoD::LoD3_2,
            "3.3" => LoD::LoD3_3,
            _ => return None,
        })
    }

    value
        .map(|lod| {
            parse_one(&lod).ok_or_else(|| invalid_argument(format!("invalid lod value '{lod}'")))
        })
        .transpose()
}

fn geometry_type_from_abi(value: cj_geometry_type_t) -> GeometryType {
    match value {
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_POINT => GeometryType::MultiPoint,
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_LINE_STRING => GeometryType::MultiLineString,
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_SURFACE => GeometryType::MultiSurface,
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_COMPOSITE_SURFACE => GeometryType::CompositeSurface,
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_SOLID => GeometryType::Solid,
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_SOLID => GeometryType::MultiSolid,
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_COMPOSITE_SOLID => GeometryType::CompositeSolid,
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_GEOMETRY_INSTANCE => GeometryType::GeometryInstance,
    }
}

fn convert_indices(indices: &[usize], name: &'static str) -> Result<Vec<u32>, AbiError> {
    indices
        .iter()
        .map(|index| {
            u32::try_from(*index)
                .map_err(|_| invalid_argument(format!("{name} index {index} exceeds u32")))
        })
        .collect()
}

fn segment_ranges(
    offsets: &[usize],
    total_len: usize,
    name: &'static str,
) -> Result<Vec<(usize, usize)>, AbiError> {
    if offsets.is_empty() {
        return Ok(if total_len == 0 {
            Vec::new()
        } else {
            vec![(0, total_len)]
        });
    }

    if offsets[0] != 0 {
        return Err(invalid_argument(format!(
            "{name} offsets must start at zero"
        )));
    }

    let mut ranges = Vec::with_capacity(offsets.len());
    for (index, start) in offsets.iter().copied().enumerate() {
        let end = offsets.get(index + 1).copied().unwrap_or(total_len);
        if start > end || end > total_len {
            return Err(invalid_argument(format!(
                "{name} offsets must be monotonically increasing and within bounds"
            )));
        }
        ranges.push((start, end));
    }

    Ok(ranges)
}

fn boundary_from_view(
    view: cj_geometry_boundary_view_t,
) -> Result<Option<Boundary<u32>>, AbiError> {
    let vertices = required_indices_view(view.vertex_indices, "boundary.vertex_indices")?;
    let ring_offsets = required_indices_view(view.ring_offsets, "boundary.ring_offsets")?;
    let surface_offsets = required_indices_view(view.surface_offsets, "boundary.surface_offsets")?;
    let shell_offsets = required_indices_view(view.shell_offsets, "boundary.shell_offsets")?;
    let solid_offsets = required_indices_view(view.solid_offsets, "boundary.solid_offsets")?;

    let vertices = convert_indices(vertices, "boundary.vertex_indices")?;
    let ring_ranges = segment_ranges(ring_offsets, vertices.len(), "boundary.ring")?;
    let rings = ring_ranges
        .iter()
        .map(|(start, end)| vertices[*start..*end].to_vec())
        .collect::<Vec<_>>();

    let boundary = match view.geometry_type {
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_POINT => {
            if !ring_offsets.is_empty()
                || !surface_offsets.is_empty()
                || !shell_offsets.is_empty()
                || !solid_offsets.is_empty()
            {
                return Err(invalid_argument(
                    "MultiPoint boundaries must not provide nested offsets",
                ));
            }
            Some(BoundaryNestedMultiPoint::from(vertices).into())
        }
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_LINE_STRING => {
            if !surface_offsets.is_empty() || !shell_offsets.is_empty() || !solid_offsets.is_empty()
            {
                return Err(invalid_argument(
                    "MultiLineString boundaries must only provide ring offsets",
                ));
            }
            Some(
                Boundary::try_from(BoundaryNestedMultiLineString::from(rings))
                    .map_err(cityjson_lib::Error::from)
                    .map_err(AbiError::from)?,
            )
        }
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_SURFACE
        | cj_geometry_type_t::CJ_GEOMETRY_TYPE_COMPOSITE_SURFACE => {
            if !shell_offsets.is_empty() || !solid_offsets.is_empty() {
                return Err(invalid_argument(
                    "surface boundaries must not provide shell or solid offsets",
                ));
            }
            let surface_ranges = segment_ranges(surface_offsets, rings.len(), "boundary.surface")?;
            let surfaces = surface_ranges
                .iter()
                .map(|(start, end)| rings[*start..*end].to_vec())
                .collect::<Vec<_>>();
            Some(
                Boundary::try_from(BoundaryNestedMultiOrCompositeSurface::from(surfaces))
                    .map_err(cityjson_lib::Error::from)
                    .map_err(AbiError::from)?,
            )
        }
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_SOLID => {
            if !solid_offsets.is_empty() {
                return Err(invalid_argument(
                    "Solid boundaries must not provide solid offsets",
                ));
            }
            let surface_ranges = segment_ranges(surface_offsets, rings.len(), "boundary.surface")?;
            let surfaces = surface_ranges
                .iter()
                .map(|(start, end)| rings[*start..*end].to_vec())
                .collect::<Vec<_>>();
            let shell_ranges = segment_ranges(shell_offsets, surfaces.len(), "boundary.shell")?;
            let shells = shell_ranges
                .iter()
                .map(|(start, end)| surfaces[*start..*end].to_vec())
                .collect::<Vec<_>>();
            Some(
                Boundary::try_from(BoundaryNestedSolid::from(shells))
                    .map_err(cityjson_lib::Error::from)
                    .map_err(AbiError::from)?,
            )
        }
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_MULTI_SOLID
        | cj_geometry_type_t::CJ_GEOMETRY_TYPE_COMPOSITE_SOLID => {
            let surface_ranges = segment_ranges(surface_offsets, rings.len(), "boundary.surface")?;
            let surfaces = surface_ranges
                .iter()
                .map(|(start, end)| rings[*start..*end].to_vec())
                .collect::<Vec<_>>();
            let shell_ranges = segment_ranges(shell_offsets, surfaces.len(), "boundary.shell")?;
            let shells = shell_ranges
                .iter()
                .map(|(start, end)| surfaces[*start..*end].to_vec())
                .collect::<Vec<_>>();
            let solid_ranges = segment_ranges(solid_offsets, shells.len(), "boundary.solid")?;
            let solids = solid_ranges
                .iter()
                .map(|(start, end)| shells[*start..*end].to_vec())
                .collect::<Vec<_>>();
            Some(
                Boundary::try_from(BoundaryNestedMultiOrCompositeSolid::from(solids))
                    .map_err(cityjson_lib::Error::from)
                    .map_err(AbiError::from)?,
            )
        }
        cj_geometry_type_t::CJ_GEOMETRY_TYPE_GEOMETRY_INSTANCE => None,
    };

    Ok(boundary)
}

fn geometry_from_boundary_view(
    view: cj_geometry_boundary_view_t,
    lod: Option<LoD>,
) -> Result<Geometry<u32, cityjson_lib::cityjson::resources::storage::OwnedStringStorage>, AbiError>
{
    let boundary = boundary_from_view(view)?;
    Ok(Geometry::from_stored_parts(StoredGeometryParts {
        type_geometry: geometry_type_from_abi(view.geometry_type),
        lod,
        boundaries: boundary,
        semantics: None,
        materials: None,
        textures: None,
        instance: None,
    }))
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
    let extension_count = model.extensions().map_or(0, |extensions| extensions.len());
    let material_count = model.material_count();
    let texture_count = model.texture_count();
    let uv_coordinate_count = model.vertices_texture().len();
    let geometry_template_count = model.geometry_template_count();
    let template_vertex_count = model.template_vertices().len();

    cj_model_summary_t {
        model_type: model.type_citymodel().into(),
        version: if model.version().is_some() {
            crate::abi::cj_version_t::CJ_VERSION_V2_0
        } else {
            crate::abi::cj_version_t::CJ_VERSION_UNKNOWN
        },
        cityobject_count: model.cityobjects().len(),
        geometry_count: model.geometry_count(),
        geometry_template_count,
        vertex_count: model.vertices().len(),
        template_vertex_count,
        uv_coordinate_count,
        semantic_count: model.semantic_count(),
        material_count,
        texture_count,
        extension_count,
        has_metadata: model.metadata().is_some(),
        has_transform: model.transform().is_some(),
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
pub extern "C" fn cj_indices_free(indices: cj_indices_t) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        if indices.data.is_null() {
            if indices.len == 0 {
                return Ok(());
            }

            return Err(invalid_argument(
                "indices data must not be null when len is non-zero",
            ));
        }

        // SAFETY: the ABI only frees buffers allocated by `indices_from_vec`.
        unsafe {
            free_indices(indices);
        }

        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_geometry_boundary_free(boundary: cj_geometry_boundary_t) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        // SAFETY: the ABI only frees boundary payloads allocated by `boundary_from_geometry`.
        unsafe {
            free_geometry_boundary(boundary);
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
        let probe = cityjson_lib::json::probe(input)?;
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
        let probe = cityjson_lib::json::probe(input)?;
        if probe.kind() != RootKind::CityJSON {
            return Err(AbiError::from(Error::ExpectedCityJSON(
                probe.kind().to_string(),
            )));
        }

        reject_unsupported_document_version(probe.version())?;
        let model = cityjson_lib::json::from_slice_assume_cityjson_v2_0(input)?;
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
        let probe = cityjson_lib::json::probe(input)?;
        if probe.kind() != RootKind::CityJSONFeature {
            return Err(AbiError::from(Error::ExpectedCityJSONFeature(
                probe.kind().to_string(),
            )));
        }

        reject_unsupported_feature_version(probe.version())?;
        let model = cityjson_lib::json::from_feature_slice_assume_cityjson_feature_v2_0(input)?;
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

        let feature_probe = cityjson_lib::json::probe(feature)?;
        if feature_probe.kind() != RootKind::CityJSONFeature {
            return Err(AbiError::from(Error::ExpectedCityJSONFeature(
                feature_probe.kind().to_string(),
            )));
        }

        reject_unsupported_feature_version(feature_probe.version())?;

        let base_probe = cityjson_lib::json::probe(base)?;
        if base_probe.kind() != RootKind::CityJSON {
            return Err(AbiError::from(Error::ExpectedCityJSON(
                base_probe.kind().to_string(),
            )));
        }

        reject_unsupported_document_version(base_probe.version())?;
        let model =
            cityjson_lib::json::staged::from_feature_slice_with_base_assume_cityjson_feature_v2_0(
                feature, base,
            )?;
        write_model_handle(out_model, model)
    }))
}

#[cfg(feature = "arrow")]
#[unsafe(no_mangle)]
pub extern "C" fn cj_model_parse_arrow_bytes(
    data: *const u8,
    len: usize,
    out_model: *mut *mut cj_model_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let input = required_bytes(data, len, "data")?;
        let model = cityjson_lib::arrow::from_reader(Cursor::new(input))?;
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
        let bytes = cityjson_lib::json::to_vec(model)?;
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
        let bytes = cityjson_lib::json::to_feature_vec_with_options(
            model,
            cityjson_lib::json::WriteOptions::default(),
        )?;
        write_bytes(out_bytes, bytes)
    }))
}

#[cfg(feature = "arrow")]
#[unsafe(no_mangle)]
pub extern "C" fn cj_model_serialize_arrow(
    model: *const cj_model_t,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model = required_model_ref(model)?;
        let mut bytes = Vec::new();
        cityjson_lib::arrow::to_writer(&mut bytes, model)?;
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
        let bytes = copy_string_bytes(model.metadata().and_then(|metadata| metadata.title()));
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
            .iter_geometries()
            .nth(index)
            .map(|(_, geometry)| *geometry.type_geometry())
            .ok_or_else(|| invalid_argument(format!("geometry index {index} is out of range")))?;
        write_value(out_type, "out_type", geometry_type.into())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_copy_geometry_boundary(
    model: *const cj_model_t,
    index: usize,
    out_boundary: *mut cj_geometry_boundary_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model = required_model_ref(model)?;
        let geometry = geometry_at(model, index)?;
        write_boundary(out_boundary, boundary_from_geometry(geometry))
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_copy_geometry_boundary_coordinates(
    model: *const cj_model_t,
    index: usize,
    out_vertices: *mut cj_vertices_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model = required_model_ref(model)?;
        let vertices = geometry_boundary_coordinates(model, index)?;
        write_vertices(out_vertices, vertices)
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
            .reserve_import(capacities.into())
            .map_err(cityjson_lib::Error::from)?;
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
            .add_vertex(vertex.into())
            .map_err(cityjson_lib::Error::from)?
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
            .add_template_vertex(vertex.into())
            .map_err(cityjson_lib::Error::from)?
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
            .add_uv_coordinate(uv.into())
            .map_err(cityjson_lib::Error::from)?
            .to_usize();
        write_value(out_index, "out_index", index)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_set_metadata_title(
    model: *mut cj_model_t,
    title: cj_string_view_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let title = view_utf8(title, "title")?;
        let metadata = required_model_mut(model)?.metadata_mut();
        metadata.set_title(title);
        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_set_metadata_identifier(
    model: *mut cj_model_t,
    identifier: cj_string_view_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let identifier = view_utf8(identifier, "identifier")?;
        let metadata = required_model_mut(model)?.metadata_mut();
        metadata.set_identifier(CityModelIdentifier::new(identifier));
        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_set_transform(
    model: *mut cj_model_t,
    transform: cj_transform_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let transform = transform_from_abi(transform);
        let transform_mut = required_model_mut(model)?.transform_mut();
        transform_mut.set_scale(transform.scale());
        transform_mut.set_translate(transform.translate());
        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_clear_transform(model: *mut cj_model_t) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let model_ref = required_model_ref(model)?;
        let bytes = cityjson_lib::json::to_vec_with_options(
            model_ref,
            cityjson_lib::json::WriteOptions {
                pretty: false,
                validate_default_themes: false,
            },
        )?;
        let mut root = match serde_json::from_slice::<serde_json::Value>(&bytes)
            .map_err(Error::from)
            .map_err(AbiError::from)?
        {
            serde_json::Value::Object(root) => root,
            _ => {
                return Err(AbiError::from(Error::Import(
                    "serialized CityJSON root is not an object".into(),
                )));
            }
        };
        root.remove("transform");
        let bytes = serde_json::to_vec(&serde_json::Value::Object(root))
            .map_err(Error::from)
            .map_err(AbiError::from)?;
        let replacement = match model_ref.type_citymodel() {
            CityModelType::CityJSON => cityjson_lib::json::from_slice(&bytes)?,
            CityModelType::CityJSONFeature => cityjson_lib::json::from_feature_slice(&bytes)?,
            other => return Err(AbiError::from(Error::UnsupportedType(other.to_string()))),
        };
        *required_model_mut(model)? = replacement;
        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_add_cityobject(
    model: *mut cj_model_t,
    id: cj_string_view_t,
    cityobject_type: cj_string_view_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let id = view_utf8(id, "id")?;
        let cityobject_type = view_utf8(cityobject_type, "cityobject_type")?;
        let cityobject_type =
            CityObjectType::from_str(&cityobject_type).map_err(cityjson_lib::Error::from)?;
        let cityobject = CityObject::new(CityObjectIdentifier::new(id), cityobject_type);
        required_model_mut(model)?
            .cityobjects_mut()
            .add(cityobject)
            .map_err(cityjson_lib::Error::from)?;
        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_remove_cityobject(
    model: *mut cj_model_t,
    id: cj_string_view_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let id = view_utf8(id, "id")?;
        let cityobjects = required_model_mut(model)?.cityobjects_mut();
        let Some(handle) = cityobjects
            .iter()
            .find_map(|(handle, cityobject)| (cityobject.id() == id).then_some(handle))
        else {
            return Err(invalid_argument(format!("CityObject '{id}' was not found")));
        };
        cityobjects.remove(handle);
        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_attach_geometry_to_cityobject(
    model: *mut cj_model_t,
    cityobject_id: cj_string_view_t,
    geometry_index: usize,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let cityobject_id = view_utf8(cityobject_id, "cityobject_id")?;
        let geometry_handle = find_geometry_handle(required_model_ref(model)?, geometry_index)?;
        find_cityobject_mut(required_model_mut(model)?, &cityobject_id)?
            .add_geometry(geometry_handle);
        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_clear_cityobject_geometry(
    model: *mut cj_model_t,
    cityobject_id: cj_string_view_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let cityobject_id = view_utf8(cityobject_id, "cityobject_id")?;
        find_cityobject_mut(required_model_mut(model)?, &cityobject_id)?.clear_geometry();
        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_add_geometry_from_boundary(
    model: *mut cj_model_t,
    boundary: cj_geometry_boundary_view_t,
    lod: cj_string_view_t,
    out_index: *mut usize,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let lod = parse_lod(optional_view_utf8(lod, "lod")?)?;
        let geometry = geometry_from_boundary_view(boundary, lod)?;
        let model = required_model_mut(model)?;
        let index = model.geometry_count();
        model
            .add_geometry(geometry)
            .map_err(cityjson_lib::Error::from)?;
        write_value(out_index, "out_index", index)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_cleanup(model: *mut cj_model_t) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let cleaned = cityjson_lib::ops::cleanup(required_model_ref(model)?)?;
        *required_model_mut(model)? = cleaned;
        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_append_model(
    target_model: *mut cj_model_t,
    source_model: *const cj_model_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let source = required_model_ref(source_model)?.clone();
        cityjson_lib::ops::append(required_model_mut(target_model)?, &source)?;
        Ok(())
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_extract_cityobjects(
    model: *const cj_model_t,
    cityobject_ids: *const cj_string_view_t,
    cityobject_count: usize,
    out_model: *mut *mut cj_model_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let views = required_string_views(cityobject_ids, cityobject_count, "cityobject_ids")?;
        let ids = views
            .iter()
            .map(|view| view_utf8(*view, "cityobject_ids[]"))
            .collect::<Result<Vec<_>, _>>()?;
        let borrowed = ids.iter().map(String::as_str).collect::<Vec<_>>();
        let extracted = cityjson_lib::ops::extract(required_model_ref(model)?, borrowed)?;
        write_model_handle(out_model, extracted)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_serialize_document_with_options(
    model: *const cj_model_t,
    options: cj_json_write_options_t,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let bytes = cityjson_lib::json::to_vec_with_options(
            required_model_ref(model)?,
            cityjson_lib::json::WriteOptions {
                pretty: options.pretty,
                validate_default_themes: options.validate_default_themes,
            },
        )?;
        write_bytes(out_bytes, bytes)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_serialize_feature_with_options(
    model: *const cj_model_t,
    options: cj_json_write_options_t,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let bytes = cityjson_lib::json::to_feature_vec_with_options(
            required_model_ref(model)?,
            cityjson_lib::json::WriteOptions {
                pretty: options.pretty,
                validate_default_themes: options.validate_default_themes,
            },
        )?;
        write_bytes(out_bytes, bytes)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_parse_feature_stream_merge_bytes(
    data: *const u8,
    len: usize,
    out_model: *mut *mut cj_model_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let input = required_bytes(data, len, "data")?;
        let model = cityjson_lib::json::merge_feature_stream_slice(input)?;
        write_model_handle(out_model, model)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_serialize_feature_stream(
    models: *const *const cj_model_t,
    model_count: usize,
    options: cj_json_write_options_t,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        if options.pretty {
            return Err(AbiError::from(Error::UnsupportedFeature(
                "pretty output is not supported for JSONL feature streams".into(),
            )));
        }

        if model_count == 0 {
            return write_bytes(out_bytes, Vec::new());
        }

        let models_ptr = NonNull::new(models.cast_mut()).ok_or_else(|| {
            invalid_argument("models must not be null when model_count is non-zero")
        })?;
        let models =
            unsafe { slice::from_raw_parts(models_ptr.as_ptr().cast_const(), model_count) };
        let refs = models
            .iter()
            .map(|handle| required_model_ref(*handle))
            .collect::<Result<Vec<_>, _>>()?;

        if options.validate_default_themes {
            for model in &refs {
                model
                    .validate_default_themes()
                    .map_err(cityjson_lib::Error::from)
                    .map_err(AbiError::from)?;
            }
        }

        let mut buffer = Vec::new();
        for (index, model) in refs.iter().enumerate() {
            match model.type_citymodel() {
                CityModelType::CityJSON => {
                    if index != 0 {
                        return Err(AbiError::from(Error::UnsupportedFeature(
                            "only the first feature-stream item may be CityJSON".into(),
                        )));
                    }
                    cityjson_lib::json::to_writer_with_options(
                        &mut buffer,
                        model,
                        cityjson_lib::json::WriteOptions {
                            pretty: false,
                            validate_default_themes: options.validate_default_themes,
                        },
                    )?;
                }
                CityModelType::CityJSONFeature => {
                    cityjson_lib::json::to_feature_writer(&mut buffer, model)?;
                }
                other => return Err(AbiError::from(Error::UnsupportedType(other.to_string()))),
            }
            buffer.push(b'\n');
        }
        write_bytes(out_bytes, buffer)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_serialize_cityjsonseq_with_transform(
    base_root: *const cj_model_t,
    features: *const *const cj_model_t,
    feature_count: usize,
    transform: cj_transform_t,
    options: cj_cityjsonseq_write_options_t,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let base_root = required_model_ref(base_root)?;
        let feature_refs = required_model_refs(features, feature_count, "features")?;
        let transform = transform_from_abi(transform);
        let _ = options;

        let mut buffer = Vec::new();
        cityjson_lib::json::write_cityjsonseq_refs(
            &mut buffer,
            base_root,
            feature_refs,
            &transform,
        )?;
        write_bytes(out_bytes, buffer)
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn cj_model_serialize_cityjsonseq_auto_transform(
    base_root: *const cj_model_t,
    features: *const *const cj_model_t,
    feature_count: usize,
    options: cj_cityjsonseq_auto_transform_options_t,
    out_bytes: *mut cj_bytes_t,
) -> cj_status_t {
    ffi_status(run_ffi::<(), AbiError, _>(|| {
        let base_root = required_model_ref(base_root)?;
        let feature_refs = required_model_refs(features, feature_count, "features")?;
        let _ = (
            options.validate_default_themes,
            options.trailing_newline,
            options.update_metadata_geographical_extent,
        );

        let mut buffer = Vec::new();
        cityjson_lib::json::write_cityjsonseq_auto_transform_refs(
            &mut buffer,
            base_root,
            feature_refs,
            [options.scale_x, options.scale_y, options.scale_z],
        )?;
        write_bytes(out_bytes, buffer)
    }))
}
