//! # Nested Boundary Representations
//!
//! This module provides nested representations of CityJSON boundaries that directly map to the
//! JSON structure defined in the CityJSON specification. These are primarily used for serialization
//! and deserialization, while the flattened representation is used for internal processing.
//!
//! ## CityJSON Compliance
//!
//! The nested representations in this module align directly with the CityJSON specification for
//! geometry boundaries. For example, a MultiSurface in CityJSON is an array of surfaces, where
//! each surface is an array of rings, and each ring is an array of vertex indices.
//!
//! ## Type Aliases
//!
//! This module provides type aliases for different vertex reference types (u16, u32, u64) to
//! accommodate different application needs:
//!
//! * 16-bit indices (u16): Suitable for small models with fewer than 65,536 vertices.
//! * 32-bit indices (u32): Suitable for most models, supporting up to ~4.3 billion vertices.
//! * 64-bit indices (u64): Suitable for very large models with more than 4.3 billion vertices.
//!
//! ## Conversion
//!
//! The module provides implementations of `From<NestedType> for Boundary<T>` for converting
//! from nested to flattened representations. The parent module provides methods like
//! `to_nested_multi_point()` for converting from flattened to nested representations.
//!
//! ## Examples
//!
//! ```
//! use cityjson::cityjson::core::boundary::Boundary;
//! use cityjson::cityjson::core::boundary::nested::*;
//!
//! // Create a nested multi-surface
//! let multi_surface: BoundaryNestedMultiOrCompositeSurface32 = vec![
//!     // First surface with one ring
//!     vec![vec![0, 1, 2, 0]],
//!     // Second surface with two rings (outer and inner)
//!     vec![vec![3, 4, 5, 3], vec![6, 7, 8, 6]],
//! ];
//!
//! // Convert to flattened representation
//! let boundary: Boundary<u32> = multi_surface.clone().into();
//!
//! // Convert back to nested representation
//! let multi_surface_again = boundary.to_nested_multi_or_composite_surface().unwrap();
//! assert_eq!(multi_surface_again, multi_surface);
//! ```

use crate::cityjson::core::boundary::Boundary;
use crate::cityjson::core::vertex::VertexIndex;
use crate::cityjson::core::vertex::VertexRef;

// Type aliases for u16
/// A collection of points (vertex indices) for a model with 16-bit indices
pub type BoundaryNestedMultiPoint16 = Vec<u16>;
/// A collection of linestrings for a model with 16-bit indices
pub type BoundaryNestedMultiLineString16 = Vec<BoundaryNestedMultiPoint16>;
/// A collection of surfaces (or composite surface) for a model with 16-bit indices
pub type BoundaryNestedMultiOrCompositeSurface16 = Vec<BoundaryNestedMultiLineString16>;
/// A solid represented as shells for a model with 16-bit indices
pub type BoundaryNestedSolid16 = Vec<BoundaryNestedMultiOrCompositeSurface16>;
/// A collection of solids (or composite solid) for a model with 16-bit indices
pub type BoundaryNestedMultiOrCompositeSolid16 = Vec<BoundaryNestedSolid16>;

// Type aliases for u32
/// A collection of points (vertex indices) for a model with 32-bit indices
pub type BoundaryNestedMultiPoint32 = Vec<u32>;
/// A collection of linestrings for a model with 32-bit indices
pub type BoundaryNestedMultiLineString32 = Vec<BoundaryNestedMultiPoint32>;
/// A collection of surfaces (or composite surface) for a model with 32-bit indices
pub type BoundaryNestedMultiOrCompositeSurface32 = Vec<BoundaryNestedMultiLineString32>;
/// A solid represented as shells for a model with 32-bit indices
pub type BoundaryNestedSolid32 = Vec<BoundaryNestedMultiOrCompositeSurface32>;
/// A collection of solids (or composite solid) for a model with 32-bit indices
pub type BoundaryNestedMultiOrCompositeSolid32 = Vec<BoundaryNestedSolid32>;

// Type aliases for u64
/// A collection of points (vertex indices) for a model with 64-bit indices
pub type BoundaryNestedMultiPoint64 = Vec<u64>;
/// A collection of linestrings for a model with 64-bit indices
pub type BoundaryNestedMultiLineString64 = Vec<BoundaryNestedMultiPoint64>;
/// A collection of surfaces (or composite surface) for a model with 64-bit indices
pub type BoundaryNestedMultiOrCompositeSurface64 = Vec<BoundaryNestedMultiLineString64>;
/// A solid represented as shells for a model with 64-bit indices
pub type BoundaryNestedSolid64 = Vec<BoundaryNestedMultiOrCompositeSurface64>;
/// A collection of solids (or composite solid) for a model with 64-bit indices
pub type BoundaryNestedMultiOrCompositeSolid64 = Vec<BoundaryNestedSolid64>;

