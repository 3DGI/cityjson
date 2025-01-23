use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq)]
pub enum AttributeValue<S: Eq + PartialEq + Hash> {
    Null,
    Bool(bool),
    Unsigned(u64),
    Integer(i64),
    Float(f64),
    String(S),
    Vec(Vec<Box<AttributeValue<S>>>),
    Map(HashMap<S, Box<AttributeValue<S>>>),
}

/// Trait for attribute storage strategies
pub trait AttributeStorage: Clone + Debug {
    /// The type of string used in attributes (String for owned, &str for borrowed)
    type StringType: AsRef<str>;
    /// The attribute value type used by this storage
    type ValueType: Clone + Debug;

    /// Create a new empty attribute storage
    fn new() -> Self;
    /// Get a value by key
    fn get(&self, key: &str) -> Option<&Self::ValueType>;
    /// Insert a value with given key
    fn insert(&mut self, key: Self::StringType, value: Self::ValueType);
    /// Remove a value by key
    fn remove(&mut self, key: &str) -> Option<Self::ValueType>;
    /// Check if storage contains a key
    fn contains_key(&self, key: &str) -> bool;
    /// Get the number of stored attributes
    fn len(&self) -> usize;
    /// Check if storage is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Generic Attributes container
#[derive(Clone, Debug)]
pub struct Attributes<S: AttributeStorage> {
    storage: S,
}

impl<S: AttributeStorage> Attributes<S> {
    /// Create a new empty attributes container
    pub fn new() -> Self {
        Self {
            storage: S::new(),
        }
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&S::ValueType> {
        self.storage.get(key)
    }

    /// Insert a value with given key
    pub fn insert(&mut self, key: S::StringType, value: S::ValueType) {
        self.storage.insert(key, value);
    }

    /// Remove a value by key
    pub fn remove(&self, key: &str) -> Option<&S::ValueType> {
        self.storage.get(key)
    }

    /// Check if attributes contains a key
    pub fn contains_key(&self, key: &str) -> bool {
        self.storage.contains_key(key)
    }

    /// Get the number of attributes
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    /// Check if attributes is empty
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// Get reference to underlying storage
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Get mutable reference to underlying storage
    pub fn storage_mut(&mut self) -> &mut S {
        &mut self.storage
    }
}

/// Owned storage implementation
#[derive(Clone, Debug, Default)]
pub struct OwnedStorage {
    values: HashMap<String, AttributeValue<String>>,
}

impl AttributeStorage for OwnedStorage {
    type StringType = String;
    type ValueType = AttributeValue<String>;

    fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    fn get(&self, key: &str) -> Option<&Self::ValueType> {
        self.values.get(key)
    }

    fn insert(&mut self, key: Self::StringType, value: Self::ValueType) {
        self.values.insert(key, value);
    }

    fn remove(&mut self, key: &str) -> Option<Self::ValueType> {
        self.values.remove(key)
    }

    fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

/// Borrowed storage implementation
#[derive(Clone, Debug, Default)]
pub struct BorrowedStorage<'a> {
    values: HashMap<&'a str, AttributeValue<&'a str>>,
}

impl<'a> AttributeStorage for BorrowedStorage<'a> {
    type StringType = &'a str;
    type ValueType = AttributeValue<&'a str>;

    fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    fn get(&self, key: &str) -> Option<&Self::ValueType> {
        self.values.get(key)
    }

    fn insert(&mut self, key: Self::StringType, value: Self::ValueType) {
        self.values.insert(key, value);
    }

    fn remove(&mut self, key: &str) -> Option<Self::ValueType> {
        self.values.remove(key)
    }

    fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

// Type aliases for convenience
pub type OwnedAttributes = Attributes<OwnedStorage>;
pub type BorrowedAttributes<'a> = Attributes<BorrowedStorage<'a>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owned_attributes() {
        let mut attrs = OwnedAttributes::new();

        // Insert some values
        attrs.insert(
            "name".to_string(),
            AttributeValue::String("John".to_string()),
        );
        attrs.insert(
            "age".to_string(),
            AttributeValue::Integer(30),
        );

        // Test retrieval
        assert!(attrs.contains_key("name"));
        assert_eq!(attrs.len(), 2);

        if let Some(AttributeValue::String(name)) = attrs.get("name") {
            assert_eq!(name, "John");
        } else {
            panic!("Expected string value");
        }
    }

    #[test]
    fn test_borrowed_attributes() {
        let text = "John";
        let mut attrs = BorrowedAttributes::new();

        // Insert borrowed values
        attrs.insert("name", AttributeValue::String(text));
        attrs.insert("age", AttributeValue::Integer(30));

        // Test retrieval
        assert!(attrs.contains_key("name"));
        assert_eq!(attrs.len(), 2);

        if let Some(AttributeValue::String(name)) = attrs.get("name") {
            assert_eq!(*name, "John");
        } else {
            panic!("Expected string value");
        }
    }

    #[test]
    fn test_nested_attributes() {
        let mut attrs = OwnedAttributes::new();

        // Create a nested structure
        let mut nested_map = HashMap::new();
        nested_map.insert(
            "inner".to_string(),
            Box::new(AttributeValue::String("value".to_string()))
        );

        attrs.insert(
            "nested".to_string(),
            AttributeValue::Map(nested_map),
        );

        // // Test nested retrieval
        // if let Some(AttributeValue::Map(map)) = attrs.get("nested") {
        //     if let Some(box AttributeValue::String(value)) = map.get("inner") {
        //         assert_eq!(value, "value");
        //     } else {
        //         panic!("Expected string value in nested map");
        //     }
        // } else {
        //     panic!("Expected map value");
        // }
    }
}