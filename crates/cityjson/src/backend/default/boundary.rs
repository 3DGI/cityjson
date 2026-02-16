//! # Boundary Representation for `CityJSON` Geometries
//!
//! This module provides representations and utilities for working with `CityJSON` geometry boundaries,
//! supporting both a memory-efficient flattened representation and a nested representation that
//! aligns with the `CityJSON` specification.
//!
//! ## `CityJSON` Compliance
//!
//! The boundary representations in this module comply with the
//! [CityJSON specification](https://www.cityjson.org/specs/) for geometry boundaries. The
//! nested representation directly maps to the JSON structure defined in the specification,
//! while the flattened representation provides an optimized internal format.
//!
//! ## Nested vs. Flattened Representations
//!
//! ### Nested Representation
//!
//! `CityJSON` defines geometry boundaries using nested arrays in JSON. For example, a `MultiSurface`
//! is represented as an array of surfaces, where each surface is an array of rings, and each ring
//! is an array of vertex indices. This nested structure is intuitive and directly maps to the
//! JSON representation, but can be inefficient for processing and memory usage due to the overhead
//! of nested vectors.
//!
//! The `nested` module provides types that match this structure:
//! - `BoundaryNestedMultiPoint<T>`
//! - `BoundaryNestedMultiLineString<T>`
//! - `BoundaryNestedMultiOrCompositeSurface<T>`
//! - `BoundaryNestedSolid<T>`
//! - `BoundaryNestedMultiOrCompositeSolid<T>`
//!
//! ### Flattened Representation
//!
//! The `Boundary<VR>` struct provides a memory-efficient "flattened" representation that uses a
//! series of offset indices to navigate through a single, contiguous array. This approach:
//! - Reduces memory overhead from nested vectors
//! - Improves cache locality for better performance
//! - Simplifies operations on the geometry
//!
//! For example, a `MultiSurface` is represented using:
//! - A single vector of vertex indices
//! - A vector of ring indices (pointing into the vertex vector)
//! - A vector of surface indices (pointing into the ring vector)
//!
//! ## Usage
//!
//! The nested representation is primarily used for serialization/deserialization to/from `CityJSON`,
//! while the flattened representation is used for internal processing within cityjson-rs. The module
//! provides methods to convert between the two representations.
//!
//! ```rust
//! use cityjson::prelude::*;
//! use cityjson::cityjson::core::boundary::nested::BoundaryNestedMultiPoint32;
//!
//! // Create a nested representation of a MultiPoint
//! let multi_point: BoundaryNestedMultiPoint32 = vec![0, 1, 2, 3];
//!
//! // Convert to flattened representation
//! let boundary: Boundary<u32> = multi_point.into();
//!
//! // Check boundary type
//! assert_eq!(boundary.check_type(), BoundaryType::MultiPoint);
//!
//! // Convert back to nested representation
//! let multi_point_again = boundary.to_nested_multi_point().unwrap();
//! assert_eq!(multi_point_again, vec![0, 1, 2, 3]);
//! ```

pub mod nested;

use crate::cityjson::core::boundary::nested::{
    BoundaryNestedMultiLineString, BoundaryNestedMultiOrCompositeSolid,
    BoundaryNestedMultiOrCompositeSurface, BoundaryNestedMultiPoint, BoundaryNestedSolid,
};
use crate::cityjson::core::vertex::VertexRef;
use crate::cityjson::core::vertex::{RawVertexView, VertexIndex};
use crate::error;

/// A generic Boundary type that can represent any `CityJSON` boundary.
///
/// This structure provides an efficient, flattened representation of geometry boundaries,
/// optimized for memory usage and processing. It can represent any `CityJSON` geometry type,
/// from `MultiPoint` to `MultiSolid`.
///
/// The `Boundary` uses a series of indices and offsets to navigate through a structure that
/// would traditionally be represented using nested arrays. Instead of nesting, each level
/// of the hierarchy (vertices, rings, surfaces, shells, solids) is stored in a flat array,
/// with indices into the next level down.
///
/// # Type Parameters
///
/// * `VR` - The vertex reference type (e.g., u16, u32, u64) that determines the maximum
///   number of vertices, rings, etc. that can be indexed
///
/// # Example
///
/// ```
/// use cityjson::prelude::*;
/// use cityjson::cityjson::core::boundary::nested::BoundaryNestedMultiLineString32;
///
/// // Create a nested representation of a multi-linestring
/// let multi_linestring: BoundaryNestedMultiLineString32 = vec![
///     vec![0, 1, 2],       // First linestring
///     vec![3, 4, 5, 6],    // Second linestring
/// ];
///
/// // Convert to flattened representation
/// let boundary: Boundary<u32> = multi_linestring.try_into().unwrap();
///
/// // Check the boundary type
/// assert_eq!(boundary.check_type(), BoundaryType::MultiLineString);
///
/// // Convert back to nested representation
/// let multi_linestring_again = boundary.to_nested_multi_linestring().unwrap();
/// assert_eq!(multi_linestring_again, vec![vec![0, 1, 2], vec![3, 4, 5, 6]]);
/// ```
#[repr(C)]
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[allow(unused)]
pub struct Boundary<VR: VertexRef> {
    /// Vertex indices that point to the global Vertices buffer.
    pub(crate) vertices: Vec<VertexIndex<VR>>,
    /// Vertex offsets that mark the start of each ring. The values point to this Boundary's vertices.
    pub(crate) rings: Vec<VertexIndex<VR>>,
    /// Ring offsets that mark the start of each surface. The values point to this Boundary's rings.
    pub(crate) surfaces: Vec<VertexIndex<VR>>,
    /// Surface offsets that mark the start of each shell. The values point to this Boundary's surfaces.
    pub(crate) shells: Vec<VertexIndex<VR>>,
    /// Shell offsets that mark the start of each solid. The values point to this Boundary's shells.
    pub(crate) solids: Vec<VertexIndex<VR>>,
}

/// Columnar representation of a [`Boundary`].
///
/// Each field is a flat buffer with offsets to the next level.
#[derive(Debug, Clone, Copy)]
pub struct BoundaryColumnar<'a, VR: VertexRef> {
    pub vertices: &'a [VertexIndex<VR>],
    pub ring_offsets: &'a [VertexIndex<VR>],
    pub surface_offsets: &'a [VertexIndex<VR>],
    pub shell_offsets: &'a [VertexIndex<VR>],
    pub solid_offsets: &'a [VertexIndex<VR>],
}

