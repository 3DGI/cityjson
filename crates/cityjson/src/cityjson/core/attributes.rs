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
//! - [`AttributePool`]: The main container for storing attribute key-value pairs in a flattened format
//! - [`OwnedAttributes`]: Type alias for attributes with owned strings
//! - [`BorrowedAttributes`]: Type alias for attributes with borrowed strings
//!
//! ## Architecture: Flattened Structure of Arrays (SoA)
//!
//! The attribute pool uses a Structure of Arrays design that maps directly to columnar
//! storage formats like Parquet. Each attribute type is stored in a separate array,
//! avoiding Rust enum unions which don't serialize cleanly to Parquet.
//!
//! ### Parquet Serialization
//!
//! The flattened design enables efficient Parquet serialization:
//!
//! - **Primitive types** (bool, integer, float, string) → Direct Parquet columns
//! - **Vec** → Parquet LIST type
//! - **Map** → Parquet MAP type
//! - **Type discriminator** → Parquet INT32 enum column
//!
//! This design avoids Parquet union types (poorly supported) while maintaining
//! efficient columnar storage and query performance.
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
//! ### Creating and using the attribute pool
//!
//! ```rust
//! use cityjson::prelude::*;
//!
//! // Create a new attributes pool
//! let mut pool = OwnedAttributePool::new();
//!
//! // Add various types of values
//! let name_id = pool.add_string(
//!     "name".to_string(),
//!     true,  // is_named
//!     "Building A".to_string(),
//!     AttributeOwnerType::CityObject,
//!     None,
//! );
//!
//! let height_id = pool.add_float(
//!     "height".to_string(),
//!     true,
//!     25.5,
//!     AttributeOwnerType::CityObject,
//!     None,
//! );
//!
//! // Retrieve values
//! if let Some(height) = pool.get_float(height_id) {
//!     println!("Building height: {} meters", height);
//! }
//! ```
//!
//! ### Working with nested attributes (Maps and Vecs)
//!
//! ```rust
//! use cityjson::prelude::*;
//! use std::collections::HashMap;
//!
//! let mut pool = OwnedAttributePool::new();
//!
//! // Create a map structure (e.g., address)
//! let street_id = pool.add_string(
//!     "street".to_string(),
//!     true,
//!     "Main St".to_string(),
//!     AttributeOwnerType::Element,
//!     None,
//! );
//!
//! let number_id = pool.add_integer(
//!     "number".to_string(),
//!     true,
//!     123,
//!     AttributeOwnerType::Element,
//!     None,
//! );
//!
//! let mut address_map = HashMap::new();
//! address_map.insert("street".to_string(), street_id);
//! address_map.insert("number".to_string(), number_id);
//!
//! let address_id = pool.add_map(
//!     "address".to_string(),
//!     true,
//!     address_map,
//!     AttributeOwnerType::CityObject,
//!     None,
//! );
//!
//! // Access nested values
//! if let Some(street_attr_id) = pool.get_map_value(address_id, "street") {
//!     if let Some(street) = pool.get_string(street_attr_id) {
//!         println!("Street: {}", street);
//!     }
//! }
//! ```
//!
//! ## Compliance
//!
//! This module implements the attribute storage needed for CityJSON objects
//! as specified in the [CityJSON specification](https://www.cityjson.org/specs/).
//! The flexible design allows for efficiently representing both simple and complex
//! attribute structures while enabling efficient serialization to Parquet.

use crate::prelude::{ResourceId32, ResourceRef};
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};
use std::collections::HashMap;
use std::fmt::{self, Debug, Display, Formatter};

/// Type alias for attribute IDs
pub type AttributeId32 = ResourceId32;

/// Container for attributes using a specific storage strategy.
///
/// Uses a Structure of Arrays (SoA) design that maps directly to Parquet columnar format.
/// Each attribute type is stored in a separate array to avoid Rust enum unions.
#[derive(Debug, Clone)]
pub struct AttributePool<SS: StringStorage, RR: ResourceRef> {
    // Metadata
    keys: Vec<SS::String>, // Attribute keys (empty for unnamed attributes)
    types: Vec<AttributeValueType>, // Type of each attribute
    generations: Vec<u16>, // Generation counter for safety
    is_named: Vec<bool>,   // Whether the attribute has a name

