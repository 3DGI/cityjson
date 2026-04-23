use std::ptr;

use crate::abi::{
    cj_bytes_list_t, cj_bytes_t, cj_cityobject_draft_t, cj_contact_t, cj_geometry_boundary_t,
    cj_geometry_draft_t, cj_geometry_types_t, cj_indices_t, cj_model_t, cj_ring_draft_t,
    cj_shell_draft_t, cj_solid_draft_t, cj_surface_draft_t, cj_uv_t, cj_uvs_t, cj_value_t,
    cj_vertex_t, cj_vertices_t,
};
use crate::authoring::{
    GeometryAuthoring, OwnedCityObject, OwnedContact, OwnedValue, RingAuthoring, ShellAuthoring,
    SolidAuthoring, SurfaceAuthoring,
};

pub fn model_into_handle(model: cityjson_lib::CityModel) -> *mut cj_model_t {
    Box::into_raw(Box::new(model)).cast::<cj_model_t>()
}

pub unsafe fn model_take(handle: *mut cj_model_t) -> Option<Box<cityjson_lib::CityModel>> {
    if handle.is_null() {
        return None;
    }

    Some(unsafe { Box::from_raw(handle.cast::<cityjson_lib::CityModel>()) })
}

pub unsafe fn model_as_ref<'a>(handle: *const cj_model_t) -> Option<&'a cityjson_lib::CityModel> {
    unsafe { handle.cast::<cityjson_lib::CityModel>().as_ref() }
}

pub unsafe fn model_as_mut<'a>(handle: *mut cj_model_t) -> Option<&'a mut cityjson_lib::CityModel> {
    unsafe { handle.cast::<cityjson_lib::CityModel>().as_mut() }
}

pub unsafe fn model_free(handle: *mut cj_model_t) {
    let _ = unsafe { model_take(handle) };
}

macro_rules! define_handle_accessors {
    ($into_fn:ident, $take_fn:ident, $as_mut_fn:ident, $free_fn:ident, $ffi_ty:ty, $rust_ty:ty) => {
        pub fn $into_fn(value: $rust_ty) -> *mut $ffi_ty {
            Box::into_raw(Box::new(value)).cast::<$ffi_ty>()
        }

        pub unsafe fn $take_fn(handle: *mut $ffi_ty) -> Option<Box<$rust_ty>> {
            if handle.is_null() {
                return None;
            }

            Some(unsafe { Box::from_raw(handle.cast::<$rust_ty>()) })
        }

        pub unsafe fn $as_mut_fn<'a>(handle: *mut $ffi_ty) -> Option<&'a mut $rust_ty> {
            unsafe { handle.cast::<$rust_ty>().as_mut() }
        }

        pub unsafe fn $free_fn(handle: *mut $ffi_ty) {
            let _ = unsafe { $take_fn(handle) };
        }
    };
}

define_handle_accessors!(
    value_into_handle,
    value_take,
    value_as_mut,
    value_free,
    cj_value_t,
    OwnedValue
);
define_handle_accessors!(
    contact_into_handle,
    contact_take,
    contact_as_mut,
    contact_free,
    cj_contact_t,
    OwnedContact
);
define_handle_accessors!(
    cityobject_draft_into_handle,
    cityobject_draft_take,
    cityobject_draft_as_mut,
    cityobject_draft_free,
    cj_cityobject_draft_t,
    OwnedCityObject
);
define_handle_accessors!(
    ring_draft_into_handle,
    ring_draft_take,
    ring_draft_as_mut,
    ring_draft_free,
    cj_ring_draft_t,
    RingAuthoring
);
define_handle_accessors!(
    surface_draft_into_handle,
    surface_draft_take,
    surface_draft_as_mut,
    surface_draft_free,
    cj_surface_draft_t,
    SurfaceAuthoring
);
define_handle_accessors!(
    shell_draft_into_handle,
    shell_draft_take,
    shell_draft_as_mut,
    shell_draft_free,
    cj_shell_draft_t,
    ShellAuthoring
);
define_handle_accessors!(
    solid_draft_into_handle,
    solid_draft_take,
    solid_draft_as_mut,
    solid_draft_free,
    cj_solid_draft_t,
    SolidAuthoring
);
define_handle_accessors!(
    geometry_draft_into_handle,
    geometry_draft_take,
    geometry_draft_as_mut,
    geometry_draft_free,
    cj_geometry_draft_t,
    GeometryAuthoring
);

