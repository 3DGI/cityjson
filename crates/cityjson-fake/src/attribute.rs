//! Attribute generation helpers.
//!
//! ```rust
//! use cjfake::attribute::AttributesBuilder;
//!
//! let attributes = AttributesBuilder::new().build();
//! assert!(attributes.is_empty());
//! ```

use cityjson::prelude::*;
use cityjson::v2_0::{AttributeValue, Attributes};
use fake::faker::lorem::raw::Word;
use fake::locales::EN;
use fake::Fake;
use rand::Rng;
use std::collections::HashMap;

/// Builder for creating `Attributes` with random values.
///
/// # Examples
///
/// ```rust
/// use cjfake::attribute::AttributesBuilder;
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
/// use cjfake::attribute::AttributesFaker;
/// use rand::SeedableRng;
///
/// let faker = AttributesFaker::default();
/// let mut rng = rand::prelude::SmallRng::seed_from_u64(1);
/// let attributes = faker.generate(&mut rng);
/// assert!(!attributes.is_empty());
/// ```
pub struct AttributesFaker {
    pub random_keys: bool,
    pub random_values: bool,
    pub max_depth: u8,
    pub min_attrs: u32,
    pub max_attrs: u32,
}

impl Default for AttributesFaker {
    fn default() -> Self {
        Self {
            random_keys: true,
            random_values: true,
            max_depth: 2,
            min_attrs: 3,
            max_attrs: 8,
        }
    }
}

impl AttributesFaker {
    /// Generates random attributes and adds them to the Attributes map
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

    /// Generates a random attribute value
    fn generate_value<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        depth: u8,
    ) -> AttributeValue<OwnedStringStorage> {
        fn leaf_value<R: Rng + ?Sized>(rng: &mut R) -> AttributeValue<OwnedStringStorage> {
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
        }

        if !self.random_values {
            return AttributeValue::String("default".into());
        }

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
