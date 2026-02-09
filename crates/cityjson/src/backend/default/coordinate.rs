//! # Coordinate
//!
//! This module provides types and functionality for handling different types of coordinates in `CityJSON`.
//! It implements various coordinate representations needed for 3D city models.
//!
//! ## Overview
//!
//! The coordinate module contains several key components:
//!
//! - [`Coordinate`]: A trait representing any type of coordinate
//! - [`FlexibleCoordinate`]: An enum that can hold either quantized or real-world coordinates
//! - [`QuantizedCoordinate`]: Integer-based coordinates used for storage efficiency
//! - [`RealWorldCoordinate`]: Floating-point coordinates representing real-world positions
//! - [`UVCoordinate`]: Texture coordinates for mapping textures to surfaces
//! - [`Vertices`]: A container for vertex coordinates with type-based size limitations
//!
//! Type aliases are provided for common vertex collection configurations:
//! - [`GeometryVertices16`], [`GeometryVertices32`], [`GeometryVertices64`]: Collections with different index sizes
//! - [`UVVertices16`], [`UVVertices32`], [`UVVertices64`]: Texture coordinate collections
//!
//! ## Usage Examples
//!
//! ### Working with `RealWorldCoordinates`
//!
//! ```rust
//! use cityjson::prelude::*;
//!
//! // Create a new coordinate
//! let coord = RealWorldCoordinate::new(10.5, 20.3, 30.7);
//!
//! // Access individual components
//! assert_eq!(coord.x(), 10.5);
//! assert_eq!(coord.y(), 20.3);
//! assert_eq!(coord.z(), 30.7);
//! ```
//!
//! ### Working with `QuantizedCoordinates`
//!
//! ```rust
//! use cityjson::prelude::*;
//!
//! // Create a quantized coordinate
//! let coord = QuantizedCoordinate::new(1000, 2000, 3000);
//!
//! // Access individual components
//! assert_eq!(coord.x(), 1000);
//! assert_eq!(coord.y(), 2000);
//! assert_eq!(coord.z(), 3000);
//! ```
//!
//! ### Using `FlexibleCoordinate`
//!
//! ```rust
//! use cityjson::prelude::*;
//!
//! // Create different coordinate types
//! let quantized = FlexibleCoordinate::Quantized(QuantizedCoordinate::new(10, 20, 30));
//! let real_world = FlexibleCoordinate::RealWorld(RealWorldCoordinate::new(10.5, 20.5, 30.5));
//!
//! // Default is a quantized coordinate
//! let default_coord = FlexibleCoordinate::default();
//! match default_coord {
//!     FlexibleCoordinate::Quantized(_) => println!("Default is quantized"),
//!     FlexibleCoordinate::RealWorld(_) => println!("Default is real-world"),
//! }
//! ```
//!
//! ### Managing Vertices
//!
//! ```rust
//! use cityjson::prelude::*;
//!
//! // Create a vertex collection
//! let mut vertices = GeometryVertices16::new();
//!
//! // Add vertices and get back indexes
//! let index1 = vertices.push(RealWorldCoordinate::new(0.0, 0.0, 0.0)).unwrap();
//! let index2 = vertices.push(RealWorldCoordinate::new(1.0, 0.0, 0.0)).unwrap();
//! let index3 = vertices.push(RealWorldCoordinate::new(1.0, 1.0, 0.0)).unwrap();
//!
//! // Retrieve vertices by index
//! let v1 = vertices.get(index1).unwrap();
//! assert_eq!(v1.x(), 0.0);
//!
//! // Get the number of vertices
//! assert_eq!(vertices.len(), 3);
//!
//! // Check if collection is empty
//! assert!(!vertices.is_empty());
//! ```
//!
//! ## Implementation Details
//!
//! The module uses generic programming to provide flexible vertex collections
//! that can store different coordinate types while being constrained by the chosen
//! index type (u16, u32, or u64). This allows for efficient memory usage based on
//! the expected number of vertices in a model.

