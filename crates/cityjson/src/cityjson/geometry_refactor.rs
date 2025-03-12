use crate::errors::{Error, Result};
use crate::prelude::{
    Boundary, CityModelTrait, CityModelTypes, Coordinate, GeometryTrait, GeometryType, LoD,
    SemanticMap, VertexIndex, VertexRef,
};
use std::collections::HashMap;

/// Represents a surface under construction with one outer ring and optional inner rings
#[derive(Default)]
struct SurfaceInProgress {
    outer_ring: Option<usize>, // index to outer ring
    inner_rings: Vec<usize>,   // indices to inner rings
}

#[derive(Default)]
struct ShellInProgress {
    outer_surfaces: Vec<usize>, // indices to outer surfaces
    inner_surfaces: Vec<usize>, // indices to inner surfaces (voids)
}

#[derive(Default)]
struct SolidInProgress {
    outer_shell: Option<usize>, // index to outer shell
    inner_shells: Vec<usize>,   // indices to inner shells (voids)
}

enum VertexOrPoint<V: VertexRef, C: Coordinate> {
    Vertex(VertexIndex<V>),
    Point(C),
}

/// Geometry builder.
///
/// The GeometryBuilder is generic over the CityModel and Coordinate type, thus it can
/// build a CityModel with either real-world coordinates or quantized coordinates,
/// for all supported CityJSON versions.
pub struct GeometryBuilder<'a, V: CityModelTypes, M: CityModelTrait<V>> {
    model: &'a mut M,
    type_geometry: GeometryType,
    lod: Option<LoD>,
    transformation_matrix: Option<[f64; 16]>,
    vertices: Vec<VertexOrPoint<V::VertexRef, V::CoordinateType>>,
    rings: Vec<Vec<usize>>,       // indices into vertices
    surfaces: Vec<SurfaceInProgress>, // surfaces with their rings
    shells: Vec<ShellInProgress>,     // A solid with its shells, each shell with their surfaces
    solids: Vec<SolidInProgress>,     // M/CSolid with its shells
    // Active element tracking
    active_linestring: Option<usize>, // active linestring being built
    active_surface: Option<usize>,    // active surface being built
    active_shell: Option<usize>,      // active shell being built
    active_solid: Option<usize>,      // active solid being built
    // Semantic storage
    point_semantics: HashMap<usize, V::ResourceRef>,
    linestring_semantics: HashMap<usize, V::ResourceRef>,
    surface_semantics: HashMap<usize, V::ResourceRef>,
    // Material storage
    surface_materials: HashMap<usize, V::ResourceRef>,
    // Texture storage
    surface_textures: HashMap<usize, V::ResourceRef>,
}

impl<'a, V: CityModelTypes, M: CityModelTrait<V>> GeometryBuilder<'a, V, M> {
    /// Instantiates a new GeometryBuilder.
    ///
    /// # Parameters
    /// * `model` - A CityModel instance.
    /// * `type_geometry` - The geometry type to build.
    pub fn new(model: &'a mut M, type_geometry: GeometryType) -> Self {
        Self {
            model,
            type_geometry,
            lod: None,
            transformation_matrix: None,
            vertices: Vec::new(),
            rings: Vec::new(),
            surfaces: Vec::new(),
            shells: Vec::new(),
            solids: Vec::new(),
            active_linestring: None,
            active_surface: None,
            active_shell: None,
            active_solid: None,
            point_semantics: Default::default(),
            linestring_semantics: Default::default(),
            surface_semantics: Default::default(),
            surface_materials: Default::default(),
            surface_textures: Default::default(),
        }
    }

    /// Set the Level of Detail on the Geometry.
    pub fn with_lod(mut self, lod: LoD) -> Self {
        self.lod = Some(lod);
        self
    }

    /// Set the Transformation Matrix on the Geometry (for `GeometryInstance` only).
    ///
    /// # Errors
    ///
    /// Returns [Error::InvalidGeometryType] if geometry is not a `GeometryInstance`.
    pub fn with_transformation_matrix(mut self, transformation_matrix: [f64; 16]) -> Result<Self> {
        if self.type_geometry != GeometryType::GeometryInstance {
            return Err(Error::InvalidGeometryType {
                expected: "GeometryInstance".to_string(),
                found: self.type_geometry.to_string(),
            });
        }
        self.transformation_matrix = Some(transformation_matrix);
        Ok(self)
    }

    /// Add a new point to the boundary by providing its coordinates. The point will be
    /// added as a new vertex to the vertex pool. Use this method when adding completely
    /// new vertices to the CityModel and the Boundary. Can be used interchangeably
    /// with [add_vertex] for building a Boundary.
    ///
    /// # Returns
    ///
    /// The index of the added vertex in the boundary.
    pub fn add_point(&mut self, point: V::CoordinateType) -> usize {
        self.vertices.push(VertexOrPoint::Point(point));
        self.vertices.len() - 1
    }