// Generic type aliases (for use in trait implementations)
/// A collection of points (vertex indices) with generic index type
pub type BoundaryNestedMultiPoint<T> = Vec<T>;
/// A collection of linestrings with generic index type
pub type BoundaryNestedMultiLineString<T> = Vec<BoundaryNestedMultiPoint<T>>;
/// A collection of surfaces (or composite surface) with generic index type
pub type BoundaryNestedMultiOrCompositeSurface<T> = Vec<BoundaryNestedMultiLineString<T>>;
/// A solid represented as shells with generic index type
pub type BoundaryNestedSolid<T> = Vec<BoundaryNestedMultiOrCompositeSurface<T>>;
/// A collection of solids (or composite solid) with generic index type
pub type BoundaryNestedMultiOrCompositeSolid<T> = Vec<BoundaryNestedSolid<T>>;

impl<T: VertexRef> From<BoundaryNestedMultiPoint<T>> for Boundary<T> {
    /// Converts a nested MultiPoint representation to a flattened Boundary.
    ///
    /// # Parameters
    ///
    /// * `value` - A vector of vertex indices representing a MultiPoint
    ///
    /// # Returns
    ///
    /// A flattened `Boundary<T>` representation of the MultiPoint
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::cityjson::core::boundary::Boundary;
    /// use cityjson::cityjson::core::boundary::nested::BoundaryNestedMultiPoint32;
    ///
    /// let multi_point: BoundaryNestedMultiPoint32 = vec![0, 1, 2, 3];
    /// let boundary: Boundary<u32> = multi_point.into();
    /// ```
    fn from(value: BoundaryNestedMultiPoint<T>) -> Self {
        if value.is_empty() {
            Self::default()
        } else {
            Self {
                vertices: value.iter().map(|v| VertexIndex::new(*v)).collect(),
                ..Self::default()
            }
        }
    }
}

impl<T: VertexRef> From<BoundaryNestedMultiLineString<T>> for Boundary<T> {
    /// Converts a nested MultiLineString representation to a flattened Boundary.
    ///
    /// # Parameters
    ///
    /// * `value` - A vector of linestrings, each a vector of vertex indices
    ///
    /// # Returns
    ///
    /// A flattened `Boundary<T>` representation of the MultiLineString
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::cityjson::core::boundary::Boundary;
    /// use cityjson::cityjson::core::boundary::nested::BoundaryNestedMultiLineString32;
    ///
    /// let multi_linestring: BoundaryNestedMultiLineString32 = vec![
    ///     vec![0, 1, 2],
    ///     vec![3, 4, 5]
    /// ];
    /// let boundary: Boundary<u32> = multi_linestring.into();
    /// ```
    fn from(value: BoundaryNestedMultiLineString<T>) -> Self {
        if value.is_empty() {
            Self::default()
        } else {
            let mut vertices = Vec::new();
            let mut rings = Vec::with_capacity(value.len());
            let mut ring_start = VertexIndex::new(T::zero());
            for ring in &value {
                rings.push(ring_start);
                for vertex in ring {
                    vertices.push(VertexIndex::new(*vertex));
                    ring_start += VertexIndex::new(T::one());
                }
            }
            Self {
                vertices,
                rings,
                ..Self::default()
            }
        }
    }
}

impl<T: VertexRef> From<BoundaryNestedMultiOrCompositeSurface<T>> for Boundary<T> {
    /// Converts a nested MultiSurface or CompositeSurface representation to a flattened Boundary.
    ///
    /// # Parameters
    ///
    /// * `value` - A vector of surfaces, each a vector of rings, each a vector of vertex indices
    ///
    /// # Returns
    ///
    /// A flattened `Boundary<T>` representation of the MultiSurface or CompositeSurface
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::cityjson::core::boundary::Boundary;
    /// use cityjson::cityjson::core::boundary::nested::BoundaryNestedMultiOrCompositeSurface32;
    ///
    /// let multi_surface: BoundaryNestedMultiOrCompositeSurface32 = vec![
    ///     vec![vec![0, 1, 2, 0]], // First surface with one ring
    ///     vec![vec![3, 4, 5, 3], vec![6, 7, 8, 6]] // Second surface with two rings
    /// ];
    /// let boundary: Boundary<u32> = multi_surface.into();
    /// ```
    fn from(value: BoundaryNestedMultiOrCompositeSurface<T>) -> Self {
        if value.is_empty() {
            return Self::default();
        }

        let mut boundary = Self::with_capacity(
            value
                .iter()
                .map(|surface| surface.iter().map(|ring| ring.len()).sum::<usize>())
                .sum::<usize>(),
            value.iter().map(|surface| surface.len()).sum::<usize>(),
            value.len(),
            0,
            0,
        );

        let mut vertex_idx = VertexIndex::new(T::zero());

        for surface in value {
            boundary
                .surfaces
                .push(VertexIndex::<T>::try_from(boundary.rings.len()).unwrap());

            for ring in surface {
                boundary.rings.push(vertex_idx);
                for vertex in ring {
                    boundary.vertices.push(VertexIndex::new(vertex));
                    vertex_idx += VertexIndex::new(T::one());
                }
            }
        }

        boundary
    }
}