use crate::cityjson::core::vertex::VertexIndex;
use crate::cityjson::core::vertex::VertexRef;
use crate::cityjson::traits::coordinate::Coordinate;
use crate::error::{Error, Result};
use std::marker::PhantomData;

/// A flexible coordinate representation that can be either quantized or real-world.
///
/// `FlexibleCoordinate` provides a way to handle both storage-efficient integer
/// coordinates and precise floating-point coordinates within the same system.
///
/// # Examples
///
/// ```
/// use cityjson::prelude::*;
///
/// // Create a quantized coordinate
/// let quantized = FlexibleCoordinate::Quantized(QuantizedCoordinate::new(100, 200, 300));
///
/// // Create a real-world coordinate
/// let real_world = FlexibleCoordinate::RealWorld(RealWorldCoordinate::new(10.5, 20.5, 30.5));
/// ```
#[repr(C, align(32))]
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum FlexibleCoordinate {
    /// A quantized (integer) coordinate representation
    Quantized(QuantizedCoordinate),
    /// A real-world (floating-point) coordinate representation
    RealWorld(RealWorldCoordinate),
}

impl Default for FlexibleCoordinate {
    /// Creates a default `FlexibleCoordinate` containing a default `QuantizedCoordinate`.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::prelude::*;
    ///
    /// let default_coord = FlexibleCoordinate::default();
    /// ```
    fn default() -> Self {
        FlexibleCoordinate::Quantized(QuantizedCoordinate::default())
    }
}

impl Coordinate for FlexibleCoordinate {}

/// A quantized coordinate using integer values for storage efficiency.
///
/// `QuantizedCoordinate` stores coordinates as integer values, typically used
/// after applying quantization to reduce storage requirements while maintaining
/// acceptable precision for 3D city models.
///
/// # Examples
///
/// ```
/// use cityjson::prelude::*;
///
/// // Create a new quantized coordinate
/// let coord = QuantizedCoordinate::new(1000, 2000, 3000);
///
/// // Access individual components
/// assert_eq!(coord.x(), 1000);
/// assert_eq!(coord.y(), 2000);
/// assert_eq!(coord.z(), 3000);
/// ```
#[repr(C, align(32))]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct QuantizedCoordinate {
    x: i64,
    y: i64,
    z: i64,
}

impl QuantizedCoordinate {
    /// Creates a new `QuantizedCoordinate` with the specified x, y, and z values.
    ///
    /// # Parameters
    ///
    /// * `x` - The x-coordinate as an integer
    /// * `y` - The y-coordinate as an integer
    /// * `z` - The z-coordinate as an integer
    ///
    /// # Returns
    ///
    /// A new `QuantizedCoordinate` instance
    #[inline]
    #[must_use]
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    /// Returns the x-coordinate value.
    ///
    /// # Returns
    ///
    /// The x-coordinate as an i64
    #[inline]
    #[must_use]
    pub fn x(&self) -> i64 {
        self.x
    }

    /// Returns the y-coordinate value.
    ///
    /// # Returns
    ///
    /// The y-coordinate as an i64
    #[inline]
    #[must_use]
    pub fn y(&self) -> i64 {
        self.y
    }

    /// Returns the z-coordinate value.
    ///
    /// # Returns
    ///
    /// The z-coordinate as an i64
    #[inline]
    #[must_use]
    pub fn z(&self) -> i64 {
        self.z
    }
}

impl Coordinate for QuantizedCoordinate {}

/// A real-world coordinate using floating-point values for accuracy.
///
/// `RealWorldCoordinate` stores coordinates as double-precision floating-point values,
/// typically used to represent actual geographic or local coordinate system positions.
///
/// # Examples
///
/// ```
/// use cityjson::prelude::*;
///
/// // Create a new real-world coordinate
/// let coord = RealWorldCoordinate::new(10.5, 20.3, 30.7);
///
/// // Access individual components
/// assert_eq!(coord.x(), 10.5);
/// assert_eq!(coord.y(), 20.3);
/// assert_eq!(coord.z(), 30.7);
/// ```
#[repr(C, align(32))]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct RealWorldCoordinate {
    x: f64,
    y: f64,
    z: f64,
}