impl<VR: VertexRef> Boundary<VR> {
    /// Creates a new empty boundary.
    ///
    /// # Returns
    ///
    /// A new `Boundary` instance with empty vectors.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::prelude::*;
    ///
    /// let boundary: Boundary<u32> = Boundary::new();
    /// assert!(boundary.is_consistent());
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new boundary with the specified capacity for each vector.
    ///
    /// # Parameters
    ///
    /// * `vertices` - Capacity for the vertices vector
    /// * `rings` - Capacity for the rings vector
    /// * `surfaces` - Capacity for the surfaces vector
    /// * `shells` - Capacity for the shells vector
    /// * `solids` - Capacity for the solids vector
    ///
    /// # Returns
    ///
    /// A new `Boundary` instance with pre-allocated vectors.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::prelude::*;
    ///
    /// // Create a boundary with pre-allocated capacity
    /// let boundary: Boundary<u32> = Boundary::with_capacity(
    ///     100, // space for 100 vertices
    ///     20,  // space for 20 rings
    ///     10,  // space for 10 surfaces
    ///     2,   // space for 2 shells
    ///     1,   // space for 1 solid
    /// );
    /// assert!(boundary.is_consistent());
    /// ```
    #[inline]
    #[must_use]
    pub fn with_capacity(
        vertices: usize,
        rings: usize,
        surfaces: usize,
        shells: usize,
        solids: usize,
    ) -> Self {
        Self {
            vertices: Vec::with_capacity(vertices),
            rings: Vec::with_capacity(rings),
            surfaces: Vec::with_capacity(surfaces),
            shells: Vec::with_capacity(shells),
            solids: Vec::with_capacity(solids),
        }
    }