impl<T: VertexRef> From<BoundaryNestedSolid<T>> for Boundary<T> {
    /// Converts a nested Solid representation to a flattened Boundary.
    ///
    /// # Parameters
    ///
    /// * `value` - A vector of shells, each a vector of surfaces, each a vector of rings,
    ///   each a vector of vertex indices
    ///
    /// # Returns
    ///
    /// A flattened `Boundary<T>` representation of the Solid
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::cityjson::core::boundary::Boundary;
    /// use cityjson::cityjson::core::boundary::nested::BoundaryNestedSolid32;
    ///
    /// // A simplified solid with one shell containing one surface with one ring
    /// let solid: BoundaryNestedSolid32 = vec![
    ///     vec![vec![vec![0, 1, 2, 0]]]
    /// ];
    /// let boundary: Boundary<u32> = solid.into();
    /// ```
    fn from(value: BoundaryNestedSolid<T>) -> Self {
        if value.is_empty() {
            return Self::default();
        }

        // Pre-calculate capacities
        let vertices_cap = value
            .iter()
            .map(|shell| {
                shell
                    .iter()
                    .map(|surface| surface.iter().map(|ring| ring.len()).sum::<usize>())
                    .sum::<usize>()
            })
            .sum::<usize>();

        let rings_cap = value
            .iter()
            .map(|shell| shell.iter().map(|surface| surface.len()).sum::<usize>())
            .sum::<usize>();

        let surfaces_cap = value.iter().map(|shell| shell.len()).sum::<usize>();

        let mut boundary =
            Self::with_capacity(vertices_cap, rings_cap, surfaces_cap, value.len(), 0);

        let mut vertex_idx = VertexIndex::new(T::zero());

        for shell in value {
            boundary
                .shells
                .push(VertexIndex::<T>::try_from(boundary.surfaces.len()).unwrap());

            for surface in shell {
                boundary
                    .surfaces
                    .push(VertexIndex::<T>::try_from(boundary.rings.len()).unwrap());

                for ring in surface {
                    boundary.rings.push(vertex_idx);
                    for vertex in ring {
                        boundary.vertices.push(VertexIndex::new(vertex));
                        vertex_idx += VertexIndex::new(T::one());
                    }
                }
            }
        }

        boundary
    }
}

impl<T: VertexRef> From<BoundaryNestedMultiOrCompositeSolid<T>> for Boundary<T> {
    /// Converts a nested MultiSolid or CompositeSolid representation to a flattened Boundary.
    ///
    /// # Parameters
    ///
    /// * `value` - A vector of solids, each a vector of shells, each a vector of surfaces,
    ///   each a vector of rings, each a vector of vertex indices
    ///
    /// # Returns
    ///
    /// A flattened `Boundary<T>` representation of the MultiSolid or CompositeSolid
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::cityjson::core::boundary::Boundary;
    /// use cityjson::cityjson::core::boundary::nested::BoundaryNestedMultiOrCompositeSolid32;
    ///
    /// // A simplified multi-solid with two solids, each with minimal structure
    /// let multi_solid: BoundaryNestedMultiOrCompositeSolid32 = vec![
    ///     vec![vec![vec![vec![0, 1, 2, 0]]]],  // First solid
    ///     vec![vec![vec![vec![3, 4, 5, 3]]]]   // Second solid
    /// ];
    /// let boundary: Boundary<u32> = multi_solid.into();
    /// ```
    fn from(value: BoundaryNestedMultiOrCompositeSolid<T>) -> Self {
        if value.is_empty() {
            return Self::default();
        }

        // Pre-calculate capacities
        let vertices_cap = value
            .iter()
            .map(|solid| {
                solid
                    .iter()
                    .map(|shell| {
                        shell
                            .iter()
                            .map(|surface| surface.iter().map(|ring| ring.len()).sum::<usize>())
                            .sum::<usize>()
                    })
                    .sum::<usize>()
            })
            .sum::<usize>();

        let rings_cap = value
            .iter()
            .map(|solid| {
                solid
                    .iter()
                    .map(|shell| shell.iter().map(|surface| surface.len()).sum::<usize>())
                    .sum::<usize>()
            })
            .sum::<usize>();

        let surfaces_cap = value
            .iter()
            .map(|solid| solid.iter().map(|shell| shell.len()).sum::<usize>())
            .sum::<usize>();

        let shells_cap = value.iter().map(|solid| solid.len()).sum::<usize>();

        let mut boundary = Self::with_capacity(
            vertices_cap,
            rings_cap,
            surfaces_cap,
            shells_cap,
            value.len(),
        );

        let mut vertex_idx = VertexIndex::new(T::zero());

        for solid in value {
            boundary
                .solids
                .push(VertexIndex::<T>::try_from(boundary.shells.len()).unwrap());

            for shell in solid {
                boundary
                    .shells
                    .push(VertexIndex::<T>::try_from(boundary.surfaces.len()).unwrap());

                for surface in shell {
                    boundary
                        .surfaces
                        .push(VertexIndex::<T>::try_from(boundary.rings.len()).unwrap());

                    for ring in surface {
                        boundary.rings.push(vertex_idx);
                        for vertex in ring {
                            boundary.vertices.push(VertexIndex::new(vertex));
                            vertex_idx += VertexIndex::new(T::one());
                        }
                    }
                }
            }
        }

        boundary
    }
}
