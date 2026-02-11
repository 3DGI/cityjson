//! # Error types
//!
//! When operations go wrong.
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    IncompatibleBoundary(String, String),
    IndexConversion {
        source_type: String,
        target_type: String,
        value: String,
    },
    IndexOverflow {
        index_type: String,
        value: String,
    },
    VerticesContainerFull {
        attempted: usize,
        maximum: usize,
    },
    ResourcePoolFull {
        attempted: usize,
        maximum: usize,
    },
    InvalidGeometry(String),
    InvalidShell {
        reason: String,
        surface_count: usize,
    },
    InvalidRing {
        reason: String,
        vertex_count: usize,
    },
    InvalidLineString {
        reason: String,
        vertex_count: usize,
    },
    NoActiveElement {
        element_type: String, // "surface", "shell", or "solid"
    },
    InvalidReference {
        element_type: String, // "surface", "shell"
        index: usize,
        max_index: usize,
    },
    MissingOuterElement {
        context: String, // e.g., "Cannot add inner ring before outer ring is set"
    },
    InvalidGeometryType {
        expected: String,
        found: String,
    },
    IncompleteGeometry(String),
    UnsupportedVersion(String, String),
    InvalidCityObjectType(String),
    InvalidJson(String),
    MissingVersion,
    UnsupportedCityJSONVersion(String),
    Import(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    fn fmt_index_related(&self, f: &mut Formatter<'_>) -> Option<std::fmt::Result> {
        match self {
            Error::IncompatibleBoundary(source_boundarytype, target_boundarytype) => Some(write!(
                f,
                "cannot convert a {source_boundarytype} to a {target_boundarytype}"
            )),
            Error::IndexConversion {
                source_type,
                target_type,
                value,
            } => Some(write!(
                f,
                "failed to convert index from {source_type} to {target_type}: value {value}"
            )),
            Error::IndexOverflow { index_type, value } => {
                Some(write!(f, "index overflow for {index_type}: value {value}"))
            }
            Error::VerticesContainerFull { attempted, maximum } => Some(write!(
                f,
                "attempted to store {attempted} vertices in a container with capacity {maximum}"
            )),
            Error::ResourcePoolFull { attempted, maximum } => Some(write!(
                f,
                "attempted to store {attempted} resources in a pool with maximum {maximum} slots"
            )),
            _ => None,
        }
    }

    fn fmt_geometry_related(&self, f: &mut Formatter<'_>) -> Option<std::fmt::Result> {
        match self {
            Error::InvalidGeometry(msg) => Some(write!(f, "{msg}")),
            Error::InvalidShell {
                reason,
                surface_count,
            } => Some(write!(
                f,
                "Invalid shell: {reason} (surface count: {surface_count})"
            )),
            Error::InvalidRing {
                reason,
                vertex_count,
            } => Some(write!(
                f,
                "Invalid ring: {reason} (vertex count: {vertex_count})"
            )),
            Error::InvalidLineString {
                reason,
                vertex_count,
            } => Some(write!(
                f,
                "Invalid linestring: {reason} (vertex count: {vertex_count})"
            )),
            Error::NoActiveElement { element_type } => {
                Some(write!(f, "No {element_type} in progress"))
            }
            Error::InvalidReference {
                element_type,
                index,
                max_index,
            } => Some(write!(
                f,
                "Invalid {element_type} index: {index} (max: {max_index})"
            )),
            Error::MissingOuterElement { context } => Some(write!(f, "{context}")),
            Error::InvalidGeometryType { expected, found } => Some(write!(
                f,
                "Invalid geometry type: expected {expected}, found {found}"
            )),
            Error::IncompleteGeometry(msg) => Some(write!(f, "Incomplete geometry: {msg}")),
            _ => None,
        }
    }

    fn fmt_version_related(&self, f: &mut Formatter<'_>) -> Option<std::fmt::Result> {
        match self {
            Error::UnsupportedVersion(v, supported) => Some(write!(
                f,
                "the CityJSON version should be one of {supported}, but got {v}"
            )),
            Error::InvalidCityObjectType(v) => Some(write!(f, "invalid CityObject type: {v}")),
            Error::InvalidJson(msg) => Some(write!(f, "Invalid JSON: {msg}")),
            Error::MissingVersion => {
                Some(write!(f, "Missing 'version' field in CityJSON document"))
            }
            Error::UnsupportedCityJSONVersion(version) => {
                Some(write!(f, "Unsupported CityJSON version: {version}"))
            }
            Error::Import(msg) => Some(write!(f, "Import error: {msg}")),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(result) = self.fmt_index_related(f) {
            return result;
        }
        if let Some(result) = self.fmt_geometry_related(f) {
            return result;
        }
        if let Some(result) = self.fmt_version_related(f) {
            return result;
        }

        unreachable!("all `Error` variants must be formatted");
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
