//! Error types.
use std::fmt::{Debug, Display, Formatter};

/// Errors returned by cityjson-rs operations.
#[derive(Clone, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Boundary type mismatch — e.g. trying to convert a `Solid` boundary into a `MultiPoint`.
    IncompatibleBoundary(String, String),
    /// A vertex index could not be converted between integer types (e.g. `u64` → `u16` overflow).
    IndexConversion {
        source_type: String,
        target_type: String,
        value: String,
    },
    /// A vertex index value exceeds the range of the target index type.
    IndexOverflow {
        index_type: String,
        value: String,
    },
    /// The vertex container is full for the chosen `VR` type (e.g. more than `u32::MAX` vertices).
    VerticesContainerFull {
        attempted: usize,
        maximum: usize,
    },
    /// A resource pool (semantics, materials, textures, or geometries) has reached its limit.
    ResourcePoolFull {
        attempted: usize,
        maximum: usize,
    },
    /// General geometry validation failure.
    InvalidGeometry(String),
    /// A shell failed validation (e.g. fewer than four surfaces for a closed solid).
    InvalidShell {
        reason: String,
        surface_count: usize,
    },
    /// A ring failed validation (e.g. fewer than three vertices).
    InvalidRing {
        reason: String,
        vertex_count: usize,
    },
    /// A linestring failed validation (e.g. fewer than two vertices).
    InvalidLineString {
        reason: String,
        vertex_count: usize,
    },
    /// A boundary index references an element that does not exist.
    InvalidReference {
        element_type: String,
        index: usize,
        max_index: usize,
    },
    /// A geometry operation expected one type but found another.
    InvalidGeometryType {
        expected: String,
        found: String,
    },
    /// A geometry is structurally incomplete (e.g. missing required fields).
    IncompleteGeometry(String),
    /// The `CityJSON` `"version"` field holds an unsupported value.
    UnsupportedVersion(String, String),
    /// The city object type string is not a known `CityJSON` type and does not start with `"+"`.
    InvalidCityObjectType(String),
    /// JSON parsing failed.
    InvalidJson(String),
    /// The `CityJSON` document is missing the required `"version"` field.
    MissingVersion,
    /// The `CityJSON` version is not supported by this crate.
    UnsupportedCityJSONVersion(String),
    /// An I/O or import error.
    Import(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IncompatibleBoundary(source_boundarytype, target_boundarytype) => {
                write!(
                    f,
                    "cannot convert a {source_boundarytype} to a {target_boundarytype}"
                )
            }
            Error::IndexConversion {
                source_type,
                target_type,
                value,
            } => write!(
                f,
                "failed to convert index from {source_type} to {target_type}: value {value}"
            ),
            Error::IndexOverflow { index_type, value } => {
                write!(f, "index overflow for {index_type}: value {value}")
            }
            Error::VerticesContainerFull { attempted, maximum } => write!(
                f,
                "attempted to store {attempted} vertices in a container with capacity {maximum}"
            ),
            Error::ResourcePoolFull { attempted, maximum } => write!(
                f,
                "attempted to store {attempted} resources in a pool with maximum {maximum} slots"
            ),
            Error::InvalidGeometry(msg) => write!(f, "{msg}"),
            Error::InvalidShell {
                reason,
                surface_count,
            } => write!(
                f,
                "Invalid shell: {reason} (surface count: {surface_count})"
            ),
            Error::InvalidRing {
                reason,
                vertex_count,
            } => write!(f, "Invalid ring: {reason} (vertex count: {vertex_count})"),
            Error::InvalidLineString {
                reason,
                vertex_count,
            } => write!(
                f,
                "Invalid linestring: {reason} (vertex count: {vertex_count})"
            ),
            Error::InvalidReference {
                element_type,
                index,
                max_index,
            } => write!(
                f,
                "Invalid {element_type} index: {index} (max: {max_index})"
            ),
            Error::InvalidGeometryType { expected, found } => {
                write!(
                    f,
                    "Invalid geometry type: expected {expected}, found {found}"
                )
            }
            Error::IncompleteGeometry(msg) => write!(f, "Incomplete geometry: {msg}"),
            Error::UnsupportedVersion(v, supported) => {
                write!(
                    f,
                    "the CityJSON version should be one of {supported}, but got {v}"
                )
            }
            Error::InvalidCityObjectType(v) => write!(f, "invalid CityObject type: {v}"),
            Error::InvalidJson(msg) => write!(f, "Invalid JSON: {msg}"),
            Error::MissingVersion => write!(f, "Missing 'version' field in CityJSON document"),
            Error::UnsupportedCityJSONVersion(version) => {
                write!(f, "Unsupported CityJSON version: {version}")
            }
            Error::Import(msg) => write!(f, "Import error: {msg}"),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Import(value.to_string())
    }
}
