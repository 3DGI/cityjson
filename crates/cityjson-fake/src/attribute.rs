//! Attribute generation helpers.
//!
//! ```rust
//! use cityjson_fake::attribute::AttributesBuilder;
//!
//! let attributes = AttributesBuilder::new().build();
//! assert!(attributes.is_empty());
//! ```

use cityjson::prelude::*;
use cityjson::v2_0::{AttributeValue, Attributes};
use fake::Fake;
use fake::RngExt;
use fake::faker::lorem::raw::Word;
use fake::locales::EN;
use rand::Rng;
use std::collections::HashMap;

/// Controls whether attribute values are heterogenous (random type per value per object)
/// or homogenous (one fixed scalar type per key, consistent across all `CityObjects`).
#[cfg_attr(feature = "cli", derive(serde::Deserialize))]
#[cfg_attr(feature = "cli", serde(rename_all = "lowercase"))]
#[derive(clap::ValueEnum, Debug, Clone, PartialEq, Default)]
pub enum AttributeValueMode {
    /// Each attribute value is a randomly chosen type; the same key can have different
    /// types across `CityObjects` ("heterogenous values").
    #[default]
    Heterogenous,
    /// A scalar type is pre-assigned to each attribute key and held constant across all
    /// `CityObjects` ("homogenous values"). Scalar types: bool, integer, unsigned, float, string.
    Homogenous,
}

/// A pre-generated type table used in homogenous mode.
/// Each entry is `(attribute_key, designated_scalar_type)`.
#[derive(Debug, Clone)]
pub struct AttributeSchema(pub Vec<(String, ScalarType)>);

/// Scalar leaf types used in homogenous mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScalarType {
    Bool,
    Integer,
    Unsigned,
    Float,
    String,
}

impl ScalarType {
    fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        match rng.random_range(0..5u8) {
            0 => Self::Bool,
            1 => Self::Integer,
            2 => Self::Unsigned,
            3 => Self::Float,
            _ => Self::String,
        }
    }

    pub fn generate_value<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
    ) -> AttributeValue<OwnedStringStorage> {
        match self {
            Self::Bool => AttributeValue::Bool(rng.random_bool(0.5)),
            Self::Integer => AttributeValue::Integer(rng.random_range(-1000..1000)),
            Self::Unsigned => AttributeValue::Unsigned(rng.random_range(0..1000)),
            Self::Float => AttributeValue::Float(rng.random_range(0.0..100.0)),
            Self::String => {
                let word: String = Word(EN).fake_with_rng(rng);
                AttributeValue::String(word)
            }
        }
    }
}

/// Builder for creating `Attributes` with random values.
///
/// # Examples
///
/// ```rust
/// use cityjson_fake::attribute::AttributesBuilder;
///
/// let attributes = AttributesBuilder::new().build();
/// assert!(attributes.is_empty());
/// ```
pub struct AttributesBuilder {
    attributes: Attributes<OwnedStringStorage>,
}

impl Default for AttributesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributesBuilder {
    /// Creates a new `AttributesBuilder`
    #[must_use]
    pub fn new() -> Self {
        Self {
            attributes: Attributes::new(),
        }
    }

    /// Generates random attributes and adds them
    #[must_use]
    pub fn with_random_attributes<R: Rng + ?Sized>(mut self, rng: &mut R) -> Self {
        let faker = AttributesFaker::default();

        let generated = faker.generate(rng);
        for (key, value) in generated.iter() {
            self.attributes.insert(key.clone(), value.clone());
        }

        self
    }

    /// Builds and returns the Attributes
    #[must_use]
    pub fn build(self) -> Attributes<OwnedStringStorage> {
        self.attributes
    }
}

/// Faker for generating random attributes.
///
/// # Examples
///
/// ```rust
/// use cityjson_fake::attribute::AttributesFaker;
/// use rand::SeedableRng;
///
/// let faker = AttributesFaker::default();
/// let mut rng = rand::prelude::SmallRng::seed_from_u64(1);
/// let attributes = faker.generate(&mut rng);
/// assert!(!attributes.is_empty());
/// ```
pub struct AttributesFaker {
    pub random_keys: bool,
    /// Attribute value mode: heterogenous (random types per value per call) or homogenous
    /// (fixed scalar type per key, determined by a pre-generated `AttributeSchema`).
    pub value_mode: AttributeValueMode,
    /// Whether null is a possible attribute value. In heterogenous mode null is included in
    /// the random type pool. In homogenous mode each value has a 1-in-7 chance of being null.
    pub allow_null: bool,
    pub max_depth: u8,
    pub min_attrs: u32,
    pub max_attrs: u32,
}

impl Default for AttributesFaker {
    fn default() -> Self {
        Self {
            random_keys: true,
            value_mode: AttributeValueMode::Heterogenous,
            allow_null: true,
            max_depth: 2,
            min_attrs: 3,
            max_attrs: 8,
        }
    }
}

