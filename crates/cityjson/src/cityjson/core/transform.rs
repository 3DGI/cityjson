//! # Transform
//!
//! This module provides types and functionality for handling CityJSON coordinate transformations.
//! It implements the [Transform object](https://www.cityjson.org/specs/1.1.3/#transform-object)
//! as specified in the CityJSON 1.1.3 standard.
//!
//! ## Overview
//!
//! The transform module contains the primary `Transform` struct that handles coordinate
//! transformations in CityJSON. CityJSON uses a mechanism to reduce the file size whereby vertices
//! are represented with integers, and these vertices need to be transformed to obtain their real
//! coordinates.
//!
//! - [`TransformCore`]: The main struct representing a transform object with scale and translation vectors
//!
//! ## Usage Examples
//!
//! ### Creating and using a transform
//!
//! ```rust
//! use cityjson::cityjson::core::transform::TransformCore;
//! use cityjson::cityjson::traits::transform::TransformTrait;
//!
//! // Create a transform with default values
//! let mut transform = TransformCore::default();
//!
//! // Default scale is [1.0, 1.0, 1.0] (no scaling)
//! assert_eq!(transform.scale(), [1.0, 1.0, 1.0]);
//!
//! // Default translation is [0.0, 0.0, 0.0] (no translation)
//! assert_eq!(transform.translate(), [0.0, 0.0, 0.0]);
//!
//! // Set scale and translation values
//! transform.set_scale([0.01, 0.01, 0.01]);
//! transform.set_translate([4424648.79, 5427344.63, 12.0]);
//!
//! // Access the values
//! assert_eq!(transform.scale(), [0.01, 0.01, 0.01]);
//! assert_eq!(transform.translate(), [4424648.79, 5427344.63, 12.0]);
//! ```
//!
//! ### Applying a transform
//!
//! When working with CityJSON files, the transform is typically applied to integer coordinates to
//! get the real-world coordinates:
//!
//! ```rust
//! use cityjson::cityjson::core::transform::TransformCore;
//! use cityjson::cityjson::traits::transform::TransformTrait;
//!
//! // Create a transform
//! let transform = TransformCore::new();
//!
//! // Example: Convert integer coordinates to real coordinates
//! // In a real application, integers would come from CityJSON vertices
//! let integer_coords = [78, 125, 3];
//! let scale = transform.scale();
//! let translate = transform.translate();
//!
//! // Apply the transformation: real_coord = integer_coord * scale + translate
//! let real_x = integer_coords[0] as f64 * scale[0] + translate[0];
//! let real_y = integer_coords[1] as f64 * scale[1] + translate[1];
//! let real_z = integer_coords[2] as f64 * scale[2] + translate[2];
//! ```
//!
//! ## Compliance
//!
//! The `Transform` type in this module is designed to comply with the
//! [CityJSON 1.1.3 specification](https://www.cityjson.org/specs/1.1.3/#transform-object).
//! It implements the required scale and translate properties as defined in the standard.
//!
//! The transformation mechanism is an important feature of CityJSON that helps reduce file sizes
//! while maintaining precision in coordinate values.

use std::fmt::{Display, Formatter};
use crate::cityjson::traits::transform::TransformTrait;

/// Transform.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#transform-object>.
///
/// # Examples
/// ```
/// # use cityjson::cityjson::core::transform::TransformCore;
/// # use cityjson::cityjson::traits::transform::TransformTrait;
/// let mut transform = TransformCore::new();
/// transform.set_scale([2.0, 2.0, 2.0]);
/// transform.set_translate([10.0, 20.0, 30.0]);
///
/// assert_eq!(transform.scale(), [2.0, 2.0, 2.0]);
/// assert_eq!(transform.translate(), [10.0, 20.0, 30.0]);
/// ```
///
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct TransformCore {
    scale: [f64; 3],
    translate: [f64; 3],
}

impl TransformTrait for TransformCore {
    fn new() -> Self {
        Self::default()
    }
    fn scale(&self) -> [f64; 3] {
        self.scale
    }
    fn translate(&self) -> [f64; 3] {
        self.translate
    }
    fn set_scale(&mut self, scale: [f64; 3]) {
        self.scale = scale;
    }
    fn set_translate(&mut self, translate: [f64; 3]) {
        self.translate = translate;
    }
}

impl Display for TransformCore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "scale: [{}, {}, {}], translate:[{}, {}, {}]",
            self.scale[0],
            self.scale[1],
            self.scale[2],
            self.translate[0],
            self.translate[1],
            self.translate[2]
        )
    }
}

impl Default for TransformCore {
    fn default() -> Self {
        Self {
            scale: [1.0, 1.0, 1.0],
            translate: [0.0, 0.0, 0.0],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn display() {
        let mut transform = TransformCore::new();
        transform.set_scale([1.5, 2.0, 2.5]);
        transform.set_translate([10.0, 20.0, 30.0]);
        println!("Transform: {}", transform);
    }
}
