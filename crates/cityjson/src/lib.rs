#![doc = include_str!("../README.md")]
//! ## Examples
//!
//! Below is an integration test that builds a `CityModel` with all possible features from the `CityJSON` v2.0 specification.
//!
//! <details>
//!
//! ```
#![doc = include_str!("../examples/cityjson_fake_complete_owned.rs")]
//! ```
//! </details>
//!
//! Its JSON representation:
//!
//! <details>
//!
//! ```json
#![doc = include_str!("../tests/data/v2_0/cityjson_fake_complete.city.json")]
//! ```
//! </details>

pub(crate) mod backend;
pub mod cityjson;
pub mod error;
pub mod raw;
pub mod resources;
pub mod v2_0;

/// The prelude module provides a convenient way to import commonly used types and traits.
pub mod prelude {
    pub use super::{CityJSON, CityJSONVersion, CityModelType};
    // Re-export from cityjson module
    pub use crate::cityjson::core::vertex::VertexIndexVec;
    // Re-export from cityjson module
    pub use crate::cityjson::core::vertex::VertexIndicesSequence;
    // Re-export from cityjson module
    pub use crate::cityjson::core::vertex::VertexRef;
    // Re-export from cityjson module
    pub use crate::cityjson::{
        core::appearance::{ImageType, TextureType, WrapMode},
        core::attributes::{AttributeValue, Attributes, BorrowedAttributes, OwnedAttributes, BorrowedAttributeValue, OwnedAttributeValue},
        core::boundary::{Boundary, Boundary16, Boundary32, Boundary64, BoundaryType},
        core::coordinate::{
            FlexibleCoordinate, GeometryVertices16, GeometryVertices32, GeometryVertices64,
            QuantizedCoordinate, RealWorldCoordinate, UVCoordinate, UVVertices16, UVVertices32,
            UVVertices64, Vertices,
        },
        core::geometry::{BuilderMode, GeometryBuilder, GeometryType, LoD},
        core::metadata::{BBox, CityModelIdentifier, Date, CRS},
        core::vertex::{VertexIndex, VertexIndex16, VertexIndex32, VertexIndex64},
        traits::coordinate::Coordinate,
        traits::semantic::SemanticTypeTrait,
    };
    // Re-export from errors module
    pub use crate::error::{Error, Result};
    // Re-export from resources module
    pub use crate::resources::{
        handles::{
            AttributeRef, CityObjectRef, GeometryRef, MaterialRef, SemanticRef,
            TemplateGeometryRef, TextureRef,
        },
        mapping::{materials::MaterialMap, semantics::SemanticMap, textures::TextureMap},
        storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage},
    };
    pub use crate::v2_0::types::{CityObjectIdentifier, ThemeName, RGB, RGBA};
    pub use crate::v2_0::{Extension, Extensions, GeometryBuilderExt, Transform};
}

use prelude::*;
use std::fmt;

/// `CityModel` type.
///
/// Marks if the `CityModel` represents a `CityJSON` object or a `CityJSONFeature` object.
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
