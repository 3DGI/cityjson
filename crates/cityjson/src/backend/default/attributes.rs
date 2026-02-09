//! # Attributes
//!
//! This module provides types and functionality for handling `CityJSON` object attributes.
//! It implements a flexible attribute system that can store various types of values,
//! supporting both owned and borrowed string storage strategies.
//!
//! ## Overview
//!
//! The attributes module contains these key components:
//!
//! - [`AttributeValue`]: The core enum representing different types of attribute values
//! - [`Attributes`]: A key-value container storing attribute values directly
//! - [`OwnedAttributes`]: Type alias for attributes with owned strings
//! - [`BorrowedAttributes`]: Type alias for attributes with borrowed strings
//!
//! ## Architecture: Array of Structures (`AoS`)
//!
//! Each object owns its attributes directly using a key-value map. Attributes are
//! stored inline rather than in a global pool, eliminating borrow checker conflicts
//! and simplifying the API.
//!
//! ## Storage Strategies
//!
//! The module supports two main string storage strategies:
//!
//! - Owned storage: Strings are owned by the attribute container (uses `String`)
//! - Borrowed storage: Strings are borrowed references (uses `&str`)
//!
//! This flexibility allows for efficient memory usage depending on the use case.
//!
//! ## Usage Examples
//!
//! ### Creating and using attributes
//!
//! ```rust
//! use cityjson::prelude::*;
//! use cityjson::cityjson::core::attributes::OwnedAttributeValue;
//!
//! // Create attribute values
//! let name = OwnedAttributeValue::String("Building A".to_string());
//! let height = OwnedAttributeValue::Float(25.5);
//!
//! // Store in attributes container
//! let mut attrs = cityjson::cityjson::core::attributes::OwnedAttributes::new();
//! attrs.insert("name".to_string(), name);
//! attrs.insert("height".to_string(), height);
//!
//! // Retrieve values
//! if let Some(height_val) = attrs.get("height") {
//!     println!("Building height: {}", height_val);
//! }
//! ```
//!
//! ## Compliance
//!
//! This module implements the attribute storage needed for `CityJSON` objects
//! as specified in the [CityJSON specification](https://www.cityjson.org/specs/).

use crate::resources::handles::GeometryRef;
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};
use std::collections::HashMap;
use std::fmt::Debug;

/// Represents the different types of values that can be stored in an attribute.
///
/// `AttributeValue` is a generic enum that can hold various types of data,
/// from simple scalars to complex nested structures like vectors and maps.
#[derive(Clone, Debug, PartialEq)]
pub enum AttributeValue<SS: StringStorage> {
    /// Represents a null or undefined value.
    Null,
    /// A boolean value (true or false).
    Bool(bool),
    /// An unsigned integer value.
    Unsigned(u64),
    /// A signed integer value.
    Integer(i64),
    /// A floating-point value.
    Float(f64),
    /// A string value using the specified storage strategy.
    String(SS::String),
    /// A vector of attribute values.
    Vec(Vec<Box<AttributeValue<SS>>>),
    /// A map of string keys to attribute values.
    Map(HashMap<SS::String, Box<AttributeValue<SS>>>),
    /// A geometry reference. Used for "address.location" which must be a `MultiPoint`.
    Geometry(GeometryRef),
}

impl<SS: StringStorage> std::fmt::Display for AttributeValue<SS>
where
    SS::String: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeValue::Null => write!(f, "null"),
            AttributeValue::Bool(value) => write!(f, "{value}"),
            AttributeValue::Unsigned(value) => write!(f, "{value}"),
            AttributeValue::Integer(value) => write!(f, "{value}"),
            AttributeValue::Float(value) => write!(f, "{value}"),
            AttributeValue::String(value) => write!(f, "\"{value}\""),
            AttributeValue::Vec(values) => {
                write!(f, "[")?;
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{value}")?;
                }
                write!(f, "]")
            }
            AttributeValue::Map(map) => {
                write!(f, "{{")?;
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{key}\": {value}")?;
                }
                write!(f, "}}")
            }
            AttributeValue::Geometry(value) => write!(f, "Geometry({value})"),
        }
    }
}

