use cjlib::{
    CityJSONVersion,
    cityjson::{CityModelType, v2_0::GeometryType},
    json::RootKind,
};

/// Stable status codes for the shared C ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum cj_status_t {
    CJ_STATUS_SUCCESS = 0,
    CJ_STATUS_INVALID_ARGUMENT = 1,
    CJ_STATUS_IO = 2,
    CJ_STATUS_SYNTAX = 3,
    CJ_STATUS_VERSION = 4,
    CJ_STATUS_SHAPE = 5,
    CJ_STATUS_UNSUPPORTED = 6,
    CJ_STATUS_MODEL = 7,
    CJ_STATUS_INTERNAL = 8,
}

impl Default for cj_status_t {
    fn default() -> Self {
        Self::CJ_STATUS_SUCCESS
    }
}

/// Stable error categories for the shared C ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum cj_error_kind_t {
    CJ_ERROR_KIND_NONE = 0,
    CJ_ERROR_KIND_INVALID_ARGUMENT = 1,
    CJ_ERROR_KIND_IO = 2,
    CJ_ERROR_KIND_SYNTAX = 3,
    CJ_ERROR_KIND_VERSION = 4,
    CJ_ERROR_KIND_SHAPE = 5,
    CJ_ERROR_KIND_UNSUPPORTED = 6,
    CJ_ERROR_KIND_MODEL = 7,
    CJ_ERROR_KIND_INTERNAL = 8,
}

impl Default for cj_error_kind_t {
    fn default() -> Self {
        Self::CJ_ERROR_KIND_NONE
    }
}

/// Stable root type discriminant for probed inputs.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum cj_root_kind_t {
    CJ_ROOT_KIND_CITY_JSON = 0,
    CJ_ROOT_KIND_CITY_JSON_FEATURE = 1,
}

impl Default for cj_root_kind_t {
    fn default() -> Self {
        Self::CJ_ROOT_KIND_CITY_JSON
    }
}

/// Stable version discriminant for probed inputs.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum cj_version_t {
    CJ_VERSION_UNKNOWN = 0,
    CJ_VERSION_V1_0 = 1,
    CJ_VERSION_V1_1 = 2,
    CJ_VERSION_V2_0 = 3,
}

impl Default for cj_version_t {
    fn default() -> Self {
        Self::CJ_VERSION_UNKNOWN
    }
}

/// Stable model type discriminant for `CityJSON` documents and features.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum cj_model_type_t {
    CJ_MODEL_TYPE_CITY_JSON = 0,
    CJ_MODEL_TYPE_CITY_JSON_FEATURE = 1,
}

impl Default for cj_model_type_t {
    fn default() -> Self {
        Self::CJ_MODEL_TYPE_CITY_JSON
    }
}

/// Stable geometry type discriminant for stored `CityJSON` geometries.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum cj_geometry_type_t {
    CJ_GEOMETRY_TYPE_MULTI_POINT = 0,
    CJ_GEOMETRY_TYPE_MULTI_LINE_STRING = 1,
    CJ_GEOMETRY_TYPE_MULTI_SURFACE = 2,
    CJ_GEOMETRY_TYPE_COMPOSITE_SURFACE = 3,
    CJ_GEOMETRY_TYPE_SOLID = 4,
    CJ_GEOMETRY_TYPE_MULTI_SOLID = 5,
    CJ_GEOMETRY_TYPE_COMPOSITE_SOLID = 6,
    CJ_GEOMETRY_TYPE_GEOMETRY_INSTANCE = 7,
}

impl Default for cj_geometry_type_t {
    fn default() -> Self {
        Self::CJ_GEOMETRY_TYPE_MULTI_POINT
    }
}

/// Opaque model handle type.
///
/// The ABI only ever passes pointers to this marker type. The actual storage is
/// a boxed `cjlib::CityModel` allocated by the Rust side.
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct cj_model_t {
    _private: [u8; 0],
}

/// Owned byte buffer returned across the ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cj_bytes_t {
    pub data: *mut u8,
    pub len: usize,
}

impl cj_bytes_t {
    pub const fn null() -> Self {
        Self {
            data: core::ptr::null_mut(),
            len: 0,
        }
    }

    pub const fn is_null(self) -> bool {
        self.data.is_null()
    }
}

/// Packed 3D coordinate copied across the ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct cj_vertex_t {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Packed UV coordinate copied across the ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct cj_uv_t {
    pub u: f32,
    pub v: f32,
}

/// Owned vertex buffer returned across the ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cj_vertices_t {
    pub data: *mut cj_vertex_t,
    pub len: usize,
}

impl cj_vertices_t {
    pub const fn null() -> Self {
        Self {
            data: core::ptr::null_mut(),
            len: 0,
        }
    }

    pub const fn is_null(self) -> bool {
        self.data.is_null()
    }
}

/// Owned UV buffer returned across the ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cj_uvs_t {
    pub data: *mut cj_uv_t,
    pub len: usize,
}

