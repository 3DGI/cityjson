
#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Boundary {
    pub(crate) vertices: Vec<u32>, // Indices into global vertices
    pub(crate) rings: Vec<u32>,    // Indices into vertices
    pub(crate) surfaces: Vec<u32>, // Indices into rings
    pub(crate) shells: Vec<u32>,   // Indices into surfaces
    pub(crate) solids: Vec<u32>,   // Indices into shells
}
