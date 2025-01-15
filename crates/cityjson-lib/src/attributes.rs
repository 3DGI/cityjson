use crate::errors;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Attributes {
    Null,
    Bool(bool),
    Unsigned(u64),
    Integer(i64),
    Float(f64),
    String(String),
    Vec(Vec<Attributes>),
    Map(HashMap<String, Attributes>),
}

impl Attributes {
    // Type checking methods
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    pub fn is_unsigned(&self) -> bool {
        matches!(self, Self::Unsigned(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_vec(&self) -> bool {
        matches!(self, Self::Vec(_))
    }

    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_unsigned(&self) -> Option<u64> {
        match self {
            Self::Unsigned(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_vec(&self) -> Option<&Vec<Attributes>> {
        match self {
            Self::Vec(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, Attributes>> {
        match self {
            Self::Map(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_vec_mut(&mut self) -> Option<&mut Vec<Attributes>> {
        match self {
            Self::Vec(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_map_mut(&mut self) -> Option<&mut HashMap<String, Attributes>> {
        match self {
            Self::Map(v) => Some(v),
            _ => None,
        }
    }
}
impl Default for Attributes {
    fn default() -> Self {
        Self::Null
    }
}

impl fmt::Display for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Unsigned(u) => write!(f, "{}", u),
            Self::Integer(i) => write!(f, "{}", i),
            Self::Float(fl) => write!(f, "{}", fl),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Vec(v) => {
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Self::Map(m) => {
                write!(f, "{{")?;
                for (i, (key, value)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl<'a> TryFrom<&'a serde_cityjson::attributes::Attributes<'a>> for Attributes {
    type Error = errors::Error;

    fn try_from(value: &'a serde_cityjson::attributes::Attributes<'a>) -> errors::Result<Self> {
        // Handle null
        if value.is_null() {
            return Ok(Self::Null);
        }

        // Handle primitive types
        if let Some(b) = value.as_bool() {
            return Ok(Self::Bool(b));
        }
        if let Some(u) = value.as_u64() {
            return Ok(Self::Unsigned(u));
        }
        if let Some(i) = value.as_i64() {
            return Ok(Self::Integer(i));
        }
        if let Some(f) = value.as_f64() {
            return Ok(Self::Float(f));
        }
        if let Some(s) = value.as_str() {
            return Ok(Self::String(s.to_string()));
        }

        // Handle arrays using AttributesArrayIter
        let mut vec = Vec::new();
        for item in value.array_iter() {
            vec.push(Self::try_from(&item)?);
        }
        if !vec.is_empty() {
            return Ok(Self::Vec(vec));
        }

        // Handle objects using AttributesObjectIter
        let mut map = HashMap::new();
        for (key, val) in value.object_iter() {
            map.insert(key.to_string(), Self::try_from(&val)?);
        }
        if !map.is_empty() {
            return Ok(Self::Map(map));
        }

        Err(errors::Error::AttributeConversionError(
            "Invalid attribute value".to_string(),
        ))
    }
}

impl<'a> TryFrom<serde_cityjson::attributes::Attributes<'a>> for Attributes {
    type Error = errors::Error;

    fn try_from(value: serde_cityjson::attributes::Attributes<'a>) -> errors::Result<Self> {
        Self::try_from(&value)
    }
}

impl<'a> TryFrom<&serde_cityjson::attributes::AttributesRef<'a>> for Attributes {
    type Error = errors::Error;

    fn try_from(value: &serde_cityjson::attributes::AttributesRef<'a>) -> errors::Result<Self> {
        // Handle primitive types
        if let Some(b) = value.as_bool() {
            return Ok(Self::Bool(b));
        }
        if let Some(u) = value.as_u64() {
            return Ok(Self::Unsigned(u));
        }
        if let Some(i) = value.as_i64() {
            return Ok(Self::Integer(i));
        }
        if let Some(f) = value.as_f64() {
            return Ok(Self::Float(f));
        }
        if let Some(s) = value.as_str() {
            return Ok(Self::String(s.to_string()));
        }

        // Handle arrays using AttributesArrayIter
        let mut vec = Vec::new();
        for item in value.array_iter() {
            vec.push(Self::try_from(&item)?);
        }
        if !vec.is_empty() {
            return Ok(Self::Vec(vec));
        }

        // Handle objects using AttributesObjectIter
        let mut map = HashMap::new();
        for (key, val) in value.object_iter() {
            map.insert(key.to_string(), Self::try_from(&val)?);
        }
        if !map.is_empty() {
            return Ok(Self::Map(map));
        }

        // If we get here, it must be null
        Ok(Self::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_cityjson::attributes::Attributes as CityjsonAttributes;
    use serde_json::json;

    #[test]
    fn test_convert_primitive_values() {
        let cityjson_attrs = CityjsonAttributes::Owned(json!({
            "null": null,
            "bool": true,
            "unsigned": 42u64,
            "integer": -42,
            "float": 3.14,
            "string": "test"
        }));

        let attrs = Attributes::try_from(&cityjson_attrs).unwrap();
        if let Attributes::Map(map) = attrs {
            assert!(map.get("null").unwrap().is_null());
            assert_eq!(map.get("bool").unwrap().as_bool(), Some(true));
            assert_eq!(map.get("unsigned").unwrap().as_unsigned(), Some(42));
            assert_eq!(map.get("integer").unwrap().as_integer(), Some(-42));
            assert_eq!(map.get("float").unwrap().as_float(), Some(3.14));
            assert_eq!(map.get("string").unwrap().as_str(), Some("test"));
        } else {
            panic!("Expected Map variant");
        }
    }

    #[test]
    fn test_convert_nested_structures() {
        let cityjson_attrs = CityjsonAttributes::Owned(json!({
            "array": [1, "text", true],
            "object": {
                "nested": {
                    "value": 42
                }
            }
        }));

        let attrs = Attributes::try_from(&cityjson_attrs).unwrap();
        if let Attributes::Map(map) = attrs {
            // Test array conversion
            if let Attributes::Vec(arr) = map.get("array").unwrap() {
                assert_eq!(arr[0].as_unsigned(), Some(1));
                assert_eq!(arr[1].as_str(), Some("text"));
                assert_eq!(arr[2].as_bool(), Some(true));
            } else {
                panic!("Expected Vec variant");
            }

            // Test nested object conversion
            if let Attributes::Map(obj) = map.get("object").unwrap() {
                if let Attributes::Map(nested) = obj.get("nested").unwrap() {
                    assert_eq!(nested.get("value").unwrap().as_unsigned(), Some(42));
                } else {
                    panic!("Expected Map variant");
                }
            } else {
                panic!("Expected Map variant");
            }
        } else {
            panic!("Expected Map variant");
        }
    }
}
