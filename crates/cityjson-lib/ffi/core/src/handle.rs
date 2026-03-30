use std::ptr;

use crate::abi::{cj_bytes_t, cj_model_t, cj_uv_t, cj_uvs_t, cj_vertex_t, cj_vertices_t};

pub fn model_into_handle(model: cjlib::CityModel) -> *mut cj_model_t {
    Box::into_raw(Box::new(model)).cast::<cj_model_t>()
}

pub unsafe fn model_take(handle: *mut cj_model_t) -> Option<Box<cjlib::CityModel>> {
    if handle.is_null() {
        return None;
    }

    Some(unsafe { Box::from_raw(handle.cast::<cjlib::CityModel>()) })
}

pub unsafe fn model_as_ref<'a>(handle: *const cj_model_t) -> Option<&'a cjlib::CityModel> {
    unsafe { handle.cast::<cjlib::CityModel>().as_ref() }
}

pub unsafe fn model_as_mut<'a>(handle: *mut cj_model_t) -> Option<&'a mut cjlib::CityModel> {
    unsafe { handle.cast::<cjlib::CityModel>().as_mut() }
}

pub unsafe fn model_free(handle: *mut cj_model_t) {
    let _ = unsafe { model_take(handle) };
}

pub fn bytes_from_vec(bytes: Vec<u8>) -> cj_bytes_t {
    let boxed = bytes.into_boxed_slice();
    let len = boxed.len();
    let data = Box::into_raw(boxed).cast::<u8>();
    cj_bytes_t { data, len }
}

pub fn bytes_from_string(bytes: String) -> cj_bytes_t {
    bytes_from_vec(bytes.into_bytes())
}

pub fn vertices_from_vec(vertices: Vec<cj_vertex_t>) -> cj_vertices_t {
    let boxed = vertices.into_boxed_slice();
    let len = boxed.len();
    let data = Box::into_raw(boxed).cast::<cj_vertex_t>();
    cj_vertices_t { data, len }
}

pub fn uvs_from_vec(uvs: Vec<cj_uv_t>) -> cj_uvs_t {
    let boxed = uvs.into_boxed_slice();
    let len = boxed.len();
    let data = Box::into_raw(boxed).cast::<cj_uv_t>();
    cj_uvs_t { data, len }
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
