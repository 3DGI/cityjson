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

    const TEST_JSON: &str = r#"{
        "string": "text",
        "float": 9.74,
        "integer": 42,
        "unsigned": 123,
        "boolean": true,
        "array": [1, 2, 3],
        "object": {"key": "value"}
    }"#;

    #[test]
    fn test_borrowed_values() {
        let value = from_str(TEST_JSON).unwrap();
        let attrs = Attributes::Borrowed(value);

        // Test primitive type access
        assert_eq!(attrs.get("string").and_then(|v| v.as_str()), Some("text"));
        assert_eq!(attrs.get("float").and_then(|v| v.as_f64()), Some(9.74));
        assert_eq!(attrs.get("integer").and_then(|v| v.as_i64()), Some(42));
        assert_eq!(attrs.get("unsigned").and_then(|v| v.as_u64()), Some(123));
        assert_eq!(attrs.get("boolean").and_then(|v| v.as_bool()), Some(true));

        // Test array access
        let array = attrs.get("array").and_then(|v| v.as_array()).unwrap();
        assert_eq!(array.len(), 3);

        // Test object access
        let object = attrs.get("object").and_then(|v| v.as_object()).unwrap();
        assert_eq!(object.get("key").and_then(|v| v.as_str()), Some("value"));
    }

    #[test]
    fn test_owned_values() {
        let value = serde_json::from_str(TEST_JSON).unwrap();
        let attrs = Attributes::Owned(value);

        // Test primitive type access
        assert_eq!(attrs.get_owned("string").and_then(|v| v.as_str()), Some("text"));
        assert_eq!(attrs.get_owned("float").and_then(|v| v.as_f64()), Some(9.74));
        assert_eq!(attrs.get_owned("integer").and_then(|v| v.as_i64()), Some(42));
        assert_eq!(attrs.get_owned("unsigned").and_then(|v| v.as_u64()), Some(123));
        assert_eq!(attrs.get_owned("boolean").and_then(|v| v.as_bool()), Some(true));

        // Test array access
        let array = attrs.get_owned("array").and_then(|v| v.as_array()).unwrap();
        assert_eq!(array.len(), 3);

        // Test object access
        let object = attrs.get_owned("object").and_then(|v| v.as_object()).unwrap();
        assert_eq!(object.get("key").and_then(|v| v.as_str()), Some("value"));
    }

    #[test]
    fn test_object_iteration() {
        let json = r#"{"a": 1, "b": "text", "c": true}"#;

        // Test borrowed iteration
        let value = from_str(json).unwrap();
        let attrs = Attributes::Borrowed(value);
        if let Some(obj) = attrs.as_object() {
            let mut count = 0;
            for (k, v) in obj.iter() {
                match k {
                    "a" => assert!(v.as_i64().is_some()),
                    "b" => assert!(v.as_str().is_some()),
                    "c" => assert!(v.as_bool().is_some()),
                    _ => panic!("Unexpected key"),
                }
                count += 1;
            }
            assert_eq!(count, 3);
        }

        // Test owned iteration
        let value = serde_json::json!({"a": 1, "b": "text", "c": true});
        let attrs = Attributes::Owned(value);
        if let Some(obj) = attrs.as_owned_object() {
            let mut count = 0;
            for (k, v) in obj.iter() {
                match k.as_str() {
                    "a" => assert!(v.as_i64().is_some()),
                    "b" => assert!(v.as_str().is_some()),
                    "c" => assert!(v.as_bool().is_some()),
                    _ => panic!("Unexpected key"),
                }
                count += 1;
            }
            assert_eq!(count, 3);
        }
    }

    #[test]
    fn test_empty_attributes() {
        let attrs = Attributes::default();
        assert!(attrs.is_empty());

        let empty_obj = serde_json::json!({});
        let attrs = Attributes::Owned(empty_obj);
        assert!(attrs.is_empty());
    }
}
