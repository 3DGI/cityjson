use crate::common::attributes::Attributes;
use crate::common::storage::StringStorage;
use crate::index::{VertexIndex, VertexIndices, VertexRef};

/// Semantic surface type.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum SemanticType {
    RoofSurface,
    GroundSurface,
    WallSurface,
    ClosureSurface,
    OuterCeilingSurface,
    OuterFloorSurface,
    Window,
    Door,
    InteriorWallSurface,
    CeilingSurface,
    FloorSurface,
    WaterSurface,
    WaterGroundSurface,
    WaterClosureSurface,
    TrafficArea,
    AuxiliaryTrafficArea,
    TransportationMarking,
    TransportationHole,
    Extension(String),
}

pub trait Semantic<VI: VertexRef, S: StringStorage> {
    /// Create a new semantic with the given type
    fn new(type_semantic: SemanticType) -> Self;
    /// Check if this semantic has any children
    fn has_children(&self) -> bool;
    /// Check if this semantic has a parent
    fn has_parent(&self) -> bool;
    /// Returns a reference to the children indices if they exist
    fn children(&self) -> Option<&VertexIndices<VI>>;
    /// Returns a mutable reference to the children indices, creating default indices if they do not exist
    fn children_mut(&mut self) -> &mut VertexIndices<VI>;
    /// Returns a reference to the parent index if it exists
    fn parent(&self) -> Option<&VertexIndex<VI>>;
    /// Returns a mutable reference to the parent index if it exists
    fn parent_mut(&mut self) -> Option<&mut VertexIndex<VI>>;
    /// Returns a reference to the attributes if they exist
    fn attributes(&self) -> Option<&Attributes<S>>;
    /// Returns a mutable reference to the attributes, creating default attributes if they do not exist
    fn attributes_mut(&mut self) -> &mut Attributes<S>;
}