impl AttributesFaker {
    /// Generates heterogenous attributes (random type per value per call).
    ///
    /// For homogenous mode use [`Self::generate_schema`] + [`Self::generate_from_schema`] instead.
    pub fn generate<R: Rng + ?Sized>(&self, rng: &mut R) -> Attributes<OwnedStringStorage> {
        let mut attributes = Attributes::new();

        let num_attrs = if self.min_attrs >= self.max_attrs {
            self.min_attrs as usize
        } else {
            rng.random_range(self.min_attrs..=self.max_attrs) as usize
        };
        for i in 0..num_attrs {
            let key = if self.random_keys {
                Word(EN).fake_with_rng(rng)
            } else {
                format!("attr_{i}")
            };

            let value = self.generate_value(rng, 0);
            attributes.insert(key, value);
        }

        attributes
    }

    /// Pre-generates the attribute type table for homogenous mode.
    ///
    /// Call once before generating `CityObjects`; pass the result to [`Self::generate_from_schema`].
    pub fn generate_schema<R: Rng + ?Sized>(&self, rng: &mut R) -> AttributeSchema {
        let num_attrs = if self.min_attrs >= self.max_attrs {
            self.min_attrs as usize
        } else {
            rng.random_range(self.min_attrs..=self.max_attrs) as usize
        };
        let entries = (0..num_attrs)
            .map(|i| {
                let key = if self.random_keys {
                    Word(EN).fake_with_rng(rng)
                } else {
                    format!("attr_{i}")
                };
                (key, ScalarType::random(rng))
            })
            .collect();
        AttributeSchema(entries)
    }

    /// Generates homogenous attributes for one object using a pre-generated type table.
    ///
    /// All objects produced from the same `schema` share the same keys and scalar types;
    /// only the concrete values differ. If `allow_null` is set, each value has a 1-in-7
    /// chance of being `Null` instead of its designated scalar type.
    pub fn generate_from_schema<R: Rng + ?Sized>(
        &self,
        schema: &AttributeSchema,
        rng: &mut R,
    ) -> Attributes<OwnedStringStorage> {
        let mut attributes = Attributes::new();
        for (key, scalar_type) in &schema.0 {
            let value = if self.allow_null && rng.random_range(0..7u8) == 0 {
                AttributeValue::Null
            } else {
                scalar_type.generate_value(rng)
            };
            attributes.insert(key.clone(), value);
        }
        attributes
    }

    /// Generates a random attribute value (used in heterogenous mode).
    fn generate_value<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        depth: u8,
    ) -> AttributeValue<OwnedStringStorage> {
        let leaf_value = |rng: &mut R| -> AttributeValue<OwnedStringStorage> {
            if self.allow_null {
                match rng.random_range(0..7u8) {
                    0 => AttributeValue::Null,
                    1 => AttributeValue::Bool(rng.random_bool(0.5)),
                    2 => AttributeValue::Integer(rng.random_range(-1000..1000)),
                    3 => AttributeValue::Unsigned(rng.random_range(0..1000)),
                    4 => AttributeValue::Float(rng.random_range(0.0..100.0)),
                    5 => {
                        let word: String = Word(EN).fake_with_rng(rng);
                        AttributeValue::String(word)
                    }
                    _ => {
                        let word1: String = Word(EN).fake_with_rng(rng);
                        let word2: String = Word(EN).fake_with_rng(rng);
                        AttributeValue::String(format!("{word1} {word2}"))
                    }
                }
            } else {
                match rng.random_range(0..6u8) {
                    0 => AttributeValue::Bool(rng.random_bool(0.5)),
                    1 => AttributeValue::Integer(rng.random_range(-1000..1000)),
                    2 => AttributeValue::Unsigned(rng.random_range(0..1000)),
                    3 => AttributeValue::Float(rng.random_range(0.0..100.0)),
                    4 => {
                        let word: String = Word(EN).fake_with_rng(rng);
                        AttributeValue::String(word)
                    }
                    _ => {
                        let word1: String = Word(EN).fake_with_rng(rng);
                        let word2: String = Word(EN).fake_with_rng(rng);
                        AttributeValue::String(format!("{word1} {word2}"))
                    }
                }
            }
        };

        if depth >= self.max_depth {
            return leaf_value(rng);
        }

        match rng.random_range(0..9u8) {
            0..=6 => leaf_value(rng),
            7 => {
                let len = rng.random_range(1..=3usize);
                AttributeValue::Vec(
                    (0..len)
                        .map(|_| self.generate_value(rng, depth + 1))
                        .collect(),
                )
            }
            _ => {
                let len = rng.random_range(1..=3usize);
                let mut map = HashMap::new();
                for idx in 0..len {
                    let key = if self.random_keys {
                        Word(EN).fake_with_rng(rng)
                    } else {
                        format!("attr_{depth}_{idx}")
                    };
                    map.insert(key, self.generate_value(rng, depth + 1));
                }
                AttributeValue::Map(map)
            }
        }
    }
}