    #[must_use]
    pub fn vertices_raw(&self) -> RawVertexView<'_, VR> {
        RawVertexView(&self.vertices)
    }

    #[must_use]
    pub fn rings_raw(&self) -> RawVertexView<'_, VR> {
        RawVertexView(&self.rings)
    }

    #[must_use]
    pub fn surfaces_raw(&self) -> RawVertexView<'_, VR> {
        RawVertexView(&self.surfaces)
    }

    #[must_use]
    pub fn shells_raw(&self) -> RawVertexView<'_, VR> {
        RawVertexView(&self.shells)
    }

    #[must_use]
    pub fn solids_raw(&self) -> RawVertexView<'_, VR> {
        RawVertexView(&self.solids)
    }

    /// Exports this boundary into a columnar view suitable for serializers.
    #[inline]
    #[must_use]
    pub fn to_columnar(&self) -> BoundaryColumnar<'_, VR> {
        BoundaryColumnar {
            vertices: &self.vertices,
            ring_offsets: &self.rings,
            surface_offsets: &self.surfaces,
            shell_offsets: &self.shells,
            solid_offsets: &self.solids,
        }
    }

    #[inline]
    #[must_use]
    pub fn vertices(&self) -> &[VertexIndex<VR>] {
        &self.vertices
    }

    /// Replaces items of the container with elements from the given iterator
    pub fn set_vertices_from_iter<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = VertexIndex<VR>>,
    {
        self.vertices = iter.into_iter().collect();
    }

    #[inline]
    #[must_use]
    pub fn rings(&self) -> &[VertexIndex<VR>] {
        &self.rings
    }

    /// Replaces items of the container with elements from the given iterator
    pub fn set_rings_from_iter<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = VertexIndex<VR>>,
    {
        self.rings = iter.into_iter().collect();
    }

    #[inline]
    #[must_use]
    pub fn surfaces(&self) -> &[VertexIndex<VR>] {
        &self.surfaces
    }

    /// Replaces items of the container with elements from the given iterator
    pub fn set_surfaces_from_iter<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = VertexIndex<VR>>,
    {
        self.surfaces = iter.into_iter().collect();
    }

    #[inline]
    #[must_use]
    pub fn shells(&self) -> &[VertexIndex<VR>] {
        &self.shells
    }

    /// Replaces items of the container with elements from the given iterator
    pub fn set_shells_from_iter<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = VertexIndex<VR>>,
    {
        self.shells = iter.into_iter().collect();
    }

    #[inline]
    #[must_use]
    pub fn solids(&self) -> &[VertexIndex<VR>] {
        &self.solids
    }

    /// Replaces items of the container with elements from the given iterator
    pub fn set_solids_from_iter<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = VertexIndex<VR>>,
    {
        self.solids = iter.into_iter().collect();
    }

    /// Converts to a nested `MultiPoint` boundary representation.
    ///
    /// This method converts the flattened boundary to a nested `MultiPoint` representation
    /// if the boundary can be interpreted as a `MultiPoint`.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the nested `MultiPoint` representation or an error
    /// if the boundary cannot be interpreted as a `MultiPoint`.
    ///
    /// # Errors
    ///
    /// Returns [`error::Error::IncompatibleBoundary`] when this boundary is not a
    /// `MultiPoint`.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::prelude::*;
    /// use cityjson::cityjson::core::boundary::nested::BoundaryNestedMultiPoint32;
    ///
    /// // Create a boundary from a MultiPoint
    /// let multi_point: BoundaryNestedMultiPoint32 = vec![0, 1, 2, 3];
    /// let boundary: Boundary<u32> = multi_point.into();
    ///
    /// // Convert back to MultiPoint
    /// let nested = boundary.to_nested_multi_point().unwrap();
    /// assert_eq!(nested, vec![0, 1, 2, 3]);
    ///
    /// // Check type
    /// assert_eq!(boundary.check_type(), BoundaryType::MultiPoint);
    /// ```
    pub fn to_nested_multi_point(&self) -> error::Result<BoundaryNestedMultiPoint<VR>> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiPoint {
            Ok(self
                .vertices
                .iter()
                .map(super::vertex::VertexIndex::value)
                .collect())
        } else {
            Err(error::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "MultiPoint".to_string(),
            ))
        }
    }

    /// Converts to a nested `MultiLineString` boundary representation.
    ///
    /// This method converts the flattened boundary to a nested `MultiLineString` representation
    /// if the boundary can be interpreted as a `MultiLineString`.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the nested `MultiLineString` representation or an error
    /// if the boundary cannot be interpreted as a `MultiLineString`.
    ///
    /// # Errors
    ///
    /// Returns [`error::Error::IncompatibleBoundary`] when this boundary is not a
    /// `MultiLineString`.
    /// Returns index-conversion errors when nested index offsets cannot be represented by `VR`.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::prelude::*;
    /// use cityjson::cityjson::core::boundary::nested::BoundaryNestedMultiLineString32;
    ///
    /// // Create a boundary from a MultiLineString
    /// let multi_linestring: BoundaryNestedMultiLineString32 = vec![
    ///     vec![0, 1, 2],
    ///     vec![3, 4, 5]
    /// ];
    /// let boundary: Boundary<u32> = multi_linestring.try_into().unwrap();
    ///
    /// // Convert back to MultiLineString
    /// let nested = boundary.to_nested_multi_linestring().unwrap();
    /// assert_eq!(nested, vec![vec![0, 1, 2], vec![3, 4, 5]]);
    /// ```
    pub fn to_nested_multi_linestring(&self) -> error::Result<BoundaryNestedMultiLineString<VR>> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiLineString {
            let mut counter = BoundaryCounter::<VR>::default();
            let mut ml = BoundaryNestedMultiLineString::with_capacity(self.rings.len());
            self.push_rings_to_surface(self.rings.as_slice(), &mut ml, &mut counter)?;
            Ok(ml)
        } else {
            Err(error::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "MultiLineString".to_string(),
            ))
        }
    }

    /// Converts to a nested Multi- or `CompositeSurface` boundary representation.
    ///
    /// This method converts the flattened boundary to a nested Multi- or `CompositeSurface`
    /// representation if the boundary can be interpreted as a Multi- or `CompositeSurface`.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the nested Multi- or `CompositeSurface` representation or an error
    /// if the boundary cannot be interpreted as a Multi- or `CompositeSurface`.
    ///
    /// # Errors
    ///
    /// Returns [`error::Error::IncompatibleBoundary`] when this boundary is not a
    /// Multi- or `CompositeSurface`.
    /// Returns index-conversion errors when nested index offsets cannot be represented by `VR`.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::prelude::*;
    /// use cityjson::cityjson::core::boundary::nested::BoundaryNestedMultiOrCompositeSurface32;
    ///
    /// // Create a boundary from a MultiSurface
    /// // A simple MultiSurface with two surfaces, each with one ring
    /// let multi_surface: BoundaryNestedMultiOrCompositeSurface32 = vec![
    ///     vec![vec![0, 1, 2, 0]], // First surface (triangle)
    ///     vec![vec![3, 4, 5, 3]]  // Second surface (triangle)
    /// ];
    /// let boundary: Boundary<u32> = multi_surface.clone().try_into().unwrap();
    ///
    /// // Convert back to MultiSurface
    /// let nested = boundary.to_nested_multi_or_composite_surface().unwrap();
    /// assert_eq!(nested, multi_surface);
    /// ```
    pub fn to_nested_multi_or_composite_surface(
        &self,
    ) -> error::Result<BoundaryNestedMultiOrCompositeSurface<VR>> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiOrCompositeSurface {
            let mut counter = BoundaryCounter::<VR>::default();
            let mut mc_surface =
                BoundaryNestedMultiOrCompositeSurface::with_capacity(self.surfaces.len());
            self.push_surfaces_to_multi_surface(
                self.surfaces.as_slice(),
                &mut mc_surface,
                &mut counter,
            )?;
            Ok(mc_surface)
        } else {
            Err(error::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "MultiOrCompositeSurface".to_string(),
            ))
        }
    }

    /// Converts to a nested Solid boundary representation.
    ///
    /// This method converts the flattened boundary to a nested Solid representation
    /// if the boundary can be interpreted as a Solid.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the nested Solid representation or an error
    /// if the boundary cannot be interpreted as a Solid.
    ///
    /// # Errors
    ///
    /// Returns [`error::Error::IncompatibleBoundary`] when this boundary is not a `Solid`.
    /// Returns index-conversion errors when nested index offsets cannot be represented by `VR`.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::prelude::*;
    /// use cityjson::cityjson::core::boundary::nested::BoundaryNestedSolid32;
    ///
    /// // Create a simplified solid representation (just one shell with one face for brevity)
    /// let solid: BoundaryNestedSolid32 = vec![
    ///     vec![vec![vec![0, 1, 2, 0]]] // One shell with one surface with one ring
    /// ];
    /// let boundary: Boundary<u32> = solid.clone().try_into().unwrap();
    ///
    /// // Check type
    /// assert_eq!(boundary.check_type(), BoundaryType::Solid);
    ///
    /// // Convert back to Solid
    /// let nested = boundary.to_nested_solid().unwrap();
    /// assert_eq!(nested, solid);
    /// ```
    pub fn to_nested_solid(&self) -> error::Result<BoundaryNestedSolid<VR>> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::Solid {
            let mut counter = BoundaryCounter::<VR>::default();
            let mut solid = BoundaryNestedSolid::with_capacity(self.shells.len());
            self.push_shells_to_solid(self.shells.as_slice(), &mut solid, &mut counter)?;
            Ok(solid)
        } else {
            Err(error::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "Solid".to_string(),
            ))
        }
    }

    /// Converts to a nested Multi- or `CompositeSolid` boundary representation.
    ///
    /// This method converts the flattened boundary to a nested Multi- or `CompositeSolid`
    /// representation if the boundary can be interpreted as a Multi- or `CompositeSolid`.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the nested Multi- or `CompositeSolid` representation or an error
    /// if the boundary cannot be interpreted as a Multi- or `CompositeSolid`.
    ///
    /// # Errors
    ///
    /// Returns [`error::Error::IncompatibleBoundary`] when this boundary is not a
    /// Multi- or `CompositeSolid`.
    /// Returns index-conversion errors when nested index offsets cannot be represented by `VR`.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::prelude::*;
    /// use cityjson::cityjson::core::boundary::nested::BoundaryNestedMultiOrCompositeSolid32;
    ///
    /// // Create a very simplified MultiSolid (just two solids with minimal structure for brevity)
    /// let multi_solid: BoundaryNestedMultiOrCompositeSolid32 = vec![
    ///     vec![vec![vec![vec![0, 1, 2, 0]]]],  // First solid
    ///     vec![vec![vec![vec![3, 4, 5, 3]]]]   // Second solid
    /// ];
    /// let boundary: Boundary<u32> = multi_solid.clone().try_into().unwrap();
    ///
    /// // Check type
    /// assert_eq!(boundary.check_type(), BoundaryType::MultiOrCompositeSolid);
    ///
    /// // Convert back to MultiSolid
    /// let nested = boundary.to_nested_multi_or_composite_solid().unwrap();
    /// assert_eq!(nested, multi_solid);
    /// ```
    pub fn to_nested_multi_or_composite_solid(
        &self,
    ) -> error::Result<BoundaryNestedMultiOrCompositeSolid<VR>> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiOrCompositeSolid {
            let mut counter = BoundaryCounter::<VR>::default();
            let mut mc_solid =
                BoundaryNestedMultiOrCompositeSolid::with_capacity(self.solids.len());
            for &shells_start_i in &self.solids {
                let shells_len = VertexIndex::<VR>::try_from(self.shells.len())?;
                let shells_end_i = self
                    .solids
                    .get(counter.try_increment_solid_idx()?.to_usize())
                    .copied()
                    .unwrap_or(shells_len);

                if let Some(shells) = self
                    .shells
                    .get(shells_start_i.to_usize()..shells_end_i.to_usize())
                {
                    let mut solid = BoundaryNestedSolid::with_capacity(shells.len());
                    self.push_shells_to_solid(shells, &mut solid, &mut counter)?;
                    mc_solid.push(solid);
                }
            }
            Ok(mc_solid)
        } else {
            Err(error::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "MultiOrCompositeSolid".to_string(),
            ))
        }
    }

    // Helper method to process shells for a solid
    fn push_shells_to_solid(
        &self,
        shells: &[VertexIndex<VR>],
        solid: &mut Vec<BoundaryNestedMultiOrCompositeSurface<VR>>,
        counter: &mut BoundaryCounter<VR>,
    ) -> error::Result<()> {
        for &surfaces_start_i in shells {
            let surfaces_len = VertexIndex::<VR>::try_from(self.surfaces.len())?;
            let surfaces_end_i = self
                .shells
                .get(counter.try_increment_shell_idx()?.to_usize())
                .copied()
                .unwrap_or(surfaces_len);

            if let Some(surfaces) = self
                .surfaces
                .get(surfaces_start_i.to_usize()..surfaces_end_i.to_usize())
            {
                let mut mc_surface =
                    BoundaryNestedMultiOrCompositeSurface::with_capacity(surfaces.len());
                self.push_surfaces_to_multi_surface(surfaces, &mut mc_surface, counter)?;
                solid.push(mc_surface);
            }
        }
        Ok(())
    }

    // Helper method to process surfaces for a shell
    fn push_surfaces_to_multi_surface(
        &self,
        surfaces: &[VertexIndex<VR>],
        mc_surface: &mut BoundaryNestedMultiOrCompositeSurface<VR>,
        counter: &mut BoundaryCounter<VR>,
    ) -> error::Result<()> {
        for &ring_start_i in surfaces {
            let rings_len = VertexIndex::<VR>::try_from(self.rings.len())?;
            let ring_end_i = self
                .surfaces
                .get(counter.try_increment_surface_idx()?.to_usize())
                .copied()
                .unwrap_or(rings_len);

            if let Some(rings) = self
                .rings
                .get(ring_start_i.to_usize()..ring_end_i.to_usize())
            {
                let mut surface = BoundaryNestedMultiLineString::with_capacity(rings.len());
                self.push_rings_to_surface(rings, &mut surface, counter)?;
                mc_surface.push(surface);
            }
        }
        Ok(())
    }

    // Helper method to process rings for a surface
    fn push_rings_to_surface(
        &self,
        rings: &[VertexIndex<VR>],
        surface: &mut BoundaryNestedMultiLineString<VR>,
        counter: &mut BoundaryCounter<VR>,
    ) -> error::Result<()> {
        for &vertices_start_i in rings {
            let vertices_len = VertexIndex::<VR>::try_from(self.vertices.len())?;
            let vertices_end_i = self
                .rings
                .get(counter.try_increment_ring_idx()?.to_usize())
                .copied()
                .unwrap_or(vertices_len);
            if let Some(vertices) = self
                .vertices
                .get(vertices_start_i.to_usize()..vertices_end_i.to_usize())
            {
                surface.push(
                    vertices
                        .iter()
                        .map(super::vertex::VertexIndex::value)
                        .collect(),
                );
            }
        }
        Ok(())
    }

    /// Determines the type of boundary stored in this instance.
    ///
    /// This method examines the structure of the boundary to determine its type.
    /// The detection follows a hierarchical approach, prioritizing the most complex
    /// structure present.
    ///
    /// # Returns
    ///
    /// A `BoundaryType` value indicating the type of the boundary.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::prelude::*;
    /// use cityjson::cityjson::core::boundary::nested::BoundaryNestedMultiLineString32;
    ///
    /// // Create a boundary from a MultiLineString
    /// let multi_linestring: BoundaryNestedMultiLineString32 = vec![vec![0, 1, 2]];
    /// let boundary: Boundary<u32> = multi_linestring.try_into().unwrap();
    ///
    /// // Check type
    /// assert_eq!(boundary.check_type(), BoundaryType::MultiLineString);
    /// ```
    #[must_use]
    pub fn check_type(&self) -> BoundaryType {
        if !self.solids.is_empty() {
            BoundaryType::MultiOrCompositeSolid
        } else if !self.shells.is_empty() {
            BoundaryType::Solid
        } else if !self.surfaces.is_empty() {
            BoundaryType::MultiOrCompositeSurface
        } else if !self.rings.is_empty() {
            BoundaryType::MultiLineString
        } else if !self.vertices.is_empty() {
            BoundaryType::MultiPoint
        } else {
            BoundaryType::None
        }
    }

    /// Verifies that the internal representation of the boundary is consistent.
    ///
    /// This method checks that all indices are valid and that there are no dangling
    /// references. It ensures that:
    /// - Ring indices point to valid vertices
    /// - Surface indices point to valid rings
    /// - Shell indices point to valid surfaces
    /// - Solid indices point to valid shells
    ///
    /// # Returns
    ///
    /// `true` if the boundary is consistent, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::prelude::*;
    ///
    /// let boundary: Boundary<u32> = Boundary::new();
    /// assert!(boundary.is_consistent());
    /// ```
    #[must_use]
    pub fn is_consistent(&self) -> bool {
        // Check that all indices are within bounds
        let vertices_len = self.vertices.len();
        let rings_len = self.rings.len();
        let surfaces_len = self.surfaces.len();
        let shells_len = self.shells.len();

        // Check ring indices point to valid vertices
        for window in self.rings.windows(2) {
            let start = window[0].to_usize();
            let end = window[1].to_usize();

            if start >= end || end > vertices_len {
                return false;
            }
        }

        // Check surface indices point to valid rings
        for window in self.surfaces.windows(2) {
            let start = window[0].to_usize();
            let end = window[1].to_usize();

            if start >= end || end > rings_len {
                return false;
            }
        }

        // Check shell indices point to valid surfaces
        for window in self.shells.windows(2) {
            let start = window[0].to_usize();
            let end = window[1].to_usize();

            if start >= end || end > surfaces_len {
                return false;
            }
        }

        // Check solid indices point to valid shells
        for window in self.solids.windows(2) {
            let start = window[0].to_usize();
            let end = window[1].to_usize();

            if start >= end || end > shells_len {
                return false;
            }
        }

        true
    }
}

