use crate::common::vertex::{OptionalVertexIndices, VertexIndices, VertexInteger};

/// Maps geometry vertices to texture coordinates and textures
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct TextureMap<VI: VertexInteger> {
    vertices: OptionalVertexIndices<VI>,      // Texture vertices
    rings: VertexIndices<VI>,                 // Indices into vertices
    ring_textures: OptionalVertexIndices<VI>, // Texture indices
    surfaces: VertexIndices<VI>,              // Indices into rings
    shells: VertexIndices<VI>,                // Indices into surfaces
    solids: VertexIndices<VI>,                // Indices into shells
}
