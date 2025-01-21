use crate::indices::{GeometryIndices, OptionalGeometryIndices};

/// Maps geometry vertices to texture coordinates and textures
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq )]
pub struct TextureMap {
    vertices: OptionalGeometryIndices,     // Texture vertices
    rings: GeometryIndices,               // Indices into vertices
    ring_textures: OptionalGeometryIndices, // Texture indices
    surfaces: GeometryIndices,            // Indices into rings
    shells: GeometryIndices,             // Indices into surfaces
    solids: GeometryIndices,            // Indices into shells
}