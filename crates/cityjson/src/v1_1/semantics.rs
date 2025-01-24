use crate::vertex::{VertexIndices, VertexInteger};
use crate::VertexIndex;
use crate::attributes::{Attributes};
use crate::storage::StringStorage;

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

#[derive(Debug, Clone)]
pub struct Semantic<VI: VertexInteger, S: StringStorage> {
    /// The type of the semantic surface
    pub type_semantic: SemanticType,
    /// Indices to child semantics in the global semantics pool
    pub children: Option<VertexIndices<VI>>,
    /// Index to parent semantic in the global semantics pool
    pub parent: Option<VertexIndex<VI>>,
    /// Additional attributes of the semantic surface
    pub attributes: Option<Attributes<S>>,
}

impl<VI: VertexInteger, S: StringStorage> Semantic<VI, S> {
    /// Create a new semantic with the given type
    #[inline]
    pub fn new(type_semantic: SemanticType) -> Self {
        Self {
            type_semantic,
            children: None,
            parent: None,
            attributes: None,
        }
    }

    /// Check if this semantic has any children
    #[inline]
    pub fn has_children(&self) -> bool {
        self.children.as_ref().map_or(false, |c| !c.is_empty())
    }

    /// Check if this semantic has a parent
    #[inline]
    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }
    /// Returns a reference to the children indices if they exist
    #[inline]
    pub fn children(&self) -> Option<&VertexIndices<VI>> {
        self.children.as_ref()
    }

    /// Returns a mutable reference to the children indices if they exist
    #[inline]
    pub fn children_mut(&mut self) -> Option<&mut VertexIndices<VI>> {
        self.children.as_mut()
    }

    /// Returns a reference to the parent index if it exists
    #[inline]
    pub fn parent(&self) -> Option<&VertexIndex<VI>> {
        self.parent.as_ref()
    }

    /// Returns a mutable reference to the parent index if it exists
    #[inline]
    pub fn parent_mut(&mut self) -> Option<&mut VertexIndex<VI>> {
        self.parent.as_mut()
    }
}
