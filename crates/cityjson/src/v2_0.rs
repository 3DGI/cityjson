//! # `CityJSON` version 2.0
//!
//! Implementation of the `CityJSON` types and traits for `CityJSON` version 2.0.
pub(crate) mod appearance;
pub(crate) mod citymodel;
pub(crate) mod cityobject;
pub(crate) mod extension;
pub(crate) mod geometry;
pub(crate) mod metadata;
pub(crate) mod transform;
pub(crate) mod types;

pub use appearance::{
    material::{BorrowedMaterial, Material, OwnedMaterial},
    texture::{BorrowedTexture, OwnedTexture, Texture},
};
pub use citymodel::{CityModel, GeometryBuilder, GeometryBuilderExt};
pub use cityobject::{
    BorrowedCityObjects, CityObject, CityObjectType, CityObjects, OwnedCityObjects,
};
pub use extension::{Extension, Extensions};
pub use geometry::{
    Geometry,
    semantic::{Semantic, SemanticType},
};
pub use metadata::{Contact, ContactRole, ContactType, Metadata};
pub use transform::Transform;
pub use types::{CityObjectIdentifier, RGB, RGBA, ThemeName};
