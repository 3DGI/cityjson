//! # Attributes
//!
//! Attribute storage for `CityJSON` objects.
//!
//! ## Architecture: Array of Structures (`AoS`)
//!
//! Each object owns its attributes directly as a key-value map, rather than
//! referencing a global pool. This avoids the ownership issues that arise when
//! attributes are pervasive across the data model (unlike geometries, which are
//! scoped and pool-managed).
//!
//! ```rust
//! use cityjson::v2_0::{OwnedAttributeValue, OwnedAttributes};
//!
//! let mut attrs = OwnedAttributes::new();
//! attrs.insert("name".to_string(), OwnedAttributeValue::String("Building A".to_string()));
//! attrs.insert("height".to_string(), OwnedAttributeValue::Float(25.5));
//! ```

use crate::resources::handles::GeometryHandle;
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};
use std::collections::HashMap;
use std::fmt::Debug;

/// Attribute value types for `CityJSON` objects.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum AttributeValue<SS: StringStorage> {
    Null,
    Bool(bool),
    Unsigned(u64),
    Integer(i64),
    Float(f64),
    String(SS::String),
    Vec(Vec<Box<AttributeValue<SS>>>),
    Map(HashMap<SS::String, Box<AttributeValue<SS>>>),
    /// Geometry reference. Used for `address.location`, which must be a `MultiPoint`.
    Geometry(GeometryHandle),
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

    /// Returns an iterator over the attribute values.
    pub fn values(&self) -> impl Iterator<Item = &AttributeValue<SS>> {
        self.values.values()
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