    // Type-specific value arrays (null everywhere except for attributes of that type)
    // These map directly to Parquet columns
    bool_values: Vec<Option<bool>>,
    unsigned_values: Vec<Option<u64>>,
    integer_values: Vec<Option<i64>>,
    float_values: Vec<Option<f64>>,
    string_values: Vec<Option<SS::String>>,
    geometry_values: Vec<Option<RR>>,

    // Owner tracking fields
    owner_types: Vec<AttributeOwnerType>,
    owner_refs: Vec<Option<RR>>,

    // Nested structures (self-referential attribute structure)
    // These map to Parquet LIST and MAP types
    vector_elements: HashMap<usize, Vec<AttributeId32>>, // Parquet LIST
    map_elements: HashMap<usize, HashMap<SS::String, AttributeId32>>, // Parquet MAP

    // Fast lookups
    key_to_index: HashMap<SS::String, usize>, // For named attributes

    // Memory management
    free_list: Vec<usize>, // Indices that can be reused
}

impl<SS: StringStorage, RR: ResourceRef> AttributePool<SS, RR> {
    /// Creates a new empty attribute pool
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            types: Vec::new(),
            generations: Vec::new(),
            is_named: Vec::new(),
            bool_values: Vec::new(),
            unsigned_values: Vec::new(),
            integer_values: Vec::new(),
            float_values: Vec::new(),
            string_values: Vec::new(),
            geometry_values: Vec::new(),
            owner_types: Vec::new(),
            owner_refs: Vec::new(),
            vector_elements: HashMap::new(),
            map_elements: HashMap::new(),
            key_to_index: HashMap::new(),
            free_list: Vec::new(),
        }
    }

    /// Creates a new attribute pool with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            keys: Vec::with_capacity(capacity),
            types: Vec::with_capacity(capacity),
            generations: Vec::with_capacity(capacity),
            is_named: Vec::with_capacity(capacity),
            bool_values: Vec::with_capacity(capacity),
            unsigned_values: Vec::with_capacity(capacity),
            integer_values: Vec::with_capacity(capacity),
            float_values: Vec::with_capacity(capacity),
            string_values: Vec::with_capacity(capacity),
            geometry_values: Vec::with_capacity(capacity),
            owner_types: Vec::with_capacity(capacity),
            owner_refs: Vec::with_capacity(capacity),
            vector_elements: HashMap::new(),
            map_elements: HashMap::new(),
            key_to_index: HashMap::new(),
            free_list: Vec::new(),
        }
    }

    /// Returns the number of attributes in the pool
    pub fn len(&self) -> usize {
        self.keys.len() - self.free_list.len()
    }

    /// Returns true if the pool is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Allocates a new slot or reuses a freed one
    fn allocate_slot(
        &mut self,
        key: SS::String,
        is_named: bool,
        attr_type: AttributeValueType,
        owner_type: AttributeOwnerType,
        owner_ref: Option<RR>,
    ) -> (usize, u16) {
        if let Some(idx) = self.free_list.pop() {
            // Reuse a freed slot
            self.keys[idx] = key.clone();
            self.types[idx] = attr_type;
            self.is_named[idx] = is_named;
            self.owner_types[idx] = owner_type;
            self.owner_refs[idx] = owner_ref;
            self.generations[idx] += 1;
            self.clear_value(idx);

            if is_named {
                self.key_to_index.insert(key.clone(), idx);
            }

            (idx, self.generations[idx])
        } else {
            // Create a new slot
            let idx = self.keys.len();
            self.keys.push(key.clone());
            self.types.push(attr_type);
            self.is_named.push(is_named);
            self.owner_types.push(owner_type);
            self.owner_refs.push(owner_ref);
            self.generations.push(0);

            // Initialize value arrays
            self.bool_values.push(None);
            self.unsigned_values.push(None);
            self.integer_values.push(None);
            self.float_values.push(None);
            self.string_values.push(None);
            self.geometry_values.push(None);

            if is_named {
                self.key_to_index.insert(key, idx);
            }

            (idx, 0)
        }
    }

    /// Clears value at the given index
    fn clear_value(&mut self, idx: usize) {
        self.bool_values[idx] = None;
        self.unsigned_values[idx] = None;
        self.integer_values[idx] = None;
        self.float_values[idx] = None;
        self.string_values[idx] = None;
        self.geometry_values[idx] = None;
        self.vector_elements.remove(&idx);
        self.map_elements.remove(&idx);
    }

    /// Checks if an attribute ID is valid
    pub fn is_valid(&self, id: AttributeId32) -> bool {
        let idx = id.index() as usize;
        idx < self.generations.len() && self.generations[idx] == id.generation()
    }

    /// Gets the index for a key
    fn find_key_index(&self, key: &str) -> Option<usize> {
        self.key_to_index.get(key).copied()
    }

    // === Attribute Creation Methods ===

    /// Adds a null value
    pub fn add_null(
        &mut self,
        key: SS::String,
        is_named: bool,
        owner_type: AttributeOwnerType,
        owner_ref: Option<RR>,
    ) -> AttributeId32 {
        let (idx, generation) = self.allocate_slot(
            key,
            is_named,
            AttributeValueType::Null,
            owner_type,
            owner_ref,
        );
        AttributeId32::new(idx as u32, generation)
    }

    /// Adds a boolean value
    pub fn add_bool(
        &mut self,
        key: SS::String,
        is_named: bool,
        value: bool,
        owner_type: AttributeOwnerType,
        owner_ref: Option<RR>,
    ) -> AttributeId32 {
        let (idx, generation) = self.allocate_slot(
            key,
            is_named,
            AttributeValueType::Bool,
            owner_type,
            owner_ref,
        );
        self.bool_values[idx] = Some(value);
        AttributeId32::new(idx as u32, generation)
    }

    /// Adds an unsigned integer value
    pub fn add_unsigned(
        &mut self,
        key: SS::String,
        is_named: bool,
        value: u64,
        owner_type: AttributeOwnerType,
        owner_ref: Option<RR>,
    ) -> AttributeId32 {
        let (idx, generation) = self.allocate_slot(
            key,
            is_named,
            AttributeValueType::Unsigned,
            owner_type,
            owner_ref,
        );
        self.unsigned_values[idx] = Some(value);
        AttributeId32::new(idx as u32, generation)
    }

    /// Adds a signed integer value
    pub fn add_integer(
        &mut self,
        key: SS::String,
        is_named: bool,
        value: i64,
        owner_type: AttributeOwnerType,
        owner_ref: Option<RR>,
    ) -> AttributeId32 {
        let (idx, generation) = self.allocate_slot(
            key,
            is_named,
            AttributeValueType::Integer,
            owner_type,
            owner_ref,
        );
        self.integer_values[idx] = Some(value);
        AttributeId32::new(idx as u32, generation)
    }

    /// Adds a floating-point value
    pub fn add_float(
        &mut self,
        key: SS::String,
        is_named: bool,
        value: f64,
        owner_type: AttributeOwnerType,
        owner_ref: Option<RR>,
    ) -> AttributeId32 {
        let (idx, generation) = self.allocate_slot(
            key,
            is_named,
            AttributeValueType::Float,
            owner_type,
            owner_ref,
        );
        self.float_values[idx] = Some(value);
        AttributeId32::new(idx as u32, generation)
    }

    /// Adds a string value
    pub fn add_string(
        &mut self,
        key: SS::String,
        is_named: bool,
        value: SS::String,
        owner_type: AttributeOwnerType,
        owner_ref: Option<RR>,
    ) -> AttributeId32 {
        let (idx, generation) = self.allocate_slot(
            key,
            is_named,
            AttributeValueType::String,
            owner_type,
            owner_ref,
        );
        self.string_values[idx] = Some(value);
        AttributeId32::new(idx as u32, generation)
    }

    /// Adds a geometry reference
    pub fn add_geometry(
        &mut self,
        key: SS::String,
        is_named: bool,
        value: RR,
        owner_type: AttributeOwnerType,
        owner_ref: Option<RR>,
    ) -> AttributeId32 {
        let (idx, generation) = self.allocate_slot(
            key,
            is_named,
            AttributeValueType::Geometry,
            owner_type,
            owner_ref,
        );
        self.geometry_values[idx] = Some(value);
        AttributeId32::new(idx as u32, generation)
    }

    /// Adds a vector of attribute references
    ///
    /// Maps to Parquet LIST type during serialization.
    pub fn add_vector(
        &mut self,
        key: SS::String,
        is_named: bool,
        elements: Vec<AttributeId32>,
        owner_type: AttributeOwnerType,
        owner_ref: Option<RR>,
    ) -> AttributeId32 {
        let (idx, generation) = self.allocate_slot(
            key,
            is_named,
            AttributeValueType::Vec,
            owner_type,
            owner_ref,
        );
        self.vector_elements.insert(idx, elements);
        AttributeId32::new(idx as u32, generation)
    }

    /// Adds a map of key-attribute pairs
    ///
    /// Maps to Parquet MAP type during serialization.
    /// Map keys are strings, values are references to other attributes in the pool.
    /// This maintains the flattened structure and supports arbitrary nesting.
    pub fn add_map(
        &mut self,
        key: SS::String,
        is_named: bool,
        elements: HashMap<SS::String, AttributeId32>,
        owner_type: AttributeOwnerType,
        owner_ref: Option<RR>,
    ) -> AttributeId32 {
        let (idx, generation) = self.allocate_slot(
            key,
            is_named,
            AttributeValueType::Map,
            owner_type,
            owner_ref,
        );
        self.map_elements.insert(idx, elements);
        AttributeId32::new(idx as u32, generation)
    }

    // === Attribute Access Methods ===

    /// Gets the type of an attribute
    pub fn get_type(&self, id: AttributeId32) -> Option<AttributeValueType> {
        if !self.is_valid(id) {
            return None;
        }
        Some(self.types[id.index() as usize])
    }

    /// Gets the key of a named attribute
    pub fn get_key(&self, id: AttributeId32) -> Option<&SS::String> {
        if !self.is_valid(id) || !self.is_named[id.index() as usize] {
            return None;
        }
        Some(&self.keys[id.index() as usize])
    }

    /// Gets a boolean value
    pub fn get_bool(&self, id: AttributeId32) -> Option<bool> {
        if !self.is_valid(id) || self.types[id.index() as usize] != AttributeValueType::Bool {
            return None;
        }
        self.bool_values[id.index() as usize]
    }

    /// Gets an unsigned integer value
    pub fn get_unsigned(&self, id: AttributeId32) -> Option<u64> {
        if !self.is_valid(id) || self.types[id.index() as usize] != AttributeValueType::Unsigned {
            return None;
        }
        self.unsigned_values[id.index() as usize]
    }

    /// Gets a signed integer value
    pub fn get_integer(&self, id: AttributeId32) -> Option<i64> {
        if !self.is_valid(id) || self.types[id.index() as usize] != AttributeValueType::Integer {
            return None;
        }
        self.integer_values[id.index() as usize]
    }

    /// Gets a floating-point value
    pub fn get_float(&self, id: AttributeId32) -> Option<f64> {
        if !self.is_valid(id) || self.types[id.index() as usize] != AttributeValueType::Float {
            return None;
        }
        self.float_values[id.index() as usize]
    }

    /// Gets a string value
    pub fn get_string(&self, id: AttributeId32) -> Option<&SS::String> {
        if !self.is_valid(id) || self.types[id.index() as usize] != AttributeValueType::String {
            return None;
        }
        self.string_values[id.index() as usize].as_ref()
    }

    /// Gets a geometry reference value
    pub fn get_geometry(&self, id: AttributeId32) -> Option<RR> {
        if !self.is_valid(id) || self.types[id.index() as usize] != AttributeValueType::Geometry {
            return None;
        }
        self.geometry_values[id.index() as usize]
    }

    /// Gets vector elements
    pub fn get_vector_elements(&self, id: AttributeId32) -> Option<&Vec<AttributeId32>> {
        if !self.is_valid(id) || self.types[id.index() as usize] != AttributeValueType::Vec {
            return None;
        }
        self.vector_elements.get(&(id.index() as usize))
    }

    /// Gets a vector element by index
    pub fn get_vector_element(
        &self,
        id: AttributeId32,
        element_idx: usize,
    ) -> Option<AttributeId32> {
        let elements = self.get_vector_elements(id)?;
        elements.get(element_idx).copied()
    }

    /// Gets the number of elements in a vector
    pub fn get_vector_length(&self, id: AttributeId32) -> Option<usize> {
        let elements = self.get_vector_elements(id)?;
        Some(elements.len())
    }

    /// Gets map elements as a HashMap
    pub fn get_map_elements(
        &self,
        id: AttributeId32,
    ) -> Option<&HashMap<SS::String, AttributeId32>> {
        if !self.is_valid(id) || self.types[id.index() as usize] != AttributeValueType::Map {
            return None;
        }
        self.map_elements.get(&(id.index() as usize))
    }

    /// Gets a map value by key
    pub fn get_map_value(&self, id: AttributeId32, key: &str) -> Option<AttributeId32> {
        let elements = self.get_map_elements(id)?;
        elements.get(key).copied()
    }

    /// Gets the number of entries in a map
    pub fn get_map_size(&self, id: AttributeId32) -> Option<usize> {
        let elements = self.get_map_elements(id)?;
        Some(elements.len())
    }

    /// Returns an iterator over map keys
    pub fn get_map_keys(&self, id: AttributeId32) -> Option<impl Iterator<Item = &SS::String>> {
        let elements = self.get_map_elements(id)?;
        Some(elements.keys())
    }

    /// Returns an iterator over map entries (key-value pairs)
    pub fn get_map_iter(
        &self,
        id: AttributeId32,
    ) -> Option<impl Iterator<Item = (&SS::String, AttributeId32)>> {
        let elements = self.get_map_elements(id)?;
        Some(elements.iter().map(|(k, &v)| (k, v)))
    }

    /// Removes an attribute from the pool
    pub fn remove(&mut self, id: AttributeId32) -> bool {
        if !self.is_valid(id) {
            return false;
        }

        let idx = id.index() as usize;

        // If this is a named attribute, remove from key index
        if self.is_named[idx] {
            self.key_to_index.remove(self.keys[idx].as_ref());
            self.is_named[idx] = false;
        }

        // Clear the value
        self.clear_value(idx);

        // Remove the owners
        self.owner_types[idx] = AttributeOwnerType::None;
        self.owner_refs[idx] = None;

        // Mark the slot as available for reuse
        self.free_list.push(idx);

        true
    }

    /// Removes a resource by key (if it exists)
    pub fn remove_by_key(&mut self, key: &str) -> bool {
        if let Some(idx) = self.find_key_index(key) {
            let id = AttributeId32::new(idx as u32, self.generations[idx]);
            self.remove(id)
        } else {
            false
        }
    }

    /// Gets an attribute ID by key (if it exists and is valid)
    pub fn get_id_by_key(&self, key: &str) -> Option<AttributeId32> {
        let idx = self.find_key_index(key)?;
        let id = AttributeId32::new(idx as u32, self.generations[idx]);
        if self.is_valid(id) { Some(id) } else { None }
    }

    /// Clears all attributes from the pool
    pub fn clear(&mut self) {
        self.keys.clear();
        self.types.clear();
        self.generations.clear();
        self.is_named.clear();
        self.bool_values.clear();
        self.unsigned_values.clear();
        self.integer_values.clear();
        self.float_values.clear();
        self.string_values.clear();
        self.geometry_values.clear();
        self.owner_types.clear();
        self.owner_refs.clear();
        self.vector_elements.clear();
        self.map_elements.clear();
        self.key_to_index.clear();
        self.free_list.clear();
    }
}

