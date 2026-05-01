//! # Coordinate
//!
//! Types and functionality for handling different types of coordinates in `CityJSON`.
//! It implements various coordinate representations needed for 3D city models.
//!
//! ## Overview
//!
//! - [`Coordinate`]: A trait representing any type of coordinate
//! - [`RealWorldCoordinate`]: Floating-point coordinates representing real-world positions
//! - [`UVCoordinate`]: Texture coordinates for mapping textures to surfaces
//!
//! ## Examples
//!
//! ```rust
//! use cityjson_types::v2_0::RealWorldCoordinate;
//!
//! // Create a new coordinate
//! let coord = RealWorldCoordinate::new(10.5, 20.3, 30.7);
//!
//! // Access individual components
//! assert_eq!(coord.x(), 10.5);
//! assert_eq!(coord.y(), 20.3);
//! assert_eq!(coord.z(), 30.7);
//! assert_eq!(coord.to_array(), [10.5, 20.3, 30.7]);
//! ```

use crate::cityjson::core::coordinate::Coordinate;
use std::fmt::{Display, Formatter};

/// A real-world coordinate using `f64` values.
#[repr(C, align(32))]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct RealWorldCoordinate {
    x: f64,
    y: f64,
    z: f64,
}

impl RealWorldCoordinate {
    #[inline]
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    #[must_use]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[inline]
    #[must_use]
    pub fn y(&self) -> f64 {
        self.y
    }

    #[inline]
    #[must_use]
    pub fn z(&self) -> f64 {
        self.z
    }

    #[inline]
    #[must_use]
    pub fn to_array(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}

impl Coordinate for RealWorldCoordinate {}

impl From<[f64; 3]> for RealWorldCoordinate {
    fn from(value: [f64; 3]) -> Self {
        Self::new(value[0], value[1], value[2])
    }
}

impl From<RealWorldCoordinate> for [f64; 3] {
    fn from(value: RealWorldCoordinate) -> Self {
        value.to_array()
    }
}

impl Display for RealWorldCoordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

/// A UV coordinate used for texture mapping, using `f32` values.
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct UVCoordinate {
    u: f32,
    v: f32,
}

impl UVCoordinate {
    #[inline]
    #[must_use]
    pub fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }

    #[inline]
    #[must_use]
    pub fn u(&self) -> f32 {
        self.u
    }

    #[inline]
    #[must_use]
    pub fn v(&self) -> f32 {
        self.v
    }

    #[inline]
    #[must_use]
    pub fn to_array(&self) -> [f32; 2] {
        [self.u, self.v]
    }
}

impl Coordinate for UVCoordinate {}

impl Display for UVCoordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.u, self.v)
    }
}

impl Default for UVCoordinate {
    fn default() -> Self {
        Self { u: 0.0, v: 0.0 }
    }
}
