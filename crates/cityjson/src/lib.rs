use std::fmt::Debug;

pub mod boundary;
pub mod boundary_nested;
pub mod coordinate;
pub mod errors;
mod resources_semantics_materials;
mod resources_textures;
pub mod vertex;

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

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Geometry<T: VertexInteger> {
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary<T>>,
    semantics: Option<SemanticMaterialMap<T>>,
    template_boundaries: Option<usize>,
    template_transformation_matrix: Option<[f64; 16]>,
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
    use crate::boundary::Boundary;
    use crate::resources_semantics_materials::SemanticMaterialMap;
    use crate::v1_1::semantics::SemanticType;
    use crate::vertex::{OptionalVertexIndices, VertexIndices};

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

    #[test]
    fn test_city_model_with_semantic_surface() {
        // Create a new CityModel using u32 for indices
        let mut model = CityModel::<u32>::new();

        // Add some vertices for a simple cube (front face only)
        let v0 = model
            .add_vertex(VertexCoordinate {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            })
            .unwrap();
        let v1 = model
            .add_vertex(VertexCoordinate {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            })
            .unwrap();
        let v2 = model
            .add_vertex(VertexCoordinate {
                x: 1.0,
                y: 1.0,
                z: 0.0,
            })
            .unwrap();
        let v3 = model
            .add_vertex(VertexCoordinate {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            })
            .unwrap();

        // Create a boundary representing a MultiSurface with one surface (square)
        let mut boundary = Boundary::new();
        boundary.vertices = VertexIndices::from_iter([v0, v1, v2, v3]);
        boundary.rings = VertexIndices::from_iter([VertexIndex::new(0u32)]);
        boundary.surfaces = VertexIndices::from_iter([VertexIndex::new(0u32)]);

        // Create a wall surface semantic
        let wall_semantic = Semantic::new(SemanticType::WallSurface);
        let wall_id = model.add_semantic(wall_semantic);

        // Create semantic mapping for the surface
        let mut semantic_map = SemanticMaterialMap::<u32>::default();
        semantic_map.surfaces =
            OptionalVertexIndices::from_iter([Some(VertexIndex::new(wall_id.index()))]);

        // Create the geometry
        let geometry = Geometry {
            type_geometry: GeometryType::MultiSurface,
            lod: Some(LoD::LoD2),
            boundaries: Some(boundary),
            semantics: Some(semantic_map),
            template_boundaries: None,
            template_transformation_matrix: None,
        };

        // Add geometry to model
        model.add_geometry(geometry);

        // Verify the model
        assert_eq!(model.vertex_count(), 4);
        assert_eq!(model.semantic_count(), 1);
        assert_eq!(model.geometry_count(), 1);

        // Verify geometry and semantics
        if let Some(geometry) = model.geometries.get(0) {
            // Check the geometry type
            assert_eq!(geometry.type_geometry, GeometryType::MultiSurface);

            // Check boundary
            if let Some(boundary) = &geometry.boundaries {
                assert_eq!(boundary.vertices.len(), 4u32);
                assert_eq!(boundary.rings.len(), 1u32);
                assert_eq!(boundary.surfaces.len(), 1u32);
            } else {
                panic!("Expected boundary");
            }

            // Check semantic mapping
            if let Some(semantic_map) = &geometry.semantics {
                let surfaces = &semantic_map.surfaces;
                assert_eq!(surfaces.len(), 1u32);

                // Verify semantic reference
                if let Some(Some(semantic_idx)) = surfaces.get(VertexIndex::new(0u32)) {
                    let semantic = model
                        .get_semantic(ResourceId::new(semantic_idx.value(), 0))
                        .expect("Semantic should exist");

                    assert!(matches!(semantic.type_semantic, SemanticType::WallSurface));
                } else {
                    panic!("Expected semantic mapping");
                }
            } else {
                panic!("Expected semantic map");
            }
        } else {
            panic!("Expected geometry");
        }
    }
}
