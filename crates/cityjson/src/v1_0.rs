//! # CityJSON version 1.0
//!
//! Implementation of the CityJSON types and traits for CityJSON version 1.0.
pub mod appearance;
pub mod citymodel;
pub mod extension;
pub mod geometry;
pub mod metadata;
pub mod transform;

pub use appearance::{
    material::{BorrowedMaterial, Material, OwnedMaterial},
    texture::{BorrowedTexture, OwnedTexture, Texture},
};
pub use extension::{Extension, Extensions};
pub use geometry::{
    semantic::{Semantic, SemanticType},
    Geometry,
};
pub use metadata::Metadata;
pub use transform::Transform;
