//! # Resource Mapping
//!
//! This module provides types for mapping between CityJSON geometries and their associated resources
//! such as semantics, materials, and textures. It implements the mapping structures needed to associate
//! these resources with specific parts of the geometry.
//!
//! ## Overview
//!
//! The mapping module contains several key components:
//!
//! - [`SemanticMap`]: Maps semantic information to geometry parts
//! - [`MaterialMap`]: Maps material properties to geometry parts
//! - [`TextureMap`]: Maps texture coordinates and textures to geometry vertices
//!
//! Under the hood, `SemanticMap` and `MaterialMap` both use the common `SemanticOrMaterialMap` structure,
//! which provides a uniform way to associate resources with different types of geometry primitives
//! (points, linestrings, surfaces, etc.).
//!
//! ## Usage Examples
//!
//! ### Creating a semantic mapping
//!
//! ```rust
//! use cityjson::resources::mapping::SemanticMap;
//! use cityjson::resources::pool::ResourceId32;
//! use cityjson::cityjson::traits::vertex::VertexRef;
//!
//! // Create a semantic mapping for a geometry with surface semantics
//! let mut semantic_map = SemanticMap::<u32, ResourceId32>::default();
//!
//! // Add semantic references to specific surfaces
//! semantic_map.add_surface(Some(ResourceId32::new(1, 0))); // Surface 0 uses semantic with ID 1
//! semantic_map.add_surface(Some(ResourceId32::new(2, 0))); // Surface 1 uses semantic with ID 2
//! semantic_map.add_surface(None);    // Surface 2 has no semantic
//! ```
//!
//! ### Working with material mappings
//!
//! ```rust
//! use cityjson::resources::mapping::MaterialMap;
//! use cityjson::resources::pool::ResourceId32;
//!
//! // Create a material mapping for a geometry with surface materials
//! let mut material_map = MaterialMap::<u32, ResourceId32>::default();
//!
//! // Add material references to specific surfaces
//! material_map.add_surface(Some(ResourceId32::new(5, 0))); // Surface 0 uses material with ID 5
//! material_map.add_surface(Some(ResourceId32::new(7, 0))); // Surface 1 uses material with ID 7
//! material_map.add_surface(None);    // Surface 2 has no material
//! ```
//!
//! ## Implementation Details
//!
//! The mapping structures are designed to work with different vertex reference types and resource
//! reference types, using generics to allow flexibility in implementation. The mappings follow the
//! hierarchical structure of CityJSON geometries, from points to solids.

pub mod materials;
pub mod semantics;
pub mod textures;

use crate::cityjson::core::boundary::BoundaryType;
use crate::cityjson::core::vertex::VertexIndex;
use crate::cityjson::traits::vertex::VertexRef;
pub use crate::resources::mapping::materials::MaterialMap;
pub use crate::resources::mapping::semantics::SemanticMap;
pub use crate::resources::mapping::textures::TextureMap;
use crate::resources::pool::ResourceRef;

/// Stores the Semantic or Material indices of a Boundary and maps them to the
/// boundary primitives.
///
/// This is a common base structure used for both semantic and material mappings,
/// allowing resources to be associated with different geometry elements based on
/// their indices.
///
/// # Type Parameters
///
/// * `VR` - The vertex reference type (e.g., u16, u32, u64) that determines indexing sizes
/// * `RR` - The resource reference type used to identify semantics or materials
///
/// # Examples
///
/// ```rust
/// use cityjson::resources::mapping::SemanticMap;
/// use cityjson::cityjson::core::boundary::BoundaryType;
/// use cityjson::resources::pool::ResourceId32;
///
/// // Create a semantic map for a multi-surface geometry
/// let mut semantic_map = SemanticMap::<u32, ResourceId32>::default();
/// semantic_map.add_surface(Some(ResourceId32::new(1, 0)));
/// semantic_map.add_surface(Some(ResourceId32::new(2, 0)));
///
/// // Check the type of boundary this map is for
/// assert_eq!(semantic_map.check_type(), BoundaryType::MultiOrCompositeSurface);
/// ```
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct SemanticOrMaterialMap<VR: VertexRef, RR: ResourceRef> {
    /// Each item corresponds to the point with the same index in a MultiPoint boundary, the value
    /// of the item is the index of the Semantic or Material object.
    pub(crate) points: Vec<Option<RR>>,
    /// Each item corresponds to the linestring with the same index in a MultiLineString boundary,
    /// the value of the item is the index of the Semantic or Material object.
    pub(crate) linestrings: Vec<Option<RR>>,
    /// Each item corresponds to the surface with the same index, the value
    /// of the item is the index of the Semantic or Material object.
    pub(crate) surfaces: Vec<Option<RR>>,
    /// Indices mapping shells to their semantic or material references
    pub(crate) shells: Vec<VertexIndex<VR>>,
    /// Indices mapping solids to their semantic or material references
    pub(crate) solids: Vec<VertexIndex<VR>>,
}

