//! # Resource management utilities
//!
//! The semantics, materials and textures are commonly referred to as *resources*.
//! The resources are mapped to geometry boundaries with [mapping::SemanticMap], [mapping::MaterialMap] and [mapping::TextureMap].
//! These maps are version agnostic, while the Semantic, Material, and Texture definitions are versioned.
//! The resources are managed by resource pools that are stored in the CityModel.
pub mod handles;
pub mod mapping;
pub mod pool;
pub mod storage;

pub use handles::{
    AttributeRef, CityObjectRef, GeometryRef, MaterialRef, SemanticRef, TemplateGeometryRef,
    TextureRef,
};
