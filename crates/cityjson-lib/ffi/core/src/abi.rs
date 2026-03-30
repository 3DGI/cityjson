use cjlib::{CityJSONVersion, json::RootKind};

/// Stable status codes for the shared C ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum cj_status_t {
    Success = 0,
    InvalidArgument = 1,
    Io = 2,
    Syntax = 3,
    Version = 4,
    Shape = 5,
    Unsupported = 6,
    Model = 7,
    Internal = 8,
}

impl Default for cj_status_t {
    fn default() -> Self {
        Self::Success
    }
}

/// Stable error categories for the shared C ABI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum cj_error_kind_t {
    None = 0,
    InvalidArgument = 1,
    Io = 2,
    Syntax = 3,
    Version = 4,
    Shape = 5,
    Unsupported = 6,
    Model = 7,
    Internal = 8,
}

impl Default for cj_error_kind_t {
    fn default() -> Self {
        Self::None
    }
}

/// Stable root type discriminant for probed inputs.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum cj_root_kind_t {
    CityJSON = 0,
    CityJSONFeature = 1,
}

impl Default for cj_root_kind_t {
    fn default() -> Self {
        Self::CityJSON
    }
}

/// Stable version discriminant for probed inputs.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum cj_version_t {
    Unknown = 0,
    V1_0 = 1,
    V1_1 = 2,
    V2_0 = 3,
}

impl Default for cj_version_t {
    fn default() -> Self {
        Self::Unknown
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
            version: probe.version().map_or(cj_version_t::Unknown, Into::into),
            has_version: probe.version().is_some(),
        }
    }
}

impl From<RootKind> for cj_root_kind_t {
    fn from(value: RootKind) -> Self {
        match value {
            RootKind::CityJSON => Self::CityJSON,
            RootKind::CityJSONFeature => Self::CityJSONFeature,
        }
    }
}

impl From<cj_root_kind_t> for RootKind {
    fn from(value: cj_root_kind_t) -> Self {
        match value {
            cj_root_kind_t::CityJSON => Self::CityJSON,
            cj_root_kind_t::CityJSONFeature => Self::CityJSONFeature,
        }
    }
}

impl From<CityJSONVersion> for cj_version_t {
    fn from(value: CityJSONVersion) -> Self {
        match value {
            CityJSONVersion::V1_0 => Self::V1_0,
            CityJSONVersion::V1_1 => Self::V1_1,
            CityJSONVersion::V2_0 => Self::V2_0,
        }
    }
}

impl TryFrom<cj_version_t> for CityJSONVersion {
    type Error = ();

    fn try_from(value: cj_version_t) -> Result<Self, Self::Error> {
        match value {
            cj_version_t::V1_0 => Ok(Self::V1_0),
            cj_version_t::V1_1 => Ok(Self::V1_1),
            cj_version_t::V2_0 => Ok(Self::V2_0),
            cj_version_t::Unknown => Err(()),
        }
    }
}

impl From<Option<CityJSONVersion>> for cj_version_t {
    fn from(value: Option<CityJSONVersion>) -> Self {
        match value {
            Some(version) => version.into(),
            None => Self::Unknown,
        }
    }
}