impl<SS: StringStorage, RR: ResourceRef> Default for AttributePool<SS, RR> {
    fn default() -> Self {
        Self::new()
    }
}

impl<SS: StringStorage, RR: ResourceRef> Display for AttributePool<SS, RR>
where
    SS::String: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "AttributePool {{")?;
        writeln!(f, "  total_slots: {}", self.keys.len())?;
        writeln!(f, "  active_attributes: {}", self.len())?;
        writeln!(f, "  free_slots: {}", self.free_list.len())?;

        // Count by type
        let mut type_counts = HashMap::new();
        for (idx, attr_type) in self.types.iter().enumerate() {
            if !self.free_list.contains(&idx) {
                *type_counts.entry(*attr_type).or_insert(0) += 1;
            }
        }

        writeln!(f, "  by_type: {{")?;
        for (attr_type, count) in type_counts.iter() {
            writeln!(f, "    {}: {}", attr_type, count)?;
        }
        writeln!(f, "  }}")?;
        writeln!(f, "}}")
    }
}

/// Type alias for attributes pool with owned strings.
pub type OwnedAttributePool = AttributePool<OwnedStringStorage, ResourceId32>;

/// Type alias for attributes pool with borrowed strings.
pub type BorrowedAttributePool<'a> = AttributePool<BorrowedStringStorage<'a>, ResourceId32>;

