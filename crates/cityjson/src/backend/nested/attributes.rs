//! # Attributes
//!
//! This module provides types and functionality for handling CityJSON object attributes.
//! It implements a flexible attribute system that can store various types of values,
//! supporting both owned and borrowed string storage strategies.
//!
//! ## Overview
//!
//! The attributes module contains these key components:
//!
//! - [`Attributes`]: The main container for storing attribute key-value pairs
//! - [`AttributeValue`]: An enum representing various types of values that attributes can hold
//! - [`OwnedAttributes`]: Type alias for attributes with owned strings
//! - [`BorrowedAttributes`]: Type alias for attributes with borrowed strings
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
//! ### Creating and using owned attributes
//!
//! ```rust
//! use cityjson::prelude::*;
//!
//! // Create a new attributes container
//! let mut attrs = OwnedAttributes::new();
//!
//! // Insert various types of values
//! attrs.insert("name".to_string(), AttributeValue::String("Building A".to_string()));
//! attrs.insert("height".to_string(), AttributeValue::Float(25.5));
//! attrs.insert("floors".to_string(), AttributeValue::Integer(5));
//! attrs.insert("is_residential".to_string(), AttributeValue::Bool(true));
//!
//! // Retrieve values
//! if let Some(AttributeValue::Float(height)) = attrs.get("height") {
//!     println!("Building height: {} meters", height);
//!     assert_eq!(*height, 25.5);
//! }
//!
//! // Modify values
//! if let Some(AttributeValue::Integer(floors)) = attrs.get_mut("floors") {
//!     *floors = 6;
//! }
//!
//! // Remove an attribute
//! let removed = attrs.remove("is_residential");
//! assert!(matches!(removed, Some(AttributeValue::Bool(true))));
//! ```
//!
//! ### Working with nested attributes
//!
//! ```rust
//! use cityjson::prelude::*;
//! use std::collections::HashMap;
//!
//! let mut attrs = OwnedAttributes::new();
//!
//! // Create a nested map structure
//! let mut address = HashMap::new();
//! address.insert("street".to_string(), Box::new(AttributeValue::String("Main St".to_string())));
//! address.insert("number".to_string(), Box::new(AttributeValue::Integer(123)));
//!
//! // Insert the nested map
//! attrs.insert("address".to_string(), AttributeValue::Map(address));
//!
//! // Create a vector of values
//! let materials = vec![
//!     Box::new(AttributeValue::String("concrete".to_string())),
//!     Box::new(AttributeValue::String("glass".to_string())),
//!     Box::new(AttributeValue::String("steel".to_string())),
//! ];
//!
//! // Insert the vector
//! attrs.insert("materials".to_string(), AttributeValue::Vec(materials));
//!
//! // Access nested values
//! if let Some(AttributeValue::Map(address_map)) = attrs.get("address") {
//!     if let Some(street_box) = address_map.get("street") {
//!         if let AttributeValue::String(street) = &**street_box {
//!             assert_eq!(street, "Main St");
//!         }
//!     }
//! }
//! ```
//!
//! ### Using borrowed attributes
//!
//! ```rust
//! use cityjson::prelude::*;
//!
//! // Static strings for demonstration
//! let name = "Building B";
//! let type_str = "commercial";
//!
//! // Create borrowed attributes
//! let mut attrs = BorrowedAttributes::new();
//! attrs.insert("name", AttributeValue::String(name));
//! attrs.insert("type", AttributeValue::String(type_str));
//!
//! // Retrieve values (note that we compare with references)
//! if let Some(AttributeValue::String(building_name)) = attrs.get("name") {
//!     assert_eq!(*building_name, "Building B");
//! }
//! ```
//!
//! ## Compliance
//!
//! This module implements the attribute storage needed for CityJSON objects
//! as specified in the [CityJSON specification](https://www.cityjson.org/specs/).
//! The flexible design allows for efficiently representing both simple and complex
//! attribute structures.

use crate::backend::nested::geometry::Geometry;
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};
use std::collections::HashMap;
use std::fmt::{self, Debug, Display, Formatter};

