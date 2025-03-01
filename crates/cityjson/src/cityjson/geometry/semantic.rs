use std::fmt;
use crate::cityjson::attributes::Attributes;
use crate::resources::pool::ResourceRef;
use crate::resources::storage::StringStorage;

pub trait SemanticType: Default + fmt::Display + Clone {}

pub trait Semantic<RR: ResourceRef, SS: StringStorage, SemType: SemanticType> {
    /// Create a new semantic with the given type
    fn new(type_semantic: SemType) -> Self;
    /// Check if this semantic has any children
    fn has_children(&self) -> bool;
    /// Check if this semantic has a parent
    fn has_parent(&self) -> bool;
    /// Returns a reference to the children indices if they exist
    fn children(&self) -> Option<&Vec<RR>>;
    /// Returns a mutable reference to the children indices, creating default indices if they do not exist
    fn children_mut(&mut self) -> &mut Vec<RR>;
    /// Returns a reference to the parent index if it exists
    fn parent(&self) -> Option<&RR>;
    /// Returns a mutable reference to the parent index if it exists
    fn parent_mut(&mut self) -> Option<&mut RR>;
    /// Returns a reference to the attributes if they exist
    fn attributes(&self) -> Option<&Attributes<SS>>;
    /// Returns a mutable reference to the attributes, creating default attributes if they do not exist
    fn attributes_mut(&mut self) -> &mut Attributes<SS>;
}
