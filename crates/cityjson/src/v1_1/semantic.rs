//! # Semantics
//!
//! Represents a [Semantic object](https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives).
use crate::common::attributes::Attributes;
use crate::common::index::{VertexIndex, VertexIndices, VertexRef};
use crate::common::semantic::SemanticType;
use crate::common::storage::StringStorage;

#[derive(Debug, Clone)]
pub struct Semantic<VR: VertexRef, SS: StringStorage> {
    /// The type of the semantic surface
    type_semantic: SemanticType,
    /// Indices to child semantics in the global semantics pool
    children: Option<VertexIndices<VR>>,
    /// Index to parent semantic in the global semantics pool
    parent: Option<VertexIndex<VR>>,
    /// Additional attributes of the semantic surface
    attributes: Option<Attributes<SS>>,
}

impl<VR: VertexRef, SS: StringStorage> crate::common::semantic::Semantic<VR, SS> for Semantic<VR, SS> {
    /// Create a new semantic with the given type
    #[inline]
    fn new(type_semantic: SemanticType) -> Self {
        Self {
            type_semantic,
            children: None,
            parent: None,
            attributes: None,
        }
    }
    /// Check if this semantic has any children
    #[inline]
    fn has_children(&self) -> bool {
        self.children.as_ref().map_or(false, |c| !c.is_empty())
    }
    /// Check if this semantic has a parent
    #[inline]
    fn has_parent(&self) -> bool {
        self.parent.is_some()
    }
    /// Returns a reference to the children indices if they exist
    #[inline]
    fn children(&self) -> Option<&VertexIndices<VR>> {
        self.children.as_ref()
    }
    /// Returns a mutable reference to the children indices, creating default indices if they do not exist
    #[inline]
    fn children_mut(&mut self) -> &mut VertexIndices<VR> {
        if self.children.is_none() {
            self.children = Some(VertexIndices::new());
        }
        self.children.as_mut().unwrap()
    }
    /// Returns a reference to the parent index if it exists
    #[inline]
    fn parent(&self) -> Option<&VertexIndex<VR>> {
        self.parent.as_ref()
    }
    /// Returns a mutable reference to the parent index if it exists
    #[inline]
    fn parent_mut(&mut self) -> Option<&mut VertexIndex<VR>> {
        self.parent.as_mut()
    }
    /// Returns a reference to the attributes if they exist
    #[inline]
    fn attributes(&self) -> Option<&Attributes<SS>> {
        self.attributes.as_ref()
    }
    /// Returns a mutable reference to the attributes, creating default attributes if they do not exist
    #[inline]
    fn attributes_mut(&mut self) -> &mut Attributes<SS> {
        if self.attributes.is_none() {
            self.attributes = Some(Attributes::new());
        }
        self.attributes.as_mut().unwrap()
    }
}

