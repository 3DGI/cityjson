use crate::cityjson::core::attributes::Attributes;
use crate::resources::pool::ResourceRef;
use crate::resources::storage::StringStorage;
use std::fmt;

pub trait SemanticTypeTrait: Default + fmt::Display + Clone {}

/// Defines the interface for semantic objects in CityJSON.
///
/// This trait provides methods for accessing and manipulating semantic information,
/// including the type, parent-child relationships, and attributes.
///
/// # Type Parameters
///
/// * `RR`: Resource reference type used for parent and children indices
/// * `SS`: String storage type used for attributes
/// * `SemType`: Type implementing the `SemanticType` trait
///
/// # Examples
///
/// ```rust
/// use cityjson::cityjson::traits::semantic::{SemanticTrait, SemanticTypeTrait};
/// use cityjson::cityjson::core::attributes::Attributes;
/// use cityjson::resources::pool::ResourceRef;
/// use cityjson::resources::storage::StringStorage;
///
/// // Define a semantic implementation
/// struct MySemantic<RR: ResourceRef, SS: StringStorage, ST: SemanticTypeTrait> {
///     type_semantic: ST,
///     children: Option<Vec<RR>>,
///     parent: Option<RR>,
///     attributes: Option<Attributes<SS, RR>>,
/// }
///
/// // impl<RR: ResourceRef, SS: StringStorage, ST: SemanticTypeTrait>
/// //    SemanticTrait<RR, SS, ST> for MySemantic<RR, SS, ST> {
/// //    // Implementation of the trait methods
/// //    // ...
/// // }
/// ```
pub trait SemanticTrait<RR: ResourceRef, SS: StringStorage, SemType: SemanticTypeTrait> {
    /// Creates a new semantic with the given type.
    ///
    /// # Parameters
    ///
    /// * `type_semantic` - The semantic surface type
    ///
    /// # Returns
    ///
    /// A new semantic object with the specified type
    fn new(type_semantic: SemType) -> Self;

    /// Returns a reference to the semantic type.
    ///
    /// # Returns
    ///
    /// A reference to the semantic type
    fn type_semantic(&self) -> &SemType;

    /// Checks if this semantic has any children.
    ///
    /// # Returns
    ///
    /// `true` if the semantic has at least one child, `false` otherwise
    fn has_children(&self) -> bool;

    /// Checks if this semantic has a parent.
    ///
    /// # Returns
    ///
    /// `true` if the semantic has a parent reference, `false` otherwise
    fn has_parent(&self) -> bool;

    /// Returns a reference to the children indices if they exist.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to a vector of child indices,
    /// or `None` if no children exist
    fn children(&self) -> Option<&Vec<RR>>;

    /// Returns a mutable reference to the children indices.
    ///
    /// If no children exist yet, a new empty vector should be created.
    ///
    /// # Returns
    ///
    /// A mutable reference to the vector of child indices
    fn children_mut(&mut self) -> &mut Vec<RR>;

    /// Returns a reference to the parent index if it exists.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the parent index,
    /// or `None` if no parent exists
    fn parent(&self) -> Option<&RR>;

    /// Returns a mutable reference to the parent index if it exists.
    ///
    /// # Returns
    ///
    /// An `Option` containing a mutable reference to the parent index,
    /// or `None` if no parent exists
    fn parent_mut(&mut self) -> Option<&mut RR>;

    /// Returns a reference to the attributes if they exist.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the attributes,
    /// or `None` if no attributes exist
    fn attributes(&self) -> Option<&Attributes<SS, RR>>;

    /// Returns a mutable reference to the attributes.
    ///
    /// If no attributes exist yet, a new empty attributes object should be created.
    ///
    /// # Returns
    ///
    /// A mutable reference to the attributes
    fn attributes_mut(&mut self) -> &mut Attributes<SS, RR>;
}
