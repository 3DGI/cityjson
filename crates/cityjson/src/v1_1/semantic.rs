//! # Semantics
//!
//! Represents a [Semantic object](https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives).
use crate::common::attributes::Attributes;
use crate::common::index::{VertexIndex, VertexIndices, VertexRef};
use crate::common::storage::StringStorage;

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
pub struct Semantic<VI: VertexRef, S: StringStorage> {
    /// The type of the semantic surface
    type_semantic: SemanticType,
    /// Indices to child semantics in the global semantics pool
    children: Option<VertexIndices<VI>>,
    /// Index to parent semantic in the global semantics pool
    parent: Option<VertexIndex<VI>>,
    /// Additional attributes of the semantic surface
    attributes: Option<Attributes<S>>,
}

impl<VI: VertexRef, S: StringStorage> Semantic<VI, S> {
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

    /// Returns a mutable reference to the children indices, creating default indices if they do not exist
    #[inline]
    pub fn children_mut(&mut self) -> &mut VertexIndices<VI> {
        if self.children.is_none() {
            self.children = Some(VertexIndices::new());
        }
        self.children.as_mut().unwrap()
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

    /// Returns a reference to the attributes if they exist
    #[inline]
    pub fn attributes(&self) -> Option<&Attributes<S>> {
        self.attributes.as_ref()
    }

    /// Returns a mutable reference to the attributes, creating default attributes if they do not exist
    #[inline]
    pub fn attributes_mut(&mut self) -> &mut Attributes<S> {
        if self.attributes.is_none() {
            self.attributes = Some(Attributes::new());
        }
        self.attributes.as_mut().unwrap()
    }
}
