use crate::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq)]
pub enum AttributeValue<S: StringStorage> {
    Null,
    Bool(bool),
    Unsigned(u64),
    Integer(i64),
    Float(f64),
    String(S::String),
    Vec(Vec<Box<AttributeValue<S>>>),
    Map(HashMap<S::String, Box<AttributeValue<S>>>),
}

/// Container for attributes using a specific storage strategy
#[derive(Clone, Debug)]
pub struct Attributes<S: StringStorage> {
    values: HashMap<S::String, AttributeValue<S>>,
}

impl<S: StringStorage> Attributes<S> {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    pub fn get(&self, key: &str) -> Option<&AttributeValue<S>> {
        self.values.get(key.borrow())
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut AttributeValue<S>> {
        self.values.get_mut(key.borrow())
    }

    pub fn insert(&mut self, key: S::String, value: AttributeValue<S>) -> Option<AttributeValue<S>> {
        self.values.insert(key, value)
    }

    pub fn remove(&mut self, key: &str) -> Option<AttributeValue<S>> {
        self.values.remove(key.borrow())
    }
}

pub type OwnedAttributes = Attributes<OwnedStringStorage>;
pub type BorrowedAttributes<'a> = Attributes<BorrowedStringStorage<'a>>;

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

        attrs.insert(
            "nested".to_string(),
            AttributeValue::Map(map),
        );

        // Test nested mutation
        if let Some(AttributeValue::Map(map)) = attrs.get_mut("nested") {
            if let Some(inner_value) = map.get_mut("inner") {
                if let AttributeValue::String(value) = &mut **inner_value {
                    *value = "modified".to_string();
                }
            }
        }

        // Verify mutation
        match attrs.get("nested") {
            Some(AttributeValue::Map(map)) => {
                match &**map.get("inner").unwrap() {
                    AttributeValue::String(value) => assert_eq!(value, "modified"),
                    _ => panic!("Expected string value"),
                }
            }
            _ => panic!("Expected map value"),
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
}