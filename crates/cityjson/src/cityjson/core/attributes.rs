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

use crate::prelude::{ResourceId32, ResourceRef};
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
pub enum AttributeValue<SS: StringStorage, RR: ResourceRef> {
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
    /// A geometry. Basically, only used for "address.location", which must be a MultiPoint.
    Geometry(RR),
}

impl<SS: StringStorage, RR: ResourceRef> Display for AttributeValue<SS, RR>
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
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use cityjson::prelude::*;
///
/// // Create a new attributes container
/// let mut attrs = Attributes::<OwnedStringStorage, ResourceId32>::new();
///
/// // Insert a value
/// attrs.insert(
///     "height".to_string(),
///     AttributeValue::Float(42.5)
/// );
///
/// // Retrieve the value
/// if let Some(AttributeValue::Float(height)) = attrs.get("height") {
///     println!("Height: {} meters", height);
///     assert_eq!(*height, 42.5);
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Attributes<SS: StringStorage, RR: ResourceRef> {
    values: HashMap<SS::String, AttributeValue<SS, RR>>,
}

impl<SS: StringStorage, RR: ResourceRef> Attributes<SS, RR> {
    /// Creates a new, empty attributes container.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let attrs = Attributes::<OwnedStringStorage, ResourceId32>::new();
    /// assert!(attrs.get("any_key").is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Retrieves a reference to the attribute value associated with the given key.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the attribute value if the key exists,
    /// or `None` if the key doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut attrs = Attributes::<OwnedStringStorage, ResourceId32>::new();
    /// attrs.insert("temperature".to_string(), AttributeValue::Float(22.5));
    ///
    /// if let Some(AttributeValue::Float(temp)) = attrs.get("temperature") {
    ///     println!("Current temperature: {}", temp);
    ///     assert_eq!(*temp, 22.5);
    /// }
    ///
    /// // Key doesn't exist
    /// assert!(attrs.get("humidity").is_none());
    /// ```
    pub fn get(&self, key: &str) -> Option<&AttributeValue<SS, RR>> {
        self.values.get(key)
    }

    /// Retrieves a mutable reference to the attribute value associated with the given key.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// An `Option` containing a mutable reference to the attribute value if the key exists,
    /// or `None` if the key doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut attrs = Attributes::<OwnedStringStorage, ResourceId32>::new();
    /// attrs.insert("counter".to_string(), AttributeValue::Integer(10));
    ///
    /// // Modify the value
    /// if let Some(AttributeValue::Integer(counter)) = attrs.get_mut("counter") {
    ///     *counter += 1;
    /// }
    ///
    /// // Verify the modification
    /// if let Some(AttributeValue::Integer(counter)) = attrs.get("counter") {
    ///     assert_eq!(*counter, 11);
    /// }
    /// ```
    pub fn get_mut(&mut self, key: &str) -> Option<&mut AttributeValue<SS, RR>> {
        self.values.get_mut(key)
    }

    /// Inserts an attribute value with the specified key.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to associate with the value
    /// * `value` - The attribute value to insert
    ///
    /// # Returns
    ///
    /// If the key already existed, returns the previous value. Otherwise, returns `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut attrs = Attributes::<OwnedStringStorage, ResourceId32>::new();
    ///
    /// // Insert a new value
    /// let previous = attrs.insert(
    ///     "status".to_string(),
    ///     AttributeValue::String("active".to_string())
    /// );
    /// assert!(previous.is_none());
    ///
    /// // Replace an existing value
    /// let previous = attrs.insert(
    ///     "status".to_string(),
    ///     AttributeValue::String("inactive".to_string())
    /// );
    /// assert!(matches!(previous, Some(AttributeValue::String(s)) if s == "active"));
    /// ```
    pub fn insert(
        &mut self,
        key: SS::String,
        value: AttributeValue<SS, RR>,
    ) -> Option<AttributeValue<SS, RR>> {
        self.values.insert(key, value)
    }

    /// Removes an attribute with the specified key.
    ///
    /// # Parameters
    ///
    /// * `key` - The key of the attribute to remove
    ///
    /// # Returns
    ///
    /// The removed attribute value if the key existed, or `None` if the key didn't exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut attrs = Attributes::<OwnedStringStorage, ResourceId32>::new();
    /// attrs.insert("temporary".to_string(), AttributeValue::Bool(true));
    ///
    /// // Remove the attribute
    /// let removed = attrs.remove("temporary");
    /// assert!(matches!(removed, Some(AttributeValue::Bool(true))));
    ///
    /// // The key no longer exists
    /// assert!(attrs.get("temporary").is_none());
    ///
    /// // Removing a non-existent key returns None
    /// let removed = attrs.remove("nonexistent");
    /// assert!(removed.is_none());
    /// ```
    pub fn remove(&mut self, key: &str) -> Option<AttributeValue<SS, RR>> {
        self.values.remove(key)
    }