/// Container for dispatching attribute values.
///
/// This enum is primarily used for building and converting attribute trees.
/// For storage, use `AttributePool` which provides a flattened Structure of Arrays design.
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

/// Type indicator for Attribute values.
///
/// Used in the flattened AttributePool to discriminate between different value types.
/// Maps to a Parquet INT32 enum column during serialization.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeValueType {
    /// Represents a null or undefined value.
    #[default]
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
    /// A geometry. Basically, only used for "address.location", which must be a MultiPoint.
    Geometry,
}

impl Display for AttributeValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Container for attribute references belonging to a specific object.
///
/// This is a lightweight container that holds references (IDs) to attributes
/// stored in the global `AttributePool`. Each CityObject instance would have
/// its own `Attributes` container.
///
/// Note: The RR parameter is kept for API compatibility but not used in the flattened design.
#[derive(Debug, Clone, PartialEq)]
pub struct Attributes<SS: StringStorage> {
    // References to attributes in the global pool
    attributes: HashMap<SS::String, AttributeId32>,
}

impl<SS: StringStorage> Attributes<SS> {
    /// Creates a new empty attributes container
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }

    /// Inserts an attribute reference
    pub fn insert(&mut self, key: SS::String, id: AttributeId32) -> Option<AttributeId32> {
        self.attributes.insert(key, id)
    }

    /// Gets an attribute reference
    pub fn get(&self, key: &str) -> Option<AttributeId32> {
        self.attributes.get(key).copied()
    }

    /// Removes an attribute reference
    pub fn remove(&mut self, key: &str) -> Option<AttributeId32> {
        self.attributes.remove(key)
    }

    /// Returns the number of attributes
    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    /// Returns true if there are no attributes
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    /// Returns true if the attribute container has the key
    pub fn contains_key(&self, key: &str) -> bool {
        self.attributes.contains_key(key)
    }

    /// Returns an iterator over all key-ID pairs
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a SS::String, AttributeId32)> + 'a {
        self.attributes.iter().map(|(k, &v)| (k, v))
    }

    /// Returns an iterator over the keys
    pub fn keys<'a>(&'a self) -> impl Iterator<Item = &'a SS::String> + 'a {
        self.attributes.keys()
    }

    /// Clears all attribute references
    pub fn clear(&mut self) {
        self.attributes.clear();
    }
}