/// The type of a `CityJSON` boundary.
///
/// This enum represents the different types of boundaries that can be represented
/// in `CityJSON`. The types follow a hierarchy of complexity, from the simplest
/// (`MultiPoint`) to the most complex (`MultiOrCompositeSolid`).
///
/// # Examples
///
/// ```
/// use cityjson::prelude::*;
/// use cityjson::cityjson::core::boundary::nested::BoundaryNestedMultiPoint32;
///
/// // Create a boundary from a MultiPoint
/// let multi_point: BoundaryNestedMultiPoint32 = vec![0, 1, 2, 3];
/// let boundary: Boundary<u32> = multi_point.into();
///
/// // Check type
/// assert_eq!(boundary.check_type(), BoundaryType::MultiPoint);
/// ```
#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[non_exhaustive]
pub enum BoundaryType {
    /// A collection of solids, possibly connected.
    MultiOrCompositeSolid,
    /// A single solid.
    Solid,
    /// A collection of surfaces, possibly connected.
    MultiOrCompositeSurface,
    /// A collection of line strings.
    MultiLineString,
    /// A collection of points.
    MultiPoint,
    /// An empty boundary.
    #[default]
    None,
}

impl std::fmt::Display for BoundaryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BoundaryType::MultiOrCompositeSolid => "MultiOrCompositeSolid",
            BoundaryType::Solid => "Solid",
            BoundaryType::MultiOrCompositeSurface => "MultiOrCompositeSurface",
            BoundaryType::MultiLineString => "MultiLineString",
            BoundaryType::MultiPoint => "MultiPoint",
            BoundaryType::None => "None",
        };
        write!(f, "{s}")
    }
}