/// Type discriminator for attribute values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeValueType {
    /// Represents a null or undefined value.
    Null,
    /// A boolean value (true or false).
    Bool,
    /// An unsigned integer value.
    Unsigned,
    /// A signed integer value.
    Integer,
    /// A floating-point value.
    Float,
    /// A string value using the specified storage strategy.
    String,
    /// A vector of attribute values.
    Vec,
    /// A map of string keys to attribute values.
    Map,
    /// A geometry. Basically, only used for "address.location", which must be a `MultiPoint`.
    Geometry,
}

impl<SS: StringStorage> AttributeValue<SS> {
    /// Returns the type of this attribute value.
    pub fn value_type(&self) -> AttributeValueType {
        match self {
            AttributeValue::Null => AttributeValueType::Null,
            AttributeValue::Bool(_) => AttributeValueType::Bool,
            AttributeValue::Unsigned(_) => AttributeValueType::Unsigned,
            AttributeValue::Integer(_) => AttributeValueType::Integer,
            AttributeValue::Float(_) => AttributeValueType::Float,
            AttributeValue::String(_) => AttributeValueType::String,
            AttributeValue::Vec(_) => AttributeValueType::Vec,
            AttributeValue::Map(_) => AttributeValueType::Map,
            AttributeValue::Geometry(_) => AttributeValueType::Geometry,
        }
    }
}

/// Container for attributes using a specific storage strategy.
///
/// `Attributes` is a key-value store where keys are strings and values are
/// `AttributeValue` instances. Each object owns its attributes directly.
#[derive(Clone, Debug, PartialEq)]
pub struct Attributes<SS: StringStorage> {
    values: HashMap<SS::String, AttributeValue<SS>>,
}

impl<SS: StringStorage> Attributes<SS> {
    /// Creates a new, empty attributes container.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Retrieves a reference to the attribute value associated with the given key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&AttributeValue<SS>> {
        self.values.get(key)
    }

    /// Retrieves a mutable reference to the attribute value associated with the given key.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut AttributeValue<SS>> {
        self.values.get_mut(key)
    }

    /// Inserts an attribute value with the specified key.
    ///
    /// If the key already existed, returns the previous value.
    pub fn insert(
        &mut self,
        key: SS::String,
        value: AttributeValue<SS>,
    ) -> Option<AttributeValue<SS>> {
        self.values.insert(key, value)
    }

    /// Removes an attribute with the specified key.
    pub fn remove(&mut self, key: &str) -> Option<AttributeValue<SS>> {
        self.values.remove(key)
    }

    /// Returns the number of attributes in the container.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Checks if the attributes container is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns an iterator over the attributes' keys and values.
    pub fn iter(&self) -> impl Iterator<Item = (&SS::String, &AttributeValue<SS>)> {
        self.values.iter()
    }

    /// Returns a mutable iterator over the attributes' keys and values.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&SS::String, &mut AttributeValue<SS>)> {
        self.values.iter_mut()
    }

    /// Returns an iterator over the attribute keys.
    pub fn keys(&self) -> impl Iterator<Item = &SS::String> {
        self.values.keys()
    }

    /// Clears the attributes container.
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Checks if the attributes container contains a key.
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
}

impl<SS: StringStorage> Default for Attributes<SS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<SS: StringStorage> std::fmt::Display for Attributes<SS>
where
    SS::String: std::fmt::Display + Eq + std::hash::Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (i, (key, value)) in self.values.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "\"{key}\": {value}")?;
        }
        write!(f, "}}")
    }
}

/// Type alias for attribute values with owned strings.
pub type OwnedAttributeValue = AttributeValue<OwnedStringStorage>;