    /// Add an existing vertex to the boundary by providing its reference in the vertex
    /// pool. Use this method when reusing existing vertices for the boundary. Can be
    /// used interchangeably with [add_point] for building a Boundary.
    ///
    /// # Returns
    ///
    /// The index of the added vertex in the boundary.
    pub fn add_vertex(&mut self, vertex: VertexIndex<V::VertexRef>) -> usize {
        self.vertices.push(VertexOrPoint::Vertex(vertex));
        self.vertices.len() - 1
    }

    /// Add a LineString to the boundary by providing its vertex indices in the boundary.
    /// The indices are returned by the [add_point] or [add_vertex] methods.
    ///
    /// # Errors
    /// * `InvalidLineString` - If less than two vertices have been provided
    ///
    /// # Returns
    ///
    /// The index of the added LineString in the boundary.
    pub fn add_linestring(&mut self, vertices: &[usize]) -> Result<usize> {
        if vertices.len() < 2 {
            return Err(Error::InvalidLineString {
                reason: "LineString must have at least 2 vertices".to_string(),
                vertex_count: vertices.len()
            });
        }
        self.rings.push(vertices.to_vec());
        Ok(self.rings.len() - 1)
    }

    /// Add a ring to the boundary by providing its vertex indices in the boundary.
    /// The indices are returned by the [add_point] or [add_vertex] methods.
    ///
    /// # Errors
    /// * `InvalidRing` - If less than three vertices have been provided
    ///
    /// # Returns
    ///
    /// The index of the added ring in the boundary.
    pub fn add_ring(&mut self, vertices: &[usize]) -> Result<usize> {
        if vertices.len() < 3 {
            return Err(Error::InvalidRing {
                reason: "ring must have at least 3 vertices".to_string(),
                vertex_count: vertices.len()
            });
        }
        self.rings.push(vertices.to_vec());
        Ok(self.rings.len() - 1)
    }

    /// Set the Semantic on a Point.
    /// A Point can only have one semantic value. The Semantic is directly added to the
    /// `model`.
    ///
    ///
    /// # Parameters
    ///
    /// * `index` - The index of the point that will get the semantic. The index is the
    /// value returned by the [add_point] or [add_vertex] methods. If
    /// `None`, the Semantic is added to the last vertex in the GeometryBuilder.
    /// * `semantic` - The semantic instance to add to the Point.
    ///
    /// # Returns
    ///
    /// The reference to the Semantic in the resource pool of the `model`.
    pub fn set_semantic_point(
        &mut self,
        index: Option<usize>,
        semantic: V::Semantic,
    ) -> V::ResourceRef {
        let semantic_ref = self.model.add_semantic(semantic);
        let vertex_i = if let Some(i) = index {
            i
        } else {
            self.vertices.len() - 1
        };
        self.point_semantics.insert(vertex_i, semantic_ref);
        semantic_ref
    }

    /// Set the Semantic on a LineString.
    /// A LineString can only have one semantic value. The Semantic is directly added to the
    /// `model`.
    ///
    ///
    /// # Parameters
    ///
    /// * `index` - The index of the LineString that will get the semantic. The index is the
    /// value returned by the [add_linestring] or [add_ring] methods. If
    /// `None`, the Semantic is added to the last LineString in the GeometryBuilder.
    /// * `semantic` - The semantic instance to add to the LineString.
    ///
    /// # Returns
    ///
    /// The reference to the Semantic in the resource pool of the `model`.
    pub fn set_semantic_linestring(
        &mut self,
        index: Option<usize>,
        semantic: V::Semantic,
    ) -> V::ResourceRef {
        let semantic_ref = self.model.add_semantic(semantic);
        let ring_i = if let Some(i) = index {
            i
        } else {
            self.rings.len() - 1
        };
        self.linestring_semantics.insert(ring_i, semantic_ref);
        semantic_ref
    }