/// A counter for tracking positions within different levels of a boundary hierarchy.
///
/// This struct is used internally during conversions between flattened and nested
/// representations to keep track of the current position in each level of the hierarchy.
#[derive(Default)]
pub(crate) struct BoundaryCounter<VR: VertexRef> {
    pub(crate) vertex: VertexIndex<VR>, // Current position in vertex list
    pub(crate) ring: VertexIndex<VR>,   // Current position in ring list
    pub(crate) surface: VertexIndex<VR>, // Current position in surface list
    pub(crate) shell: VertexIndex<VR>,  // Current position in shell list
    pub(crate) solid: VertexIndex<VR>,  // Current position in solid list
}

impl<VR: VertexRef> BoundaryCounter<VR> {
    #[inline]
    fn increment_checked(offset: &mut VertexIndex<VR>) -> error::Result<VertexIndex<VR>> {
        *offset = offset.next().ok_or_else(|| error::Error::IndexOverflow {
            index_type: std::any::type_name::<VR>().to_string(),
            value: offset.value().to_string(),
        })?;
        Ok(*offset)
    }

    // Increment methods - return new position after incrementing
    pub(crate) fn increment_vertex_idx(&mut self) -> VertexIndex<VR> {
        self.vertex += VertexIndex::new(VR::one());
        self.vertex
    }

    pub(crate) fn increment_ring_idx(&mut self) -> VertexIndex<VR> {
        self.ring += VertexIndex::new(VR::one());
        self.ring
    }

    pub(crate) fn increment_surface_idx(&mut self) -> VertexIndex<VR> {
        self.surface += VertexIndex::new(VR::one());
        self.surface
    }

    pub(crate) fn increment_shell_idx(&mut self) -> VertexIndex<VR> {
        self.shell += VertexIndex::new(VR::one());
        self.shell
    }

    pub(crate) fn increment_solid_idx(&mut self) -> VertexIndex<VR> {
        self.solid += VertexIndex::new(VR::one());
        self.solid
    }

    pub(crate) fn try_increment_ring_idx(&mut self) -> error::Result<VertexIndex<VR>> {
        Self::increment_checked(&mut self.ring)
    }

    pub(crate) fn try_increment_surface_idx(&mut self) -> error::Result<VertexIndex<VR>> {
        Self::increment_checked(&mut self.surface)
    }

    pub(crate) fn try_increment_shell_idx(&mut self) -> error::Result<VertexIndex<VR>> {
        Self::increment_checked(&mut self.shell)
    }

    pub(crate) fn try_increment_solid_idx(&mut self) -> error::Result<VertexIndex<VR>> {
        Self::increment_checked(&mut self.solid)
    }

    // Get current offsets without incrementing
    pub(crate) fn vertex_offset(&self) -> VertexIndex<VR> {
        self.vertex
    }

    pub(crate) fn ring_offset(&self) -> VertexIndex<VR> {
        self.ring
    }

    pub(crate) fn surface_offset(&self) -> VertexIndex<VR> {
        self.surface
    }

    pub(crate) fn shell_offset(&self) -> VertexIndex<VR> {
        self.shell
    }

    #[allow(unused)]
    pub(crate) fn solid_offset(&self) -> VertexIndex<VR> {
        self.solid
    }
}