impl<SS: StringStorage> Default for Attributes<SS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<SS: StringStorage> Display for Attributes<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "keys: {}",
            self.attributes
                .keys()
                .map(|k| k.as_ref())
                .collect::<Vec<&str>>()
                .join(",")
        )
    }
}

/// Type alias for attributes with owned strings.
pub type OwnedAttributes = Attributes<OwnedStringStorage>;

/// Type alias for attributes with borrowed strings.
pub type BorrowedAttributes<'a> = Attributes<BorrowedStringStorage<'a>>;

/// Indicates what type of entity owns an attribute
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeOwnerType {
    /// For deleted attributes
    None,
    /// Owned by a CityObject
    CityObject,
    /// Owned by a Semantic surface
    Semantic,
    /// Owned by Metadata
    Metadata,
    /// Owned by the CityModel itself
    CityModel,
    /// For attributes that are part of a vector/array or map
    Element,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_pool_basic() {
        let mut pool = OwnedAttributePool::new();

        // Add different types
        let bool_id = pool.add_bool(
            "active".to_string(),
            true,
            true,
            AttributeOwnerType::CityObject,
            None,
        );

        let int_id = pool.add_integer(
            "floors".to_string(),
            true,
            5,
            AttributeOwnerType::CityObject,
            None,
        );

        let float_id = pool.add_float(
            "height".to_string(),
            true,
            25.5,
            AttributeOwnerType::CityObject,
            None,
        );

        let string_id = pool.add_string(
            "name".to_string(),
            true,
            "Building A".to_string(),
            AttributeOwnerType::CityObject,
            None,
        );

        // Test retrieval
        assert_eq!(pool.get_bool(bool_id), Some(true));
        assert_eq!(pool.get_integer(int_id), Some(5));
        assert_eq!(pool.get_float(float_id), Some(25.5));
        assert_eq!(pool.get_string(string_id), Some(&"Building A".to_string()));

        // Test type checking
        assert_eq!(pool.get_type(bool_id), Some(AttributeValueType::Bool));
        assert_eq!(pool.get_type(int_id), Some(AttributeValueType::Integer));
        assert_eq!(pool.get_type(float_id), Some(AttributeValueType::Float));
        assert_eq!(pool.get_type(string_id), Some(AttributeValueType::String));

        // Test key retrieval
        assert_eq!(pool.get_key(bool_id), Some(&"active".to_string()));
        assert_eq!(pool.get_key(int_id), Some(&"floors".to_string()));
    }

    #[test]
    fn test_attribute_pool_vectors() {
        let mut pool = OwnedAttributePool::new();

        // Create vector elements
        let elem1 = pool.add_integer("".to_string(), false, 1, AttributeOwnerType::Element, None);
        let elem2 = pool.add_integer("".to_string(), false, 2, AttributeOwnerType::Element, None);
        let elem3 = pool.add_integer("".to_string(), false, 3, AttributeOwnerType::Element, None);

        // Create vector
        let vec_id = pool.add_vector(
            "numbers".to_string(),
            true,
            vec![elem1, elem2, elem3],
            AttributeOwnerType::CityObject,
            None,
        );

        // Test vector access
        assert_eq!(pool.get_type(vec_id), Some(AttributeValueType::Vec));
        assert_eq!(pool.get_vector_length(vec_id), Some(3));

        let elem = pool.get_vector_element(vec_id, 0).unwrap();
        assert_eq!(pool.get_integer(elem), Some(1));

        let elem = pool.get_vector_element(vec_id, 1).unwrap();
        assert_eq!(pool.get_integer(elem), Some(2));

        let elem = pool.get_vector_element(vec_id, 2).unwrap();
        assert_eq!(pool.get_integer(elem), Some(3));
    }

    #[test]
    fn test_attribute_pool_maps() {
        let mut pool = OwnedAttributePool::new();

        // Create map elements
        let street = pool.add_string(
            "street".to_string(),
            true,
            "Main St".to_string(),
            AttributeOwnerType::Element,
            None,
        );

        let number = pool.add_integer(
            "number".to_string(),
            true,
            123,
            AttributeOwnerType::Element,
            None,
        );

        let city = pool.add_string(
            "city".to_string(),
            true,
            "Springfield".to_string(),
            AttributeOwnerType::Element,
            None,
        );

        // Create map
        let mut address_map = HashMap::new();
        address_map.insert("street".to_string(), street);
        address_map.insert("number".to_string(), number);
        address_map.insert("city".to_string(), city);

        let map_id = pool.add_map(
            "address".to_string(),
            true,
            address_map,
            AttributeOwnerType::CityObject,
            None,
        );

        // Test map access
        assert_eq!(pool.get_type(map_id), Some(AttributeValueType::Map));
        assert_eq!(pool.get_map_size(map_id), Some(3));

        // Test individual value access
        let street_id = pool.get_map_value(map_id, "street").unwrap();
        assert_eq!(pool.get_string(street_id), Some(&"Main St".to_string()));

        let number_id = pool.get_map_value(map_id, "number").unwrap();
        assert_eq!(pool.get_integer(number_id), Some(123));

        let city_id = pool.get_map_value(map_id, "city").unwrap();
        assert_eq!(pool.get_string(city_id), Some(&"Springfield".to_string()));

        // Test iteration
        let keys: Vec<_> = pool.get_map_keys(map_id).unwrap().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&&"street".to_string()));
        assert!(keys.contains(&&"number".to_string()));
        assert!(keys.contains(&&"city".to_string()));
    }

    #[test]
    fn test_nested_maps_and_vectors() {
        let mut pool = OwnedAttributePool::new();

        // Create coordinates vector
        let lat = pool.add_float(
            "".to_string(),
            false,
            40.7128,
            AttributeOwnerType::Element,
            None,
        );
        let lon = pool.add_float(
            "".to_string(),
            false,
            -74.0060,
            AttributeOwnerType::Element,
            None,
        );
        let coords_vec = pool.add_vector(
            "coordinates".to_string(),
            true,
            vec![lat, lon],
            AttributeOwnerType::Element,
            None,
        );

        // Create address map
        let street = pool.add_string(
            "street".to_string(),
            true,
            "Broadway".to_string(),
            AttributeOwnerType::Element,
            None,
        );

        let mut address_map = HashMap::new();
        address_map.insert("street".to_string(), street);
        address_map.insert("coordinates".to_string(), coords_vec);

        let address_id = pool.add_map(
            "address".to_string(),
            true,
            address_map,
            AttributeOwnerType::CityObject,
            None,
        );

        // Test nested access: address.coordinates[0]
        let coords_id = pool.get_map_value(address_id, "coordinates").unwrap();
        let lat_id = pool.get_vector_element(coords_id, 0).unwrap();
        assert_eq!(pool.get_float(lat_id), Some(40.7128));

        // Test nested access: address.street
        let street_id = pool.get_map_value(address_id, "street").unwrap();
        assert_eq!(pool.get_string(street_id), Some(&"Broadway".to_string()));
    }

    #[test]
    fn test_remove_and_reuse() {
        let mut pool = OwnedAttributePool::new();

        // Add an attribute
        let id1 = pool.add_integer(
            "test".to_string(),
            true,
            42,
            AttributeOwnerType::CityObject,
            None,
        );
        assert_eq!(pool.len(), 1);
        assert_eq!(pool.get_integer(id1), Some(42));

        // Remove it
        assert!(pool.remove(id1));
        assert_eq!(pool.len(), 0);
        assert_eq!(pool.get_integer(id1), None);

        // Add another attribute - should reuse the slot
        let id2 = pool.add_integer(
            "test2".to_string(),
            true,
            99,
            AttributeOwnerType::CityObject,
            None,
        );
        assert_eq!(pool.len(), 1);
        assert_eq!(pool.get_integer(id2), Some(99));

        // Old ID should still be invalid (generation changed)
        assert_eq!(pool.get_integer(id1), None);
    }

    #[test]
    fn test_attribute_container() {
        let mut pool = OwnedAttributePool::new();
        let mut attrs = OwnedAttributes::new();

        // Add attributes to pool
        let name_id = pool.add_string(
            "name".to_string(),
            true,
            "Building A".to_string(),
            AttributeOwnerType::CityObject,
            None,
        );

        let height_id = pool.add_float(
            "height".to_string(),
            true,
            25.5,
            AttributeOwnerType::CityObject,
            None,
        );

        // Link them to the container
        attrs.insert("name".to_string(), name_id);
        attrs.insert("height".to_string(), height_id);

        // Test container operations
        assert_eq!(attrs.len(), 2);
        assert!(attrs.contains_key("name"));
        assert!(attrs.contains_key("height"));

        // Retrieve and verify via pool
        let retrieved_name_id = attrs.get("name").unwrap();
        assert_eq!(
            pool.get_string(retrieved_name_id),
            Some(&"Building A".to_string())
        );
    }

    #[test]
    fn test_display_implementations() {
        let mut pool = OwnedAttributePool::new();

        pool.add_bool(
            "active".to_string(),
            true,
            true,
            AttributeOwnerType::CityObject,
            None,
        );

        pool.add_integer(
            "count".to_string(),
            true,
            42,
            AttributeOwnerType::CityObject,
            None,
        );

        let display_str = format!("{}", pool);
        assert!(display_str.contains("AttributePool"));
        assert!(display_str.contains("active_attributes: 2"));
        assert!(display_str.contains("Bool: 1"));
        assert!(display_str.contains("Integer: 1"));
    }

    #[test]
    fn test_get_by_key() {
        let mut pool = OwnedAttributePool::new();

        let id = pool.add_string(
            "name".to_string(),
            true,
            "Test".to_string(),
            AttributeOwnerType::CityObject,
            None,
        );

        // Test get by key
        let retrieved_id = pool.get_id_by_key("name").unwrap();
        assert_eq!(retrieved_id, id);
        assert_eq!(pool.get_string(retrieved_id), Some(&"Test".to_string()));

        // Test remove by key
        assert!(pool.remove_by_key("name"));
        assert!(pool.get_id_by_key("name").is_none());
    }

    #[test]
    fn test_clear() {
        let mut pool = OwnedAttributePool::new();

        pool.add_integer(
            "a".to_string(),
            true,
            1,
            AttributeOwnerType::CityObject,
            None,
        );

        pool.add_integer(
            "b".to_string(),
            true,
            2,
            AttributeOwnerType::CityObject,
            None,
        );

        assert_eq!(pool.len(), 2);

        pool.clear();

        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
        assert!(pool.get_id_by_key("a").is_none());
    }
}