impl RealWorldCoordinate {
    /// Creates a new `RealWorldCoordinate` with the specified x, y, and z values.
    ///
    /// # Parameters
    ///
    /// * `x` - The x-coordinate as a double-precision floating-point
    /// * `y` - The y-coordinate as a double-precision floating-point
    /// * `z` - The z-coordinate as a double-precision floating-point
    ///
    /// # Returns
    ///
    /// A new `RealWorldCoordinate` instance
    #[inline]
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Returns the x-coordinate value.
    ///
    /// # Returns
    ///
    /// The x-coordinate as an f64
    #[inline]
    #[must_use]
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Returns the y-coordinate value.
    ///
    /// # Returns
    ///
    /// The y-coordinate as an f64
    #[inline]
    #[must_use]
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Returns the z-coordinate value.
    ///
    /// # Returns
    ///
    /// The z-coordinate as an f64
    #[inline]
    #[must_use]
    pub fn z(&self) -> f64 {
        self.z
    }
}

impl Coordinate for RealWorldCoordinate {}

/// A UV coordinate used for texture mapping.
///
/// `UVCoordinate` stores 2D texture coordinates used to map textures onto surfaces.
/// The u-coordinate typically runs horizontally, and the v-coordinate runs vertically.
///
/// # Examples
///
/// ```
/// use cityjson::prelude::*;
///
/// // Create a UV coordinate
/// let uv = UVCoordinate::new(0.5, 0.25);
///
/// // Access individual components
/// assert_eq!(uv.u(), 0.5);
/// assert_eq!(uv.v(), 0.25);
/// ```
#[repr(C, align(32))]
#[derive(Clone, Debug)]
pub struct UVCoordinate {
    u: f32,
    v: f32,
}

impl UVCoordinate {
    /// Creates a new `UVCoordinate` with the specified u and v values.
    ///
    /// # Parameters
    ///
    /// * `u` - The u-coordinate (horizontal texture coordinate)
    /// * `v` - The v-coordinate (vertical texture coordinate)
    ///
    /// # Returns
    ///
    /// A new `UVCoordinate` instance
    #[inline]
    #[must_use]
    pub fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }

    /// Returns the u-coordinate value.
    ///
    /// # Returns
    ///
    /// The u-coordinate as an f32
    #[inline]
    #[must_use]
    pub fn u(&self) -> f32 {
        self.u
    }

    /// Returns the v-coordinate value.
    ///
    /// # Returns
    ///
    /// The v-coordinate as an f32
    #[inline]
    #[must_use]
    pub fn v(&self) -> f32 {
        self.v
    }
}

impl Coordinate for UVCoordinate {}

impl Default for UVCoordinate {
    /// Creates a default `UVCoordinate` with u=0.0 and v=0.0.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::prelude::*;
    ///
    /// let default_uv = UVCoordinate::default();
    /// assert_eq!(default_uv.u(), 0.0);
    /// assert_eq!(default_uv.v(), 0.0);
    /// ```
    fn default() -> Self {
        Self { u: 0.0, v: 0.0 }
    }
}

