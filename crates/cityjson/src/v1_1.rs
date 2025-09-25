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

pub use appearance::{
    material::{BorrowedMaterial, Material, OwnedMaterial},
    texture::{BorrowedTexture, OwnedTexture, Texture},
};
pub use citymodel::CityModel;
pub use cityobject::{
    BorrowedCityObjects, CityObject, CityObjectType, CityObjects, OwnedCityObjects,
};
pub use extension::{Extension, Extensions};
pub use geometry::{
    semantic::{Semantic, SemanticType},
    Geometry,
};
pub use metadata::{Contact, ContactRole, ContactType, Metadata};
pub use transform::Transform;