/// Represents the different types of values that can be stored in an attribute.
///
/// `AttributeValue` is a generic enum that can hold various types of data,
/// from simple scalars to complex nested structures like vectors and maps.
/// It uses a string storage strategy specified by the type parameter `SS`.
///
/// # Type Parameter
///
/// * `SS` - The string storage strategy to use (e.g., `OwnedStringStorage` or `BorrowedStringStorage`)
///
/// # Examples
///
/// ```rust
/// use cityjson::prelude::*;
///
/// // Create different types of attribute values
/// let null_value = AttributeValue::<OwnedStringStorage, ResourceId32>::Null;
/// let bool_value = AttributeValue::<OwnedStringStorage, ResourceId32>::Bool(true);
/// let int_value = AttributeValue::<OwnedStringStorage, ResourceId32>::Integer(-42);
/// let float_value = AttributeValue::<OwnedStringStorage, ResourceId32>::Float(std::f64::consts::PI);
/// let string_value = AttributeValue::<OwnedStringStorage, ResourceId32>::String("example".to_string());
///
/// // Create a vector of values
/// let vec_value = AttributeValue::<OwnedStringStorage, ResourceId32>::Vec(vec![
///     Box::new(AttributeValue::Integer(1)),
///     Box::new(AttributeValue::Integer(2)),
///     Box::new(AttributeValue::Integer(3)),
/// ]);
/// ```
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
    /// A geometry. Basically, only used for "address.location", which must be a MultiPoint.
    Geometry(Box<Geometry<SS>>),
}

impl<SS: StringStorage> Display for AttributeValue<SS>
where
    SS::String: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AttributeValue::Null => write!(f, "null"),
            AttributeValue::Bool(value) => write!(f, "{}", value),
            AttributeValue::Unsigned(value) => write!(f, "{}", value),
            AttributeValue::Integer(value) => write!(f, "{}", value),
            AttributeValue::Float(value) => write!(f, "{}", value),
            AttributeValue::String(value) => write!(f, "\"{}\"", value),
            AttributeValue::Vec(values) => {
                write!(f, "[")?;
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                }
                write!(f, "]")
            }
            AttributeValue::Map(map) => {
                write!(f, "{{")?;
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            }
            AttributeValue::Geometry(value) => write!(f, "Geometry {}", value),
        }
    }
}

/// Container for attributes using a specific storage strategy.
///
/// `Attributes` is a key-value store where keys are strings (using the specified storage
/// strategy) and values are `AttributeValue` instances. It provides methods to add,
/// retrieve, modify, and remove attributes.
///
/// # Type Parameter
///
/// * `SS` - The string storage strategy to use (e.g., `OwnedStringStorage` or `BorrowedStringStorage`)
///
#[derive(Clone, Debug, PartialEq)]
pub struct Attributes<SS: StringStorage> {
    values: HashMap<SS::String, AttributeValue<SS>>,
}

impl<SS: StringStorage> Attributes<SS> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&AttributeValue<SS>> {
        self.values.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut AttributeValue<SS>> {
        self.values.get_mut(key)
    }

    pub fn insert(
        &mut self,
        key: SS::String,
        value: AttributeValue<SS>,
    ) -> Option<AttributeValue<SS>> {
        self.values.insert(key, value)
    }

    pub fn remove(&mut self, key: &str) -> Option<AttributeValue<SS>> {
        self.values.remove(key)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&SS::String, &AttributeValue<SS>)> {
        self.values.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&SS::String, &mut AttributeValue<SS>)> {
        self.values.iter_mut()
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
}

impl<SS: StringStorage> Default for Attributes<SS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<SS: StringStorage> Display for Attributes<SS>
where
    SS::String: Display + Eq + std::hash::Hash,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (i, (key, value)) in self.values.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "\"{}\": {}", key, value)?;
        }
        write!(f, "}}")
    }
}

pub type OwnedAttributes = Attributes<OwnedStringStorage>;

pub type BorrowedAttributes<'a> = Attributes<BorrowedStringStorage<'a>>;
