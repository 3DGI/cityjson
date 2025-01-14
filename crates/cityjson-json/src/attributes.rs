//! Module for handling CityJSON attributes with efficient zero-copy deserialization.
//!
//! This module provides the [`Attributes`] enum which can hold either borrowed or owned JSON values.
//! It is designed to handle CityJSON attributes efficiently by:
//! - Using zero-copy deserialization with [`serde_json_borrow::Value`] when reading data
//! - Supporting owned values with [`serde_json::Value`] when creating or modifying data
//!
//! # Examples
//!
//! Reading attributes from JSON (borrowed):
//! ```
//! # use serde_json::from_str;
//! # use serde_cityjson::attributes::Attributes;
//! let json = r#"{"b3_dak_type": "slanted", "b3_h_dak_50p": 9.74}"#;
//! let value = from_str(json).unwrap();
//! let attrs = Attributes::Borrowed(value);
//!
//! assert_eq!(attrs.get("b3_dak_type").and_then(|v| v.as_str()), Some("slanted"));
//! ```
//!
//! Creating new attributes (owned):
//! ```
//! # use serde_cityjson::attributes::Attributes;
//! let value = serde_json::json!({
//!     "b3_dak_type": "slanted",
//!     "b3_h_dak_50p": 9.74
//! });
//! let attrs = Attributes::Owned(value);
//!
//! assert_eq!(attrs.get_owned("b3_dak_type").and_then(|v| v.as_str()), Some("slanted"));
//! ```
//!
//! Iterating over object members:
//! ```
//! # use serde_cityjson::attributes::Attributes;
//! # use serde_json::from_str;
//! let json = r#"{"key1": "value1", "key2": "value2"}"#;
//! let value = from_str(json).unwrap();
//! let attrs = Attributes::Borrowed(value);
//!
//! if let Some(obj) = attrs.as_object() {
//!     for (key, value) in obj.iter() {
//!         println!("{}: {}", key, value);
//!     }
//! }
//! ```
//!
//! # Performance Considerations
//!
//! - Use borrowed values (`Attributes::Borrowed`) when reading data as it provides zero-copy deserialization
//! - Use owned values (`Attributes::Owned`) when creating new data or modifying existing data
//! - Methods are separated for borrowed and owned variants to avoid unnecessary conversions
//!
//! # Note
//!
//! The borrowed variant uses [`serde_json_borrow::Value`] which maintains references to the original JSON data,
//! while the owned variant uses [`serde_json::Value`] which owns its data. Choose the appropriate variant
//! based on your use case.
use serde::{Deserialize, Serialize, Serializer};
use serde_json_borrow::Value;
use std::fmt::{Display, Formatter};

// NOTE OPTIMIZATION:
// - Consider using `smallvec` for small arrays/objects
// - Add capacity hints for iterators
// - Consider adding a compact representation for small string values

/// Represents CityJSON attributes that can be either borrowed or owned.
///
/// - `Borrowed` variant uses zero-copy deserialization with [`serde_json_borrow::Value`]
/// - `Owned` variant uses [`serde_json::Value`] for creating or modifying data
///
/// # Example
/// ```
/// # use serde_json::from_str;
/// # use serde_cityjson::attributes::Attributes;
/// // Borrowed (reading)
/// let json = r#"{"height": 9.74}"#;
/// let value = from_str(json).unwrap();
/// let attrs = Attributes::Borrowed(value);
///
/// // Owned (creating)
/// let value = serde_json::json!({"height": 9.74});
/// let attrs = Attributes::Owned(value);
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Attributes<'cm> {
    Borrowed(Value<'cm>),
    Owned(serde_json::Value),
}

