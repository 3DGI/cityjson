#![allow(clippy::all, clippy::pedantic)]

//! Shared low-level FFI core for non-Rust bindings.
//!
//! This crate will own the common low-level substrate used by the C++, Python,
//! and wasm layers. The public contract stays intentionally narrow: opaque
//! model handles, explicit ownership, stable status/error categories, and
//! bytes-in/bytes-out entry points on top.

pub mod abi;
pub mod authoring;
pub mod error;
pub mod exports;
pub mod handle;
pub mod ids;

pub use cityjson_lib;

pub use abi::{
    cj_affine_transform_4x4_t, cj_bbox_t, cj_bytes_t, cj_cityjsonseq_auto_transform_options_t,
    cj_cityjsonseq_write_options_t, cj_cityobject_id_t, cj_contact_role_t, cj_contact_t,
    cj_contact_type_t, cj_error_kind_t, cj_geometry_boundary_t, cj_geometry_boundary_view_t,
    cj_geometry_draft_t, cj_geometry_id_t, cj_geometry_template_id_t, cj_geometry_type_t,
    cj_image_type_t, cj_indices_t, cj_indices_view_t, cj_json_write_options_t, cj_material_id_t,
    cj_model_capacities_t, cj_model_summary_t, cj_model_t, cj_model_type_t, cj_probe_t, cj_rgb_t,
    cj_rgba_t, cj_ring_draft_t, cj_root_kind_t, cj_semantic_id_t, cj_shell_draft_t,
    cj_solid_draft_t, cj_status_t, cj_string_view_t, cj_surface_draft_t, cj_texture_id_t,
    cj_texture_type_t, cj_transform_t, cj_uv_t, cj_uvs_t, cj_value_kind_t, cj_value_t,
    cj_version_t, cj_vertex_t, cj_vertices_t, cj_wrap_mode_t,
};
pub use authoring::{
    GeometryAuthoring, LineStringAuthoring, OwnedCityObject, OwnedContact, OwnedGeometryDraft,
    OwnedMaterial, OwnedSemantic, OwnedTexture, OwnedValue, PointAuthoring, RingAuthoring,
    RingTextureAuthoring, ShellAuthoring, SolidAuthoring, SurfaceAuthoring, UvAuthoring,
    VertexAuthoring,
};
pub use error::{
    AbiError, clear_last_error, copy_last_error_message, last_error_kind, last_error_message_len,
    last_error_status, run_ffi, set_last_error, set_last_error_from_cityjson_lib_error,
};
pub use handle::{
    bytes_free, bytes_from_string, bytes_from_vec, cityobject_draft_as_mut, cityobject_draft_free,
    cityobject_draft_into_handle, cityobject_draft_take, contact_as_mut, contact_free,
    contact_into_handle, contact_take, geometry_boundary_free, geometry_draft_as_mut,
    geometry_draft_free, geometry_draft_into_handle, geometry_draft_take, indices_free,
    indices_from_vec, model_as_mut, model_as_ref, model_free, model_into_handle, model_take,
    ring_draft_as_mut, ring_draft_free, ring_draft_into_handle, ring_draft_take,
    shell_draft_as_mut, shell_draft_free, shell_draft_into_handle, shell_draft_take,
    solid_draft_as_mut, solid_draft_free, solid_draft_into_handle, solid_draft_take,
    surface_draft_as_mut, surface_draft_free, surface_draft_into_handle, surface_draft_take,
    uvs_free, uvs_from_vec, value_as_mut, value_free, value_into_handle, value_take, vertices_free,
    vertices_from_vec,
};
