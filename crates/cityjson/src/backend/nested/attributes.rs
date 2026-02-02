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
//! use cityjson::backend::nested::attributes::{AttributeValue, OwnedAttributes};
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
//! use cityjson::backend::nested::attributes::{AttributeValue, OwnedAttributes};
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
//! use cityjson::backend::nested::attributes::{AttributeValue, BorrowedAttributes};
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
use std::marker::PhantomData;

/// Represents the different types of values that can be stored in an attribute.
///
/// `AttributeValue` is a generic enum that can hold various types of data,
/// from simple scalars to complex nested structures like vectors and maps.
/// It uses a string storage strategy specified by the type parameter `SS`
/// and a resource reference type specified by the type parameter `RR`.
///
/// # Type Parameters
///
/// * `SS` - The string storage strategy to use (e.g., `OwnedStringStorage` or `BorrowedStringStorage`)
/// * `RR` - The resource reference type to use (unused in the nested backend)
///
/// # Examples
///
/// ```rust
/// use cityjson::backend::nested::attributes::AttributeValue;
/// use cityjson::resources::storage::OwnedStringStorage;
///
/// // Create different types of attribute values
/// let null_value = AttributeValue::<OwnedStringStorage, ()>::Null;
/// let bool_value = AttributeValue::<OwnedStringStorage, ()>::Bool(true);
/// let int_value = AttributeValue::<OwnedStringStorage, ()>::Integer(-42);
/// let float_value = AttributeValue::<OwnedStringStorage, ()>::Float(std::f64::consts::PI);
/// let string_value = AttributeValue::<OwnedStringStorage, ()>::String("example".to_string());
///
/// // Create a vector of values
/// let vec_value = AttributeValue::<OwnedStringStorage, ()>::Vec(vec![
///     Box::new(AttributeValue::Integer(1)),
///     Box::new(AttributeValue::Integer(2)),
///     Box::new(AttributeValue::Integer(3)),
/// ]);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum AttributeValue<SS: StringStorage, RR> {
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
    Vec(Vec<Box<AttributeValue<SS, RR>>>),
    /// A map of string keys to attribute values.
    Map(HashMap<SS::String, Box<AttributeValue<SS, RR>>>),
    /// A geometry value. Used for "address.location" which must be a MultiPoint.
    Geometry(Box<Geometry<SS, RR>>),
    #[doc(hidden)]
    __Marker(PhantomData<RR>),
}

impl<SS: StringStorage, RR> Display for AttributeValue<SS, RR>
where
    SS::String: Display,
    Geometry<SS, RR>: Display,
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
            AttributeValue::Geometry(value) => write!(f, "Geometry({})", value),
            AttributeValue::__Marker(_) => write!(f, "<marker>"),
        }
    }
}

/// Container for attributes using a specific storage strategy.
///
/// `Attributes` is a key-value store where keys are strings (using the specified storage
/// strategy) and values are `AttributeValue` instances. It provides methods to add,
/// retrieve, modify, and remove attributes.
///
/// # Type Parameters
///
/// * `SS` - The string storage strategy to use (e.g., `OwnedStringStorage` or `BorrowedStringStorage`)
/// * `RR` - The resource reference type to use (unused in the nested backend)
///
#[derive(Clone, Debug, PartialEq)]
pub struct Attributes<SS: StringStorage, RR> {
    values: HashMap<SS::String, AttributeValue<SS, RR>>,
    _marker: PhantomData<RR>,
}

impl<SS: StringStorage, RR> Attributes<SS, RR> {
    /// Creates a new, empty attributes container.
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            _marker: PhantomData,
        }
    }

    /// Retrieves a reference to the attribute value associated with the given key.
    pub fn get(&self, key: &str) -> Option<&AttributeValue<SS, RR>> {
        self.values.get(key)
    }

    /// Retrieves a mutable reference to the attribute value associated with the given key.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut AttributeValue<SS, RR>> {
        self.values.get_mut(key)
    }

    /// Inserts an attribute value with the specified key.
    ///
    /// If the key already existed, returns the previous value.
    pub fn insert(
        &mut self,
        key: SS::String,
        value: AttributeValue<SS, RR>,
    ) -> Option<AttributeValue<SS, RR>> {
        self.values.insert(key, value)
    }

    /// Removes an attribute with the specified key.
    pub fn remove(&mut self, key: &str) -> Option<AttributeValue<SS, RR>> {
        self.values.remove(key)
    }

    /// Returns the number of attributes in the container.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Checks if the attributes container is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns an iterator over the attributes' keys and values.
    pub fn iter(&self) -> impl Iterator<Item = (&SS::String, &AttributeValue<SS, RR>)> {
        self.values.iter()
    }

    /// Returns a mutable iterator over the attributes' keys and values.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&SS::String, &mut AttributeValue<SS, RR>)> {
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
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
}

impl<SS: StringStorage, RR> Default for Attributes<SS, RR> {
    fn default() -> Self {
        Self::new()
    }
}

impl<SS: StringStorage, RR> Display for Attributes<SS, RR>
where
    SS::String: Display + Eq + std::hash::Hash,
    Geometry<SS, RR>: Display,
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

/// Type alias for attribute values with owned strings and inline geometry.
pub type OwnedAttributeValue = AttributeValue<OwnedStringStorage, ()>;

/// Type alias for attribute values with borrowed strings and inline geometry.
pub type BorrowedAttributeValue<'a> = AttributeValue<BorrowedStringStorage<'a>, ()>;

/// Type alias for attributes container with owned strings and inline geometry.
pub type OwnedAttributes = Attributes<OwnedStringStorage, ()>;

/// Type alias for attributes container with borrowed strings and inline geometry.
pub type BorrowedAttributes<'a> = Attributes<BorrowedStringStorage<'a>, ()>;
