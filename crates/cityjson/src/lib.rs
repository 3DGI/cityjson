#![doc = include_str!("../README.md")]
pub(crate) mod backend;
mod cityjson;
pub mod error;
pub mod raw;
pub mod resources;
pub mod v2_0;

pub mod prelude {
    pub use super::{CityJSON, CityJSONVersion, CityModelType};
    pub use crate::error::{Error, Result};
    pub use crate::resources::{
        handles::{
            CityObjectHandle, GeometryHandle, GeometryTemplateHandle, MaterialHandle,
            SemanticHandle, TextureHandle,
        },
        mapping::{materials::MaterialMap, semantics::SemanticMap, textures::TextureMap},
        storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage},
    };
}

use crate::error::{Error, Result};
use crate::resources::storage::StringStorage;
use crate::v2_0::VertexRef;
use std::fmt;

/// Whether a [`CityModel`](v2_0::citymodel::CityModel) is a full `CityJSON` document or a
/// single-feature `CityJSONFeature` (used for streaming and tiling).
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CityModelType {
    #[default]
    CityJSON,
    CityJSONFeature,
}

impl fmt::Display for CityModelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CityModelType::CityJSON => {
                write!(f, "CityJSON")
            }
            CityModelType::CityJSONFeature => {
                write!(f, "CityJSONFeature")
            }
        }
    }
}

impl CityModelType {
    fn _from_str(value: &str) -> error::Result<CityModelType> {
        match value {
            "CityJSON" => Ok(CityModelType::CityJSON),
            "CityJSONFeature" => Ok(CityModelType::CityJSONFeature),
            _ => Err(Error::UnsupportedVersion(
                value.to_string(),
                "CityJSON, CityJSONFeature".to_string(),
            )),
        }
    }
}

impl TryFrom<&str> for CityModelType {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        CityModelType::_from_str(value)
    }
}

impl TryFrom<String> for CityModelType {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        CityModelType::_from_str(value.as_ref())
    }
}

/// Supported `CityJSON` spec versions. Currently only v2.0.
#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum CityJSONVersion {
    #[default]
    V2_0,
}

impl CityJSONVersion {
    fn _from_str(value: &str) -> error::Result<CityJSONVersion> {
        match value {
            "2.0" | "2.0.0" | "2.0.1" => Ok(CityJSONVersion::V2_0),
            _ => Err(Error::UnsupportedVersion(
                value.to_string(),
                "2.0, 2.0.0, 2.0.1".to_string(),
            )),
        }
    }
}

impl fmt::Display for CityJSONVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CityJSONVersion::V2_0 => {
                write!(f, "2.0")
            }
        }
    }
}

impl TryFrom<&str> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        CityJSONVersion::_from_str(value)
    }
}

impl TryFrom<String> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        CityJSONVersion::_from_str(value.as_ref())
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum CityJSON<VR: VertexRef, SS: StringStorage> {
    V2_0(v2_0::CityModel<VR, SS>),
}

fn format_option<T: std::fmt::Display>(option: Option<&T>) -> String {
    option.map_or_else(|| "None".to_string(), std::string::ToString::to_string)
}