/// Container for vertex coordinates with size limited by the chosen index type.
///
/// `Vertices` provides a generic collection for storing coordinates of any type
/// that implements the `Coordinate` trait. The collection size is constrained by
/// the vertex reference type `VR`, which determines the maximum number of vertices.
///
/// # Type Parameters
///
/// * `VR` - The vertex reference type (e.g., u16, u32, u64) that determines the maximum collection size
/// * `V` - The coordinate type that implements the `Coordinate` trait
///
/// # Examples
///
/// ```
/// use cityjson::prelude::*;
///
/// // Create a new vertex collection
/// let mut vertices = GeometryVertices16::new();
///
/// // Add vertices
/// let index1 = vertices.push(RealWorldCoordinate::new(0.0, 0.0, 0.0)).unwrap();
/// let index2 = vertices.push(RealWorldCoordinate::new(1.0, 0.0, 0.0)).unwrap();
///
/// // Retrieve a vertex by index
/// let coord = vertices.get(index1).unwrap();
/// assert_eq!(coord.x(), 0.0);
/// assert_eq!(coord.y(), 0.0);
/// assert_eq!(coord.z(), 0.0);
/// ```
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Vertices<VR: VertexRef, V: Coordinate> {
    coordinates: Vec<V>,
    _phantom: PhantomData<VR>,
}

impl<VR: VertexRef, V: Coordinate> Vertices<VR, V> {
    /// Creates a new empty Vertices collection.
    ///
    /// # Returns
    ///
    /// A new empty `Vertices` collection
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            coordinates: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Creates a new empty Vertices collection with the provided initial capacity.
    ///
    /// # Returns
    ///
    /// A new empty `Vertices` collection
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            coordinates: Vec::with_capacity(capacity),
            _phantom: PhantomData,
        }
    }

    /// Reserves capacity for at least `additional` more elements to be inserted in the
    /// `Vertices`.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    ///
    /// # Errors
    ///
    /// * `VerticesContainerFull` if the current or new capacity is equal or greater
    ///   than the maximum indexable size by the vertex reference type
    ///
    #[inline]
    pub fn reserve(&mut self, additional_capacity: usize) -> Result<()> {
        let max = VR::MAX.try_into().unwrap_or(usize::MAX);
        if self.coordinates.len() >= max || self.coordinates.len() + additional_capacity > max {
            return Err(Error::VerticesContainerFull {
                attempted: self.coordinates.len() + 1,
                maximum: max,
            });
        }
        self.coordinates.reserve(additional_capacity);
        Ok(())
    }

    /// Returns the number of vertices in the collection.
    ///
    /// # Returns
    ///
    /// The number of vertices
    #[must_use]
    pub fn len(&self) -> usize {
        self.coordinates.len()
    }

    /// Adds a new coordinate to the collection.
    ///
    /// # Parameters
    ///
    /// * `coordinate` - The coordinate to add
    ///
    /// # Returns
    ///
    /// A `Result` containing the index of the newly added coordinate or an error
    /// if the collection has reached its maximum capacity
    ///
    /// # Errors
    ///
    /// Returns [`Error::VerticesContainerFull`] when adding the coordinate would exceed the
    /// maximum number of vertices representable by `VR`.
    /// Returns index-conversion errors when converting the coordinate position to `VertexIndex<VR>`.
    pub fn push(&mut self, coordinate: V) -> Result<VertexIndex<VR>> {
        if self.coordinates.len() >= VR::MAX.try_into().unwrap_or(usize::MAX) {
            return Err(Error::VerticesContainerFull {
                attempted: self.coordinates.len() + 1,
                maximum: VR::MAX.try_into().unwrap_or(usize::MAX),
            });
        }
        let index = VertexIndex::<VR>::try_from(self.coordinates.len())?;
        self.coordinates.push(coordinate);
        Ok(index)
    }

    /// Returns a reference to the coordinate at the specified index.
    ///
    /// # Parameters
    ///
    /// * `index` - The vertex index
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the coordinate if the index is valid,
    /// or `None` if the index is out of bounds
    #[inline]
    pub fn get(&self, index: VertexIndex<VR>) -> Option<&V> {
        self.coordinates.get(index.to_usize())
    }

    /// Returns true if the collection is empty.
    ///
    /// # Returns
    ///
    /// `true` if the collection contains no vertices, `false` otherwise
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.coordinates.is_empty()
    }

    /// Returns a slice of all coordinates.
    ///
    /// # Returns
    ///
    /// A slice containing all coordinates in the collection
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[V] {
        &self.coordinates
    }

    /// Clears the collection, removing all vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::prelude::*;
    ///
    /// let mut vertices = GeometryVertices16::new();
    /// vertices.push(RealWorldCoordinate::new(1.0, 2.0, 3.0)).unwrap();
    /// assert_eq!(vertices.len(), 1);
    ///
    /// vertices.clear();
    /// assert_eq!(vertices.len(), 0);
    /// assert!(vertices.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.coordinates.clear();
    }
}

