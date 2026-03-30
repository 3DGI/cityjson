#![allow(clippy::all, clippy::pedantic)]

//! Shared low-level FFI core for non-Rust bindings.
//!
//! This crate will own the common low-level substrate used by the C++, Python,
//! and wasm layers. The public contract stays intentionally narrow: opaque
//! model handles, explicit ownership, stable status/error categories, and
//! bytes-in/bytes-out entry points on top.

pub mod abi;
pub mod error;
pub mod exports;
pub mod handle;

pub use cjlib;

pub use abi::{
    cj_bytes_t, cj_error_kind_t, cj_geometry_type_t, cj_model_capacities_t, cj_model_summary_t,
    cj_model_t, cj_model_type_t, cj_probe_t, cj_root_kind_t, cj_status_t, cj_uv_t, cj_uvs_t,
    cj_version_t, cj_vertex_t, cj_vertices_t,
};
pub use error::{
    AbiError, clear_last_error, copy_last_error_message, last_error_kind, last_error_message_len,
    last_error_status, run_ffi, set_last_error, set_last_error_from_cjlib_error,
};
pub use handle::{
    bytes_free, bytes_from_string, bytes_from_vec, model_as_mut, model_as_ref, model_free,
    model_into_handle, model_take, uvs_free, uvs_from_vec, vertices_free, vertices_from_vec,
};