    /// Returns the number of attributes in the container.
    ///
    /// # Returns
    ///
    /// The number of key-value pairs in the attributes container.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut attrs = Attributes::<OwnedStringStorage, ResourceId32>::new();
    /// assert_eq!(attrs.len(), 0);
    ///
    /// attrs.insert("key1".to_string(), AttributeValue::Integer(1));
    /// attrs.insert("key2".to_string(), AttributeValue::Integer(2));
    /// assert_eq!(attrs.len(), 2);
    ///
    /// attrs.remove("key1");
    /// assert_eq!(attrs.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Checks if the attributes container is empty.
    ///
    /// # Returns
    ///
    /// `true` if the container has no attributes, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut attrs = Attributes::<OwnedStringStorage, ResourceId32>::new();
    /// assert!(attrs.is_empty());
    ///
    /// attrs.insert("key".to_string(), AttributeValue::Integer(1));
    /// assert!(!attrs.is_empty());
    ///
    /// attrs.remove("key");
    /// assert!(attrs.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns an iterator over the attributes' keys and values.
    ///
    /// # Returns
    ///
    /// An iterator yielding tuples of (&SS::String, &AttributeValue<SS>).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    /// use std::collections::HashSet;
    ///
    /// let mut attrs = Attributes::<OwnedStringStorage, ResourceId32>::new();
    /// attrs.insert("width".to_string(), AttributeValue::Float(10.0));
    /// attrs.insert("height".to_string(), AttributeValue::Float(20.0));
    ///
    /// // Collect keys into a set
    /// let keys: HashSet<&String> = attrs.iter().map(|(k, _)| k).collect();
    /// assert!(keys.contains(&"width".to_string()));
    /// assert!(keys.contains(&"height".to_string()));
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&SS::String, &AttributeValue<SS, RR>)> {
        self.values.iter()
    }

    /// Returns a mutable iterator over the attributes' keys and values.
    ///
    /// # Returns
    ///
    /// A mutable iterator yielding tuples of (&SS::String, &mut AttributeValue<SS>).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut attrs = Attributes::<OwnedStringStorage, ResourceId32>::new();
    /// attrs.insert("width".to_string(), AttributeValue::Float(10.0));
    /// attrs.insert("height".to_string(), AttributeValue::Float(20.0));
    ///
    /// // Double all float values
    /// for (_, value) in attrs.iter_mut() {
    ///     if let AttributeValue::Float(f) = value {
    ///         *f *= 2.0;
    ///     }
    /// }
    ///
    /// // Verify the changes
    /// if let Some(AttributeValue::Float(width)) = attrs.get("width") {
    ///     assert_eq!(*width, 20.0);
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&SS::String, &mut AttributeValue<SS, RR>)> {
        self.values.iter_mut()
    }

    /// Clears the attributes container, removing all key-value pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut attrs = Attributes::<OwnedStringStorage, ResourceId32>::new();
    /// attrs.insert("key1".to_string(), AttributeValue::Integer(1));
    /// attrs.insert("key2".to_string(), AttributeValue::Integer(2));
    /// assert_eq!(attrs.len(), 2);
    ///
    /// attrs.clear();
    /// assert_eq!(attrs.len(), 0);
    /// assert!(attrs.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Checks if the attributes container contains a key.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to check for
    ///
    /// # Returns
    ///
    /// `true` if the key exists, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut attrs = Attributes::<OwnedStringStorage, ResourceId32>::new();
    /// attrs.insert("exists".to_string(), AttributeValue::Bool(true));
    ///
    /// assert!(attrs.contains_key("exists"));
    /// assert!(!attrs.contains_key("nonexistent"));
    /// ```
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
}

