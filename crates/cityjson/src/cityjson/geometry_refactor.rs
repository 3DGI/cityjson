use crate::errors::{Error, Result};
use crate::prelude::{CityModelTrait, CityModelTypes, Coordinate, GeometryType, LoD, VertexIndex, VertexRef};
use std::collections::HashMap;

#[derive(Default)]
struct RingInProgress {
    vertices: Vec<usize>,
}

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
    Point(C)
}

pub struct GeometryBuilder<'a, V: CityModelTypes, M: CityModelTrait<V>> {
    model: &'a mut M,
    type_geometry: GeometryType,
    lod: Option<LoD>,
    transformation_matrix: Option<[f64; 16]>,
    vertices: Vec<VertexOrPoint<V::VertexRef, V::CoordinateType>>,
    rings: Vec<RingInProgress>,           // indices into vertices
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
    pub fn add_point(&mut self, point: V::CoordinateType) -> usize {
        self.vertices.push(VertexOrPoint::Point(point));
        self.vertices.len() - 1
    }

    /// Add an existing vertex to the boundary by providing its reference in the vertex
    /// pool. Use this method when reusing existing vertices for the boundary. Can be
    /// used interchangeably with [add_point] for building a Boundary.
    pub fn add_vertex(&mut self, vertex: VertexIndex<V::VertexRef>) -> usize {
        self.vertices.push(VertexOrPoint::Vertex(vertex));
        self.vertices.len() - 1
    }

    /// Set the Semantic on a point.
    /// A point can only have one semantic value.
    ///
    /// # Parameters
    ///
    /// * `index` - The index of the point that will get the semantic. The index is the
    /// value returned by the [add_point] or [add_vertex] methods. If
    /// `None`, the Semantic is added to the last vertex in the GeometryBuilder.
    /// * `semantic` - The semantic instance to add to the point.
    pub fn set_point_semantic(&mut self, index: Option<usize>, semantic: V::Semantic) -> V::ResourceRef {
        let semantic_ref = self.model.add_semantic(semantic);
        let vertex_i= if let Some(i) = index {
            i
        } else {
            self.vertices.len() - 1
        };
        self.point_semantics.insert(vertex_i, semantic_ref);
        semantic_ref
    }
}