impl cj_uvs_t {
    pub const fn null() -> Self {
        Self {
            data: core::ptr::null_mut(),
            len: 0,
        }
    }

    pub const fn is_null(self) -> bool {
        self.data.is_null()
    }
}

/// Owned index buffer returned across the ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cj_indices_t {
    pub data: *mut usize,
    pub len: usize,
}

impl cj_indices_t {
    pub const fn null() -> Self {
        Self {
            data: core::ptr::null_mut(),
            len: 0,
        }
    }

    pub const fn is_null(self) -> bool {
        self.data.is_null()
    }
}

/// Borrowed UTF-8 string view passed into the ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cj_string_view_t {
    pub data: *const u8,
    pub len: usize,
}

impl cj_string_view_t {
    pub const fn null() -> Self {
        Self {
            data: core::ptr::null(),
            len: 0,
        }
    }
}

/// Borrowed index-slice view passed into the ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cj_indices_view_t {
    pub data: *const usize,
    pub len: usize,
}

impl cj_indices_view_t {
    pub const fn null() -> Self {
        Self {
            data: core::ptr::null(),
            len: 0,
        }
    }
}

/// Owned flat boundary payload returned across the ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cj_geometry_boundary_t {
    pub geometry_type: cj_geometry_type_t,
    pub has_boundaries: bool,
    pub vertex_indices: cj_indices_t,
    pub ring_offsets: cj_indices_t,
    pub surface_offsets: cj_indices_t,
    pub shell_offsets: cj_indices_t,
    pub solid_offsets: cj_indices_t,
}

/// Borrowed flat boundary payload passed into the ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cj_geometry_boundary_view_t {
    pub geometry_type: cj_geometry_type_t,
    pub vertex_indices: cj_indices_view_t,
    pub ring_offsets: cj_indices_view_t,
    pub surface_offsets: cj_indices_view_t,
    pub shell_offsets: cj_indices_view_t,
    pub solid_offsets: cj_indices_view_t,
}

/// Probe result returned by the low-level ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cj_probe_t {
    pub root_kind: cj_root_kind_t,
    pub version: cj_version_t,
    pub has_version: bool,
}

impl cj_probe_t {
    pub fn from_probe(probe: &cjlib::json::Probe) -> Self {
        Self {
            root_kind: probe.kind().into(),
            version: probe
                .version()
                .map_or(cj_version_t::CJ_VERSION_UNKNOWN, Into::into),
            has_version: probe.version().is_some(),
        }
    }
}

/// Aggregate model inspection summary returned across the ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cj_model_summary_t {
    pub model_type: cj_model_type_t,
    pub version: cj_version_t,
    pub cityobject_count: usize,
    pub geometry_count: usize,
    pub geometry_template_count: usize,
    pub vertex_count: usize,
    pub template_vertex_count: usize,
    pub uv_coordinate_count: usize,
    pub semantic_count: usize,
    pub material_count: usize,
    pub texture_count: usize,
    pub extension_count: usize,
    pub has_metadata: bool,
    pub has_transform: bool,
    pub has_templates: bool,
    pub has_appearance: bool,
}

/// Capacity hints for bulk import and model-building paths.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cj_model_capacities_t {
    pub cityobjects: usize,
    pub vertices: usize,
    pub semantics: usize,
    pub materials: usize,
    pub textures: usize,
    pub geometries: usize,
    pub template_vertices: usize,
    pub template_geometries: usize,
    pub uv_coordinates: usize,
}

/// Explicit JSON write options for document, feature, and feature-stream output.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cj_json_write_options_t {
    pub pretty: bool,
    pub validate_default_themes: bool,
}

/// Explicit strict `CityJSONSeq` write options.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct cj_cityjsonseq_write_options_t {
    pub validate_default_themes: bool,
    pub trailing_newline: bool,
    pub update_metadata_geographical_extent: bool,
}

/// Auto-transform options for strict `CityJSONSeq` writing.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct cj_cityjsonseq_auto_transform_options_t {
    pub scale_x: f64,
    pub scale_y: f64,
    pub scale_z: f64,
    pub validate_default_themes: bool,
    pub trailing_newline: bool,
    pub update_metadata_geographical_extent: bool,
}

/// Explicit root-transform state for JSON write and edit workflows.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct cj_transform_t {
    pub scale_x: f64,
    pub scale_y: f64,
    pub scale_z: f64,
    pub translate_x: f64,
    pub translate_y: f64,
    pub translate_z: f64,
}

impl From<RootKind> for cj_root_kind_t {
    fn from(value: RootKind) -> Self {
        match value {
            RootKind::CityJSON => Self::CJ_ROOT_KIND_CITY_JSON,
            RootKind::CityJSONFeature => Self::CJ_ROOT_KIND_CITY_JSON_FEATURE,
        }
    }
}