pub fn bytes_from_vec(bytes: Vec<u8>) -> cj_bytes_t {
    let boxed = bytes.into_boxed_slice();
    let len = boxed.len();
    let data = Box::into_raw(boxed).cast::<u8>();
    cj_bytes_t { data, len }
}

pub fn bytes_from_string(bytes: String) -> cj_bytes_t {
    bytes_from_vec(bytes.into_bytes())
}

pub fn bytes_list_from_vec(bytes: Vec<cj_bytes_t>) -> cj_bytes_list_t {
    let boxed = bytes.into_boxed_slice();
    let len = boxed.len();
    let data = Box::into_raw(boxed).cast::<cj_bytes_t>();
    cj_bytes_list_t { data, len }
}

pub fn vertices_from_vec(vertices: Vec<cj_vertex_t>) -> cj_vertices_t {
    let boxed = vertices.into_boxed_slice();
    let len = boxed.len();
    let data = Box::into_raw(boxed).cast::<cj_vertex_t>();
    cj_vertices_t { data, len }
}

pub fn geometry_types_from_vec(types: Vec<crate::abi::cj_geometry_type_t>) -> cj_geometry_types_t {
    let boxed = types.into_boxed_slice();
    let len = boxed.len();
    let data = Box::into_raw(boxed).cast::<crate::abi::cj_geometry_type_t>();
    cj_geometry_types_t { data, len }
}

pub fn uvs_from_vec(uvs: Vec<cj_uv_t>) -> cj_uvs_t {
    let boxed = uvs.into_boxed_slice();
    let len = boxed.len();
    let data = Box::into_raw(boxed).cast::<cj_uv_t>();
    cj_uvs_t { data, len }
}

pub fn indices_from_vec(indices: Vec<usize>) -> cj_indices_t {
    let boxed = indices.into_boxed_slice();
    let len = boxed.len();
    let data = Box::into_raw(boxed).cast::<usize>();
    cj_indices_t { data, len }
}

pub unsafe fn bytes_free(bytes: cj_bytes_t) {
    if bytes.data.is_null() {
        return;
    }

    let slice = ptr::slice_from_raw_parts_mut(bytes.data, bytes.len);
    unsafe {
        drop(Box::from_raw(slice));
    }
}

pub unsafe fn bytes_list_free(bytes: cj_bytes_list_t) {
    if bytes.data.is_null() {
        return;
    }

    let slice = ptr::slice_from_raw_parts_mut(bytes.data, bytes.len);
    // SAFETY: each element was allocated through `bytes_from_string`/`bytes_from_vec`.
    unsafe {
        for item in &*slice {
            bytes_free(*item);
        }
        drop(Box::from_raw(slice));
    }
}

pub unsafe fn vertices_free(vertices: cj_vertices_t) {
    if vertices.data.is_null() {
        return;
    }

    let slice = ptr::slice_from_raw_parts_mut(vertices.data, vertices.len);
    unsafe {
        drop(Box::from_raw(slice));
    }
}

pub unsafe fn uvs_free(uvs: cj_uvs_t) {
    if uvs.data.is_null() {
        return;
    }

    let slice = ptr::slice_from_raw_parts_mut(uvs.data, uvs.len);
    unsafe {
        drop(Box::from_raw(slice));
    }
}

pub unsafe fn indices_free(indices: cj_indices_t) {
    if indices.data.is_null() {
        return;
    }

    let slice = ptr::slice_from_raw_parts_mut(indices.data, indices.len);
    unsafe {
        drop(Box::from_raw(slice));
    }
}

pub unsafe fn geometry_types_free(types: cj_geometry_types_t) {
    if types.data.is_null() {
        return;
    }

    let slice = ptr::slice_from_raw_parts_mut(types.data, types.len);
    unsafe {
        drop(Box::from_raw(slice));
    }
}

pub unsafe fn geometry_boundary_free(boundary: cj_geometry_boundary_t) {
    unsafe {
        indices_free(boundary.vertex_indices);
        indices_free(boundary.ring_offsets);
        indices_free(boundary.surface_offsets);
        indices_free(boundary.shell_offsets);
        indices_free(boundary.solid_offsets);
    }
}
