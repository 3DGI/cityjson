//! # Texture Mapping
//!
//! This module provides types for mapping textures to CityJSON geometry elements.
//! It defines specialized map types that associate texture resources and texture coordinates
//! with specific parts of the geometry.
//!
//! ## Overview
//!
//! The texture mapping module contains:
//!
//! - [`TextureMap`]: A type for mapping textures and texture coordinates to geometry vertices
//!
//! Unlike semantic and material mappings which use a common base structure, texture mapping
//! requires a distinct structure due to the need to associate both texture resources and
//! texture coordinates with geometry.
//!
//! ## Usage Examples
//!
//! ### Creating a texture mapping
//!
//! ```rust
//! use cityjson::prelude::*;
//!
//! // Create a texture mapping
//! let mut texture_map = TextureMap::<u32, ResourceId32>::new();
//!
//! // Add texture vertex references
//! texture_map.add_vertex(Some(VertexIndex::new(0)));
//! texture_map.add_vertex(Some(VertexIndex::new(1)));
//! texture_map.add_vertex(Some(VertexIndex::new(2)));
//!
//! // Add a ring and its texture
//! texture_map.add_ring(VertexIndex::new(0));
//! texture_map.add_ring_texture(Some(ResourceId32::new(5, 1))); // Associate texture ID 5 with this ring
//! ```
//!
//! ## Implementation Details
//!
//! The `TextureMap` structure follows the hierarchical organization of CityJSON geometries,
//! from vertices up to solids, allowing texture information to be associated at different levels.

use crate::cityjson::shared::vertex::VertexIndex;
use crate::traits::vertex::VertexRef;
use crate::resources::pool::ResourceRef;

/// Maps geometry vertices to texture coordinates and textures.
///
/// This structure provides associations between geometry elements and texture information,
/// including texture coordinates (UV vertices) and texture resources. It follows the
/// hierarchy of CityJSON geometries from vertices to solids.
///
/// # Type Parameters
///
/// * `VR` - The vertex reference type (e.g., u16, u32, u64) that determines indexing sizes
/// * `RR` - The resource reference type used to identify textures
///
/// # Examples
///
/// ```rust
/// use cityjson::prelude::*;
///
/// // Create a texture map
/// let mut texture_map = TextureMap::<u32, ResourceId32>::new();
///
/// // Add texture vertex references and ring indices
/// texture_map.add_vertex(Some(VertexIndex::new(0)));
/// texture_map.add_vertex(Some(VertexIndex::new(1)));
/// texture_map.add_vertex(Some(VertexIndex::new(2)));
///
/// // Add a ring starting at vertex 0
/// texture_map.add_ring(VertexIndex::new(0));
///
/// // Associate a texture with this ring
/// texture_map.add_ring_texture(Some(ResourceId32::new(5, 1))); // Reference to texture ID 5
/// ```
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct TextureMap<VR: VertexRef, RR: ResourceRef> {
    /// References to texture vertices (UV coordinates) for each geometry vertex
    vertices: Vec<Option<VertexIndex<VR>>>,

    /// Indices marking the start of each ring in the vertices vector
    rings: Vec<VertexIndex<VR>>,

    /// Texture resource references for each ring
    ring_textures: Vec<Option<RR>>,

    /// Indices marking the start of each surface in the rings vector
    surfaces: Vec<VertexIndex<VR>>,

    /// Indices marking the start of each shell in the surfaces vector
    shells: Vec<VertexIndex<VR>>,

    /// Indices marking the start of each solid in the shells vector
    solids: Vec<VertexIndex<VR>>,
}