impl<VR: VertexRef, RR: ResourceRef> SemanticOrMaterialMap<VR, RR> {
    /// Creates a new empty mapping.
    ///
    /// # Returns
    ///
    /// A new `SemanticOrMaterialMap` instance with no mappings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::resources::mapping::SemanticMap;
    /// use cityjson::resources::pool::ResourceId32;
    ///
    /// let map = SemanticMap::<u32, ResourceId32>::new();
    /// assert!(map.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the map contains no mappings.
    ///
    /// # Returns
    ///
    /// `true` if all collections in the map are empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::resources::mapping::MaterialMap;
    /// use cityjson::resources::pool::ResourceId32;
    ///
    /// let map = MaterialMap::<u32, ResourceId32>::new();
    /// assert!(map.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
            && self.linestrings.is_empty()
            && self.surfaces.is_empty()
            && self.shells.is_empty()
            && self.solids.is_empty()
    }

    /// Adds a point semantic or material reference.
    ///
    /// # Parameters
    ///
    /// * `resource` - Optional reference to a semantic or material resource, or None
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::resources::mapping::SemanticMap;
    /// use cityjson::resources::pool::ResourceId32;
    ///
    /// let mut map = SemanticMap::<u32, ResourceId32>::new();
    /// map.add_point(Some(ResourceId32::new(1, 0))); // Point 0 has semantic with ID 1
    /// map.add_point(None);    // Point 1 has no semantic
    /// ```
    pub fn add_point(&mut self, resource: Option<RR>) {
        self.points.push(resource);
    }

    /// Adds a linestring semantic or material reference.
    ///
    /// # Parameters
    ///
    /// * `resource` - Optional reference to a semantic or material resource, or None
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::resources::mapping::MaterialMap;
    /// use cityjson::resources::pool::ResourceId32;
    ///
    /// let mut map = MaterialMap::<u32, ResourceId32>::new();
    /// map.add_linestring(Some(ResourceId32::new(2, 0))); // Linestring 0 has material with ID 2
    /// ```
    pub fn add_linestring(&mut self, resource: Option<RR>) {
        self.linestrings.push(resource);
    }

    /// Adds a surface semantic or material reference.
    ///
    /// # Parameters
    ///
    /// * `resource` - Optional reference to a semantic or material resource, or None
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::resources::mapping::SemanticMap;
    /// use cityjson::resources::pool::ResourceId32;
    ///
    /// let mut map = SemanticMap::<u32, ResourceId32>::new();
    /// map.add_surface(Some(ResourceId32::new(3, 0))); // Surface 0 has semantic with ID 3
    /// ```
    pub fn add_surface(&mut self, resource: Option<RR>) {
        self.surfaces.push(resource);
    }

    /// Adds a shell index.
    ///
    /// # Parameters
    ///
    /// * `shell_index` - Index of the shell
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::resources::mapping::SemanticMap;
    /// use cityjson::cityjson::core::vertex::VertexIndex;
    /// use cityjson::resources::pool::ResourceId32;
    ///
    /// let mut map = SemanticMap::<u32, ResourceId32>::new();
    /// map.add_shell(VertexIndex::new(0));
    /// ```
    pub fn add_shell(&mut self, shell_index: VertexIndex<VR>) {
        self.shells.push(shell_index);
    }

    /// Adds a solid index.
    ///
    /// # Parameters
    ///
    /// * `solid_index` - Index of the solid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::resources::mapping::MaterialMap;
    /// use cityjson::cityjson::core::vertex::VertexIndex;
    /// use cityjson::resources::pool::ResourceId32;
    ///
    /// let mut map = MaterialMap::<u32, ResourceId32>::new();
    /// map.add_solid(VertexIndex::new(0));
    /// ```
    pub fn add_solid(&mut self, solid_index: VertexIndex<VR>) {
        self.solids.push(solid_index);
    }

    /// Returns a reference to the point semantic or material references.
    ///
    /// # Returns
    ///
    /// A slice containing resource references for points.
    pub fn points(&self) -> &[Option<RR>] {
        &self.points
    }

    /// Returns a reference to the linestring semantic or material references.
    ///
    /// # Returns
    ///
    /// A slice containing resource references for linestrings.
    pub fn linestrings(&self) -> &[Option<RR>] {
        &self.linestrings
    }

    /// Returns a reference to the surface semantic or material references.
    ///
    /// # Returns
    ///
    /// A slice containing resource references for surfaces.
    pub fn surfaces(&self) -> &[Option<RR>] {
        &self.surfaces
    }

    /// Returns a reference to the shell indices.
    ///
    /// # Returns
    ///
    /// A slice containing shell indices.
    pub fn shells(&self) -> &[VertexIndex<VR>] {
        &self.shells
    }

    /// Returns a reference to the solid indices.
    ///
    /// # Returns
    ///
    /// A slice containing solid indices.
    pub fn solids(&self) -> &[VertexIndex<VR>] {
        &self.solids
    }

    /// Determines what type of boundary this mapping is associated with based on its contents.
    ///
    /// This method examines which vectors in the mapping contain data and returns the
    /// corresponding boundary type. It follows the hierarchy from most complex (solids)
    /// to simplest (points).
    ///
    /// # Returns
    ///
    /// A `BoundaryType` value indicating the type of boundary this mapping corresponds to.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::resources::mapping::MaterialMap;
    /// use cityjson::resources::pool::ResourceId32;
    /// use cityjson::cityjson::core::boundary::BoundaryType;
    ///
    /// // Create a material map for point features
    /// let mut material_map = MaterialMap::<u32, ResourceId32>::default();
    /// material_map.add_point(Some(ResourceId32::new(1, 0)));
    /// material_map.add_point(Some(ResourceId32::new(2, 0)));
    ///
    /// // Check the type
    /// assert_eq!(material_map.check_type(), BoundaryType::MultiPoint);
    ///
    /// // Create a material map for line features
    /// let mut line_map = MaterialMap::<u32, ResourceId32>::default();
    /// line_map.add_linestring(Some(ResourceId32::new(3, 0)));
    ///
    /// // Check the type
    /// assert_eq!(line_map.check_type(), BoundaryType::MultiLineString);
    /// ```
    pub fn check_type(&self) -> BoundaryType {
        if !self.solids.is_empty() {
            BoundaryType::MultiOrCompositeSolid
        } else if !self.shells.is_empty() {
            BoundaryType::Solid
        } else if !self.surfaces.is_empty() {
            BoundaryType::MultiOrCompositeSurface
        } else if !self.linestrings.is_empty() {
            BoundaryType::MultiLineString
        } else if !self.points.is_empty() {
            BoundaryType::MultiPoint
        } else {
            BoundaryType::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cityjson::core::boundary::BoundaryType;
    use crate::cityjson::core::vertex::VertexIndex;
    use crate::resources::pool::ResourceId32;

    #[test]
    fn test_semantic_or_material_map_check_type() {
        // Test empty map
        let empty_map = SemanticOrMaterialMap::<u32, ResourceId32>::default();
        assert_eq!(empty_map.check_type(), BoundaryType::None);

        // Test point map
        let mut point_map = SemanticOrMaterialMap::<u32, ResourceId32>::default();
        point_map.add_point(Some(ResourceId32::new(1, 0)));
        assert_eq!(point_map.check_type(), BoundaryType::MultiPoint);

        // Test linestring map
        let mut line_map = SemanticOrMaterialMap::<u32, ResourceId32>::default();
        line_map.add_linestring(Some(ResourceId32::new(1, 0)));
        assert_eq!(line_map.check_type(), BoundaryType::MultiLineString);

        // Test surface map
        let mut surface_map = SemanticOrMaterialMap::<u32, ResourceId32>::default();
        surface_map.add_surface(Some(ResourceId32::new(1, 0)));
        assert_eq!(
            surface_map.check_type(),
            BoundaryType::MultiOrCompositeSurface
        );

        // Test shell map
        let mut shell_map = SemanticOrMaterialMap::<u32, ResourceId32>::default();
        shell_map.add_shell(VertexIndex::new(0));
        assert_eq!(shell_map.check_type(), BoundaryType::Solid);

        // Test solid map
        let mut solid_map = SemanticOrMaterialMap::<u32, ResourceId32>::default();
        solid_map.add_solid(VertexIndex::new(0));
        assert_eq!(solid_map.check_type(), BoundaryType::MultiOrCompositeSolid);

        // Test hierarchy (solids take precedence)
        let mut mixed_map = SemanticOrMaterialMap::<u32, ResourceId32>::default();
        mixed_map.add_point(Some(ResourceId32::new(1, 0)));
        mixed_map.add_linestring(Some(ResourceId32::new(2, 0)));
        mixed_map.add_surface(Some(ResourceId32::new(3, 0)));
        mixed_map.add_shell(VertexIndex::new(0));
        mixed_map.add_solid(VertexIndex::new(0));
        assert_eq!(mixed_map.check_type(), BoundaryType::MultiOrCompositeSolid);
    }

    #[test]
    fn test_is_empty() {
        let empty_map = SemanticOrMaterialMap::<u32, ResourceId32>::new();
        assert!(empty_map.is_empty());

        let mut not_empty_map = SemanticOrMaterialMap::<u32, ResourceId32>::new();
        not_empty_map.add_point(Some(ResourceId32::new(1, 0)));
        assert!(!not_empty_map.is_empty());
    }

    #[test]
    fn test_accessors() {
        let mut map = SemanticOrMaterialMap::<u32, ResourceId32>::new();

        // Add data
        map.add_point(Some(ResourceId32::new(1, 0)));
        map.add_point(Some(ResourceId32::new(2, 0)));
        map.add_linestring(Some(ResourceId32::new(3, 0)));
        map.add_surface(Some(ResourceId32::new(4, 0)));
        map.add_shell(VertexIndex::new(0));
        map.add_solid(VertexIndex::new(1));

        // Test accessors
        assert_eq!(map.points().len(), 2);
        assert_eq!(map.points()[0], Some(ResourceId32::new(1, 0)));
        assert_eq!(map.linestrings().len(), 1);
        assert_eq!(map.linestrings()[0], Some(ResourceId32::new(3, 0)));
        assert_eq!(map.surfaces().len(), 1);
        assert_eq!(map.surfaces()[0], Some(ResourceId32::new(4, 0)));
        assert_eq!(map.shells().len(), 1);
        assert_eq!(map.shells()[0], VertexIndex::new(0));
        assert_eq!(map.solids().len(), 1);
        assert_eq!(map.solids()[0], VertexIndex::new(1));
    }

    #[test]
    fn test_material_map_type_alias() {
        // Test that MaterialMap is correctly aliased
        let mut material_map = MaterialMap::<u32, ResourceId32>::default();
        material_map.add_surface(Some(ResourceId32::new(5, 0)));
        assert_eq!(
            material_map.check_type(),
            BoundaryType::MultiOrCompositeSurface
        );
    }

    #[test]
    fn test_semantic_map_type_alias() {
        // Test that SemanticMap is correctly aliased
        let mut semantic_map = SemanticMap::<u32, ResourceId32>::default();
        semantic_map.add_surface(Some(ResourceId32::new(3, 0)));
        assert_eq!(
            semantic_map.check_type(),
            BoundaryType::MultiOrCompositeSurface
        );
    }
}
