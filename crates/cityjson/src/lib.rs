#![doc = include_str!("../README.md")]
//! ## Examples
//!
//! Below is an integration test that builds a CityModel with all possible features from the CityJSON v1.1 specification.
//!
//! <details>
//!
//! ```
#![doc = include_str!("../tests/build_dummy_complete_owned.rs")]
//! ```
//! </details>
//!
//! Its JSON representation:
//!
//! <details>
//!
//! ```json
#![doc = include_str!("../tests/data/v1_1/cityjson_dummy_complete.city.json")]
//! ```
//! </details>

pub mod cityjson;
pub mod error;
pub mod resources;
pub mod v1_0;
pub mod v1_1;
pub mod v2_0;

/// The prelude module provides a convenient way to import commonly used types and traits.
pub mod prelude {
    pub use super::{CityJSON, CityJSONVersion, CityModelType};
    // Re-export from cityjson module
    pub use crate::cityjson::{
        core::appearance::{ImageType, TextureType, WrapMode, RGB, RGBA},
        core::attributes::{AttributeValue, Attributes, BorrowedAttributes, OwnedAttributes},
        core::boundary::{
            nested::{
                BoundaryNestedMultiLineString, BoundaryNestedMultiLineString16,
                BoundaryNestedMultiLineString32, BoundaryNestedMultiLineString64,
                BoundaryNestedMultiOrCompositeSolid, BoundaryNestedMultiOrCompositeSolid16,
                BoundaryNestedMultiOrCompositeSolid32, BoundaryNestedMultiOrCompositeSolid64,
                BoundaryNestedMultiOrCompositeSurface, BoundaryNestedMultiOrCompositeSurface16,
                BoundaryNestedMultiOrCompositeSurface32, BoundaryNestedMultiOrCompositeSurface64,
                BoundaryNestedMultiPoint, BoundaryNestedMultiPoint16, BoundaryNestedMultiPoint32,
                BoundaryNestedMultiPoint64, BoundaryNestedSolid, BoundaryNestedSolid16,
                BoundaryNestedSolid32, BoundaryNestedSolid64,
            },
            Boundary, Boundary16, Boundary32, Boundary64, BoundaryType,
        },
        core::coordinate::{
            FlexibleCoordinate, GeometryVertices16, GeometryVertices32, GeometryVertices64,
            QuantizedCoordinate, RealWorldCoordinate, UVCoordinate, UVVertices16, UVVertices32,
            UVVertices64, Vertices,
        },
        core::extension::{ExtensionCore, ExtensionsCore},
        core::geometry::{BuilderMode, GeometryBuilder, GeometryType, LoD},
        core::metadata::{BBox, CityModelIdentifier, Date, CRS},
        core::vertex::{VertexIndex, VertexIndex16, VertexIndex32, VertexIndex64, RawVertexView},
        traits::appearance::{material::MaterialTrait, texture::TextureTrait},
        traits::citymodel::{CityModelTrait, CityModelTypes},
        traits::cityobject::{CityObjectTrait, CityObjectTypeTrait, CityObjectsTrait},
        traits::coordinate::Coordinate,
        traits::extension::{ExtensionTrait, ExtensionsTrait},
        traits::geometry::GeometryTrait,
        traits::metadata::{BBoxTrait, MetadataTrait},
        traits::semantic::{SemanticTrait, SemanticTypeTrait},
        traits::transform::TransformTrait,
        traits::vertex::{VertexIndexVec, VertexIndicesSequence, VertexRef},
    };
    // Re-export from errors module
    pub use crate::error::{Error, Result};

    // Re-export from resources module
    pub use crate::resources::{
        mapping::{materials::MaterialMap, semantics::SemanticMap, textures::TextureMap},
        pool::{DefaultResourcePool, ResourceId32, ResourcePool, ResourceRef},
        storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage},
    };
}

use prelude::*;
use std::fmt;

/// CityModel type.
///
/// Marks if the [CityModel] represents a CityJSON object or a CityJSONFeature object.
#[repr(C)]
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
pub enum CityJSONVersion {
    V1_0,
    V1_1,
    #[default]
    V2_0,
}

impl CityJSONVersion {
    fn _from_str(value: &str) -> error::Result<CityJSONVersion> {
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
pub enum CityJSON<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    V1_0(v1_0::CityModel<VR, RR, SS>),
    V1_1(v1_1::CityModel<VR, RR, SS>),
    V2_0(v2_0::CityModel<VR, RR, SS>),
}

fn format_option<T: std::fmt::Display>(option: &Option<T>) -> String {
    option
        .as_ref()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "None".to_string())
}