impl<SS: StringStorage, RR: ResourceRef> Default for Attributes<SS, RR> {
    /// Creates a new, empty attributes container.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let attrs: Attributes<OwnedStringStorage, ResourceId32> = Default::default();
    /// assert!(attrs.is_empty());
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl<SS: StringStorage, RR: ResourceRef> Display for Attributes<SS, RR>
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

/// Type alias for attributes with owned strings.
///
/// This is a convenience type that uses `OwnedStringStorage` for the string storage strategy.
///
/// # Examples
///
/// ```rust
/// use cityjson::prelude::*;
///
/// let mut attrs = OwnedAttributes::new();
/// attrs.insert("name".to_string(), AttributeValue::String("Example".to_string()));
/// ```
pub type OwnedAttributes = Attributes<OwnedStringStorage, ResourceId32>;

/// Type alias for attributes with borrowed strings.
///
/// This is a convenience type that uses `BorrowedStringStorage` for the string storage strategy.
///
/// # Type Parameter
///
/// * `'a` - The lifetime of the borrowed strings
///
/// # Examples
///
/// ```rust
/// use cityjson::prelude::*;
///
/// let text = "Example";
/// let mut attrs = BorrowedAttributes::new();
/// attrs.insert("name", AttributeValue::String(text));
/// ```
pub type BorrowedAttributes<'a> = Attributes<BorrowedStringStorage<'a>, ResourceId32>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owned_attributes() {
        let mut attrs = OwnedAttributes::new();

        // Test insert
        attrs.insert(
            "name".to_string(),
            AttributeValue::String("John".to_string()),
        );

        // Test get
        match attrs.get("name") {
            Some(AttributeValue::String(name)) => assert_eq!(name, "John"),
            _ => panic!("Expected string value"),
        }

        // Test mutation
        if let Some(AttributeValue::String(name)) = attrs.get_mut("name") {
            *name = "Jane".to_string();
        }

        match attrs.get("name") {
            Some(AttributeValue::String(name)) => assert_eq!(name, "Jane"),
            _ => panic!("Expected modified string value"),
        }

        // Test remove
        let removed = attrs.remove("name");
        assert!(matches!(removed, Some(AttributeValue::String(s)) if s == "Jane"));
        assert!(attrs.get("name").is_none());
    }

    #[test]
    fn test_nested_attributes() {
        let mut attrs = OwnedAttributes::new();

        // Create and insert nested structure
        let mut map = HashMap::new();
        map.insert(
            "inner".to_string(),
            Box::new(AttributeValue::String("value".to_string())),
        );

        attrs.insert("nested".to_string(), AttributeValue::Map(map));

        // Test nested mutation
        if let Some(AttributeValue::Map(map)) = attrs.get_mut("nested") {
            if let Some(inner_value) = map.get_mut("inner") {
                if let AttributeValue::String(value) = &mut **inner_value {
                    *value = "modified".to_string();
                }
            }
        }

        // Verify mutation
        if let Some(AttributeValue::Map(map)) = attrs.get("nested") {
            if let Some(inner_box) = map.get("inner") {
                if let AttributeValue::String(value) = inner_box.as_ref() {
                    assert_eq!(value, "modified");
                } else {
                    panic!("Expected string value");
                }
            } else {
                panic!("Expected inner key to exist");
            }
        } else {
            panic!("Expected map value");
        }
    }

    #[test]
    fn test_borrowed_attributes() {
        let text = "John";
        let mut attrs = BorrowedAttributes::new();

        attrs.insert("name", AttributeValue::String(text));

        // Test get
        match attrs.get("name") {
            Some(AttributeValue::String(name)) => assert_eq!(*name, "John"),
            _ => panic!("Expected string value"),
        }

        // Test remove
        let removed = attrs.remove("name");
        assert!(matches!(removed, Some(AttributeValue::String(s)) if s == "John"));
        assert!(attrs.get("name").is_none());
    }

