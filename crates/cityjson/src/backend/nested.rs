//! Nested backend implementation (work in progress).
//!
//! This backend provides an alternative nested representation of CityJSON data structures.
//! It is currently a skeleton implementation and not yet functional.

#![allow(dead_code)]

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

// Re-export key types (will be used after integration in Phase 6)
#[allow(unused_imports)]
pub use citymodel::CityModel;
#[allow(unused_imports)]
pub use cityobject::CityObject;
#[allow(unused_imports)]
pub use geometry::Geometry;
#[allow(unused_imports)]
pub use geometry_builder::{BuilderMode, GeometryBuilder};

// Type aliases for common usage
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage};

/// CityModel with owned string storage
#[allow(dead_code)]
pub type OwnedCityModel = CityModel<OwnedStringStorage>;

/// CityModel with borrowed string storage
#[allow(dead_code)]
pub type BorrowedCityModel<'a> = CityModel<BorrowedStringStorage<'a>>;

/// CityObject with owned string storage
#[allow(dead_code)]
pub type OwnedCityObject = CityObject<OwnedStringStorage>;

/// CityObject with borrowed string storage
#[allow(dead_code)]
pub type BorrowedCityObject<'a> = CityObject<BorrowedStringStorage<'a>>;

/// Geometry with owned string storage
#[allow(dead_code)]
pub type OwnedGeometry = Geometry<OwnedStringStorage>;

/// Geometry with borrowed string storage
#[allow(dead_code)]
pub type BorrowedGeometry<'a> = Geometry<BorrowedStringStorage<'a>>;
