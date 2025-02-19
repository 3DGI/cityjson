use crate::common::index::VertexRef;
use crate::resources::pool::ResourceRef;

/// Maps geometry vertices to texture coordinates and textures
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct TextureMap<VR: VertexRef, RR: ResourceRef> {
    vertices: Vec<Option<VR>>,      // Texture vertices
    rings: Vec<VR>,                 // Indices into vertices
    ring_textures: Vec<Option<RR>>, // Texture indices
    surfaces: Vec<VR>,              // Indices into rings
    shells: Vec<VR>,                // Indices into surfaces
    solids: Vec<VR>,                // Indices into shells
}