impl<VR: VertexRef, RR: ResourceRef> TextureMap<VR, RR> {
    /// Creates a new empty TextureMap.
    ///
    /// # Returns
    ///
    /// A new TextureMap instance with no mappings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let texture_map = TextureMap::<u32, ResourceId32>::new();
    /// assert!(texture_map.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new TextureMap with the specified capacities for its collections.
    ///
    /// This method allows efficient memory allocation when the approximate sizes
    /// of the collections are known in advance.
    ///
    /// # Parameters
    ///
    /// * `vertex_capacity` - Capacity for the vertices vector
    /// * `ring_capacity` - Capacity for the rings vector
    /// * `ring_texture_capacity` - Capacity for the ring_textures vector
    /// * `surface_capacity` - Capacity for the surfaces vector
    /// * `shell_capacity` - Capacity for the shells vector
    /// * `solid_capacity` - Capacity for the solids vector
    ///
    /// # Returns
    ///
    /// A new TextureMap instance with preallocated memory for its collections.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// // Create a texture map with preallocated capacity
    /// let texture_map = TextureMap::<u32, ResourceId32>::with_capacity(100, 10, 10, 5, 2, 1);
    /// ```
    pub fn with_capacity(
        vertex_capacity: usize,
        ring_capacity: usize,
        ring_texture_capacity: usize,
        surface_capacity: usize,
        shell_capacity: usize,
        solid_capacity: usize,
    ) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_capacity),
            rings: Vec::with_capacity(ring_capacity),
            ring_textures: Vec::with_capacity(ring_texture_capacity),
            surfaces: Vec::with_capacity(surface_capacity),
            shells: Vec::with_capacity(shell_capacity),
            solids: Vec::with_capacity(solid_capacity),
        }
    }

    /// Returns true if the texture map contains no mappings.
    ///
    /// # Returns
    ///
    /// `true` if all collections in the map are empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let texture_map = TextureMap::<u32, ResourceId32>::new();
    /// assert!(texture_map.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
            && self.rings.is_empty()
            && self.ring_textures.is_empty()
            && self.surfaces.is_empty()
            && self.shells.is_empty()
            && self.solids.is_empty()
    }

    /// Adds a texture vertex reference.
    ///
    /// # Parameters
    ///
    /// * `vertex` - Optional reference to a texture vertex, or None if no texture
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut texture_map = TextureMap::<u32, ResourceId32>::new();
    /// texture_map.add_vertex(Some(VertexIndex::new(0)));
    /// texture_map.add_vertex(None); // No texture for this vertex
    /// ```
    pub fn add_vertex(&mut self, vertex: Option<VertexIndex<VR>>) {
        self.vertices.push(vertex);
    }

    /// Adds a ring index.
    ///
    /// # Parameters
    ///
    /// * `ring_start` - Index marking the start of a ring in the vertices vector
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut texture_map = TextureMap::<u32, ResourceId32>::new();
    /// texture_map.add_ring(VertexIndex::new(0));
    /// ```
    pub fn add_ring(&mut self, ring_start: VertexIndex<VR>) {
        self.rings.push(ring_start);
    }

    /// Adds a texture reference for a ring.
    ///
    /// # Parameters
    ///
    /// * `texture` - Optional reference to a texture resource, or None if no texture
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut texture_map = TextureMap::<u32, ResourceId32>::new();
    /// texture_map.add_ring_texture(Some(ResourceId32::new(5, 1))); // Reference to texture ID 5
    /// texture_map.add_ring_texture(None);    // No texture for this ring
    /// ```
    pub fn add_ring_texture(&mut self, texture: Option<RR>) {
        self.ring_textures.push(texture);
    }

    /// Adds a surface index.
    ///
    /// # Parameters
    ///
    /// * `surface_start` - Index marking the start of a surface in the rings vector
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut texture_map = TextureMap::<u32, ResourceId32>::new();
    /// texture_map.add_surface(VertexIndex::new(0));
    /// ```
    pub fn add_surface(&mut self, surface_start: VertexIndex<VR>) {
        self.surfaces.push(surface_start);
    }

    /// Adds a shell index.
    ///
    /// # Parameters
    ///
    /// * `shell_start` - Index marking the start of a shell in the surfaces vector
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut texture_map = TextureMap::<u32, ResourceId32>::new();
    /// texture_map.add_shell(VertexIndex::new(0));
    /// ```
    pub fn add_shell(&mut self, shell_start: VertexIndex<VR>) {
        self.shells.push(shell_start);
    }

    /// Adds a solid index.
    ///
    /// # Parameters
    ///
    /// * `solid_start` - Index marking the start of a solid in the shells vector
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut texture_map = TextureMap::<u32, ResourceId32>::new();
    /// texture_map.add_solid(VertexIndex::new(0));
    /// ```
    pub fn add_solid(&mut self, solid_start: VertexIndex<VR>) {
        self.solids.push(solid_start);
    }

    /// Returns a reference to the texture vertices.
    ///
    /// # Returns
    ///
    /// A slice containing references to texture vertices.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut texture_map = TextureMap::<u32, ResourceId32>::new();
    /// texture_map.add_vertex(Some(VertexIndex::new(0)));
    /// texture_map.add_vertex(Some(VertexIndex::new(1)));
    ///
    /// let vertices = texture_map.vertices();
    /// assert_eq!(vertices.len(), 2);
    /// ```
    pub fn vertices(&self) -> &[Option<VertexIndex<VR>>] {
        &self.vertices
    }

    pub fn vertices_mut(&mut self) -> &mut [Option<VertexIndex<VR>>] {
        &mut self.vertices
    }

    /// Returns a reference to the ring indices.
    ///
    /// # Returns
    ///
    /// A slice containing ring indices.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut texture_map = TextureMap::<u32, ResourceId32>::new();
    /// texture_map.add_ring(VertexIndex::new(0));
    ///
    /// let rings = texture_map.rings();
    /// assert_eq!(rings.len(), 1);
    /// ```
    pub fn rings(&self) -> &[VertexIndex<VR>] {
        &self.rings
    }

    pub fn rings_mut(&mut self) -> &mut [VertexIndex<VR>] {
        &mut self.rings
    }

    /// Returns a reference to the ring texture references.
    ///
    /// # Returns
    ///
    /// A slice containing texture references for rings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cityjson::prelude::*;
    ///
    /// let mut texture_map = TextureMap::<u32, ResourceId32>::new();
    /// texture_map.add_ring_texture(Some(ResourceId32::new(5, 1)));
    ///
    /// let textures = texture_map.ring_textures();
    /// assert_eq!(textures.len(), 1);
    /// ```
    pub fn ring_textures(&self) -> &[Option<RR>] {
        &self.ring_textures
    }
    pub fn ring_textures_mut(&mut self) -> &mut [Option<RR>] {
        &mut self.ring_textures
    }

    /// Returns a reference to the surface indices.
    ///
    /// # Returns
    ///
    /// A slice containing surface indices.
    pub fn surfaces(&self) -> &[VertexIndex<VR>] {
        &self.surfaces
    }
    pub fn surfaces_mut(&mut self) -> &mut [VertexIndex<VR>] {
        &mut self.surfaces
    }

    /// Returns a reference to the shell indices.
    ///
    /// # Returns
    ///
    /// A slice containing shell indices.
    pub fn shells(&self) -> &[VertexIndex<VR>] {
        &self.shells
    }
    pub fn shells_mut(&mut self) -> &mut [VertexIndex<VR>] {
        &mut self.shells
    }

    /// Returns a reference to the solid indices.
    ///
    /// # Returns
    ///
    /// A slice containing solid indices.
    pub fn solids(&self) -> &[VertexIndex<VR>] {
        &self.solids
    }
    pub fn solids_mut(&mut self) -> &mut [VertexIndex<VR>] {
        &mut self.solids
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cityjson::vertex::VertexIndex;
    use crate::resources::pool::ResourceId32;

    #[test]
    fn test_texture_map_creation() {
        // Test creating an empty texture map
        let texture_map = TextureMap::<u32, ResourceId32>::new();
        assert!(texture_map.is_empty());
        assert!(texture_map.vertices().is_empty());
        assert!(texture_map.rings().is_empty());
        assert!(texture_map.ring_textures().is_empty());
        assert!(texture_map.surfaces().is_empty());
        assert!(texture_map.shells().is_empty());
        assert!(texture_map.solids().is_empty());
    }

    #[test]
    fn test_texture_map_population() {
        // Create and populate a texture map
        let mut texture_map = TextureMap::<u32, ResourceId32>::new();

        // Add vertices
        texture_map.add_vertex(Some(VertexIndex::new(0)));
        texture_map.add_vertex(Some(VertexIndex::new(1)));
        texture_map.add_vertex(Some(VertexIndex::new(2)));
        texture_map.add_vertex(None); // No texture for fourth vertex

        // Add rings
        texture_map.add_ring(VertexIndex::new(0));
        texture_map.add_ring(VertexIndex::new(3));

        // Add textures for rings
        texture_map.add_ring_texture(Some(ResourceId32::new(5, 0)));
        texture_map.add_ring_texture(Some(ResourceId32::new(6, 0)));

        // Verify structure
        assert_eq!(texture_map.vertices().len(), 4);
        assert_eq!(texture_map.rings().len(), 2);
        assert_eq!(texture_map.ring_textures().len(), 2);

        // Verify content
        assert_eq!(texture_map.vertices()[0], Some(VertexIndex::new(0)));
        assert_eq!(texture_map.vertices()[3], None);
        assert_eq!(texture_map.rings()[0], VertexIndex::new(0));
        assert_eq!(
            texture_map.ring_textures()[1],
            Some(ResourceId32::new(6, 0))
        );
    }

    #[test]
    fn test_texture_map_hierarchy() {
        // Test the hierarchical structure of TextureMap
        let mut texture_map = TextureMap::<u32, ResourceId32>::new();

        // Add vertices
        for i in 0..8 {
            texture_map.add_vertex(Some(VertexIndex::new(i)));
        }

        // Add rings
        texture_map.add_ring(VertexIndex::new(0)); // First ring starts at vertex 0
        texture_map.add_ring(VertexIndex::new(4)); // Second ring starts at vertex 4

        // Add ring textures
        texture_map.add_ring_texture(Some(ResourceId32::new(10, 0)));
        texture_map.add_ring_texture(Some(ResourceId32::new(11, 0)));

        // Add surfaces
        texture_map.add_surface(VertexIndex::new(0)); // First surface starts at ring 0
        texture_map.add_surface(VertexIndex::new(1)); // Second surface starts at ring 1

        // Add shells
        texture_map.add_shell(VertexIndex::new(0)); // First shell starts at surface 0

        // Add solids
        texture_map.add_solid(VertexIndex::new(0)); // First solid starts at shell 0

        // Verify structure
        assert_eq!(texture_map.vertices().len(), 8);
        assert_eq!(texture_map.rings().len(), 2);
        assert_eq!(texture_map.ring_textures().len(), 2);
        assert_eq!(texture_map.surfaces().len(), 2);
        assert_eq!(texture_map.shells().len(), 1);
        assert_eq!(texture_map.solids().len(), 1);
    }

    #[test]
    fn test_accessor_methods() {
        let mut texture_map = TextureMap::<u32, ResourceId32>::new();

        // Add data
        texture_map.add_vertex(Some(VertexIndex::new(5)));
        texture_map.add_ring(VertexIndex::new(0));
        texture_map.add_ring_texture(Some(ResourceId32::new(3, 0)));
        texture_map.add_surface(VertexIndex::new(0));
        texture_map.add_shell(VertexIndex::new(0));
        texture_map.add_solid(VertexIndex::new(0));

        // Test accessors
        assert_eq!(texture_map.vertices()[0], Some(VertexIndex::new(5)));
        assert_eq!(texture_map.rings()[0], VertexIndex::new(0));
        assert_eq!(
            texture_map.ring_textures()[0],
            Some(ResourceId32::new(3, 0))
        );
        assert_eq!(texture_map.surfaces()[0], VertexIndex::new(0));
        assert_eq!(texture_map.shells()[0], VertexIndex::new(0));
        assert_eq!(texture_map.solids()[0], VertexIndex::new(0));
    }
}
