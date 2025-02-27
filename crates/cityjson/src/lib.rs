//! The cityjson-rs library defines the types and methods for representing the complete CityJSON data model in Rust.
//! *cityjson-rs* is meant to be a core dependency in Rust-based CityJSON software, so that the dependent applications can extend the types with their specific functionality.
//! Therefore, *citjson-rs* is designed with performance, flexibility, and ease-of-use in mind.
//! The three criteria are implemented in the following features:
//!
//! - The Geometry representation is flattened into densely packed containers to minimize allocations, improve cache-locality, and enable SIMD operations. This is very different to the nested arrays defined by the CityJSON schema. However, the implementation details are hidden from the API.
//! - Vertex indices, and consequently boundaries, semantics, and appearances can be specialized with either `u16`, `u32` or `u64` types to enable various use cases and memory optimizations.
//! - Supports both borrowed and owned values.
//! - Getter and setter methods are implemented for each CityJSON object and their members to provide a stable API and hide implementation details.
//! - The API is thoroughly documented, including usage examples.
//! - Supports CityJSON Extensions.
//! - Supports multiple CityJSON versions, such as v1.0, v1.1, v2.0, and it is extensible for future versions.

pub mod cityjson;
pub mod errors;
pub mod resources;
pub mod v1_0;
pub mod v1_1;
pub mod v2_0;

use crate::cityjson::vertex::VertexRef;
use crate::errors::Error;
use crate::resources::pool::ResourceRef;
use crate::resources::storage::StringStorage;
use std::fmt;
pub use cityjson::coordinate;



/// CityModel type.
///
/// Marks if the [CityModel] represents a CityJSON object or a CityJSONFeature object.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
pub enum CityJSONVersion {
    V1_0,
    #[default]
    V1_1,
    V2_0,
}

impl CityJSONVersion {
    fn _from_str(value: &str) -> errors::Result<CityJSONVersion> {
        match value {
            "1.0" | "1.0.0" | "1.0.1" | "1.0.2" | "1.0.3" => Ok(CityJSONVersion::V1_0),
            "1.1" | "1.1.0" | "1.1.1" | "1.1.2" | "1.1.3" => Ok(CityJSONVersion::V1_1),
            "2.0" | "2.0.0" | "2.0.1" => Ok(CityJSONVersion::V2_0),
            _ => Err(Error::UnsupportedVersion(
                value.to_string(),
                "1.0, 1.0.0, 1.0.1, 1.0.2, 1.0.3, 1.1, 1.1.0, 1.1.1, 1.1.2, 1.1.3, 2.0, 2.0.0, 2.0.1".to_string(),
            )),
        }
    }
}

impl fmt::Display for CityJSONVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CityJSONVersion::V1_0 => {
                write!(f, "1.0")
            }
            CityJSONVersion::V1_1 => {
                write!(f, "1.1")
            }
            CityJSONVersion::V2_0 => {
                write!(f, "2.0")
            }
        }
    }
}

impl TryFrom<&str> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: &str) -> errors::Result<Self> {
        CityJSONVersion::_from_str(value)
    }
}

impl TryFrom<String> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: String) -> errors::Result<Self> {
        CityJSONVersion::_from_str(value.as_ref())
    }
}

#[derive(Debug)]
pub enum CityJSON<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    V1_1(v1_1::CityModel<VR, RR, SS>),
    V2_0(v2_0::CityModel),
}

fn format_option<T: std::fmt::Display>(option: &Option<T>) -> String {
    option
        .as_ref()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "None".to_string())
}