    /// Builds the geometry and adds it to the `model`.
    ///
    /// # Errors
    /// * The geometry type does not match the structure (`InvalidGeometryType`)
    /// * The `model`'s vertex container has reached its maximum capacity (`VerticesContainerFull`)
    pub fn build(self) -> Result<V::ResourceRef> {
        // Validate structure before building
        self.validate_structure()?;

        let mut boundary = Boundary::with_capacity(
            self.vertices.len(),
            self.rings.len(),
            self.surfaces.len(),
            self.shells.len(),
            self.solids.len(),
        );
        let cnt_new_vertices = self.vertices.iter().filter(|v| matches!(v, VertexOrPoint::Point(_))).count();
        if cnt_new_vertices > 0 {
            self.model
                .vertices_mut()
                .reserve(cnt_new_vertices)?;
        }

        let mut semantic_map_optional = None;

        match self.type_geometry {
            GeometryType::MultiPoint => {
                for point in self.vertices {
                    match point {
                        VertexOrPoint::Vertex(v) => {
                            boundary.vertices.push(v);
                        }
                        VertexOrPoint::Point(p) => {
                            boundary.vertices.push(self.model.add_vertex(p)?)
                        }
                    }
                }
                if !self.point_semantics.is_empty() {
                    let mut semantic_map = SemanticMap::<V::VertexRef, V::ResourceRef>::default();
                    for i in 0..boundary.vertices.len() {
                        semantic_map
                            .points
                            .push(self.point_semantics.get(&i).copied());
                    }
                    semantic_map_optional = Some(semantic_map);
                }
            }
            _ => {
                unimplemented!()
            }
        }

        // Create the geometry
        let geometry = V::Geometry::new(
            self.type_geometry,
            self.lod,
            Some(boundary),
            semantic_map_optional,
            None,
            None,
            None,
            None,
        );

        Ok(self.model.add_geometry(geometry))
    }

    fn validate_structure(&self) -> Result<()> {
        match self.type_geometry {
            GeometryType::MultiPoint => {
                if !self.solids.is_empty()
                    || !self.shells.is_empty()
                    || !self.surfaces.is_empty()
                    || !self.rings.is_empty()
                    || self.vertices.is_empty()
                {
                    return Err(Error::InvalidGeometryType {
                        expected: "multi point geometry".to_string(),
                        found: self.format_counts(),
                    });
                }
            }
            _ => {
                unimplemented!()
            }
        }
        Ok(())
    }

