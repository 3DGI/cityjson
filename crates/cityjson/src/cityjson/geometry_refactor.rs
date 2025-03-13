use crate::cityjson::geometry::boundary::BoundaryCounter;
use crate::errors;
use crate::errors::{Error, Result};
use crate::prelude::{
    Boundary, CityModelTrait, CityModelTypes, Coordinate, GeometryTrait, GeometryType, LoD,
    MaterialMap, SemanticMap, TextureMap, UVCoordinate, VertexIndex, VertexRef,
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
    // UV coordinates storage
    uv_coordinates: Vec<UVCoordinate>,
    // Maps geometry vertex indices to UV coordinate indices
    vertex_uv_mapping: HashMap<usize, usize>,
    rings: Vec<Vec<usize>>,           // indices into vertices
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
    // Maps ring index to texture reference
    ring_textures: HashMap<usize, V::ResourceRef>,
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
            uv_coordinates: Vec::new(),
            vertex_uv_mapping: Default::default(),
            ring_textures: Default::default(),
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
        self.vertices.len().saturating_sub(1)
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
        self.vertices.len().saturating_sub(1)
    }

    /// Add a new UV coordinate and return its index.
    ///
    /// # Returns
    ///
    /// The index of the added UV coordinate.
    pub fn add_uv_coordinate(&mut self, u: f32, v: f32) -> usize {
        self.uv_coordinates.push(UVCoordinate::new(u, v));
        self.uv_coordinates.len().saturating_sub(1)
    }

    /// Map a boundary vertex to a UV coordinate.
    ///
    /// # Parameters
    /// - `vertex_idx`: Index of the target vertex, as returned from [add_point] or
    /// [add_vertex].
    /// - `uv_idx`: Index of the corresponding UV coordinate, as returned by
    /// [add_uv_coordinate].
    pub fn map_vertex_to_uv(&mut self, vertex_idx: usize, uv_idx: usize) {
        self.vertex_uv_mapping.insert(vertex_idx, uv_idx);
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
                vertex_count: vertices.len(),
            });
        }
        self.rings.push(vertices.to_vec());
        Ok(self.rings.len().saturating_sub(1))
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
                vertex_count: vertices.len(),
            });
        }
        self.rings.push(vertices.to_vec());
        Ok(self.rings.len().saturating_sub(1))
    }

    /// Starts a new surface.
    ///
    /// Returns the index of the new surface.
    pub fn start_surface(&mut self) -> usize {
        let idx = self.surfaces.len();
        self.surfaces.push(SurfaceInProgress::default());
        self.active_surface = Some(idx);
        idx
    }

    /// Sets the outer ring for the active surface.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No surface is currently being built (`NoActiveElement`)
    /// - The ring is invalid (`InvalidRing`)
    /// - An outer ring is already set (`InvalidGeometry`)
    pub fn add_surface_outer_ring(&mut self, vertices: &[usize]) -> Result<()> {
        let surface_idx = self.active_surface.ok_or_else(|| Error::NoActiveElement {
            element_type: "surface".to_string(),
        })?;
        if self.surfaces[surface_idx].outer_ring.is_some() {
            return Err(Error::InvalidGeometry(
                "An outer ring is already set on the surface".to_string(),
            ));
        }
        let ring_idx = self.add_ring(vertices)?;
        self.surfaces[surface_idx].outer_ring = Some(ring_idx);
        Ok(())
    }

    /// Adds an inner ring (hole) to the active surface.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No surface is currently being built (`NoActiveElement`)
    /// - The current surface has no outer ring (`MissingOuterElement`)
    /// - The ring is invalid (`InvalidRing`)
    pub fn add_surface_inner_ring(&mut self, vertices: &[usize]) -> errors::Result<()> {
        let surface_idx = self.active_surface.ok_or_else(|| Error::NoActiveElement {
            element_type: "surface".to_string(),
        })?;

        if self.surfaces[surface_idx].outer_ring.is_none() {
            return Err(Error::MissingOuterElement {
                context: "Cannot add inner ring before outer ring is set".to_string(),
            });
        }

        let ring_idx = self.add_ring(vertices)?;
        self.surfaces[surface_idx].inner_rings.push(ring_idx);
        Ok(())
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
    ) -> Result<V::ResourceRef> {
        let semantic_ref = self.model.add_semantic(semantic);
        let vertex_i = if let Some(i) = index {
            if i >= self.vertices.len() {
                return Err(Error::InvalidReference {
                    element_type: "vertex".to_string(),
                    index: i,
                    max_index: self.vertices.len().saturating_sub(1),
                });
            }
            i
        } else {
            self.vertices.len().saturating_sub(1)
        };

        self.point_semantics.insert(vertex_i, semantic_ref);

        Ok(semantic_ref)
    }

    /// Set the Semantic on a LineString.
    /// A LineString can only have one semantic value. The Semantic is directly added to the
    /// `model`.
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
    ) -> Result<V::ResourceRef> {
        let semantic_ref = self.model.add_semantic(semantic);
        let ring_i = if let Some(i) = index {
            if i >= self.rings.len() {
                return Err(Error::InvalidReference {
                    element_type: "ring".to_string(),
                    index: i,
                    max_index: self.rings.len().saturating_sub(1),
                });
            }
            i
        } else {
            self.rings.len().saturating_sub(1)
        };

        self.linestring_semantics.insert(ring_i, semantic_ref);

        Ok(semantic_ref)
    }

    /// Set the Semantic on a surface.
    /// A surface can only have one semantic value. The Semantic is directly added to the
    /// `model`.
    ///
    /// # Parameters
    ///
    /// * `index` - The index of the surface that will get the semantic. The index is the
    /// value returned by the [add_surface] method. If
    /// `None`, the Semantic is added to the last surface in the GeometryBuilder.
    /// * `semantic` - The Semantic instance to add to the surface.
    ///
    /// # Returns
    ///
    /// The reference to the Semantic in the resource pool of the `model`.
    pub fn set_semantic_surface(
        &mut self,
        index: Option<usize>,
        semantic: V::Semantic,
    ) -> Result<V::ResourceRef> {
        let semantic_ref = self.model.add_semantic(semantic);
        let surface_i = if let Some(i) = index {
            if i >= self.surfaces.len() {
                return Err(Error::InvalidReference {
                    element_type: "surface".to_string(),
                    index: i,
                    max_index: self.surfaces.len().saturating_sub(1),
                });
            }
            i
        } else {
            self.surfaces.len().saturating_sub(1)
        };

        self.surface_semantics.insert(surface_i, semantic_ref);

        Ok(semantic_ref)
    }

    /// Set the Material on a surface.
    /// A surface can only have one material value. The Material is directly added to the
    /// `model`.
    ///
    /// # Parameters
    ///
    /// * `index` - The index of the surface that will get the material. The index is the
    /// value returned by the [add_surface] method. If
    /// `None`, the Material is added to the last surface in the GeometryBuilder.
    /// * `material` - The Material instance to add to the surface.
    ///
    /// # Returns
    ///
    /// The reference to the Material in the resource pool of the `model`.
    pub fn set_material_surface(
        &mut self,
        index: Option<usize>,
        material: V::Material,
    ) -> Result<V::ResourceRef> {
        let material_ref = self.model.add_material(material);
        let surface_i = if let Some(i) = index {
            if i >= self.surfaces.len() {
                return Err(Error::InvalidReference {
                    element_type: "surface".to_string(),
                    index: i,
                    max_index: self.surfaces.len().saturating_sub(1),
                });
            }
            i
        } else {
            self.surfaces.len().saturating_sub(1)
        };

        self.surface_materials.insert(surface_i, material_ref);

        Ok(material_ref)
    }

    /// Set the Texture on a surface.
    /// A surface can only have one material value. The Material is directly added to the
    /// `model`.
    ///
    /// # Parameters
    ///
    /// * `index` - The index of the surface that will get the material. The index is the
    /// value returned by the [add_surface] method. If
    /// `None`, the Material is added to the last surface in the GeometryBuilder.
    /// * `material` - The Material instance to add to the surface.
    ///
    /// # Returns
    ///
    /// The reference to the Material in the resource pool of the `model`.
    pub fn set_texture_surface(
        &mut self,
        index: Option<usize>,
        texture: V::Texture,
    ) -> Result<V::ResourceRef> {
        let texture_ref = self.model.add_texture(texture);
        let surface_i = if let Some(i) = index {
            if i >= self.surfaces.len() {
                return Err(Error::InvalidReference {
                    element_type: "surface".to_string(),
                    index: i,
                    max_index: self.surfaces.len().saturating_sub(1),
                });
            }
            i
        } else {
            self.surfaces.len().saturating_sub(1)
        };

        self.surface_textures.insert(surface_i, texture_ref);

        Ok(texture_ref)
    }

    /// Builds the geometry and adds it to the `model`.
    ///
    /// # Errors
    /// * The geometry type does not match the structure (`InvalidGeometryType`)
    /// * The `model`'s vertex container has reached its maximum capacity (`VerticesContainerFull`)
    pub fn build(self) -> Result<V::ResourceRef> {
        // Validate structure before building
        self.validate_structure()?;

        // Pre-allocate the Boundary
        let mut vertices_capacity = 0;
        let mut rings_capacity = 0;
        let mut surfaces_capacity = 0;
        if self.type_geometry == GeometryType::MultiPoint {
            vertices_capacity = self.vertices.len();
        } else if self.type_geometry == GeometryType::MultiLineString {
            vertices_capacity = self.rings.iter().map(|ring| ring.len()).sum();
            rings_capacity = self.rings.len();
        } else if self.type_geometry == GeometryType::MultiSurface
            || self.type_geometry == GeometryType::CompositeSurface
        {
            // For MultiSurface, calculate total vertices from all rings in all surfaces
            rings_capacity = self
                .surfaces
                .iter()
                .map(|s| {
                    let outer = s.outer_ring.map_or(0, |_| 1);
                    outer + s.inner_rings.len()
                })
                .sum();

            vertices_capacity = self.rings.iter().map(|ring| ring.len()).sum();
            surfaces_capacity = self.surfaces.len();
        }
        let mut boundary = Boundary::with_capacity(
            vertices_capacity,
            rings_capacity,
            surfaces_capacity,
            self.shells.len(),
            self.solids.len(),
        );
        let cnt_new_vertices = self
            .vertices
            .iter()
            .filter(|v| matches!(v, VertexOrPoint::Point(_)))
            .count();
        if cnt_new_vertices > 0 {
            self.model.vertices_mut().reserve(cnt_new_vertices)?;
        }

        let mut counter = BoundaryCounter::<V::VertexRef>::default();

        let mut semantic_map_optional = None;
        let mut material_map_optional = None;

        // Each Boundary type has vertices
        let vertex_indices: Vec<VertexIndex<V::VertexRef>> = self
            .vertices
            .into_iter()
            .map(|v| match v {
                VertexOrPoint::Vertex(idx) => Ok(idx),
                VertexOrPoint::Point(p) => self.model.add_vertex(p),
            })
            .collect::<Result<Vec<_>>>()?;

        match self.type_geometry {
            GeometryType::MultiPoint => {
                boundary.vertices = vertex_indices;

                semantic_map_optional = if !self.point_semantics.is_empty() {
                    let mut semantic_map = SemanticMap::<V::VertexRef, V::ResourceRef>::default();
                    semantic_map.points = (0..boundary.vertices.len())
                        .map(|i| self.point_semantics.get(&i).copied())
                        .collect();
                    Some(semantic_map)
                } else {
                    None
                }
            }
            GeometryType::MultiLineString => {
                for ring in &self.rings {
                    boundary.rings.push(counter.vertex_offset());
                    for &vert_idx in ring {
                        boundary.vertices.push(vertex_indices[vert_idx]);
                        counter.increment_vertex_idx();
                    }
                }

                semantic_map_optional = if !self.linestring_semantics.is_empty() {
                    let mut semantic_map = SemanticMap::<V::VertexRef, V::ResourceRef>::default();
                    semantic_map.linestrings = (0..self.rings.len())
                        .map(|i| self.linestring_semantics.get(&i).copied())
                        .collect();
                    Some(semantic_map)
                } else {
                    None
                }
            }
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                for surface in &self.surfaces {
                    if let Some(outer_ring_idx) = surface.outer_ring {
                        boundary.surfaces.push(counter.ring_offset());

                        // Add the outer ring first
                        boundary.rings.push(counter.vertex_offset());
                        for &vertex_idx in &self.rings[outer_ring_idx] {
                            boundary.vertices.push(vertex_indices[vertex_idx]);
                            counter.increment_vertex_idx();
                        }
                        counter.increment_ring_idx();

                        // Add all inner rings for this surface
                        for &inner_ring_idx in &surface.inner_rings {
                            boundary.rings.push(counter.vertex_offset());
                            for &vertex_idx in &self.rings[inner_ring_idx] {
                                boundary.vertices.push(vertex_indices[vertex_idx]);
                                counter.increment_vertex_idx();
                            }
                            counter.increment_ring_idx();
                        }
                    }
                }

                if !self.surface_semantics.is_empty() {
                    let mut semantic_map = SemanticMap::<V::VertexRef, V::ResourceRef>::default();
                    semantic_map.surfaces = (0..self.surfaces.len())
                        .map(|i| self.surface_semantics.get(&i).copied())
                        .collect();
                    semantic_map_optional = Some(semantic_map);
                }

                if !self.surface_materials.is_empty() {
                    let mut material_map = MaterialMap::<V::VertexRef, V::ResourceRef>::default();
                    material_map.surfaces = (0..self.surfaces.len())
                        .map(|i| self.surface_materials.get(&i).copied())
                        .collect();
                    material_map_optional = Some(material_map);
                }
            }
            _ => {
                unimplemented!()
            }
        }

        let texture_map_optional = if self.surface_textures.is_empty()
            && self.ring_textures.is_empty()
            && self.vertex_uv_mapping.is_empty()
        {
            None
        } else {
            Some(build_texture_map::<V, M>(
                &boundary,
                &self.ring_textures,
                &self.surface_textures,
                &self.vertex_uv_mapping,
            ))
        };
        if texture_map_optional.is_some() {
            for uv in self.uv_coordinates {
                self.model.add_uv_coordinate(uv)?;
            }
        }

        // Create the geometry
        let geometry = V::Geometry::new(
            self.type_geometry,
            self.lod,
            Some(boundary),
            semantic_map_optional,
            material_map_optional,
            texture_map_optional,
            None,
            self.transformation_matrix,
        );

        Ok(self.model.add_geometry(geometry))
    }

    fn validate_structure(&self) -> Result<()> {
        match self.type_geometry {
            GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                if self.solids.is_empty()
                    || self.shells.is_empty()
                    || self.surfaces.is_empty()
                    || self.rings.is_empty()
                    || self.vertices.is_empty()
                {
                    return Err(Error::InvalidGeometryType {
                        expected: "multi- or composite solid geometry".to_string(),
                        found: self.format_counts(),
                    });
                }
            }
            GeometryType::Solid => {
                if !self.solids.is_empty()
                    || self.shells.is_empty()
                    || self.surfaces.is_empty()
                    || self.rings.is_empty()
                    || self.vertices.is_empty()
                {
                    return Err(Error::InvalidGeometryType {
                        expected: "single solid geometry".to_string(),
                        found: self.format_counts(),
                    });
                }
            }
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                if !self.solids.is_empty()
                    || !self.shells.is_empty()
                    || self.surfaces.is_empty()
                    || self.rings.is_empty()
                    || self.vertices.is_empty()
                {
                    return Err(Error::InvalidGeometryType {
                        expected: "multi- or composite surface geometry".to_string(),
                        found: self.format_counts(),
                    });
                }
            }
            GeometryType::MultiLineString => {
                if !self.solids.is_empty()
                    || !self.shells.is_empty()
                    || !self.surfaces.is_empty()
                    || self.rings.is_empty()
                    || self.vertices.is_empty()
                {
                    return Err(Error::InvalidGeometryType {
                        expected: "multi linestring geometry".to_string(),
                        found: self.format_counts(),
                    });
                }
                return Ok(());
            }
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
                return Ok(());
            }
            _ => {
                unimplemented!()
            }
        }

        // Verify surfaces
        for (i, surface) in self.surfaces.iter().enumerate() {
            if surface.outer_ring.is_none() {
                return Err(Error::IncompleteGeometry(format!(
                    "Surface {} missing outer ring",
                    i
                )));
            }
        }

        // Verify shells
        for (i, shell) in self.shells.iter().enumerate() {
            if shell.outer_surfaces.is_empty() {
                return Err(Error::IncompleteGeometry(format!(
                    "Shell {} has no outer surfaces",
                    i
                )));
            }
        }

        // Verify solids
        for (i, solid) in self.solids.iter().enumerate() {
            if solid.outer_shell.is_none() {
                return Err(Error::IncompleteGeometry(format!(
                    "Solid {} missing outer shell",
                    i
                )));
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

fn build_texture_map<V: CityModelTypes, M: CityModelTrait<V>>(
    boundary: &Boundary<V::VertexRef>,
    ring_textures: &HashMap<usize, V::ResourceRef>,
    surface_textures: &HashMap<usize, V::ResourceRef>,
    vertex_uv_mapping: &HashMap<usize, usize>,
) -> TextureMap<V::VertexRef, V::ResourceRef> {
    // Pre-allocate the texture map with correct capacity
    let mut texture_map = TextureMap::<V::VertexRef, V::ResourceRef>::with_capacity(
        boundary.vertices.len(),
        boundary.rings.len(),
        ring_textures.len(),
        boundary.surfaces.len(),
        boundary.shells.len(),
        boundary.solids.len(),
    );

    // Initialize the vertices vector with None values
    for _ in 0..boundary.vertices.len() {
        texture_map.add_vertex(None);
    }

    // Use BoundaryCounter to track positions within the boundary
    let mut counter = BoundaryCounter::<V::VertexRef>::default();
    let mut orig_builder_idx_to_boundary_idx = HashMap::new();
    let mut builder_ring_idx = 0;

    // Process each surface
    for s_idx in 0..boundary.surfaces.len() {
        let surface_start = boundary.surfaces[s_idx].to_usize();
        let surface_end = boundary
            .surfaces
            .get(s_idx + 1)
            .map_or(boundary.rings.len(), |idx| idx.to_usize());

        // Get the texture for this surface (if any)
        let surface_texture = surface_textures.get(&s_idx).copied();

        // Process each ring in this surface
        for r_idx in surface_start..surface_end {
            let ring_start = boundary.rings[r_idx].to_usize();
            let ring_end = boundary
                .rings
                .get(r_idx + 1)
                .map_or(boundary.vertices.len(), |idx| idx.to_usize());

            // Get ring-specific texture (if any) or fall back to surface texture
            let texture_ref = ring_textures.get(&builder_ring_idx).copied().or(surface_texture);

            // If we have a texture for this ring, add it to the texture map
            if let Some(texture_ref) = texture_ref {
                texture_map.add_ring(boundary.rings[r_idx]);
                texture_map.add_ring_texture(Some(texture_ref));

                // Map each vertex in this ring from builder index to boundary index
                let current_vertex_offset = counter.vertex_offset();
                for v_offset in 0..(ring_end - ring_start) {
                    let builder_vertex_idx = v_offset;
                    let boundary_vertex_idx = current_vertex_offset.to_usize() + v_offset;
                    orig_builder_idx_to_boundary_idx.insert(builder_vertex_idx, boundary_vertex_idx);
                }
            }

            // Advance vertex counter for this ring
            for _ in ring_start..ring_end {
                counter.increment_vertex_idx();
            }

            builder_ring_idx += 1;
        }
    }

    // Map the UV coordinates to boundary vertices
    for (builder_vertex_idx, uv_idx) in vertex_uv_mapping {
        if let Some(boundary_idx) = orig_builder_idx_to_boundary_idx.get(builder_vertex_idx) {
            // Convert UV index to VertexIndex
            if let Ok(uv_vertex_idx) = VertexIndex::<V::VertexRef>::try_from(*uv_idx) {
                // Assign the UV coordinate to the boundary vertex
                if *boundary_idx < texture_map.vertices().len() {
                    texture_map.vertices_mut()[*boundary_idx] = Some(uv_vertex_idx);
                }
            }
        }
    }

    texture_map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cityjson::geometry::GeometryType;
    use crate::prelude::{
        ImageType, MaterialTrait, QuantizedCoordinate, ResourcePool, SemanticTrait, TextureTrait,
    };
    use crate::resources::pool::ResourceId32;
    use crate::resources::storage::OwnedStringStorage;
    use crate::v1_1::{CityModel, OwnedMaterial, OwnedTexture, Semantic, SemanticType};
    use crate::CityModelType;

    // Test helper to create a new model
    fn create_test_model() -> CityModel<u32, ResourceId32, OwnedStringStorage> {
        CityModel::new(CityModelType::CityJSON)
    }

    #[test]
    fn test_multipoint_with_add_vertex() {
        let mut model = create_test_model();

        // First, add some vertices to the model
        let v0 = model.add_vertex(QuantizedCoordinate::new(1, 2, 3)).unwrap();
        let v1 = model.add_vertex(QuantizedCoordinate::new(4, 5, 6)).unwrap();
        let v2 = model.add_vertex(QuantizedCoordinate::new(7, 8, 9)).unwrap();

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
        let geometry = model
            .geometries()
            .get(geom_ref)
            .expect("Failed to get geometry");

        // Check geometry type and LoD
        assert_eq!(geometry.type_geometry(), &GeometryType::MultiPoint);
        assert_eq!(geometry.lod(), Some(&LoD::LoD1));

        // Get the boundary and convert to nested representation
        let boundary = geometry.boundaries().expect("No boundary found");
        let nested = boundary
            .to_nested_multi_point()
            .expect("Failed to convert to nested");

        // Verify the nested representation (should have 3 points)
        assert_eq!(model.vertex_count(), 3);
        assert_eq!(nested, vec![0, 1, 2]);
    }

    #[test]
    fn test_multipoint_with_mixed_adds() {
        let mut model = create_test_model();

        // First add a vertex to the citymodel
        let v0 = model.add_vertex(QuantizedCoordinate::new(1, 2, 3)).unwrap();
        let v1 = model
            .add_vertex(QuantizedCoordinate::new(10, 11, 12))
            .unwrap();

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
        assert_eq!(nested, vec![0, 1, 2]);

        // Check semantics
        let semantics = geometry.semantics().expect("No semantics found");
        let semantic_points = semantics.points();

        // Verify points have semantics applied correctly
        assert_eq!(semantic_points.len(), 3);

        // Verify the semantic references are the ones we set
        let sem_refs: Vec<ResourceId32> = semantic_points
            .iter()
            .filter_map(|s| s.as_ref())
            .cloned()
            .collect();
        assert!(sem_refs.contains(sem_ref0.as_ref().unwrap()));
        assert!(sem_refs.contains(sem_ref1.as_ref().unwrap()));

        // Verify the semantics themselves
        let semantic0 = model
            .get_semantic(sem_ref0.unwrap())
            .expect("Semantic 0 not found");
        assert_eq!(semantic0.type_semantic(), &SemanticType::TransportationHole);

        let semantic1 = model
            .get_semantic(sem_ref1.unwrap())
            .expect("Semantic 1 not found");
        assert_eq!(
            semantic1.type_semantic(),
            &SemanticType::TransportationMarking
        );
    }

    #[test]
    fn test_multilinestring() {
        let mut model = create_test_model();

        // First add some vertices to the model
        let v0 = model.add_vertex(QuantizedCoordinate::new(0, 0, 0)).unwrap();
        let v1 = model.add_vertex(QuantizedCoordinate::new(1, 0, 0)).unwrap();

        // Create a builder for MultiLineString geometry
        let mut builder = GeometryBuilder::new(&mut model, GeometryType::MultiLineString);

        // Add a mix of existing vertices and new points
        let p0 = builder.add_vertex(v0);
        let p1 = builder.add_vertex(v1);
        let p2 = builder.add_point(QuantizedCoordinate::new(1, 1, 0));
        let p3 = builder.add_point(QuantizedCoordinate::new(0, 1, 0));
        let p4 = builder.add_point(QuantizedCoordinate::new(2, 0, 0));
        let p5 = builder.add_point(QuantizedCoordinate::new(2, 2, 0));

        // Create three linestrings
        // First linestring: square
        builder
            .add_linestring(&[p0, p1, p2, p3, p4])
            .expect("Failed to add linestring");
        // Second linestring: diagonal
        let ls2 = builder
            .add_linestring(&[p0, p2])
            .expect("Failed to add linestring");
        // Third linestring: another line
        builder
            .add_linestring(&[p1, p4, p5])
            .expect("Failed to add linestring");

        // Create semantic for the second linestring
        let sem = Semantic::new(SemanticType::TransportationMarking);

        // Set semantic for the second linestring
        let sem_ref = builder.set_semantic_linestring(Some(ls2), sem);

        // Build the geometry
        let geom_ref = builder.build().expect("Failed to build geometry");

        // Get the geometry from the model
        let geometry = model
            .geometries()
            .get(geom_ref)
            .expect("Failed to get geometry");

        // Check geometry type
        assert_eq!(geometry.type_geometry(), &GeometryType::MultiLineString);

        // Get the boundary and convert to nested representation
        let boundary = geometry.boundaries().expect("No boundary found");
        let nested = boundary
            .to_nested_multi_linestring()
            .expect("Failed to convert to nested");

        // Verify the nested representation
        assert_eq!(model.vertex_count(), 6);
        assert_eq!(nested, vec![vec![0, 1, 2, 3, 4], vec![0, 2], vec![1, 4, 5]]);

        // Check semantics
        let semantics = geometry.semantics().expect("No semantics found");
        let linestring_semantics = semantics.linestrings();

        // Verify linestrings have semantics applied correctly
        assert_eq!(linestring_semantics.len(), 3); // Should have entries for all linestrings

        // Only the second linestring should have a semantic
        assert!(linestring_semantics[0].is_none());
        assert_eq!(linestring_semantics[1], Some(sem_ref.clone().unwrap()));
        assert!(linestring_semantics[2].is_none());

        // Verify the semantic itself
        let semantic = model
            .get_semantic(sem_ref.unwrap())
            .expect("Semantic not found");
        assert_eq!(
            semantic.type_semantic(),
            &SemanticType::TransportationMarking
        );
    }

    #[test]
    fn test_multisurface() {
        let mut model = create_test_model();

        // First add some vertices to the model using QuantizedCoordinate
        let v0 = model.add_vertex(QuantizedCoordinate::new(0, 0, 0)).unwrap();
        let v1 = model
            .add_vertex(QuantizedCoordinate::new(10, 0, 0))
            .unwrap();
        let v2 = model
            .add_vertex(QuantizedCoordinate::new(10, 10, 0))
            .unwrap();
        let v3 = model
            .add_vertex(QuantizedCoordinate::new(0, 10, 0))
            .unwrap();

        // Create a builder for MultiSurface geometry
        let mut builder = GeometryBuilder::new(&mut model, GeometryType::MultiSurface);

        // Add a mix of existing vertices and new points
        let p0 = builder.add_vertex(v0);
        let p1 = builder.add_vertex(v1);
        let p2 = builder.add_vertex(v2);
        let p3 = builder.add_vertex(v3);
        let p4 = builder.add_point(QuantizedCoordinate::new(5, 15, 0));
        let p5 = builder.add_point(QuantizedCoordinate::new(15, 5, 0));
        let p6 = builder.add_point(QuantizedCoordinate::new(20, 0, 0));
        let p7 = builder.add_point(QuantizedCoordinate::new(20, 10, 0));
        let p8 = builder.add_point(QuantizedCoordinate::new(15, 15, 0));

        // Create three surfaces

        // Surface 1: Triangle (no semantic or material)
        let surface0 = builder.start_surface();
        builder
            .add_surface_outer_ring(&[p0, p1, p4])
            .expect("Failed to add outer ring");

        // Surface 2: Square with semantic and texture
        let surface1 = builder.start_surface();
        builder
            .add_surface_outer_ring(&[p1, p2, p5, p6])
            .expect("Failed to add outer ring");
        builder
            .add_surface_inner_ring(&[p0, p1, p2])
            .expect("Failed to add inner ring");

        // Add UV coordinates for each vertex
        let uv0 = builder.add_uv_coordinate(0.0, 0.0);
        let uv1 = builder.add_uv_coordinate(1.0, 0.0);
        let uv2 = builder.add_uv_coordinate(1.0, 1.0);
        let uv3 = builder.add_uv_coordinate(0.0, 1.0);
        // Map vertices to UV coordinates
        builder.map_vertex_to_uv(p1, uv0);
        builder.map_vertex_to_uv(p2, uv1);
        builder.map_vertex_to_uv(p5, uv2);
        builder.map_vertex_to_uv(p6, uv3);
        // Create a texture
        let wall_texture = OwnedTexture::new("facade.jpg".to_string(), ImageType::Jpg);
        // Set the texture for the surface
        let texture_ref = builder.set_texture_surface(Some(surface0), wall_texture);

        // Create and assign semantic for the second surface
        let roof_semantic = Semantic::new(SemanticType::RoofSurface);
        let sem_ref = builder.set_semantic_surface(Some(surface1), roof_semantic);

        // Surface 3: Polygon with material
        let surface2 = builder.start_surface();
        builder
            .add_surface_outer_ring(&[p2, p3, p4, p8, p7])
            .expect("Failed to add outer ring");

        // Create and assign material for the third surface
        let mut wall_material = OwnedMaterial::new("Wall".to_string());
        wall_material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
        wall_material.set_ambient_intensity(Some(0.5));
        let mat_ref = builder.set_material_surface(Some(surface2), wall_material);

        // Build the geometry
        let geom_ref = builder.build().expect("Failed to build geometry");

        // Get the geometry from the model
        let geometry = model
            .geometries()
            .get(geom_ref)
            .expect("Failed to get geometry");

        // Check geometry type
        assert_eq!(geometry.type_geometry(), &GeometryType::MultiSurface);

        // Get the boundary and convert to nested representation
        let boundary = geometry.boundaries().expect("No boundary found");
        let nested = boundary
            .to_nested_multi_or_composite_surface()
            .expect("Failed to convert to nested");

        // Verify the nested representation
        let nested_expected = vec![
            vec![vec![0, 1, 4]],
            vec![vec![1, 2, 5, 6], vec![0, 1, 2]],
            vec![vec![2, 3, 4, 8, 7]],
        ];
        assert_eq!(model.vertex_count(), 9);
        assert_eq!(nested, nested_expected);

        // Check semantics
        let semantics = geometry.semantics().expect("No semantics found");
        let surface_semantics = semantics.surfaces();

        // Verify surface semantics
        assert_eq!(surface_semantics.len(), 3); // Should have entries for all surfaces

        // Only the second surface should have a semantic
        assert!(surface_semantics[0].is_none());
        assert_eq!(surface_semantics[1], Some(sem_ref.clone().unwrap()));
        assert!(surface_semantics[2].is_none());

        // Verify the semantic itself
        let semantic = model
            .get_semantic(sem_ref.unwrap())
            .expect("Semantic not found");
        assert_eq!(semantic.type_semantic(), &SemanticType::RoofSurface);

        // Check materials
        let materials = geometry.materials().expect("No materials found");
        let surface_materials = materials.surfaces();

        // Verify surface materials
        assert_eq!(surface_materials.len(), 3); // Should have entries for all surfaces

        // Only the third surface should have a material
        assert!(surface_materials[0].is_none());
        assert!(surface_materials[1].is_none());
        assert_eq!(surface_materials[2], Some(mat_ref.clone().unwrap()));

        // Verify the material itself
        let material = model
            .get_material(mat_ref.unwrap())
            .expect("Material not found");
        assert_eq!(material.name(), "Wall");
        assert!(material.diffuse_color().is_some());
        assert_eq!(material.ambient_intensity().unwrap(), 0.5);

        // Check textures
        let textures = geometry.textures().expect("No textures found");

        // Verify we have texture mappings
        assert!(textures.vertices().len() > 0, "No texture vertices found");
        assert!(textures.rings().len() > 0, "No texture rings found");
        assert!(textures.ring_textures().len() > 0, "No ring textures found");

        // Verify the texture references
        let texture_refs: Vec<ResourceId32> = textures
            .ring_textures()
            .iter()
            .filter_map(|t| t.as_ref())
            .cloned()
            .collect();

        assert!(
            texture_refs.contains(texture_ref.as_ref().unwrap()),
            "First texture reference not found"
        );

        // Verify the texture objects themselves
        let texture1 = model
            .get_texture(texture_ref.unwrap())
            .expect("Texture 1 not found");
        assert_eq!(texture1.image(), "facade.jpg");
        assert_eq!(texture1.image_type(), &ImageType::Jpg);
    }
}
