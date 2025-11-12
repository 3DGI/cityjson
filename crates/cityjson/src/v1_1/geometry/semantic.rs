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
//!
//! // Create a semantic object for a roof surface
//! let mut roof = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
//!
//! // Add attributes if needed
//! let mut attrs = roof.attributes_mut();
//! attrs.insert("material".to_string(), AttributeValue::String("slate".to_string()));
//! attrs.insert("year_constructed".to_string(), AttributeValue::Integer(1985));
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
use crate::cityjson::traits::semantic::{SemanticTrait, SemanticTypeTrait};
use crate::format_option;
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
///
/// // Create a new wall surface semantic
/// let mut wall = Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);
///
/// // Add attributes
/// let mut attrs = wall.attributes_mut();
/// attrs.insert("material".to_string(), AttributeValue::String("brick".to_string()));
/// attrs.insert("insulated".to_string(), AttributeValue::Bool(true));
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
    attributes: Option<Attributes<SS, RR>>,
}

impl<RR: ResourceRef, SS: StringStorage> SemanticTrait<RR, SS, SemanticType<SS>>
    for Semantic<RR, SS>
{
    #[inline]
    fn new(type_semantic: SemanticType<SS>) -> Self {
        Self {
            type_semantic,
            children: None,
            parent: None,
            attributes: None,
        }
    }
    #[inline]
    fn type_semantic(&self) -> &SemanticType<SS> {
        &self.type_semantic
    }
    #[inline]
    fn has_children(&self) -> bool {
        self.children.as_ref().is_some_and(|c| !c.is_empty())
    }
    #[inline]
    fn has_parent(&self) -> bool {
        self.parent.is_some()
    }
    #[inline]
    fn children(&self) -> Option<&Vec<RR>> {
        self.children.as_ref()
    }
    #[inline]
    fn children_mut(&mut self) -> &mut Vec<RR> {
        if self.children.is_none() {
            self.children = Some(Vec::new());
        }
        self.children.as_mut().unwrap()
    }
    #[inline]
    fn parent(&self) -> Option<&RR> {
        self.parent.as_ref()
    }
    #[inline]
    fn set_parent(&mut self, parent_ref: RR) {
        self.parent = Some(parent_ref);
    }
    #[inline]
    fn attributes(&self) -> Option<&Attributes<SS, RR>> {
        self.attributes.as_ref()
    }
    #[inline]
    fn attributes_mut(&mut self) -> &mut Attributes<SS, RR> {
        if self.attributes.is_none() {
            self.attributes = Some(Attributes::new());
        }
        self.attributes.as_mut().unwrap()
    }
}

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
    use crate::cityjson::core::attributes::AttributeValue;
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

        // Get mutable reference and add attributes
        let attrs = semantic.attributes_mut();
        attrs.insert(
            "material".to_string(),
            AttributeValue::String("brick".to_string()),
        );
        attrs.insert(
            "color".to_string(),
            AttributeValue::String("red".to_string()),
        );

        // Now attributes should exist
        assert!(semantic.attributes().is_some());
        match semantic.attributes().unwrap().get("material") {
            Some(AttributeValue::String(v)) => assert_eq!(v, "brick"),
            _ => panic!("Expected string value"),
        }
        match semantic.attributes().unwrap().get("color") {
            Some(AttributeValue::String(v)) => assert_eq!(v, "red"),
            _ => panic!("Expected string value"),
        }
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

        // Add attributes and check display again
        let attrs = semantic.attributes_mut();
        attrs.insert(
            "material".to_string(),
            AttributeValue::String("tile".to_string()),
        );

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

        semantic10.attributes_mut().insert(
            "material".to_string(),
            AttributeValue::String("tile".to_string()),
        );
        semantic10
            .attributes_mut()
            .insert("year".to_string(), AttributeValue::Integer(2020));

        semantic11.attributes_mut().insert(
            "material".to_string(),
            AttributeValue::String("tile".to_string()),
        );
        semantic11
            .attributes_mut()
            .insert("year".to_string(), AttributeValue::Integer(2020));
        assert_eq!(semantic10, semantic11);

        // Test 8: Two semantics with different attributes are not equal
        let mut semantic12 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::RoofSurface);
        semantic12.attributes_mut().insert(
            "material".to_string(),
            AttributeValue::String("slate".to_string()),
        );
        assert_ne!(semantic10, semantic12);

        // Test 9: Two semantics with all fields equal are equal
        let mut semantic13 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);
        let mut semantic14 =
            Semantic::<ResourceId32, OwnedStringStorage>::new(SemanticType::WallSurface);

        semantic13.children_mut().push(ResourceId32::new(1, 0));
        semantic13.set_parent(ResourceId32::new(5, 0));
        semantic13.attributes_mut().insert(
            "color".to_string(),
            AttributeValue::String("blue".to_string()),
        );

        semantic14.children_mut().push(ResourceId32::new(1, 0));
        semantic14.set_parent(ResourceId32::new(5, 0));
        semantic14.attributes_mut().insert(
            "color".to_string(),
            AttributeValue::String("blue".to_string()),
        );
        assert_eq!(semantic13, semantic14);
    }
}