    fn format_counts(&self) -> String {
        format!(
            "{} solids, {} shells, {} surfaces, {} rings, {} vertices",
            self.solids.len(),
            self.shells.len(),
            self.surfaces.len(),
            self.rings.len(),
            self.vertices.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cityjson::geometry::GeometryType;
    use crate::prelude::{QuantizedCoordinate, ResourcePool, SemanticTrait};
    use crate::resources::pool::ResourceId32;
    use crate::resources::storage::OwnedStringStorage;
    use crate::v1_1::{CityModel, Semantic, SemanticType};
    use crate::CityModelType;

    // Test helper to create a new model
    fn create_test_model() -> CityModel<u32, ResourceId32, OwnedStringStorage> {
        CityModel::new(CityModelType::CityJSON)
    }

    #[test]
    fn test_multipoint_with_add_vertex() {
        let mut model = create_test_model();

        // First, add some vertices to the model
        let v0 = model
            .add_vertex(QuantizedCoordinate::new(1, 2, 3))
            .unwrap();
        let v1 = model
            .add_vertex(QuantizedCoordinate::new(4, 5, 6))
            .unwrap();
        let v2 = model
            .add_vertex(QuantizedCoordinate::new(7, 8, 9))
            .unwrap();

        // Create a builder for MultiPoint geometry
        let mut builder = GeometryBuilder::new(&mut model, GeometryType::MultiPoint);

        // Add existing vertices
        builder.add_vertex(v0);
        builder.add_vertex(v1);
        builder.add_vertex(v2);
        builder.add_vertex(v1);

        // Build the geometry
        let geom_ref = builder.build().expect("Failed to build geometry");

        // Get the geometry from the model
        let geometry = model
            .geometries()
            .get(geom_ref)
            .expect("Failed to get geometry");

        // Check geometry type
        assert_eq!(geometry.type_geometry(), &GeometryType::MultiPoint);

        // Get the boundary and convert to nested representation
        let boundary = geometry.boundaries().expect("No boundary found");
        let nested = boundary
            .to_nested_multi_point()
            .expect("Failed to convert to nested");

        // Verify the nested representation (should have 3 points)
        assert_eq!(model.vertex_count(), 3);
        assert_eq!(nested, vec![0, 1, 2, 1]);
    }

    #[test]
    fn test_multipoint_with_add_point() {
        let mut model = create_test_model();

        // Create a builder for MultiPoint geometry
        let mut builder = GeometryBuilder::new(&mut model, GeometryType::MultiPoint);

        // Add points
        builder.add_point(QuantizedCoordinate::new(1, 2, 3));
        builder.add_point(QuantizedCoordinate::new(4, 5, 6));
        builder.add_point(QuantizedCoordinate::new(7, 8, 9));

        // Set LoD (optional)
        builder = builder.with_lod(LoD::LoD1);

        // Build the geometry
        let geom_ref = builder.build().expect("Failed to build geometry");

        // Get the geometry from the model
        let geometry = model.geometries().get(geom_ref).expect("Failed to get geometry");

        // Check geometry type and LoD
        assert_eq!(geometry.type_geometry(), &GeometryType::MultiPoint);
        assert_eq!(geometry.lod(), Some(&LoD::LoD1));

        // Get the boundary and convert to nested representation
        let boundary = geometry.boundaries().expect("No boundary found");
        let nested = boundary.to_nested_multi_point().expect("Failed to convert to nested");

        // Verify the nested representation (should have 3 points)
        assert_eq!(model.vertex_count(), 3);
        assert_eq!(nested, vec![0, 1, 2]);

    }

    #[test]
    fn test_multipoint_with_mixed_adds() {
        let mut model = create_test_model();

        // First add a vertex to the citymodel
        let v0 = model.add_vertex(QuantizedCoordinate::new(1, 2, 3)).unwrap();
        let v1 = model.add_vertex(QuantizedCoordinate::new(10, 11, 12)).unwrap();

        // Create a builder for MultiPoint geometry
        let mut builder = GeometryBuilder::new(&mut model, GeometryType::MultiPoint);

        // Mix adding vertices and points
        builder.add_vertex(v0);
        builder.add_point(QuantizedCoordinate::new(4, 5, 6)); // 2
        builder.add_vertex(v1);
        builder.add_point(QuantizedCoordinate::new(7, 8, 9)); // 3
        builder.add_vertex(v0);

        // Build the geometry
        let geom_ref = builder.build().expect("Failed to build geometry");

        // Get the geometry from the model
        let geometry = model.geometries().get(geom_ref).expect("Failed to get geometry");

        // Check geometry type
        assert_eq!(geometry.type_geometry(), &GeometryType::MultiPoint);

        // Get the boundary and convert to nested representation
        let boundary = geometry.boundaries().expect("No boundary found");
        let nested = boundary.to_nested_multi_point().expect("Failed to convert to nested");

        // Verify the nested representation (should have 3 points)
        assert_eq!(model.vertex_count(), 4);
        assert_eq!(nested, vec![0, 2, 1, 3, 0]);
    }

    #[test]
    fn test_multipoint_with_semantics() {
        let mut model = create_test_model();

        // Create a builder for MultiPoint geometry
        let mut builder = GeometryBuilder::new(&mut model, GeometryType::MultiPoint);

        // Add points
        let p0 = builder.add_point(QuantizedCoordinate::new(1, 2, 3));
        let _p1 = builder.add_point(QuantizedCoordinate::new(4, 5, 6));
        let p2 = builder.add_point(QuantizedCoordinate::new(7, 8, 9));

        // Create semantics
        let sem0 = Semantic::new(SemanticType::TransportationHole);
        let sem1 = Semantic::new(SemanticType::TransportationMarking);

        // Set semantics for two of the points
        let sem_ref0 = builder.set_semantic_point(Some(p0), sem0);
        let sem_ref1 = builder.set_semantic_point(Some(p2), sem1);

        // Build the geometry
        let geom_ref = builder.build().expect("Failed to build geometry");

        // Get the geometry from the model
        let geometry = model.geometries().get(geom_ref).expect("Failed to get geometry");

        // Check geometry type
        assert_eq!(geometry.type_geometry(), &GeometryType::MultiPoint);

        // Get the boundary and convert to nested representation
        let boundary = geometry.boundaries().expect("No boundary found");
        let nested = boundary.to_nested_multi_point().expect("Failed to convert to nested");

        // Verify the nested representation (should have 3 points)
        assert_eq!(model.vertex_count(), 3);
        assert_eq!(nested, vec![0, 1, 2]);

        // Check semantics
        let semantics = geometry.semantics().expect("No semantics found");
        let semantic_points = semantics.points();

        // Verify points have semantics applied correctly
        assert_eq!(semantic_points.len(), 3);

        // Verify the semantic references are the ones we set
        let sem_refs: Vec<ResourceId32> = semantic_points.iter()
            .filter_map(|s| s.as_ref())
            .cloned()
            .collect();
        assert!(sem_refs.contains(&sem_ref0));
        assert!(sem_refs.contains(&sem_ref1));

        // Verify the semantics themselves
        let semantic0 = model.get_semantic(sem_ref0).expect("Semantic 0 not found");
        assert_eq!(semantic0.type_semantic(), &SemanticType::TransportationHole);

        let semantic1 = model.get_semantic(sem_ref1).expect("Semantic 1 not found");
        assert_eq!(semantic1.type_semantic(), &SemanticType::TransportationMarking);
    }
}
