use crate::vertex::{OptionalVertexIndices, VertexIndices, VertexInteger};

/// Maps geometry vertices to texture coordinates and textures
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct TextureMap<T: VertexInteger> {
    vertices: OptionalVertexIndices<T>,      // Texture vertices
    rings: VertexIndices<T>,                 // Indices into vertices
    ring_textures: OptionalVertexIndices<T>, // Texture indices
    surfaces: VertexIndices<T>,              // Indices into rings
    shells: VertexIndices<T>,                // Indices into surfaces
    solids: VertexIndices<T>,                // Indices into shells
}
