//! Appearance types: materials and textures.
//!
//! `CityJSON` supports two appearance mechanisms: [`material`] (rendering properties) and
//! [`texture`] (image mapping). Both are theme-based: geometry assigns appearance per
//! named theme, so a surface can have different materials or textures depending on the
//! rendering context.
//!
//! [`RGB`] and [`RGBA`] represent colors with components in the range 0.0–1.0.

pub use crate::cityjson::core::appearance::{
    ImageType, RGB, RGBA, TextureType, ThemeName, WrapMode,
};

pub mod material;
pub mod texture;
