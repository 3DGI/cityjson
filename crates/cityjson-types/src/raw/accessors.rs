//! Traits for raw data access.

use crate::raw::views::{RawPoolView, RawSliceView};

/// Trait for types that expose their internals for zero-copy serialization.
pub trait RawAccess {
    type Vertex;
    type Geometry;
    type Semantic;
    type Material;
    type Texture;

    fn vertices_raw(&self) -> RawSliceView<'_, Self::Vertex>;
    fn geometries_raw(&self) -> RawPoolView<'_, Self::Geometry>;
    fn semantics_raw(&self) -> RawPoolView<'_, Self::Semantic>;
    fn materials_raw(&self) -> RawPoolView<'_, Self::Material>;
    fn textures_raw(&self) -> RawPoolView<'_, Self::Texture>;
}
