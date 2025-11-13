//! # Semantics
//!
//! This module provides types and functionality for handling semantics in CityJSON.
//! It implements the [Semantic object](https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives)
//! as specified in the CityJSON standard, allowing for semantic classification of geometric primitives.
//!
//! ## Overview
//!
//! The semantics module contains several key components:
//!
//! - [`Semantic`]: The main struct representing a semantic object with type information and relationships
//! - [`SemanticType`]: An enumeration of standardized semantic surface types
//! - [`SemanticTrait`] trait: A trait defining the interface for semantic objects
//! - [`SemanticTypeTrait`] trait: A marker trait for types that can be used as semantic types
//!
//! ## Key Features
//!
//! - Support for standard CityJSON semantic surface types (roof, wall, floor, etc.)
//! - Hierarchical relationships (parent-child) between semantic objects
//! - Extensibility through custom attributes
//! - Support for CityJSON extensions with custom semantic types
//!
//! ## Usage Examples
//!
//! ### Creating a semantic object
//!
//! ```rust
//! use cityjson::prelude::*;
//! use cityjson::v1_1::*;
//! use cityjson::cityjson::core::attributes::{AttributeOwnerType, OwnedAttributePool};
//!
//! // Create a semantic object for a roof surface
//! let mut roof = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
//!
//! // Add attributes if needed
//! let mut pool = OwnedAttributePool::new();
//! let material_id = pool.add_string(
//!     "material".to_string(),
//!     true,
//!     "slate".to_string(),
//!     AttributeOwnerType::Semantic,
//!     None,
//! );
//! let year_id = pool.add_integer(
//!     "year_constructed".to_string(),
//!     true,
//!     1985,
//!     AttributeOwnerType::Semantic,
//!     None,
//! );
//! let mut attrs = roof.attributes_mut();
//! attrs.insert("material".to_string(), material_id);
//! attrs.insert("year_constructed".to_string(), year_id);
//! ```
//!
//! ### Working with semantic hierarchies
//!
//! ```rust
//! use cityjson::prelude::*;
//! use cityjson::v1_1::*;
//!
//! // Create a parent semantic (building)
//! let parent_id = 1; // Would typically come from a ResourcePool
//!
//! // Create a child semantic (wall)
//! let mut wall = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);
//!
//! // Set parent relationship
//! *wall.children_mut() = vec![ResourceId32::new(2, 0), ResourceId32::new(3, 0), ResourceId32::new(4, 0)]; // Child surface IDs
//! ```
//!
//! ### Using custom semantic types (extensions)
//!
//! ```rust
//! use cityjson::prelude::*;
//! use cityjson::v1_1::*;
//!
//! // Create a semantic with a custom type from an extension
//! let custom_type = SemanticType::Extension("SolarPanel".to_string());
//! let solar_panel = Semantic::<ResourceId32, OwnedStringStorage>::new(custom_type);
//! ```
//!
//! ## Compliance
//!
//! All types in this module are designed to comply with the
//! [CityJSON 1.1.3 specification](https://www.cityjson.org/specs/1.1.3/) and later versions.
//! The module implements all standard semantic surface types defined in the specification.

use crate::cityjson::core::attributes::Attributes;
use crate::cityjson::traits::semantic::SemanticTypeTrait;
use crate::format_option;
use crate::macros::impl_semantic_trait;
use crate::resources::pool::ResourceRef;
use crate::resources::storage::StringStorage;
use std::fmt::{Display, Formatter};

/// Represents a semantic surface in CityJSON.
///
/// Semantic surfaces provide meaning to geometric objects by classifying them according
/// to their real-world function or purpose. Each semantic surface has a type (e.g., roof,
/// wall, floor), can have hierarchical relationships with other semantics, and can carry
/// additional attributes.
///
/// # Type Parameters
///
/// * `RR`: Resource reference type for referring to other semantics
/// * `SS`: String storage type for attributes and extension types
///
/// # Examples
///
/// ```rust
/// use cityjson::prelude::*;
/// use cityjson::v1_1::*;
/// use cityjson::cityjson::core::attributes::{AttributeOwnerType, OwnedAttributePool};
///
/// // Create a new wall surface semantic
/// let mut wall = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);
///
/// // Add attributes
/// let mut pool = OwnedAttributePool::new();
/// let material_id = pool.add_string(
///     "material".to_string(),
///     true,
///     "brick".to_string(),
///     AttributeOwnerType::Semantic,
///     None,
/// );
/// let insulated_id = pool.add_bool(
///     "insulated".to_string(),
///     true,
///     true,
///     AttributeOwnerType::Semantic,
///     None,
/// );
/// let mut attrs = wall.attributes_mut();
/// attrs.insert("material".to_string(), material_id);
/// attrs.insert("insulated".to_string(), insulated_id);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Semantic<RR: ResourceRef, SS: StringStorage> {
    /// The type of the semantic surface
    type_semantic: SemanticType<SS>,
    /// Indices to child semantics in the global semantics pool
    children: Option<Vec<RR>>,
    /// Index to parent semantic in the global semantics pool
    parent: Option<RR>,
    /// Additional attributes of the semantic surface
    attributes: Option<Attributes<SS>>,
}

