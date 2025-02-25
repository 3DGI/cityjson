//! # Transform
//!
//! Represents a [Transform object](https://www.cityjson.org/specs/1.1.3/#transform-object).

use std::fmt::{Display, Formatter};
use crate::cityjson;

#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub scale: [f64; 3],
    pub translate: [f64; 3],
}

impl Transform {
    pub fn new() -> Self {
        Self {
            scale: [1.0, 1.0, 1.0],
            translate: [0.0, 0.0, 0.0],
        }
    }
}

impl Display for Transform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(scale: [{}, {}, {}], translate:[{}, {}, {}])",
            self.scale[0],
            self.scale[1],
            self.scale[2],
            self.translate[0],
            self.translate[1],
            self.translate[2]
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl cityjson::transform::Transform for Transform {}