impl<VR: VertexRef, V: Coordinate> Default for Vertices<VR, V> {
    /// Creates a default empty `Vertices` collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::prelude::*;
    ///
    /// let vertices = GeometryVertices32::default();
    /// assert!(vertices.is_empty());
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl<VR: VertexRef, V: Coordinate> From<Vec<V>> for Vertices<VR, V> {
    fn from(value: Vec<V>) -> Self {
        Self {
            coordinates: value,
            _phantom: PhantomData,
        }
    }
}

impl<VR: VertexRef, V: Coordinate> From<&[V]> for Vertices<VR, V> {
    fn from(value: &[V]) -> Self {
        Self {
            coordinates: Vec::from(value),
            _phantom: PhantomData,
        }
    }
}

// Type aliases for convenience
/// A collection of real-world coordinates with u16 indexing (up to 65,535 vertices)
pub type GeometryVertices16 = Vertices<u16, RealWorldCoordinate>;
/// A collection of real-world coordinates with u32 indexing (up to 4,294,967,295 vertices)
pub type GeometryVertices32 = Vertices<u32, RealWorldCoordinate>;
/// A collection of real-world coordinates with u64 indexing (virtually unlimited vertices)
pub type GeometryVertices64 = Vertices<u64, RealWorldCoordinate>;

/// A collection of UV texture coordinates with u16 indexing (up to 65,535 vertices)
pub type UVVertices16 = Vertices<u16, UVCoordinate>;
/// A collection of UV texture coordinates with u32 indexing (up to 4,294,967,295 vertices)
pub type UVVertices32 = Vertices<u32, UVCoordinate>;
/// A collection of UV texture coordinates with u64 indexing (virtually unlimited vertices)
pub type UVVertices64 = Vertices<u64, UVCoordinate>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertices16_limit() {
        let mut vertices = GeometryVertices16::new();

        // Add vertices and get valid indices
        for i in 0..5 {
            let _ = vertices
                .push(RealWorldCoordinate {
                    x: f64::from(i),
                    y: 0.0,
                    z: 0.0,
                })
                .unwrap();
        }

        // Fill up to u16::MAX
        for _ in 5..usize::from(u16::MAX) {
            vertices
                .push(RealWorldCoordinate {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                })
                .unwrap();
        }

        // One more should fail
        let result = vertices.push(RealWorldCoordinate {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_vertices_indexing() {
        let mut vertices = GeometryVertices16::new();
        let idx = vertices
            .push(RealWorldCoordinate {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            })
            .unwrap();

        let coord = vertices.get(idx).unwrap();
        assert_eq!(coord.x(), 1.0);
        assert_eq!(coord.y(), 2.0);
        assert_eq!(coord.z(), 3.0);
    }

    #[test]
    fn test_quantized_coordinate() {
        let coord = QuantizedCoordinate::new(10, 20, 30);
        assert_eq!(coord.x(), 10);
        assert_eq!(coord.y(), 20);
        assert_eq!(coord.z(), 30);

        let default_coord = QuantizedCoordinate::default();
        assert_eq!(default_coord.x(), 0);
        assert_eq!(default_coord.y(), 0);
        assert_eq!(default_coord.z(), 0);
    }

    #[test]
    fn test_real_world_coordinate() {
        let coord = RealWorldCoordinate::new(10.5, 20.5, 30.5);
        assert_eq!(coord.x(), 10.5);
        assert_eq!(coord.y(), 20.5);
        assert_eq!(coord.z(), 30.5);

        let default_coord = RealWorldCoordinate::default();
        assert_eq!(default_coord.x(), 0.0);
        assert_eq!(default_coord.y(), 0.0);
        assert_eq!(default_coord.z(), 0.0);
    }

    #[test]
    fn test_uv_coordinate() {
        let uv = UVCoordinate { u: 0.5, v: 0.75 };
        assert_eq!(uv.u(), 0.5);
        assert_eq!(uv.v(), 0.75);

        let uv = UVCoordinate::new(0.25, 0.35);
        assert_eq!(uv.u(), 0.25);
        assert_eq!(uv.v(), 0.35);
    }

    #[test]
    fn test_flexible_coordinate() {
        let quantized = FlexibleCoordinate::Quantized(QuantizedCoordinate::new(10, 20, 30));
        let real_world = FlexibleCoordinate::RealWorld(RealWorldCoordinate::new(10.5, 20.5, 30.5));

        if let FlexibleCoordinate::Quantized(coord) = quantized {
            assert_eq!(coord.x(), 10);
            assert_eq!(coord.y(), 20);
            assert_eq!(coord.z(), 30);
        } else {
            panic!("Expected Quantized variant");
        }

        if let FlexibleCoordinate::RealWorld(coord) = real_world {
            assert_eq!(coord.x(), 10.5);
            assert_eq!(coord.y(), 20.5);
            assert_eq!(coord.z(), 30.5);
        } else {
            panic!("Expected RealWorld variant");
        }

        // Test default implementation
        let default_coord = FlexibleCoordinate::default();
        match default_coord {
            FlexibleCoordinate::Quantized(coord) => {
                assert_eq!(coord.x(), 0);
                assert_eq!(coord.y(), 0);
                assert_eq!(coord.z(), 0);
            }
            FlexibleCoordinate::RealWorld(_) => {
                panic!("Default should be Quantized variant");
            }
        }
    }

    #[test]
    fn test_vertices_methods() {
        let mut vertices = GeometryVertices32::new();
        assert!(vertices.is_empty());
        assert_eq!(vertices.len(), 0);

        // Add vertices
        let idx1 = vertices
            .push(RealWorldCoordinate::new(1.0, 2.0, 3.0))
            .unwrap();
        let idx2 = vertices
            .push(RealWorldCoordinate::new(4.0, 5.0, 6.0))
            .unwrap();

        // Test length
        assert_eq!(vertices.len(), 2);
        assert!(!vertices.is_empty());

        // Test get
        let coord1 = vertices.get(idx1).unwrap();
        assert_eq!(coord1.x(), 1.0);
        assert_eq!(coord1.y(), 2.0);
        assert_eq!(coord1.z(), 3.0);

        let coord2 = vertices.get(idx2).unwrap();
        assert_eq!(coord2.x(), 4.0);
        assert_eq!(coord2.y(), 5.0);
        assert_eq!(coord2.z(), 6.0);

        // Test as_slice
        let slice = vertices.as_slice();
        assert_eq!(slice.len(), 2);
        assert_eq!(slice[0].x(), 1.0);
        assert_eq!(slice[1].x(), 4.0);
    }

    #[test]
    fn test_vertices32_capacity() {
        let mut vertices = GeometryVertices32::new();

        // We can't test adding u32::MAX elements due to memory constraints,
        // but we can add a reasonable number to verify the basic behavior
        for i in 0..1000 {
            let idx = vertices
                .push(RealWorldCoordinate::new(f64::from(i), 0.0, 0.0))
                .unwrap();
            assert_eq!(vertices.get(idx).unwrap().x(), f64::from(i));
        }

        assert_eq!(vertices.len(), 1000);
    }

    #[test]
    fn test_vertices_default() {
        let vertices: GeometryVertices32 = Vertices::default();
        assert!(vertices.is_empty());
        assert_eq!(vertices.len(), 0);
    }
}
