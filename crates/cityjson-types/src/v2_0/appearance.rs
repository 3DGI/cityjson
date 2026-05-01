//! Appearance types: materials and textures (spec §6).
//!
//! `CityJSON` supports two appearance mechanisms: [`material`] (rendering properties such as
//! diffuse color and shininess) and [`texture`] (image mapping via UV coordinates). Both are
//! theme-based: the same surface can carry different appearances for different rendering
//! contexts by assigning them to named themes.
//!
//! Resources (`Material`, `Texture`) are registered once on [`CityModel`] via
//! [`add_material`](super::citymodel::CityModel::add_material) /
//! [`add_texture`](super::citymodel::CityModel::add_texture), which return typed handles. Each
//! geometry then holds a [`MaterialMap`] and a [`TextureMap`] that map theme names to
//! per-surface handle arrays.
//!
//! [`RGB`] and [`RGBA`] represent colors with components in the range 0.0–1.0.
//!
//! [`CityModel`]: super::citymodel::CityModel
//! [`MaterialMap`]: crate::resources::mapping::materials::MaterialMap
//! [`TextureMap`]: crate::resources::mapping::textures::TextureMap

pub use crate::cityjson::core::appearance::{
    ImageType, RGB, RGBA, TextureType, ThemeName, WrapMode,
};

pub mod material;
pub mod texture;
