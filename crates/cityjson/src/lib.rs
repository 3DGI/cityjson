use std::fmt::Debug;

pub mod boundary;
pub mod boundary_nested;
pub mod coordinate;
pub mod errors;
mod resources_semantics_materials;
mod resources_textures;
pub mod vertex;
pub mod geometry;

mod resource_pool;
pub mod v1_1;

use crate::coordinate::Vertices;
use crate::errors::Result;
use crate::resource_pool::{DefaultResourcePool, ResourceId, ResourcePool};
use crate::v1_1::semantics::Semantic;
use crate::vertex::VertexInteger;
pub use boundary::Boundary;
pub use coordinate::VertexCoordinate;
pub use resources_semantics_materials::SemanticMaterialMap;
pub use resources_textures::TextureMap;
pub use vertex::VertexIndex;
pub use geometry::Geometry;


pub type CityModel<T> = GenericCityModel<T, DefaultResourcePool<Semantic<T>>>;

#[derive(Debug)]
pub struct GenericCityModel<T: VertexInteger, P: ResourcePool<Semantic<T>>> {
    /// Pool of vertex coordinates
    vertices: Vertices<T>,
    /// Pool of semantic objects
    semantics: P,
    /// Collection of geometries
    geometries: Vec<Geometry<T>>,
}

impl<T: VertexInteger, P: ResourcePool<Semantic<T>>> GenericCityModel<T, P> {
    /// Create a new empty CityModel
    pub fn new() -> Self {
        Self {
            vertices: Vertices::new(),
            semantics: P::new(),
            geometries: Vec::new(),
        }
    }

    /// Create a new CityModel with the specified capacity
    pub fn with_capacity(
        _vertex_capacity: usize,
        semantic_capacity: usize,
        geometry_capacity: usize,
    ) -> Self {
        Self {
            vertices: Vertices::new(), // Vertices handle capacity internally
            semantics: P::with_capacity(semantic_capacity),
            geometries: Vec::with_capacity(geometry_capacity),
        }
    }

    /// Add a semantic object to the pool
    pub fn add_semantic(&mut self, semantic: Semantic<T>) -> ResourceId {
        self.semantics.add(semantic)
    }

    /// Get a reference to a semantic object
    pub fn get_semantic(&self, id: ResourceId) -> Option<&Semantic<T>> {
        self.semantics.get(id)
    }

    /// Get a mutable reference to a semantic object
    pub fn get_semantic_mut(&mut self, id: ResourceId) -> Option<&mut Semantic<T>> {
        self.semantics.get_mut(id)
    }

    /// Add a geometry to the model
    pub fn add_geometry(&mut self, geometry: Geometry<T>) {
        self.geometries.push(geometry);
    }

    /// Add a vertex coordinate
    pub fn add_vertex(&mut self, coordinate: VertexCoordinate) -> Result<VertexIndex<T>> {
        self.vertices.push(coordinate)
    }

    /// Get a reference to a vertex coordinate
    pub fn get_vertex(&self, index: VertexIndex<T>) -> Option<&VertexCoordinate> {
        self.vertices.get(index)
    }

    /// Get the number of geometries
    pub fn geometry_count(&self) -> usize {
        self.geometries.len()
    }

    /// Get the number of semantics
    pub fn semantic_count(&self) -> usize {
        self.semantics.iter().count()
    }

    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.as_slice().len()
    }
}

// Implement default for convenience
impl<T: VertexInteger, P: ResourcePool<Semantic<T>>> Default for GenericCityModel<T, P> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum GeometryType {
    MultiPoint,
    MultiLineString,
    MultiSurface,
    CompositeSurface,
    Solid,
    MultiSolid,
    CompositeSolid,
    GeometryInstance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LoD {
    LoD0,
    LoD0_0,
    LoD0_1,
    LoD0_2,
    LoD0_3,
    LoD1,
    LoD1_0,
    LoD1_1,
    LoD1_2,
    LoD1_3,
    LoD2,
    LoD2_0,
    LoD2_1,
    LoD2_2,
    LoD2_3,
    LoD3,
    LoD3_0,
    LoD3_1,
    LoD3_2,
    LoD3_3,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometry_type_equality() {
        assert_eq!(GeometryType::MultiPoint, GeometryType::MultiPoint);
        assert_ne!(GeometryType::MultiPoint, GeometryType::MultiSurface);
    }

    #[test]
    fn test_lod_ordering() {
        assert!(LoD::LoD0 < LoD::LoD1);
        assert!(LoD::LoD1 < LoD::LoD2);
        assert!(LoD::LoD2 < LoD::LoD3);
        assert!(LoD::LoD0_1 > LoD::LoD0);
        assert!(LoD::LoD1_2 > LoD::LoD1);
    }
}