/// Type alias for attribute values with borrowed strings.
pub type BorrowedAttributeValue<'a> = AttributeValue<BorrowedStringStorage<'a>>;

/// Type alias for attributes container with owned strings.
pub type OwnedAttributes = Attributes<OwnedStringStorage>;

/// Type alias for attributes container with borrowed strings.
pub type BorrowedAttributes<'a> = Attributes<BorrowedStringStorage<'a>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_value_types() {
        let null_val: OwnedAttributeValue = AttributeValue::Null;
        assert_eq!(null_val.value_type(), AttributeValueType::Null);

        let bool_val: OwnedAttributeValue = AttributeValue::Bool(true);
        assert_eq!(bool_val.value_type(), AttributeValueType::Bool);

        let int_val: OwnedAttributeValue = AttributeValue::Integer(42);
        assert_eq!(int_val.value_type(), AttributeValueType::Integer);

        let float_val: OwnedAttributeValue = AttributeValue::Float(std::f64::consts::PI);
        assert_eq!(float_val.value_type(), AttributeValueType::Float);

        let string_val: OwnedAttributeValue = AttributeValue::String("test".to_string());
        assert_eq!(string_val.value_type(), AttributeValueType::String);

        let vec_val: OwnedAttributeValue = AttributeValue::Vec(vec![]);
        assert_eq!(vec_val.value_type(), AttributeValueType::Vec);

        let map_val: OwnedAttributeValue = AttributeValue::Map(HashMap::new());
        assert_eq!(map_val.value_type(), AttributeValueType::Map);
    }

    #[test]
    fn test_attributes_basic() {
        let mut attrs = OwnedAttributes::new();

        // Add different types of values
        attrs.insert("active".to_string(), AttributeValue::Bool(true));
        attrs.insert("floors".to_string(), AttributeValue::Integer(5));
        attrs.insert("height".to_string(), AttributeValue::Float(25.5));
        attrs.insert(
            "name".to_string(),
            AttributeValue::String("Building A".to_string()),
        );

        // Test retrieval
        assert_eq!(attrs.get("active"), Some(&AttributeValue::Bool(true)));
        assert_eq!(attrs.get("floors"), Some(&AttributeValue::Integer(5)));
        assert_eq!(attrs.get("height"), Some(&AttributeValue::Float(25.5)));
        assert_eq!(
            attrs.get("name"),
            Some(&AttributeValue::String("Building A".to_string()))
        );

        // Test type checking
        assert_eq!(
            attrs.get("active").map(|v| v.value_type()),
            Some(AttributeValueType::Bool)
        );
        assert_eq!(
            attrs.get("floors").map(|v| v.value_type()),
            Some(AttributeValueType::Integer)
        );
    }

    #[test]
    fn test_attributes_vectors() {
        let mut attrs = OwnedAttributes::new();

        // Create a vector value
        let vector_value = AttributeValue::Vec(vec![
            Box::new(AttributeValue::Integer(1)),
            Box::new(AttributeValue::Integer(2)),
            Box::new(AttributeValue::Integer(3)),
        ]);

        attrs.insert("numbers".to_string(), vector_value);

        // Retrieve and verify
        if let Some(AttributeValue::Vec(values)) = attrs.get("numbers") {
            assert_eq!(values.len(), 3);
            assert_eq!(*values[0], AttributeValue::Integer(1));
            assert_eq!(*values[1], AttributeValue::Integer(2));
            assert_eq!(*values[2], AttributeValue::Integer(3));
        } else {
            panic!("Expected Vec value");
        }
    }

    #[test]
    fn test_attributes_maps() {
        let mut attrs = OwnedAttributes::new();

        // Create a map value
        let mut map_content = HashMap::new();
        map_content.insert(
            "street".to_string(),
            Box::new(AttributeValue::String("Main St".to_string())),
        );
        map_content.insert("number".to_string(), Box::new(AttributeValue::Integer(123)));
        map_content.insert(
            "city".to_string(),
            Box::new(AttributeValue::String("Springfield".to_string())),
        );

        let map_value = AttributeValue::Map(map_content);
        attrs.insert("address".to_string(), map_value);

        // Retrieve and verify
        if let Some(AttributeValue::Map(map)) = attrs.get("address") {
            assert_eq!(map.len(), 3);
            assert_eq!(
                map.get("street"),
                Some(&Box::new(AttributeValue::String("Main St".to_string())))
            );
            assert_eq!(
                map.get("number"),
                Some(&Box::new(AttributeValue::Integer(123)))
            );
        } else {
            panic!("Expected Map value");
        }
    }

    #[test]
    fn test_attributes_remove() {
        let mut attrs = OwnedAttributes::new();

        attrs.insert("test".to_string(), AttributeValue::Integer(42));
        assert_eq!(attrs.len(), 1);

        let removed = attrs.remove("test");
        assert_eq!(removed, Some(AttributeValue::Integer(42)));
        assert_eq!(attrs.len(), 0);
    }

    #[test]
    fn test_attributes_contains_key() {
        let mut attrs = OwnedAttributes::new();

        attrs.insert(
            "name".to_string(),
            AttributeValue::String("Test".to_string()),
        );

        assert!(attrs.contains_key("name"));
        assert!(!attrs.contains_key("missing"));
    }

    #[test]
    fn test_attributes_iter() {
        let mut attrs = OwnedAttributes::new();

        attrs.insert("a".to_string(), AttributeValue::Integer(1));
        attrs.insert("b".to_string(), AttributeValue::Integer(2));
        attrs.insert("c".to_string(), AttributeValue::Integer(3));

        let mut keys: Vec<&str> = attrs.keys().map(|k| k.as_ref()).collect();
        keys.sort();

        assert_eq!(keys, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_attributes_display() {
        let mut attrs = OwnedAttributes::new();

        attrs.insert(
            "name".to_string(),
            AttributeValue::String("Building".to_string()),
        );
        attrs.insert("height".to_string(), AttributeValue::Float(25.5));

        let display_str = format!("{}", attrs);
        assert!(display_str.contains("\"name\""));
        assert!(display_str.contains("\"height\""));
    }

    #[test]
    fn test_attribute_value_display() {
        let values: Vec<(OwnedAttributeValue, &str)> = vec![
            (AttributeValue::Null, "null"),
            (AttributeValue::Bool(true), "true"),
            (AttributeValue::Integer(42), "42"),
            (
                AttributeValue::Float(std::f64::consts::PI),
                "3.141592653589793",
            ),
            (AttributeValue::String("test".to_string()), "\"test\""),
        ];

        for (val, expected) in values {
            let display_str = format!("{}", val);
            assert_eq!(display_str, expected);
        }
    }

    #[test]
    fn test_nested_structures() {
        let mut attrs = OwnedAttributes::new();

        // Create nested structure: address with coordinates
        let mut address_map = HashMap::new();
        address_map.insert(
            "street".to_string(),
            Box::new(AttributeValue::String("Broadway".to_string())),
        );

        // Create coordinates vector
        let coords_vec = AttributeValue::Vec(vec![
            Box::new(AttributeValue::Float(40.7128)),
            Box::new(AttributeValue::Float(-74.0060)),
        ]);

        address_map.insert("coordinates".to_string(), Box::new(coords_vec));

        let address = AttributeValue::Map(address_map);
        attrs.insert("address".to_string(), address);

        // Access nested values
        if let Some(AttributeValue::Map(address)) = attrs.get("address") {
            assert_eq!(address.len(), 2);

            // Get street
            if let Some(AttributeValue::String(street)) = address.get("street").map(|sb| &**sb) {
                assert_eq!(street, "Broadway");
            }

            // Get coordinates
            if let Some(coords_box) = address.get("coordinates")
                && let AttributeValue::Vec(coords) = &**coords_box
            {
                assert_eq!(coords.len(), 2);
            }
        }
    }
}