// Type aliases for convenience
/// A boundary using 16-bit vertex indices (suitable for up to 65,535 vertices)
pub type Boundary16 = Boundary<u16>;
/// A boundary using 32-bit vertex indices (suitable for up to ~4.3 billion vertices)
pub type Boundary32 = Boundary<u32>;
/// A boundary using 64-bit vertex indices (suitable for virtually unlimited vertices)
pub type Boundary64 = Boundary<u64>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cityjson::core::boundary::nested::{
        BoundaryNestedMultiLineString32, BoundaryNestedMultiOrCompositeSolid32,
        BoundaryNestedMultiOrCompositeSurface32, BoundaryNestedMultiPoint32, BoundaryNestedSolid32,
    };
    use crate::cityjson::core::vertex::VertexIndex;

    // Helper function to create vertex indices
    fn vi<T: VertexRef>(value: T) -> VertexIndex<T> {
        VertexIndex::new(value)
    }

    #[test]
    fn test_empty_boundary() {
        let boundary: Boundary<u32> = Boundary::new();
        assert_eq!(boundary.check_type(), BoundaryType::None);
        assert!(boundary.is_consistent());
    }

    #[test]
    fn test_boundary_with_capacity() {
        let boundary: Boundary<u32> = Boundary::with_capacity(
            10, // vertices capacity
            5,  // rings capacity
            3,  // surfaces capacity
            2,  // shells capacity
            1,  // solids capacity
        );
        assert_eq!(boundary.check_type(), BoundaryType::None);
        assert!(boundary.is_consistent());
    }

    #[test]
    fn test_boundary_type_detection() {
        // Create various boundary types
        let mut multi_point_boundary: Boundary<u32> = Boundary::new();
        multi_point_boundary.vertices = vec![vi(0), vi(1), vi(2)];
        assert_eq!(multi_point_boundary.check_type(), BoundaryType::MultiPoint);

        let mut multi_line_boundary: Boundary<u32> = Boundary::new();
        multi_line_boundary.vertices = vec![vi(0), vi(1), vi(2)];
        multi_line_boundary.rings = vec![vi(0)];
        assert_eq!(
            multi_line_boundary.check_type(),
            BoundaryType::MultiLineString
        );

        let mut multi_surface_boundary: Boundary<u32> = Boundary::new();
        multi_surface_boundary.vertices = vec![vi(0), vi(1), vi(2)];
        multi_surface_boundary.rings = vec![vi(0)];
        multi_surface_boundary.surfaces = vec![vi(0)];
        assert_eq!(
            multi_surface_boundary.check_type(),
            BoundaryType::MultiOrCompositeSurface
        );

        let mut solid_boundary: Boundary<u32> = Boundary::new();
        solid_boundary.vertices = vec![vi(0), vi(1), vi(2)];
        solid_boundary.rings = vec![vi(0)];
        solid_boundary.surfaces = vec![vi(0)];
        solid_boundary.shells = vec![vi(0)];
        assert_eq!(solid_boundary.check_type(), BoundaryType::Solid);

        let mut multi_solid_boundary: Boundary<u32> = Boundary::new();
        multi_solid_boundary.vertices = vec![vi(0), vi(1), vi(2)];
        multi_solid_boundary.rings = vec![vi(0)];
        multi_solid_boundary.surfaces = vec![vi(0)];
        multi_solid_boundary.shells = vec![vi(0)];
        multi_solid_boundary.solids = vec![vi(0)];
        assert_eq!(
            multi_solid_boundary.check_type(),
            BoundaryType::MultiOrCompositeSolid
        );
    }

    #[test]
    fn test_boundary_consistency() {
        // Consistent boundary - basic multilinestring
        let mut consistent: Boundary<u32> = Boundary::new();
        consistent.vertices = vec![vi(0), vi(1), vi(2), vi(3)];
        consistent.rings = vec![vi(0), vi(2)];
        assert!(consistent.is_consistent());

        // Consistent boundary - multi-surface
        let mut consistent2: Boundary<u32> = Boundary::new();
        consistent2.vertices = vec![vi(0), vi(1), vi(2), vi(3), vi(4), vi(5)];
        consistent2.rings = vec![vi(0), vi(3), vi(6)]; // Note: vi(6) is out of bounds, but it's allowed as the "end" pointer
        consistent2.surfaces = vec![vi(0), vi(2)];
        assert!(consistent2.is_consistent());

        // Inconsistent boundary - ring references out of bounds
        let mut inconsistent: Boundary<u32> = Boundary::new();
        inconsistent.vertices = vec![vi(0), vi(1)];
        inconsistent.rings = vec![vi(0), vi(3)]; // references vertex 3, which doesn't exist
        assert!(!inconsistent.is_consistent());

        // Inconsistent boundary - surface references out of bounds
        let mut inconsistent2: Boundary<u32> = Boundary::new();
        inconsistent2.vertices = vec![vi(0), vi(1), vi(2), vi(3)];
        inconsistent2.rings = vec![vi(0)];
        inconsistent2.surfaces = vec![vi(0), vi(2)]; // references ring 2, which doesn't exist
        assert!(!inconsistent2.is_consistent());
    }

    #[test]
    fn test_multi_point_conversion() {
        // Create nested multi-point
        let nested: BoundaryNestedMultiPoint32 = vec![0, 1, 2, 3];

        // Convert to flattened
        let flattened: Boundary<u32> = nested.clone().into();
        assert_eq!(flattened.check_type(), BoundaryType::MultiPoint);

        // Convert back to nested
        let round_trip = flattened.to_nested_multi_point().unwrap();
        assert_eq!(round_trip, nested);

        // Test incompatible conversion
        let mut multi_line_boundary: Boundary<u32> = Boundary::new();
        multi_line_boundary.vertices = vec![vi(0), vi(1), vi(2)];
        multi_line_boundary.rings = vec![vi(0)];
        assert!(multi_line_boundary.to_nested_multi_point().is_err());
    }

    #[test]
    fn test_multi_linestring_conversion() {
        // Create nested multi-linestring
        let nested: BoundaryNestedMultiLineString32 = vec![vec![0, 1, 2], vec![3, 4, 5, 6]];

        // Convert to flattened
        let flattened: Boundary<u32> = nested.clone().try_into().unwrap();
        assert_eq!(flattened.check_type(), BoundaryType::MultiLineString);

        // Convert back to nested
        let round_trip = flattened.to_nested_multi_linestring().unwrap();
        assert_eq!(round_trip, nested);

        // Test incompatible conversion
        let mut multi_point_boundary: Boundary<u32> = Boundary::new();
        multi_point_boundary.vertices = vec![vi(0), vi(1), vi(2)];
        assert!(multi_point_boundary.to_nested_multi_linestring().is_err());
    }

    #[test]
    fn test_multi_surface_conversion() {
        // Create nested multi-surface
        let nested: BoundaryNestedMultiOrCompositeSurface32 = vec![
            // First surface with one ring
            vec![vec![0, 1, 2, 0]],
            // Second surface with two rings (outer and inner)
            vec![vec![3, 4, 5, 3], vec![6, 7, 8, 6]],
        ];

        // Convert to flattened
        let flattened: Boundary<u32> = nested.clone().try_into().unwrap();
        assert_eq!(
            flattened.check_type(),
            BoundaryType::MultiOrCompositeSurface
        );

        // Convert back to nested
        let round_trip = flattened.to_nested_multi_or_composite_surface().unwrap();
        assert_eq!(round_trip, nested);

        // Test incompatible conversion
        let mut multi_point_boundary: Boundary<u32> = Boundary::new();
        multi_point_boundary.vertices = vec![vi(0), vi(1), vi(2)];
        assert!(
            multi_point_boundary
                .to_nested_multi_or_composite_surface()
                .is_err()
        );
    }

    #[test]
    fn test_solid_conversion() {
        // Create nested solid (a simple cube)
        let nested: BoundaryNestedSolid32 = vec![
            // Outer shell with 6 faces
            vec![
                vec![vec![0, 1, 2, 3, 0]], // front face
                vec![vec![4, 5, 6, 7, 4]], // back face
                vec![vec![0, 3, 7, 4, 0]], // left face
                vec![vec![1, 2, 6, 5, 1]], // right face
                vec![vec![0, 1, 5, 4, 0]], // bottom face
                vec![vec![3, 2, 6, 7, 3]], // top face
            ],
        ];

        // Convert to flattened
        let flattened: Boundary<u32> = nested.clone().try_into().unwrap();
        assert_eq!(flattened.check_type(), BoundaryType::Solid);

        // Convert back to nested
        let round_trip = flattened.to_nested_solid().unwrap();
        assert_eq!(round_trip, nested);

        // Test incompatible conversion
        let mut multi_point_boundary: Boundary<u32> = Boundary::new();
        multi_point_boundary.vertices = vec![vi(0), vi(1), vi(2)];
        assert!(multi_point_boundary.to_nested_solid().is_err());
    }

    #[test]
    fn test_multi_solid_conversion() {
        // Create nested multi-solid (two simple cubes)
        let nested: BoundaryNestedMultiOrCompositeSolid32 = vec![
            // First solid - just a single triangular face for simplicity
            vec![vec![vec![vec![0, 1, 2, 0]]]],
            // Second solid - also a single triangular face
            vec![vec![vec![vec![3, 4, 5, 3]]]],
        ];

        // Convert to flattened
        let flattened: Boundary<u32> = nested.clone().try_into().unwrap();
        assert_eq!(flattened.check_type(), BoundaryType::MultiOrCompositeSolid);

        // Convert back to nested
        let round_trip = flattened.to_nested_multi_or_composite_solid().unwrap();
        assert_eq!(round_trip, nested);

        // Test incompatible conversion
        let mut multi_point_boundary: Boundary<u32> = Boundary::new();
        multi_point_boundary.vertices = vec![vi(0), vi(1), vi(2)];
        assert!(
            multi_point_boundary
                .to_nested_multi_or_composite_solid()
                .is_err()
        );
    }

    #[test]
    fn test_display_boundary_type() {
        assert_eq!(BoundaryType::None.to_string(), "None");
        assert_eq!(BoundaryType::MultiPoint.to_string(), "MultiPoint");
        assert_eq!(BoundaryType::MultiLineString.to_string(), "MultiLineString");
        assert_eq!(
            BoundaryType::MultiOrCompositeSurface.to_string(),
            "MultiOrCompositeSurface"
        );
        assert_eq!(BoundaryType::Solid.to_string(), "Solid");
        assert_eq!(
            BoundaryType::MultiOrCompositeSolid.to_string(),
            "MultiOrCompositeSolid"
        );
    }

    #[test]
    fn test_boundary_counter() {
        let mut counter = BoundaryCounter::<u32>::default();

        // Initial values should be zero
        assert_eq!(counter.vertex_offset().value(), 0);
        assert_eq!(counter.ring_offset().value(), 0);
        assert_eq!(counter.surface_offset().value(), 0);
        assert_eq!(counter.shell_offset().value(), 0);
        assert_eq!(counter.solid_offset().value(), 0);

        // Test increments
        assert_eq!(counter.increment_vertex_idx().value(), 1);
        assert_eq!(counter.increment_vertex_idx().value(), 2);

        assert_eq!(counter.increment_ring_idx().value(), 1);
        assert_eq!(counter.increment_surface_idx().value(), 1);
        assert_eq!(counter.increment_shell_idx().value(), 1);
        assert_eq!(counter.increment_solid_idx().value(), 1);

        // Current values after increments
        assert_eq!(counter.vertex_offset().value(), 2);
        assert_eq!(counter.ring_offset().value(), 1);
        assert_eq!(counter.surface_offset().value(), 1);
        assert_eq!(counter.shell_offset().value(), 1);
        assert_eq!(counter.solid_offset().value(), 1);
    }
}