impl<'cm> Attributes<'cm> {
    /// Operations on borrowed variant
    pub fn as_borrowed(&self) -> Option<&Value<'cm>> {
        match self {
            Self::Borrowed(v) => Some(v),
            Self::Owned(_) => None,
        }
    }

    /// Operations on owned variant
    pub fn as_owned(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Borrowed(_) => None,
            Self::Owned(v) => Some(v),
        }
    }

    pub fn get(&'cm self, key: &'cm str) -> Option<&'cm Value<'cm>> {
        match self {
            Self::Borrowed(v) => {
                let value = v.get(key);
                if value == &Value::Null {
                    None
                } else {
                    Some(value)
                }
            }
            Self::Owned(_) => None,
        }
    }

    pub fn get_owned(&self, key: &str) -> Option<&serde_json::Value> {
        match self {
            Self::Borrowed(_) => None,
            Self::Owned(v) => v.get(key),
        }
    }

    /// Check if attributes object is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Borrowed(v) => v.as_object().map_or(true, |o| o.is_empty()),
            Self::Owned(v) => v.as_object().map_or(true, |o| o.is_empty()),
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Self::Borrowed(v) => v.is_null(),
            Self::Owned(v) => v.is_null(),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Borrowed(v) => v.as_str(),
            Self::Owned(v) => v.as_str(),
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Borrowed(v) => v.as_f64(),
            Self::Owned(v) => v.as_f64(),
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Borrowed(v) => v.as_bool(),
            Self::Owned(v) => v.as_bool(),
        }
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Self::Borrowed(v) => v.as_array(),
            Self::Owned(_) => None,
        }
    }

    pub fn as_owned_array(&self) -> Option<&Vec<serde_json::Value>> {
        match self {
            Self::Borrowed(_) => None,
            Self::Owned(v) => v.as_array(),
        }
    }

    pub fn as_object(&self) -> Option<&serde_json_borrow::ObjectAsVec> {
        match self {
            Self::Borrowed(v) => v.as_object(),
            Self::Owned(_) => None,
        }
    }

    pub fn as_owned_object(&self) -> Option<&serde_json::Map<String, serde_json::Value>> {
        match self {
            Self::Borrowed(_) => None,
            Self::Owned(v) => v.as_object(),
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Borrowed(v) => v.as_u64(),
            Self::Owned(v) => v.as_u64(),
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Borrowed(v) => v.as_i64(),
            Self::Owned(v) => v.as_i64(),
        }
    }

    /// Returns an iterator over array elements as references
    pub fn array_iter(&'cm self) -> AttributesArrayIter<'cm> {
        // Static empty Vec to avoid temporary allocation issues
        static EMPTY_VEC: Vec<serde_json::Value> = Vec::new();
        match self {
            Self::Borrowed(v) => AttributesArrayIter::Borrowed(v.as_array().unwrap_or(&[]).iter()),
            Self::Owned(v) => AttributesArrayIter::Owned(v.as_array().unwrap_or(&EMPTY_VEC).iter()),
        }
    }

    /// Returns an iterator over object key-value pairs as references
    pub fn object_iter(&'cm self) -> AttributesObjectIter<'cm> {
        match self {
            Self::Borrowed(v) => {
                if let Some(obj) = v.as_object() {
                    AttributesObjectIter::Borrowed(Box::new(
                        obj.iter().map(|(k, v)| (k, AttributesRef::Borrowed(v))),
                    ))
                } else {
                    // Empty iterator for non-objects
                    AttributesObjectIter::Borrowed(Box::new(std::iter::empty()))
                }
            }
            Self::Owned(v) => match v.as_object() {
                Some(obj) => AttributesObjectIter::Owned(Box::new(
                    obj.iter()
                        .map(|(k, v)| (k.as_ref(), AttributesRef::Owned(v))),
                )),
                None => AttributesObjectIter::Owned(Box::new(std::iter::empty())),
            },
        }
    }
}

impl<'cm> Default for Attributes<'cm> {
    fn default() -> Self {
        Self::Owned(Default::default())
    }
}

impl<'cm> Display for Attributes<'cm> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Borrowed(v) => write!(f, "{}", v),
            Self::Owned(v) => write!(f, "{}", v),
        }
    }
}

/// Iterator over references to array elements
pub enum AttributesArrayIter<'cm> {
    Borrowed(std::slice::Iter<'cm, Value<'cm>>),
    Owned(std::slice::Iter<'cm, serde_json::Value>),
}

impl<'cm> Iterator for AttributesArrayIter<'cm> {
    // Return a reference to an Attributes that borrows from self
    type Item = AttributesRef<'cm>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Borrowed(iter) => iter.next().map(AttributesRef::Borrowed),
            Self::Owned(iter) => iter.next().map(AttributesRef::Owned),
        }
    }
}

/// Iterator over references to object key-value pairs
pub enum AttributesObjectIter<'cm> {
    Borrowed(Box<dyn Iterator<Item = (&'cm str, AttributesRef<'cm>)> + 'cm>),
    Owned(Box<dyn Iterator<Item = (&'cm str, AttributesRef<'cm>)> + 'cm>),
}

impl<'cm> Iterator for AttributesObjectIter<'cm> {
    type Item = (&'cm str, AttributesRef<'cm>);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Borrowed(iter) => iter.next(),
            Self::Owned(iter) => iter.next(),
        }
    }
}

/// A reference type that mirrors Attributes but holds references
#[derive(Debug, Clone)]
pub enum AttributesRef<'cm> {
    Borrowed(&'cm Value<'cm>),
    Owned(&'cm serde_json::Value),
}

// Implement methods on AttributesRef that mirror Attributes
impl<'cm> AttributesRef<'cm> {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Borrowed(v) => v.as_str(),
            Self::Owned(v) => v.as_str(),
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Borrowed(v) => v.as_f64(),
            Self::Owned(v) => v.as_f64(),
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Borrowed(v) => v.as_bool(),
            Self::Owned(v) => v.as_bool(),
        }
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Self::Borrowed(v) => v.as_array(),
            Self::Owned(_) => None,
        }
    }

    pub fn as_owned_array(&self) -> Option<&Vec<serde_json::Value>> {
        match self {
            Self::Borrowed(_) => None,
            Self::Owned(v) => v.as_array(),
        }
    }

    pub fn as_object(&self) -> Option<&serde_json_borrow::ObjectAsVec> {
        match self {
            Self::Borrowed(v) => v.as_object(),
            Self::Owned(_) => None,
        }
    }

    pub fn as_owned_object(&self) -> Option<&serde_json::Map<String, serde_json::Value>> {
        match self {
            Self::Borrowed(_) => None,
            Self::Owned(v) => v.as_object(),
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Borrowed(v) => v.as_u64(),
            Self::Owned(v) => v.as_u64(),
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Borrowed(v) => v.as_i64(),
            Self::Owned(v) => v.as_i64(),
        }
    }
}

