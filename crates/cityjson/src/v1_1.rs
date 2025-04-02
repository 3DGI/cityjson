//! # CityJSON version 1.1
//!
//! Implementation of the CityJSON types and traits for CityJSON version 1.1.
pub mod appearance;
pub mod citymodel;
pub mod cityobject;
pub mod extension;
pub mod geometry;
pub mod metadata;
pub mod transform;

// Re-export main types from appearance
pub use appearance::{
    material::{BorrowedMaterial, Material, OwnedMaterial},
    texture::{BorrowedTexture, OwnedTexture, Texture},
};

// Re-export main types from citymodel
pub use citymodel::CityModel;

// Re-export main types from cityobject
pub use cityobject::{
    BorrowedCityObjects, CityObject, CityObjectType, CityObjects, OwnedCityObjects,
};

pub use extension::{Extension, Extensions};

// Re-export main types from geometry
pub use geometry::{
    semantic::{Semantic, SemanticType},
    Geometry,
};

// Re-export main types from metadata
pub use metadata::{
    Contact, ContactRole, ContactType, Metadata,
};

// Re-export main types from transform
pub use transform::Transform;