#[cfg(test)]
mod nested_tests {
    use super::*;
    use crate::cityjson::core::boundary::nested::{
        BoundaryNestedMultiLineString16, BoundaryNestedMultiLineString32,
        BoundaryNestedMultiOrCompositeSolid16, BoundaryNestedMultiOrCompositeSolid32,
        BoundaryNestedMultiOrCompositeSurface16, BoundaryNestedMultiOrCompositeSurface32,
        BoundaryNestedMultiPoint16, BoundaryNestedMultiPoint32, BoundaryNestedSolid16,
        BoundaryNestedSolid32,
    };
    use crate::cityjson::core::vertex::VertexIndex;

    const U16_MAX_PLUS_ONE: usize = (u16::MAX as usize) + 1;

    #[test]
    fn test_empty_nested_conversions() {
        // Test empty MultiPoint
        let empty_multi_point: BoundaryNestedMultiPoint32 = vec![];
        let boundary: Boundary<u32> = empty_multi_point.into();
        assert_eq!(boundary.check_type(), BoundaryType::None);

        // Test empty MultiLineString
        let empty_multi_linestring: BoundaryNestedMultiLineString32 = vec![];
        let boundary: Boundary<u32> = empty_multi_linestring.try_into().unwrap();
        assert_eq!(boundary.check_type(), BoundaryType::None);

        // Test empty MultiSurface
        let empty_multi_surface: BoundaryNestedMultiOrCompositeSurface32 = vec![];
        let boundary: Boundary<u32> = empty_multi_surface.try_into().unwrap();
        assert_eq!(boundary.check_type(), BoundaryType::None);

        // Test empty Solid
        let empty_solid: BoundaryNestedSolid32 = vec![];
        let boundary: Boundary<u32> = empty_solid.try_into().unwrap();
        assert_eq!(boundary.check_type(), BoundaryType::None);

        // Test empty MultiSolid
        let empty_multisolid: BoundaryNestedMultiOrCompositeSolid32 = vec![];
        let boundary: Boundary<u32> = empty_multisolid.try_into().unwrap();
        assert_eq!(boundary.check_type(), BoundaryType::None);
    }