    #[test]
    fn test_attribute_value_types() {
        // Test creation of different attribute value types
        let null = AttributeValue::<OwnedStringStorage, ResourceId32>::Null;
        let boolean = AttributeValue::<OwnedStringStorage, ResourceId32>::Bool(true);
        let unsigned = AttributeValue::<OwnedStringStorage, ResourceId32>::Unsigned(42);
        let integer = AttributeValue::<OwnedStringStorage, ResourceId32>::Integer(-42);
        let float = AttributeValue::<OwnedStringStorage, ResourceId32>::Float(std::f64::consts::PI);
        let string = AttributeValue::<OwnedStringStorage, ResourceId32>::String("test".to_string());
        let geometry =
            AttributeValue::<OwnedStringStorage, ResourceId32>::Geometry(ResourceId32::new(0, 0));

        // Test equality
        assert_eq!(
            null,
            AttributeValue::<OwnedStringStorage, ResourceId32>::Null
        );
        assert_eq!(
            boolean,
            AttributeValue::<OwnedStringStorage, ResourceId32>::Bool(true)
        );
        assert_ne!(
            boolean,
            AttributeValue::<OwnedStringStorage, ResourceId32>::Bool(false)
        );
        assert_eq!(
            unsigned,
            AttributeValue::<OwnedStringStorage, ResourceId32>::Unsigned(42)
        );
        assert_ne!(
            unsigned,
            AttributeValue::<OwnedStringStorage, ResourceId32>::Unsigned(43)
        );
        assert_eq!(
            integer,
            AttributeValue::<OwnedStringStorage, ResourceId32>::Integer(-42)
        );
        assert_ne!(
            integer,
            AttributeValue::<OwnedStringStorage, ResourceId32>::Integer(-43)
        );
        assert_eq!(
            float,
            AttributeValue::<OwnedStringStorage, ResourceId32>::Float(std::f64::consts::PI)
        );
        assert_ne!(
            float,
            AttributeValue::<OwnedStringStorage, ResourceId32>::Float(std::f64::consts::PI)
        );
        assert_eq!(
            string,
            AttributeValue::<OwnedStringStorage, ResourceId32>::String("test".to_string())
        );
        assert_ne!(
            string,
            AttributeValue::<OwnedStringStorage, ResourceId32>::String("test2".to_string())
        );
        assert_eq!(
            geometry,
            AttributeValue::<OwnedStringStorage, ResourceId32>::Geometry(ResourceId32::new(0, 0))
        );
    }

    #[test]
    fn test_vector_attributes() {
        let mut attrs = OwnedAttributes::new();

        // Create and insert a vector of values
        let vec_values = vec![
            Box::new(AttributeValue::Integer(1)),
            Box::new(AttributeValue::Integer(2)),
            Box::new(AttributeValue::Integer(3)),
        ];

        attrs.insert("numbers".to_string(), AttributeValue::Vec(vec_values));

        // Test retrieval and modification
        if let Some(AttributeValue::Vec(numbers)) = attrs.get_mut("numbers") {
            // Add a new value
            numbers.push(Box::new(AttributeValue::Integer(4)));

            // Modify an existing value
            if let AttributeValue::Integer(mut _value) = *numbers[0] {
                _value = 10;
            }
        }

        // Verify changes
        if let Some(AttributeValue::Vec(numbers)) = attrs.get("numbers") {
            assert_eq!(numbers.len(), 4);

            // Check the modified first value
            if let AttributeValue::Integer(value) = numbers[0].as_ref() {
                assert_eq!(*value, 1);
            } else {
                panic!("Expected Integer value");
            }

            // Check the added value
            if let AttributeValue::Integer(value) = numbers[3].as_ref() {
                assert_eq!(*value, 4);
            } else {
                panic!("Expected Integer value");
            }
        } else {
            panic!("Expected Vec value");
        }
    }

