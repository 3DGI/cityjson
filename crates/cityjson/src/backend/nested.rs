//! Nested backend implementation (work in progress).
//!
//! This backend provides an alternative nested representation of CityJSON data structures.
//! It is currently a skeleton implementation and not yet functional.

pub mod appearance;
pub mod attributes;
pub mod boundary;
pub mod citymodel;
pub mod cityobject;
pub mod coordinate;
pub mod extension;
pub mod geometry;
pub mod geometry_builder;
pub mod geometry_struct;
pub mod metadata;
pub mod semantics;
pub mod transform;
pub mod vertex;

// Re-export key types
pub use citymodel::CityModel;
pub use cityobject::CityObject;
pub use geometry::Geometry;
pub use geometry_builder::{BuilderMode, GeometryBuilder};
