use crate::cityjson::index::{VertexIndex, VertexRef};
use crate::resources::pool::ResourceRef;

/// Maps geometry vertices to texture coordinates and textures
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct TextureMap<VR: VertexRef, RR: ResourceRef> {
    vertices: Vec<Option<VertexIndex<VR>>>, // Texture vertices
    rings: Vec<VertexIndex<VR>>,            // Indices into vertices
    ring_textures: Vec<Option<RR>>,         // Texture indices
    surfaces: Vec<VertexIndex<VR>>,         // Indices into rings
    shells: Vec<VertexIndex<VR>>,           // Indices into surfaces
    solids: Vec<VertexIndex<VR>>,           // Indices into shells
}