impl<'cm> PartialEq for AttributesRef<'cm> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Borrowed(a), Self::Borrowed(b)) => a == b,
            (Self::Owned(a), Self::Owned(b)) => a == b,
            _ => false,
        }
    }
}

pub fn deserialize_attributes<'de: 'cm, 'cm, D>(
    deserializer: D,
) -> Result<Option<Attributes<'cm>>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s = Value::deserialize(deserializer)?;
    Ok((!s.is_null()).then_some(Attributes::Borrowed(s)))
}

pub fn serialize_attributes<S>(
    attributes: &Option<Attributes>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // We unwrap here, because the attributes are always set to
    // 'skip_serializing_if = "Option::is_none"'.
    let a = attributes.as_ref().unwrap();
    match a {
        Attributes::Borrowed(a) => a.serialize(serializer),
        Attributes::Owned(a) => a.serialize(serializer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;
    use serde_json::json;

    #[test]
    fn test_primitive_access() {
        let json = r#"{
            "string": "text",
            "float": 9.74,
            "integer": 42,
            "boolean": true
        }"#;

        // Test borrowed access
        let attrs = Attributes::Borrowed(from_str(json).unwrap());
        assert_eq!(attrs.get("string").and_then(|v| v.as_str()), Some("text"));
        assert_eq!(attrs.get("float").and_then(|v| v.as_f64()), Some(9.74));
        assert_eq!(attrs.get("integer").and_then(|v| v.as_i64()), Some(42));
        assert_eq!(attrs.get("boolean").and_then(|v| v.as_bool()), Some(true));

        // Test owned access
        let attrs = Attributes::Owned(
            json!({"string": "text", "float": 9.74, "integer": 42, "boolean": true}),
        );
        assert_eq!(
            attrs.get_owned("string").and_then(|v| v.as_str()),
            Some("text")
        );
        assert_eq!(
            attrs.get_owned("float").and_then(|v| v.as_f64()),
            Some(9.74)
        );
        assert_eq!(
            attrs.get_owned("integer").and_then(|v| v.as_i64()),
            Some(42)
        );
        assert_eq!(
            attrs.get_owned("boolean").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn test_array_operations() {
        let json = "[1, \"text\", true]";

        // Test borrowed array
        let attrs = Attributes::Borrowed(from_str(json).unwrap());
        assert!(attrs.as_array().is_some());
        let mut iter = attrs.array_iter();
        assert_eq!(iter.next().unwrap().as_i64(), Some(1));
        assert_eq!(iter.next().unwrap().as_str(), Some("text"));
        assert_eq!(iter.next().unwrap().as_bool(), Some(true));
        assert!(iter.next().is_none());

        // Test owned array
        let attrs = Attributes::Owned(json!([1, "text", true]));
        assert!(attrs.as_owned_array().is_some());
        let mut iter = attrs.array_iter();
        assert_eq!(iter.next().unwrap().as_i64(), Some(1));
        assert_eq!(iter.next().unwrap().as_str(), Some("text"));
        assert_eq!(iter.next().unwrap().as_bool(), Some(true));
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_object_operations() {
        let json = r#"{"a": 1, "b": "text", "c": true}"#;

        // Test borrowed object
        let attrs = Attributes::Borrowed(from_str(json).unwrap());
        let mut entries: Vec<_> = attrs.object_iter().collect();
        entries.sort_by_key(|(k, _)| *k);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].1.as_i64(), Some(1));
        assert_eq!(entries[1].1.as_str(), Some("text"));
        assert_eq!(entries[2].1.as_bool(), Some(true));

        // Test owned object
        let attrs = Attributes::Owned(json!({"a": 1, "b": "text", "c": true}));
        let mut entries: Vec<_> = attrs.object_iter().collect();
        entries.sort_by_key(|(k, _)| *k);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].1.as_i64(), Some(1));
        assert_eq!(entries[1].1.as_str(), Some("text"));
        assert_eq!(entries[2].1.as_bool(), Some(true));
    }

    #[test]
    fn test_empty_and_null() {
        // Test empty object
        let attrs = Attributes::default();
        assert!(attrs.is_empty());

        let attrs = Attributes::Owned(json!({}));
        assert!(attrs.is_empty());

        // Test null value
        let attrs = Attributes::Owned(json!(null));
        assert!(attrs.is_null());

        let attrs = Attributes::Borrowed(from_str("null").unwrap());
        assert!(attrs.is_null());
    }
}