impl From<cj_root_kind_t> for RootKind {
    fn from(value: cj_root_kind_t) -> Self {
        match value {
            cj_root_kind_t::CJ_ROOT_KIND_CITY_JSON => Self::CityJSON,
            cj_root_kind_t::CJ_ROOT_KIND_CITY_JSON_FEATURE => Self::CityJSONFeature,
        }
    }
}

impl From<CityJSONVersion> for cj_version_t {
    fn from(value: CityJSONVersion) -> Self {
        match value {
            CityJSONVersion::V1_0 => Self::CJ_VERSION_V1_0,
            CityJSONVersion::V1_1 => Self::CJ_VERSION_V1_1,
            CityJSONVersion::V2_0 => Self::CJ_VERSION_V2_0,
        }
    }
}

impl From<CityModelType> for cj_model_type_t {
    fn from(value: CityModelType) -> Self {
        match value {
            CityModelType::CityJSON => Self::CJ_MODEL_TYPE_CITY_JSON,
            CityModelType::CityJSONFeature => Self::CJ_MODEL_TYPE_CITY_JSON_FEATURE,
            _ => Self::CJ_MODEL_TYPE_CITY_JSON,
        }
    }
}

impl From<cj_model_type_t> for CityModelType {
    fn from(value: cj_model_type_t) -> Self {
        match value {
            cj_model_type_t::CJ_MODEL_TYPE_CITY_JSON => Self::CityJSON,
            cj_model_type_t::CJ_MODEL_TYPE_CITY_JSON_FEATURE => Self::CityJSONFeature,
        }
    }
}

impl From<GeometryType> for cj_geometry_type_t {
    fn from(value: GeometryType) -> Self {
        match value {
            GeometryType::MultiPoint => Self::CJ_GEOMETRY_TYPE_MULTI_POINT,
            GeometryType::MultiLineString => Self::CJ_GEOMETRY_TYPE_MULTI_LINE_STRING,
            GeometryType::MultiSurface => Self::CJ_GEOMETRY_TYPE_MULTI_SURFACE,
            GeometryType::CompositeSurface => Self::CJ_GEOMETRY_TYPE_COMPOSITE_SURFACE,
            GeometryType::Solid => Self::CJ_GEOMETRY_TYPE_SOLID,
            GeometryType::MultiSolid => Self::CJ_GEOMETRY_TYPE_MULTI_SOLID,
            GeometryType::CompositeSolid => Self::CJ_GEOMETRY_TYPE_COMPOSITE_SOLID,
            GeometryType::GeometryInstance => Self::CJ_GEOMETRY_TYPE_GEOMETRY_INSTANCE,
            _ => Self::CJ_GEOMETRY_TYPE_MULTI_POINT,
        }
    }
}

impl TryFrom<cj_version_t> for CityJSONVersion {
    type Error = ();

    fn try_from(value: cj_version_t) -> Result<Self, Self::Error> {
        match value {
            cj_version_t::CJ_VERSION_V1_0 => Ok(Self::V1_0),
            cj_version_t::CJ_VERSION_V1_1 => Ok(Self::V1_1),
            cj_version_t::CJ_VERSION_V2_0 => Ok(Self::V2_0),
            cj_version_t::CJ_VERSION_UNKNOWN => Err(()),
        }
    }
}

impl From<Option<CityJSONVersion>> for cj_version_t {
    fn from(value: Option<CityJSONVersion>) -> Self {
        match value {
            Some(version) => version.into(),
            None => Self::CJ_VERSION_UNKNOWN,
        }
    }
}

impl From<cjlib::cityjson::v2_0::RealWorldCoordinate> for cj_vertex_t {
    fn from(value: cjlib::cityjson::v2_0::RealWorldCoordinate) -> Self {
        Self {
            x: value.x(),
            y: value.y(),
            z: value.z(),
        }
    }
}

impl From<cj_vertex_t> for cjlib::cityjson::v2_0::RealWorldCoordinate {
    fn from(value: cj_vertex_t) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl From<cjlib::cityjson::v2_0::UVCoordinate> for cj_uv_t {
    fn from(value: cjlib::cityjson::v2_0::UVCoordinate) -> Self {
        Self {
            u: value.u(),
            v: value.v(),
        }
    }
}

impl From<cj_uv_t> for cjlib::cityjson::v2_0::UVCoordinate {
    fn from(value: cj_uv_t) -> Self {
        Self::new(value.u, value.v)
    }
}

impl From<cj_model_capacities_t> for cjlib::cityjson::v2_0::CityModelCapacities {
    fn from(value: cj_model_capacities_t) -> Self {
        Self {
            cityobjects: value.cityobjects,
            vertices: value.vertices,
            semantics: value.semantics,
            materials: value.materials,
            textures: value.textures,
            geometries: value.geometries,
            template_vertices: value.template_vertices,
            template_geometries: value.template_geometries,
            uv_coordinates: value.uv_coordinates,
        }
    }
}