    #[test]
    fn test_attributes_methods() {
        let mut attrs = OwnedAttributes::new();

        // Test is_empty and len
        assert!(attrs.is_empty());
        assert_eq!(attrs.len(), 0);

        // Insert some values
        attrs.insert("key1".to_string(), AttributeValue::Integer(1));
        attrs.insert("key2".to_string(), AttributeValue::Integer(2));

        assert!(!attrs.is_empty());
        assert_eq!(attrs.len(), 2);

        // Test contains_key
        assert!(attrs.contains_key("key1"));
        assert!(attrs.contains_key("key2"));
        assert!(!attrs.contains_key("key3"));

        // Test iter
        let mut keys = Vec::new();
        for (key, _) in attrs.iter() {
            keys.push(key.clone());
        }
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));

        // Test iter_mut
        for (_, value) in attrs.iter_mut() {
            if let AttributeValue::Integer(i) = value {
                *i *= 10;
            }
        }

        // Verify changes
        match attrs.get("key1") {
            Some(AttributeValue::Integer(value)) => assert_eq!(*value, 10),
            _ => panic!("Expected Integer value"),
        }
        match attrs.get("key2") {
            Some(AttributeValue::Integer(value)) => assert_eq!(*value, 20),
            _ => panic!("Expected Integer value"),
        }

        // Test clear
        attrs.clear();
        assert!(attrs.is_empty());
        assert_eq!(attrs.len(), 0);
        assert!(!attrs.contains_key("key1"));
        assert!(!attrs.contains_key("key2"));
    }

    #[test]
    fn test_default_implementation() {
        let attrs: OwnedAttributes = Default::default();
        assert!(attrs.is_empty());
        assert_eq!(attrs.len(), 0);
    }

    #[test]
    fn test_mixed_attribute_types() {
        let mut attrs = OwnedAttributes::new();

        // Insert different types of values
        attrs.insert("null".to_string(), AttributeValue::Null);
        attrs.insert("bool".to_string(), AttributeValue::Bool(true));
        attrs.insert("unsigned".to_string(), AttributeValue::Unsigned(42));
        attrs.insert("integer".to_string(), AttributeValue::Integer(-42));
        attrs.insert(
            "float".to_string(),
            AttributeValue::Float(std::f64::consts::PI),
        );
        attrs.insert(
            "string".to_string(),
            AttributeValue::String("test".to_string()),
        );

        // Test type-specific retrieval
        assert!(matches!(attrs.get("null"), Some(AttributeValue::Null)));
        assert!(matches!(
            attrs.get("bool"),
            Some(AttributeValue::Bool(true))
        ));
        assert!(matches!(
            attrs.get("unsigned"),
            Some(AttributeValue::Unsigned(42))
        ));
        assert!(matches!(
            attrs.get("integer"),
            Some(AttributeValue::Integer(-42))
        ));
        assert!(
            matches!(attrs.get("float"), Some(AttributeValue::Float(f)) if *f == std::f64::consts::PI)
        );
        assert!(matches!(attrs.get("string"), Some(AttributeValue::String(s)) if s == "test"));

        // Test non-existent key
        assert!(attrs.get("nonexistent").is_none());
    }

    #[test]
    fn test_complex_nested_structure() {
        let mut attrs = OwnedAttributes::new();

        // Create a complex nested structure
        // person: {
        //   name: "Alice",
        //   age: 30,
        //   address: {
        //     street: "123 Main St",
        //     city: "Anytown",
        //     coordinates: [40.7128, -74.0060]
        //   },
        //   hobbies: ["reading", "hiking", "coding"]
        // }

        // Create address map
        let mut address = HashMap::new();
        address.insert(
            "street".to_string(),
            Box::new(AttributeValue::String("123 Main St".to_string())),
        );
        address.insert(
            "city".to_string(),
            Box::new(AttributeValue::String("Anytown".to_string())),
        );

        // Create coordinates vector
        let coordinates = vec![
            Box::new(AttributeValue::Float(40.7128)),
            Box::new(AttributeValue::Float(-74.0060)),
        ];
        address.insert(
            "coordinates".to_string(),
            Box::new(AttributeValue::Vec(coordinates)),
        );

        // Create hobbies vector
        let hobbies = vec![
            Box::new(AttributeValue::String("reading".to_string())),
            Box::new(AttributeValue::String("hiking".to_string())),
            Box::new(AttributeValue::String("coding".to_string())),
        ];

        // Create person map
        let mut person = HashMap::new();
        person.insert(
            "name".to_string(),
            Box::new(AttributeValue::String("Alice".to_string())),
        );
        person.insert("age".to_string(), Box::new(AttributeValue::Integer(30)));
        person.insert(
            "address".to_string(),
            Box::new(AttributeValue::Map(address)),
        );
        person.insert(
            "hobbies".to_string(),
            Box::new(AttributeValue::Vec(hobbies)),
        );

        // Insert into attributes
        attrs.insert("person".to_string(), AttributeValue::Map(person));

        // Test deep nested access
        if let Some(AttributeValue::Map(person_map)) = attrs.get("person") {
            // Check name
            if let Some(name_box) = person_map.get("name") {
                if let AttributeValue::String(name) = name_box.as_ref() {
                    assert_eq!(name, "Alice");
                } else {
                    panic!("Expected String value for name");
                }
            }

            // Check address
            if let Some(address_box) = person_map.get("address") {
                if let AttributeValue::Map(address_map) = address_box.as_ref() {
                    // Check street
                    if let Some(street_box) = address_map.get("street") {
                        if let AttributeValue::String(street) = street_box.as_ref() {
                            assert_eq!(street, "123 Main St");
                        } else {
                            panic!("Expected String value for street");
                        }
                    }

                    // Check coordinates
                    if let Some(coordinates_box) = address_map.get("coordinates") {
                        if let AttributeValue::Vec(coordinates_vec) = coordinates_box.as_ref() {
                            assert_eq!(coordinates_vec.len(), 2);
                            if let AttributeValue::Float(lat) = coordinates_vec[0].as_ref() {
                                assert_eq!(*lat, 40.7128);
                            } else {
                                panic!("Expected Float value for latitude");
                            }

                            if let AttributeValue::Float(lon) = coordinates_vec[1].as_ref() {
                                assert_eq!(*lon, -74.0060);
                            } else {
                                panic!("Expected Float value for longitude");
                            }
                        }
                    } else {
                        panic!("Expected Vec value for coordinates");
                    }
                }
            }

            // Check hobbies
            if let Some(hobbies_box) = person_map.get("hobbies") {
                if let AttributeValue::Vec(hobbies_vec) = hobbies_box.as_ref() {
                    assert_eq!(hobbies_vec.len(), 3);

                    if let AttributeValue::String(hobby) = hobbies_vec[0].as_ref() {
                        assert_eq!(hobby, "reading");
                    } else {
                        panic!("Expected String value for hobby");
                    }
                } else {
                    panic!("Expected Vec value for hobbies");
                }
            }
        }
    }

    #[test]
    fn test_attribute_value_display() {
        // Test primitive values
        let null = AttributeValue::<OwnedStringStorage, ResourceId32>::Null;
        let boolean = AttributeValue::<OwnedStringStorage, ResourceId32>::Bool(true);
        let unsigned = AttributeValue::<OwnedStringStorage, ResourceId32>::Unsigned(42);
        let integer = AttributeValue::<OwnedStringStorage, ResourceId32>::Integer(-100);
        let float = AttributeValue::<OwnedStringStorage, ResourceId32>::Float(3.12345);
        let string =
            AttributeValue::<OwnedStringStorage, ResourceId32>::String("hello".to_string());
        let geometry =
            AttributeValue::<OwnedStringStorage, ResourceId32>::Geometry(ResourceId32::new(0, 0));

        assert_eq!(format!("{}", null), "null");
        assert_eq!(format!("{}", boolean), "true");
        assert_eq!(format!("{}", unsigned), "42");
        assert_eq!(format!("{}", integer), "-100");
        assert_eq!(format!("{}", float), "3.12345");
        assert_eq!(format!("{}", string), "\"hello\"");
        assert_eq!(format!("{}", geometry), "Geometry index: 0, generation: 0");

        // Test vector
        let vec_values = vec![
            Box::new(AttributeValue::<OwnedStringStorage, ResourceId32>::Integer(
                1,
            )),
            Box::new(AttributeValue::<OwnedStringStorage, ResourceId32>::Integer(
                2,
            )),
            Box::new(AttributeValue::<OwnedStringStorage, ResourceId32>::Integer(
                3,
            )),
        ];

        let vec_attr = AttributeValue::<OwnedStringStorage, ResourceId32>::Vec(vec_values);
        assert_eq!(format!("{}", vec_attr), "[1, 2, 3]");

        // Test map
        let mut map = HashMap::new();
        map.insert(
            "name".to_string(),
            Box::new(AttributeValue::<OwnedStringStorage, ResourceId32>::String(
                "City Hall".to_string(),
            )),
        );
        map.insert(
            "height".to_string(),
            Box::new(AttributeValue::<OwnedStringStorage, ResourceId32>::Float(
                45.5,
            )),
        );

        let map_attr = AttributeValue::<OwnedStringStorage, ResourceId32>::Map(map);
        // Since HashMap iteration order is non-deterministic, we need to check parts
        let map_str = format!("{}", map_attr);
        assert!(map_str.starts_with("{"));
        assert!(map_str.ends_with("}"));
        assert!(map_str.contains("\"name\": \"City Hall\""));
        assert!(map_str.contains("\"height\": 45.5"));
    }

    #[test]
    fn test_attributes_display() {
        let mut attrs = Attributes::<OwnedStringStorage, ResourceId32> {
            values: HashMap::new(),
        };

        attrs.values.insert(
            "name".to_string(),
            AttributeValue::String("Building 42".to_string()),
        );
        attrs
            .values
            .insert("year_built".to_string(), AttributeValue::Integer(1985));
        attrs
            .values
            .insert("is_heritage".to_string(), AttributeValue::Bool(true));

        // Since HashMap iteration order is non-deterministic, we need to check parts
        let attrs_str = format!("{}", attrs);
        assert!(attrs_str.starts_with("{"));
        assert!(attrs_str.ends_with("}"));
        assert!(attrs_str.contains("\"name\": \"Building 42\""));
        assert!(attrs_str.contains("\"year_built\": 1985"));
        assert!(attrs_str.contains("\"is_heritage\": true"));
        println!("{}", attrs);
    }
}