    #[test]
    fn test_nested_multilinestring_with_empty_linestrings() {
        // Create a nested multi-linestring with an empty linestring
        let nested: BoundaryNestedMultiLineString32 = vec![
            vec![0, 1, 2],
            vec![], // Empty linestring
            vec![3, 4, 5],
        ];

        // Convert to flattened
        let flattened: Boundary<u32> = nested.clone().try_into().unwrap();
        assert_eq!(flattened.check_type(), BoundaryType::MultiLineString);

        // Convert back to nested
        let round_trip = flattened.to_nested_multi_linestring().unwrap();

        // The empty linestring should be preserved
        assert_eq!(round_trip.len(), 3);
        assert_eq!(round_trip[0], vec![0, 1, 2]);
        assert_eq!(round_trip[1], Vec::<u32>::new()); // Empty linestring preserved
        assert_eq!(round_trip[2], vec![3, 4, 5]);
    }

    #[test]
    fn test_nested_multisurface_with_empty_components() {
        // Create a nested multi-surface with an empty surface
        let nested: BoundaryNestedMultiOrCompositeSurface<u32> = vec![
            vec![vec![0, 1, 2, 0]],
            vec![], // Empty surface (no rings)
            vec![vec![3, 4, 5, 3]],
        ];

        // Convert to flattened
        let flattened: Boundary<u32> = nested.clone().try_into().unwrap();
        assert_eq!(
            flattened.check_type(),
            BoundaryType::MultiOrCompositeSurface
        );

        // Convert back to nested
        let round_trip = flattened.to_nested_multi_or_composite_surface().unwrap();

        let empty_surface = BoundaryNestedMultiLineString32::default();
        // The empty surface should be preserved
        assert_eq!(round_trip.len(), 3);
        assert_eq!(round_trip[1], empty_surface); // Empty surface preserved
    }

    #[test]
    fn test_type_alias_consistency() {
        // Ensure type aliases are consistent with each other

        // Create a simple multi-point with u16 indices
        let mp16: BoundaryNestedMultiPoint16 = vec![0, 1, 2];
        let boundary16: Boundary16 = mp16.clone().into();

        // Create the same with u32 indices
        let mp32: BoundaryNestedMultiPoint32 = vec![0, 1, 2];
        let boundary32: Boundary32 = mp32.into();

        // The boundaries should have the same structure despite different index types
        assert_eq!(boundary16.check_type(), boundary32.check_type());
        assert_eq!(boundary16.vertices.len(), boundary32.vertices.len());

        // Check round-trip conversion
        let mp16_again = boundary16.to_nested_multi_point().unwrap();
        assert_eq!(mp16_again, mp16);
    }

    #[test]
    fn test_multisolid_with_complex_structure() {
        // Test a multi-solid with multiple levels of nesting
        let nested: BoundaryNestedMultiOrCompositeSolid32 = vec![
            // First solid with two shells
            vec![
                // Outer shell with two surfaces
                vec![
                    vec![vec![0, 1, 2, 0]], // First surface
                    vec![vec![3, 4, 5, 3]], // Second surface
                ],
                // Inner shell with one surface
                vec![vec![vec![6, 7, 8, 6]]],
            ],
            // Second solid with one shell
            vec![
                // One shell with one surface with two rings (outer and inner)
                vec![vec![vec![9, 10, 11, 9], vec![12, 13, 14, 12]]],
            ],
        ];

        // Convert to flattened
        let flattened: Boundary<u32> = nested.clone().try_into().unwrap();
        assert_eq!(flattened.check_type(), BoundaryType::MultiOrCompositeSolid);

        // Convert back to nested
        let round_trip = flattened.to_nested_multi_or_composite_solid().unwrap();

        // The complex structure should be preserved
        assert_eq!(round_trip, nested);
    }

    #[test]
    fn test_nested_multilinestring_overflow_returns_err() {
        let nested: BoundaryNestedMultiLineString16 = vec![vec![0; U16_MAX_PLUS_ONE]];
        let result = Boundary::<u16>::try_from(nested);
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_multisurface_overflow_returns_err() {
        let nested: BoundaryNestedMultiOrCompositeSurface16 =
            vec![vec![vec![]; U16_MAX_PLUS_ONE], vec![]];
        let result = Boundary::<u16>::try_from(nested);
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_solid_overflow_returns_err() {
        let nested: BoundaryNestedSolid16 = vec![vec![vec![]; U16_MAX_PLUS_ONE], vec![]];
        let result = Boundary::<u16>::try_from(nested);
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_multisolid_overflow_returns_err() {
        let nested: BoundaryNestedMultiOrCompositeSolid16 =
            vec![vec![vec![]; U16_MAX_PLUS_ONE], vec![]];
        let result = Boundary::<u16>::try_from(nested);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_nested_multilinestring_overflow_returns_err_without_panic() {
        let mut boundary = Boundary::<u16>::new();
        boundary.vertices = vec![VertexIndex::new(0); U16_MAX_PLUS_ONE];
        boundary.rings = vec![VertexIndex::new(0)];

        let result = std::panic::catch_unwind(|| boundary.to_nested_multi_linestring());
        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }

    #[test]
    fn test_to_nested_multisurface_overflow_returns_err_without_panic() {
        let mut boundary = Boundary::<u16>::new();
        boundary.rings = vec![VertexIndex::new(0); U16_MAX_PLUS_ONE];
        boundary.surfaces = vec![VertexIndex::new(0)];

        let result = std::panic::catch_unwind(|| boundary.to_nested_multi_or_composite_surface());
        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }

    #[test]
    fn test_to_nested_solid_overflow_returns_err_without_panic() {
        let mut boundary = Boundary::<u16>::new();
        boundary.surfaces = vec![VertexIndex::new(0); U16_MAX_PLUS_ONE];
        boundary.shells = vec![VertexIndex::new(0)];

        let result = std::panic::catch_unwind(|| boundary.to_nested_solid());
        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }

    #[test]
    fn test_to_nested_multisolid_overflow_returns_err_without_panic() {
        let mut boundary = Boundary::<u16>::new();
        boundary.shells = vec![VertexIndex::new(0); U16_MAX_PLUS_ONE];
        boundary.solids = vec![VertexIndex::new(0)];

        let result = std::panic::catch_unwind(|| boundary.to_nested_multi_or_composite_solid());
        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }
}
