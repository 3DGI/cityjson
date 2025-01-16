/// Maps geometry surfaces to semantics or materials
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct SemanticMaterialMapping {
    /// Each item corresponds to the point with the same index in a MultiPoint boundary, the value
    /// of the item is the index of the Semantic or Material object.
    pub points: Vec<Option<u32>>,
    /// Each item corresponds to the linestring with the same index in a MultiLineString boundary,
    /// the value of the item is the index of the Semantic or Material object.
    pub linestrings: Vec<Option<u32>>,
    /// Each item corresponds to the surface with the same index, the value
    /// of the item is the index of the Semantic or Material object.
    pub surfaces: Vec<Option<u32>>,
    pub shells: Vec<u32>,
    pub solids: Vec<u32>,
}

/// Maps geometry vertices to texture coordinates and textures
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq )]
pub struct TextureMapping {
    vertices: Vec<Option<u32>>,     // Texture vertices
    rings: Vec<u32>,               // Indices into vertices
    ring_textures: Vec<Option<u32>>, // Texture indices
    surfaces: Vec<u32>,            // Indices into rings
    shells: Vec<u32>,             // Indices into surfaces
    solids: Vec<u32>,            // Indices into shells
}