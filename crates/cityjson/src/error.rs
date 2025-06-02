//! # Error types
//!
//! When operations go wrong.
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Hash, PartialEq, Eq)]
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
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IncompatibleBoundary(source_boundarytype, target_boundarytype) => {
                write!(
                    f,
                    "cannot convert a {} to a {}",
                    source_boundarytype, target_boundarytype
                )
            }
            Error::IndexConversion {
                source_type,
                target_type,
                value,
            } => {
                write!(
                    f,
                    "failed to convert index from {} to {}: value {}",
                    source_type, target_type, value
                )
            }
            Error::IndexOverflow { index_type, value } => {
                write!(f, "index overflow for {}: value {}", index_type, value)
            }
            Error::VerticesContainerFull { attempted, maximum } => {
                write!(
                    f,
                    "attempted to store {} vertices in a container with capacity {}",
                    attempted, maximum
                )
            }
            Error::InvalidGeometry(msg) => {
                write!(f, "{}", msg)
            }
            Error::InvalidShell {
                reason,
                surface_count,
            } => {
                write!(
                    f,
                    "Invalid shell: {} (surface count: {})",
                    reason, surface_count
                )
            }
            Error::InvalidRing {
                reason,
                vertex_count,
            } => {
                write!(
                    f,
                    "Invalid ring: {} (vertex count: {})",
                    reason, vertex_count
                )
            }
            Error::InvalidLineString {
                reason,
                vertex_count,
            } => {
                write!(
                    f,
                    "Invalid linestring: {} (vertex count: {})",
                    reason, vertex_count
                )
            }
            Error::NoActiveElement { element_type } => {
                write!(f, "No {} in progress", element_type)
            }
            Error::InvalidReference {
                element_type,
                index,
                max_index,
            } => {
                write!(
                    f,
                    "Invalid {} index: {} (max: {})",
                    element_type, index, max_index
                )
            }
            Error::MissingOuterElement { context } => {
                write!(f, "{}", context)
            }
            Error::InvalidGeometryType { expected, found } => {
                write!(
                    f,
                    "Invalid geometry type: expected {}, found {}",
                    expected, found
                )
            }
            Error::IncompleteGeometry(msg) => {
                write!(f, "Incomplete geometry: {}", msg)
            }
            Error::UnsupportedVersion(v, supported) => {
                write!(
                    f,
                    "the CityJSON version should be one of {}, but got {}",
                    supported, v
                )
            },
            Error::InvalidCityObjectType(v) =>  {
                write!(f, "invalid CityObject type: {}", v)
            }
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for Error {}