impl_semantic_trait!(SemanticType<SS>);

impl<RR: ResourceRef, SS: StringStorage> Display for Semantic<RR, SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type: {}, children: {:?}, parent: {:?}, attributes: {}",
            self.type_semantic,
            self.children,
            self.parent,
            format_option(&self.attributes)
        )
    }
}

/// Semantic surface type.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives>.
#[derive(Debug, Default, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum SemanticType<SS: StringStorage> {
    #[default]
    Default,
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
    Extension(SS::String),
}

impl<SS: StringStorage> Display for SemanticType<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<SS: StringStorage> SemanticTypeTrait for SemanticType<SS> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cityjson::core::attributes::{AttributeOwnerType, OwnedAttributePool};
    use crate::resources::pool::ResourceId32;
    use crate::resources::storage::OwnedStringStorage;

    #[test]
    fn test_semantic_creation() {
        let semantic = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
        assert!(!semantic.has_children());
        assert!(!semantic.has_parent());
        assert!(semantic.children().is_none());
        assert!(semantic.parent().is_none());
        assert!(semantic.attributes().is_none());
    }

    #[test]
    fn test_semantic_attributes() {
        let mut semantic =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);

        // Initially no attributes
        assert!(semantic.attributes().is_none());

        // Create attribute pool
        let mut pool = OwnedAttributePool::new();

        // Get mutable reference and add attributes
        let attrs = semantic.attributes_mut();
        let material_id = pool.add_string(
            "material".to_string(),
            true,
            "brick".to_string(),
            AttributeOwnerType::Semantic,
            None,
        );
        let color_id = pool.add_string(
            "color".to_string(),
            true,
            "red".to_string(),
            AttributeOwnerType::Semantic,
            None,
        );
        attrs.insert("material".to_string(), material_id);
        attrs.insert("color".to_string(), color_id);

        // Now attributes should exist
        assert!(semantic.attributes().is_some());
        let retrieved_material_id = semantic.attributes().unwrap().get("material");
        assert!(retrieved_material_id.is_some());
        assert_eq!(
            pool.get_string(retrieved_material_id.unwrap()),
            Some(&"brick".to_string())
        );
        let retrieved_color_id = semantic.attributes().unwrap().get("color");
        assert!(retrieved_color_id.is_some());
        assert_eq!(
            pool.get_string(retrieved_color_id.unwrap()),
            Some(&"red".to_string())
        );
    }

    #[test]
    fn test_semantic_children() {
        let mut semantic =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);

        // Initially no children
        assert!(!semantic.has_children());

        // Add children
        let children = semantic.children_mut();
        children.push(ResourceId32::new(1, 0));
        children.push(ResourceId32::new(2, 0));

        // Now should have children
        assert!(semantic.has_children());
        assert_eq!(semantic.children().unwrap().len(), 2);
        assert_eq!(semantic.children().unwrap()[0], ResourceId32::new(1, 0));
        assert_eq!(semantic.children().unwrap()[1], ResourceId32::new(2, 0));
    }

    #[test]
    fn test_semantic_parent() {
        let mut semantic = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::Window);

        // Initially no parent
        assert!(!semantic.has_parent());
        assert!(semantic.parent().is_none());

        // Set parent manually
        semantic.parent = Some(ResourceId32::new(5, 0));

        // Now should have parent
        assert!(semantic.has_parent());
        assert_eq!(*semantic.parent().unwrap(), ResourceId32::new(5, 0));

        semantic.set_parent(ResourceId32::new(10, 0));
        assert_eq!(*semantic.parent().unwrap(), ResourceId32::new(10, 0));
    }

    #[test]
    fn test_semantic_display() {
        let mut semantic =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
        let display_str = format!("{}", semantic);
        assert!(display_str.contains("RoofSurface"));

        // Create attribute pool
        let mut pool = OwnedAttributePool::new();

        // Add attributes and check display again
        let attrs = semantic.attributes_mut();
        let material_id = pool.add_string(
            "material".to_string(),
            true,
            "tile".to_string(),
            AttributeOwnerType::Semantic,
            None,
        );
        attrs.insert("material".to_string(), material_id);

        let display_str = format!("{}", semantic);
        assert!(display_str.contains("RoofSurface"));
        assert!(display_str.contains("attributes"));
        println!("{}", semantic);
    }

    #[test]
    fn test_semantic_type_extension() {
        let extension_type = SemanticType::Extension("CustomType".to_string());
        let semantic = Semantic::<ResourceId32, OwnedStringStorage>::new(extension_type);
        let display_str = format!("{}", semantic);
        assert!(display_str.contains("Extension"));
    }

    #[test]
    fn test_semantic_equality() {
        // Test 1: Two semantics with same type and no other fields are equal
        let semantic1 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
        let semantic2 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
        assert_eq!(semantic1, semantic2);

        // Test 2: Two semantics with different types are not equal
        let semantic3 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);
        assert_ne!(semantic1, semantic3);

        // Test 3: Two semantics with same type and same children are equal
        let mut semantic4 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);
        let mut semantic5 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);

        semantic4.children_mut().push(ResourceId32::new(1, 0));
        semantic4.children_mut().push(ResourceId32::new(2, 0));
        semantic5.children_mut().push(ResourceId32::new(1, 0));
        semantic5.children_mut().push(ResourceId32::new(2, 0));
        assert_eq!(semantic4, semantic5);

        // Test 4: Two semantics with different children are not equal
        let mut semantic6 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);
        semantic6.children_mut().push(ResourceId32::new(3, 0));
        assert_ne!(semantic4, semantic6);

        // Test 5: Two semantics with same parent are equal
        let mut semantic7 = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::Window);
        let mut semantic8 = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::Window);
        semantic7.set_parent(ResourceId32::new(10, 0));
        semantic8.set_parent(ResourceId32::new(10, 0));
        assert_eq!(semantic7, semantic8);

        // Test 6: Two semantics with different parents are not equal
        let mut semantic9 = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::Window);
        semantic9.set_parent(ResourceId32::new(20, 0));
        assert_ne!(semantic7, semantic9);

        // Test 7: Two semantics with same attributes are equal
        let mut semantic10 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
        let mut semantic11 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);

        // Create attribute pool for test 7, 8, and 9
        let mut pool = OwnedAttributePool::new();
        let material_id = pool.add_string(
            "material".to_string(),
            true,
            "tile".to_string(),
            AttributeOwnerType::Semantic,
            None,
        );
        let year_id = pool.add_integer(
            "year".to_string(),
            true,
            2020,
            AttributeOwnerType::Semantic,
            None,
        );

        // Use the same attribute IDs for both semantics to make them equal
        semantic10
            .attributes_mut()
            .insert("material".to_string(), material_id);
        semantic10
            .attributes_mut()
            .insert("year".to_string(), year_id);

        semantic11
            .attributes_mut()
            .insert("material".to_string(), material_id);
        semantic11
            .attributes_mut()
            .insert("year".to_string(), year_id);
        assert_eq!(semantic10, semantic11);

        // Test 8: Two semantics with different attributes are not equal
        let mut semantic12 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
        let material_id12 = pool.add_string(
            "material".to_string(),
            true,
            "slate".to_string(),
            AttributeOwnerType::Semantic,
            None,
        );
        semantic12
            .attributes_mut()
            .insert("material".to_string(), material_id12);
        assert_ne!(semantic10, semantic12);

        // Test 9: Two semantics with all fields equal are equal
        let mut semantic13 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);
        let mut semantic14 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);

        let color_id = pool.add_string(
            "color".to_string(),
            true,
            "blue".to_string(),
            AttributeOwnerType::Semantic,
            None,
        );

        // Use the same attribute ID for both semantics to make them equal
        semantic13.children_mut().push(ResourceId32::new(1, 0));
        semantic13.set_parent(ResourceId32::new(5, 0));
        semantic13
            .attributes_mut()
            .insert("color".to_string(), color_id);

        semantic14.children_mut().push(ResourceId32::new(1, 0));
        semantic14.set_parent(ResourceId32::new(5, 0));
        semantic14
            .attributes_mut()
            .insert("color".to_string(), color_id);
        assert_eq!(semantic13, semantic14);
    }
}
