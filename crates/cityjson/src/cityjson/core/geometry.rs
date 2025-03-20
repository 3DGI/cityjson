//#![doc = include_str!("../../docs/boundary_guide.md")]

use crate::cityjson::core::boundary::BoundaryCounter;
use crate::cityjson::traits::coordinate::Coordinate;
use crate::cityjson::traits::vertex::VertexRef;
use crate::errors::{Error, Result};
use crate::prelude::{
    Boundary, CityModelTrait, CityModelTypes, GeometryTrait, MaterialMap, RealWorldCoordinate,
    SemanticMap, StringStorage, TextureMap, UVCoordinate, VertexIndex,
};
use std::collections::HashMap;

/// Represents a surface under construction with one outer ring and optional inner rings
#[derive(Default)]
struct SurfaceInProgress {
    outer_ring: Option<usize>, // index to outer ring
    inner_rings: Vec<usize>,   // indices to inner rings
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

enum TemplateVertexOrPoint<V: VertexRef> {
    Vertex(VertexIndex<V>),
    Point(RealWorldCoordinate),
}

/// Controls the [GeometryBuilder] to build a regular geometry or a geometry template.
pub enum BuilderMode {
    /// Build a regular geometry
    Regular,
    /// Build a geometry template
    Template,
}

/// Geometry builder.
///
/// The GeometryBuilder is generic over the CityModel and Coordinate type, thus it can
/// build a CityModel with either real-world coordinates or quantized coordinates,
/// for all supported CityJSON versions.
pub struct GeometryBuilder<'a, V: CityModelTypes, M: CityModelTrait<V>, SS: StringStorage> {
    model: &'a mut M,
    type_geometry: GeometryType,
    builder_mode: BuilderMode,
    lod: Option<LoD>,
    template_geometry: Option<V::ResourceRef>,
    transformation_matrix: Option<[f64; 16]>,
    template_vertices: Vec<TemplateVertexOrPoint<V::VertexRef>>,
    vertices: Vec<VertexOrPoint<V::VertexRef, V::CoordinateType>>,
    // UV coordinates storage
    uv_coordinates: Vec<UVCoordinate>,
    // Maps geometry vertex indices to UV coordinate indices
    vertex_uv_mapping: HashMap<usize, usize>,
    rings: Vec<Vec<usize>>,           // indices into vertices
    surfaces: Vec<SurfaceInProgress>, // surfaces with their rings
    shells: Vec<Vec<usize>>,          // A solid with its shells, each shell with their surfaces
    solids: Vec<SolidInProgress>,     // M/CSolid with its shells
    // Active element tracking
    active_surface: Option<usize>, // active surface being built
    active_solid: Option<usize>,   // active solid being built
    // Semantic storage
    point_semantics: HashMap<usize, V::ResourceRef>,
    linestring_semantics: HashMap<usize, V::ResourceRef>,
    surface_semantics: HashMap<usize, V::ResourceRef>,
    // Material storage with themes as [(theme, [(surface idx, material ref)])]
    surface_materials: Vec<(SS::String, Vec<(usize, V::ResourceRef)>)>,
    // Maps ring index to texture reference
    ring_textures: Vec<(SS::String, Vec<(usize, V::ResourceRef)>)>,
}

impl<'a, V: CityModelTypes<StringStorage = SS>, M: CityModelTrait<V>, SS: StringStorage>
    GeometryBuilder<'a, V, M, SS>
{
    /// Instantiates a new GeometryBuilder.
    ///
    /// # Parameters
    /// * `model` - A CityModel instance.
    /// * `type_geometry` - The geometry type to build.
    pub fn new(model: &'a mut M, type_geometry: GeometryType, builder_mode: BuilderMode) -> Self {
        Self {
            model,
            type_geometry,
            builder_mode,
            lod: None,
            template_geometry: None,
            transformation_matrix: None,
            template_vertices: Vec::new(),
            vertices: Vec::new(),
            uv_coordinates: Vec::new(),
            vertex_uv_mapping: Default::default(),
            rings: Vec::new(),
            surfaces: Vec::new(),
            shells: Vec::new(),
            solids: Vec::new(),
            active_surface: None,
            active_solid: None,
            point_semantics: Default::default(),
            linestring_semantics: Default::default(),
            surface_semantics: Default::default(),
            surface_materials: Default::default(),
            ring_textures: Default::default(),
        }
    }

    /// Set the Level of Detail on the Geometry.
    pub fn with_lod(mut self, lod: LoD) -> Self {
        self.lod = Some(lod);
        self
    }

    /// Specifies the template geometry to reference (for a `GeometryInstance` only).
    ///
    /// # Parameters
    ///
    /// * `template_ref` - Reference to a geometry in the model
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Errors
    /// * [Error::InvalidGeometryType] if geometry is not a `GeometryInstance`.
    pub fn with_template(mut self, template_ref: V::ResourceRef) -> Result<Self> {
        if self.type_geometry != GeometryType::GeometryInstance {
            return Err(Error::InvalidGeometryType {
                expected: "GeometryInstance".to_string(),
                found: self.type_geometry.to_string(),
            });
        }
        self.template_geometry = Some(template_ref);
        Ok(self)
    }

    /// Set the Transformation Matrix on the Geometry (for `GeometryInstance` only).
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Errors
    /// * [Error::InvalidGeometryType] if geometry is not a `GeometryInstance`.
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

    pub fn with_reference_point(mut self, point: V::CoordinateType) -> Self {
        self.add_point(point);
        self
    }

    pub fn with_reference_vertex(mut self, vertex: VertexIndex<V::VertexRef>) -> Self {
        self.add_vertex(vertex);
        self
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

    pub fn add_template_point(&mut self, point: RealWorldCoordinate) -> usize {
        self.template_vertices
            .push(TemplateVertexOrPoint::Point(point));
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

    pub fn add_template_vertex(&mut self, vertex: VertexIndex<V::VertexRef>) -> usize {
        self.template_vertices
            .push(TemplateVertexOrPoint::Vertex(vertex));
        self.template_vertices.len().saturating_sub(1)
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
        // if vertices.len() < 2 {
        //     return Err(Error::InvalidLineString {
        //         reason: "LineString must have at least 2 vertices".to_string(),
        //         vertex_count: vertices.len(),
        //     });
        // }
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
        // if vertices.len() < 3 {
        //     return Err(Error::InvalidRing {
        //         reason: "ring must have at least 3 vertices".to_string(),
        //         vertex_count: vertices.len(),
        //     });
        // }
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
    pub fn add_surface_outer_ring(&mut self, ring_idx: usize) -> Result<()> {
        let surface_idx = self.active_surface.ok_or_else(|| Error::NoActiveElement {
            element_type: "surface".to_string(),
        })?;
        if self.surfaces[surface_idx].outer_ring.is_some() {
            return Err(Error::InvalidGeometry(
                "An outer ring is already set on the surface".to_string(),
            ));
        }
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
    pub fn add_surface_inner_ring(&mut self, ring_idx: usize) -> Result<()> {
        let surface_idx = self.active_surface.ok_or_else(|| Error::NoActiveElement {
            element_type: "surface".to_string(),
        })?;

        if self.surfaces[surface_idx].outer_ring.is_none() {
            return Err(Error::MissingOuterElement {
                context: "Cannot add inner ring before outer ring is set".to_string(),
            });
        }
        self.surfaces[surface_idx].inner_rings.push(ring_idx);
        Ok(())
    }

    /// Adds a shell to the boundary.
    ///
    /// # Errors
    ///
    /// - `InvalidShell`: If less than 4 surfaces are provided.
    pub fn add_shell(&mut self, surfaces: &[usize]) -> Result<usize> {
        // if surfaces.len() < 4 {
        //     return Err(Error::InvalidShell {
        //         reason: "shell must have at least 4 surfaces".to_string(),
        //         surface_count: surfaces.len(),
        //     });
        // }
        self.shells.push(surfaces.to_vec());
        Ok(self.shells.len().saturating_sub(1))
    }

    /// Starts a new solid.
    ///
    /// Returns the index of the new solid.
    pub fn start_solid(&mut self) -> usize {
        let idx = self.solids.len();
        self.solids.push(SolidInProgress::default());
        self.active_solid = Some(idx);
        idx
    }

    /// Sets the outer shell for the current solid.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No solid is currently being built (`NoActiveElement`)
    /// - The shell index is invalid (`InvalidReference`)
    /// - An outer shell is already set (`InvalidGeometry`)
    pub fn add_solid_outer_shell(&mut self, shell_idx: usize) -> Result<()> {
        let solid_idx = self.active_solid.ok_or_else(|| Error::NoActiveElement {
            element_type: "solid".to_string(),
        })?;

        if shell_idx >= self.shells.len() {
            return Err(Error::InvalidReference {
                element_type: "shell".to_string(),
                index: shell_idx,
                max_index: self.shells.len().saturating_sub(1),
            });
        }

        self.solids[solid_idx].outer_shell = Some(shell_idx);
        Ok(())
    }

    /// Adds an inner shell to the active solid.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No solid is currently being built (`NoActiveElement`)
    /// - The shell index is invalid (`InvalidReference`)
    /// - The solid has no outer shell (`MissingOuterElement`)
    pub fn add_solid_inner_shell(&mut self, shell_idx: usize) -> Result<()> {
        let solid_idx = self.active_solid.ok_or_else(|| Error::NoActiveElement {
            element_type: "solid".to_string(),
        })?;

        if shell_idx >= self.shells.len() {
            return Err(Error::InvalidReference {
                element_type: "shell".to_string(),
                index: shell_idx,
                max_index: self.shells.len().saturating_sub(1),
            });
        }

        if self.solids[solid_idx].outer_shell.is_none() {
            return Err(Error::MissingOuterElement {
                context: "Cannot add inner shell before outer shell is set".to_string(),
            });
        }

        self.solids[solid_idx].inner_shells.push(shell_idx);
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
    /// The Material is directly added to the `model`.
    ///
    /// # Parameters
    ///
    /// * `index` - The index of the surface that will get the material. The index is the
    /// value returned by the [add_surface] method. If
    /// `None`, the Material is added to the last surface in the GeometryBuilder.
    /// * `material` - The Material instance to add to the surface.
    /// * `theme` - The theme of the material.
    ///
    /// # Returns
    ///
    /// The reference to the Material in the resource pool of the `model`.
    pub fn set_material_surface(
        &mut self,
        index: Option<usize>,
        material: V::Material,
        theme: SS::String,
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

        // Find or create the theme entry
        if let Some(pos) = self.surface_materials.iter().position(|(t, _)| t == &theme) {
            // Theme exists, find or update surface
            let surface_maps = &mut self.surface_materials[pos].1;
            if let Some(pos) = surface_maps.iter().position(|(s, _)| *s == surface_i) {
                // Update existing surface
                surface_maps[pos].1 = material_ref;
            } else {
                // Add new surface
                surface_maps.push((surface_i, material_ref));
            }
        } else {
            // Create new theme with this surface
            self.surface_materials
                .push((theme, vec![(surface_i, material_ref)]));
        }

        Ok(material_ref)
    }

    /// Set the Texture on a ring.
    /// The Texture is directly added to the `model`.
    ///
    /// # Parameters
    ///
    /// * `index` - The index of the ring that will get the texture. The index is the
    /// value returned by the [add_ring] method. If
    /// `None`, the Texture is added to the last ring in the GeometryBuilder.
    /// * `texture` - The Texture instance to add to the ring.
    /// * `theme` - The theme of the texture.
    ///
    /// # Returns
    ///
    /// The reference to the Texture in the resource pool of the `model`.
    pub fn set_texture_ring(
        &mut self,
        index: Option<usize>,
        texture: V::Texture,
        theme: SS::String,
    ) -> Result<V::ResourceRef> {
        let texture_ref = self.model.add_texture(texture);
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

        if let Some(pos) = self.ring_textures.iter().position(|(t, _)| t == &theme) {
            let ring_maps = &mut self.ring_textures[pos].1;
            if let Some(pos) = ring_maps.iter().position(|(r, _)| *r == ring_i) {
                ring_maps[pos].1 = texture_ref;
            } else {
                ring_maps.push((ring_i, texture_ref));
            }
        } else {
            self.ring_textures
                .push((theme, vec![(ring_i, texture_ref)]));
        }

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
        } else if self.type_geometry == GeometryType::GeometryInstance {
            vertices_capacity = 1;
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
        let cnt_new_template_vertices = self
            .template_vertices
            .iter()
            .filter(|v| matches!(v, TemplateVertexOrPoint::Point(_)))
            .count();
        if cnt_new_template_vertices > 0 {
            self.model
                .template_vertices_mut()
                .reserve(cnt_new_template_vertices)?;
        }

        let mut counter = BoundaryCounter::<V::VertexRef>::default();

        let mut semantic_map_option = None;
        let mut material_map_option = None;
        let mut instance_reference_point = None;

        let nr_builder_vertices = self.vertices.len();
        // Each Boundary type has vertices
        let vertex_indices: Vec<VertexIndex<V::VertexRef>> = match self.builder_mode {
            BuilderMode::Regular => self
                .vertices
                .into_iter()
                .map(|v| match v {
                    VertexOrPoint::Vertex(idx) => Ok(idx),
                    VertexOrPoint::Point(p) => self.model.add_vertex(p),
                })
                .collect::<Result<Vec<_>>>()?,
            BuilderMode::Template => self
                .template_vertices
                .into_iter()
                .map(|v| match v {
                    TemplateVertexOrPoint::Vertex(idx) => Ok(idx),
                    TemplateVertexOrPoint::Point(p) => self.model.add_template_vertex(p),
                })
                .collect::<Result<Vec<_>>>()?,
        };

        match self.type_geometry {
            GeometryType::GeometryInstance => {
                instance_reference_point = Some(vertex_indices[0]);
            }
            GeometryType::MultiPoint => {
                boundary.vertices = vertex_indices;

                semantic_map_option = build_semantic_map::<V, M>(
                    &self.type_geometry,
                    &self.point_semantics,
                    nr_builder_vertices,
                );
            }
            GeometryType::MultiLineString => {
                for ring in &self.rings {
                    boundary.rings.push(counter.vertex_offset());
                    for &vert_idx in ring {
                        boundary.vertices.push(vertex_indices[vert_idx]);
                        counter.increment_vertex_idx();
                    }
                }

                semantic_map_option = build_semantic_map::<V, M>(
                    &self.type_geometry,
                    &self.linestring_semantics,
                    self.rings.len(),
                );
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

                semantic_map_option = build_semantic_map::<V, M>(
                    &self.type_geometry,
                    &self.surface_semantics,
                    self.surfaces.len(),
                );

                material_map_option =
                    build_material_map::<V, M, SS>(&self.surface_materials, &self.surfaces);
            }
            GeometryType::Solid => {
                // Add shell index
                boundary.shells.push(counter.surface_offset());

                // Process surfaces for this shell
                for &surface_idx in &self.shells[0] {
                    if surface_idx < self.surfaces.len() {
                        boundary.surfaces.push(counter.ring_offset());

                        // Add outer ring for this surface
                        if let Some(outer_ring_idx) = self.surfaces[surface_idx].outer_ring {
                            boundary.rings.push(counter.vertex_offset());
                            for &vertex_idx in &self.rings[outer_ring_idx] {
                                boundary.vertices.push(vertex_indices[vertex_idx]);
                                counter.increment_vertex_idx();
                            }
                            counter.increment_ring_idx();

                            // Add inner rings if any
                            for &inner_ring_idx in &self.surfaces[surface_idx].inner_rings {
                                boundary.rings.push(counter.vertex_offset());
                                for &vertex_idx in &self.rings[inner_ring_idx] {
                                    boundary.vertices.push(vertex_indices[vertex_idx]);
                                    counter.increment_vertex_idx();
                                }
                                counter.increment_ring_idx();
                            }
                        }
                        counter.increment_surface_idx();
                    }
                }

                semantic_map_option = build_semantic_map::<V, M>(
                    &self.type_geometry,
                    &self.surface_semantics,
                    self.surfaces.len(),
                );

                material_map_option =
                    build_material_map::<V, M, SS>(&self.surface_materials, &self.surfaces);
            }
            GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                // Process each solid
                for solid in &self.solids {
                    if let Some(outer_shell_idx) = solid.outer_shell {
                        // Add this solid to the boundary
                        boundary.solids.push(counter.shell_offset());

                        // Process the outer shell first
                        if outer_shell_idx < self.shells.len() {
                            boundary.shells.push(counter.surface_offset());

                            // Process surfaces for the outer shell
                            for &surface_idx in &self.shells[outer_shell_idx] {
                                if surface_idx < self.surfaces.len() {
                                    boundary.surfaces.push(counter.ring_offset());

                                    // Add outer ring for this surface
                                    if let Some(outer_ring_idx) =
                                        self.surfaces[surface_idx].outer_ring
                                    {
                                        boundary.rings.push(counter.vertex_offset());
                                        for &vertex_idx in &self.rings[outer_ring_idx] {
                                            boundary.vertices.push(vertex_indices[vertex_idx]);
                                            counter.increment_vertex_idx();
                                        }
                                        counter.increment_ring_idx();

                                        // Add inner rings if any
                                        for &inner_ring_idx in
                                            &self.surfaces[surface_idx].inner_rings
                                        {
                                            boundary.rings.push(counter.vertex_offset());
                                            for &vertex_idx in &self.rings[inner_ring_idx] {
                                                boundary.vertices.push(vertex_indices[vertex_idx]);
                                                counter.increment_vertex_idx();
                                            }
                                            counter.increment_ring_idx();
                                        }
                                    }
                                    counter.increment_surface_idx();
                                }
                            }
                            counter.increment_shell_idx();
                        }

                        // Now process any inner shells (voids) for this solid
                        for &inner_shell_idx in &solid.inner_shells {
                            if inner_shell_idx < self.shells.len() {
                                boundary.shells.push(counter.surface_offset());

                                // Process surfaces for this inner shell
                                for &surface_idx in &self.shells[inner_shell_idx] {
                                    if surface_idx < self.surfaces.len() {
                                        boundary.surfaces.push(counter.ring_offset());

                                        // Add outer ring for this surface
                                        if let Some(outer_ring_idx) =
                                            self.surfaces[surface_idx].outer_ring
                                        {
                                            boundary.rings.push(counter.vertex_offset());
                                            for &vertex_idx in &self.rings[outer_ring_idx] {
                                                boundary.vertices.push(vertex_indices[vertex_idx]);
                                                counter.increment_vertex_idx();
                                            }
                                            counter.increment_ring_idx();

                                            // Add inner rings if any
                                            for &inner_ring_idx in
                                                &self.surfaces[surface_idx].inner_rings
                                            {
                                                boundary.rings.push(counter.vertex_offset());
                                                for &vertex_idx in &self.rings[inner_ring_idx] {
                                                    boundary
                                                        .vertices
                                                        .push(vertex_indices[vertex_idx]);
                                                    counter.increment_vertex_idx();
                                                }
                                                counter.increment_ring_idx();
                                            }
                                        }
                                        counter.increment_surface_idx();
                                    }
                                }
                                counter.increment_shell_idx();
                            }
                        }
                        counter.increment_solid_idx();
                    }
                }

                semantic_map_option = build_semantic_map::<V, M>(
                    &self.type_geometry,
                    &self.surface_semantics,
                    self.surfaces.len(),
                );

                material_map_option =
                    build_material_map::<V, M, SS>(&self.surface_materials, &self.surfaces);
            }
        }

        let texture_map_option =
            if self.ring_textures.is_empty() && self.vertex_uv_mapping.is_empty() {
                None
            } else {
                Some(build_texture_map::<V, M, SS>(
                    &boundary,
                    &self.ring_textures,
                    &self.vertex_uv_mapping,
                ))
            };
        if texture_map_option.is_some() {
            for uv in self.uv_coordinates {
                self.model.add_uv_coordinate(uv)?;
            }
        }

        let boundary_option = if self.type_geometry == GeometryType::GeometryInstance {
            None
        } else {
            Some(boundary)
        };
        // Create the geometry
        let geometry = V::Geometry::new(
            self.type_geometry,
            self.lod,
            boundary_option,
            semantic_map_option,
            material_map_option,
            texture_map_option,
            self.template_geometry,
            instance_reference_point,
            self.transformation_matrix,
        );

        match self.builder_mode {
            BuilderMode::Regular => Ok(self.model.add_geometry(geometry)),
            BuilderMode::Template => Ok(self.model.add_template_geometry(geometry)),
        }
    }

    fn validate_structure(&self) -> Result<()> {
        match self.type_geometry {
            GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                let mut template_str = "";
                let vertices_empty = match self.builder_mode {
                    BuilderMode::Regular => self.vertices.is_empty(),
                    BuilderMode::Template => {
                        template_str = "template";
                        self.template_vertices.is_empty()
                    }
                };
                if self.solids.is_empty()
                    || self.shells.is_empty()
                    || self.surfaces.is_empty()
                    || self.rings.is_empty()
                    || vertices_empty
                {
                    return Err(Error::InvalidGeometryType {
                        expected: format!("multi- or composite solid geometry {}", template_str),
                        found: self.format_counts(),
                    });
                }
            }
            GeometryType::Solid => {
                let mut template_str = "";
                let vertices_empty = match self.builder_mode {
                    BuilderMode::Regular => self.vertices.is_empty(),
                    BuilderMode::Template => {
                        template_str = "template";
                        self.template_vertices.is_empty()
                    }
                };
                if !self.solids.is_empty()
                    || self.shells.is_empty()
                    || self.surfaces.is_empty()
                    || self.rings.is_empty()
                    || vertices_empty
                {
                    return Err(Error::InvalidGeometryType {
                        expected: format!("single solid geometry {}", template_str),
                        found: self.format_counts(),
                    });
                }
            }
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                let mut template_str = "";
                let vertices_empty = match self.builder_mode {
                    BuilderMode::Regular => self.vertices.is_empty(),
                    BuilderMode::Template => {
                        template_str = "template";
                        self.template_vertices.is_empty()
                    }
                };
                if !self.solids.is_empty()
                    || !self.shells.is_empty()
                    || self.surfaces.is_empty()
                    || self.rings.is_empty()
                    || vertices_empty
                {
                    return Err(Error::InvalidGeometryType {
                        expected: format!("multi- or composite surface geometry {}", template_str),
                        found: self.format_counts(),
                    });
                }
            }
            GeometryType::MultiLineString => {
                let mut template_str = "";
                let vertices_empty = match self.builder_mode {
                    BuilderMode::Regular => self.vertices.is_empty(),
                    BuilderMode::Template => {
                        template_str = "template";
                        self.template_vertices.is_empty()
                    }
                };
                if !self.solids.is_empty()
                    || !self.shells.is_empty()
                    || !self.surfaces.is_empty()
                    || self.rings.is_empty()
                    || vertices_empty
                {
                    return Err(Error::InvalidGeometryType {
                        expected: format!("multi linestring geometry {}", template_str),
                        found: self.format_counts(),
                    });
                }
                return Ok(());
            }
            GeometryType::MultiPoint => {
                let mut template_str = "";
                let vertices_empty = match self.builder_mode {
                    BuilderMode::Regular => self.vertices.is_empty(),
                    BuilderMode::Template => {
                        template_str = "template";
                        self.template_vertices.is_empty()
                    }
                };
                if !self.solids.is_empty()
                    || !self.shells.is_empty()
                    || !self.surfaces.is_empty()
                    || !self.rings.is_empty()
                    || vertices_empty
                {
                    return Err(Error::InvalidGeometryType {
                        expected: format!("multi point geometry {}", template_str),
                        found: self.format_counts(),
                    });
                }
                return Ok(());
            }
            GeometryType::GeometryInstance => {
                if self.template_geometry.is_none() {
                    return Err(Error::IncompleteGeometry(
                        "GeometryInstance requires a geometry template".to_string(),
                    ));
                }
                if self.transformation_matrix.is_none() {
                    return Err(Error::IncompleteGeometry(
                        "GeometryInstance requires a transformation matrix".to_string(),
                    ));
                }
                if !self.solids.is_empty()
                    || !self.shells.is_empty()
                    || !self.surfaces.is_empty()
                    || !self.rings.is_empty()
                    || self.vertices.len() != 1
                {
                    return Err(Error::IncompleteGeometry (
                        "GeometryInstance must have a boundary with only a single vertex, which is the reference point for the template transformations".to_string()
                    ));
                }
                return Ok(());
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
            "{} solids, {} shells, {} surfaces, {} rings, {} vertices, {} template vertices",
            self.solids.len(),
            self.shells.len(),
            self.surfaces.len(),
            self.rings.len(),
            self.vertices.len(),
            self.template_vertices.len()
        )
    }
}

fn build_semantic_map<V: CityModelTypes, M: CityModelTrait<V>>(
    type_geometry: &GeometryType,
    builder_semantics: &HashMap<usize, V::ResourceRef>,
    nr_primitives: usize,
) -> Option<SemanticMap<V::VertexRef, V::ResourceRef>> {
    match type_geometry {
        GeometryType::GeometryInstance => None,
        GeometryType::MultiPoint => {
            if !builder_semantics.is_empty() {
                let mut semantic_map = SemanticMap::<V::VertexRef, V::ResourceRef>::default();
                semantic_map.points = (0..nr_primitives)
                    .map(|i| builder_semantics.get(&i).copied())
                    .collect();
                Some(semantic_map)
            } else {
                None
            }
        }
        GeometryType::MultiLineString => {
            if !builder_semantics.is_empty() {
                let mut semantic_map = SemanticMap::<V::VertexRef, V::ResourceRef>::default();
                semantic_map.linestrings = (0..nr_primitives)
                    .map(|i| builder_semantics.get(&i).copied())
                    .collect();
                Some(semantic_map)
            } else {
                None
            }
        }
        _ => {
            // Handle semantics, materials and textures for surfaces
            if !builder_semantics.is_empty() {
                let mut semantic_map = SemanticMap::<V::VertexRef, V::ResourceRef>::default();
                semantic_map.surfaces = (0..nr_primitives)
                    .map(|i| builder_semantics.get(&i).copied())
                    .collect();
                Some(semantic_map)
            } else {
                None
            }
        }
    }
}

fn build_material_map<V: CityModelTypes, M: CityModelTrait<V>, SS: StringStorage>(
    surface_materials: &Vec<(SS::String, Vec<(usize, V::ResourceRef)>)>,
    surfaces: &Vec<SurfaceInProgress>,
) -> Option<Vec<(SS::String, MaterialMap<V::VertexRef, V::ResourceRef>)>> {
    if !surface_materials.is_empty() {
        // Create a vector to hold all theme/materialmap pairs
        let mut themed_materials = Vec::with_capacity(surface_materials.len());

        // For each theme, create a MaterialMap
        for (theme_name, surface_mappings) in surface_materials {
            let mut material_map = MaterialMap::<V::VertexRef, V::ResourceRef>::default();

            // We need to ensure the materials vector has entries for all surfaces
            // by creating an array of the right size with all None values
            material_map.surfaces = vec![None; surfaces.len()];

            // Now fill in the materials that are defined for this theme
            for (surface_idx, material_ref) in surface_mappings {
                if *surface_idx < surfaces.len() {
                    material_map.surfaces[*surface_idx] = Some(*material_ref);
                }
            }

            // Add this theme and its material map to our collection
            themed_materials.push((theme_name.clone(), material_map));
        }

        if !themed_materials.is_empty() {
            Some(themed_materials)
        } else {
            None
        }
    } else {
        None
    }
}

fn build_texture_map<
    V: CityModelTypes<StringStorage = SS>,
    M: CityModelTrait<V>,
    SS: StringStorage,
>(
    boundary: &Boundary<V::VertexRef>,
    ring_textures: &[(SS::String, Vec<(usize, V::ResourceRef)>)],
    vertex_uv_mapping: &HashMap<usize, usize>,
) -> Vec<(SS::String, TextureMap<V::VertexRef, V::ResourceRef>)> {
    let mut themed_texture_maps = Vec::new();

    for (theme_name, ring_mappings) in ring_textures {
        let mut texture_map = TextureMap::<V::VertexRef, V::ResourceRef>::default();

        // Initialize vertices with None values
        for _ in 0..boundary.vertices.len() {
            texture_map.add_vertex(None);
        }

        // Process each ring mapping for this theme
        for (ring_idx, texture_ref) in ring_mappings {
            // Check if the ring index is valid
            if *ring_idx < boundary.rings.len() {
                // Add the ring to the texture map
                texture_map.add_ring(boundary.rings[*ring_idx]);
                // Assign the texture to this ring
                texture_map.add_ring_texture(Some(*texture_ref));
            }
        }

        // Map UV coordinates to vertices
        for (vertex_idx, uv_idx) in vertex_uv_mapping {
            if *vertex_idx < texture_map.vertices_mut().len() {
                if let Ok(uv_vertex_idx) = VertexIndex::<V::VertexRef>::try_from(*uv_idx) {
                    texture_map.vertices_mut()[*vertex_idx] = Some(uv_vertex_idx);
                }
            }
        }

        // Only add the texture map if it has at least one ring with texture
        if texture_map.rings().len() > 0 {
            themed_texture_maps.push((theme_name.clone(), texture_map));
        }
    }

    themed_texture_maps
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

impl std::fmt::Display for GeometryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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

impl std::fmt::Display for LoD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            LoD::LoD0 => write!(f, "0"),
            LoD::LoD0_0 => write!(f, "0.0"),
            LoD::LoD0_1 => write!(f, "0.1"),
            LoD::LoD0_2 => write!(f, "0.2"),
            LoD::LoD0_3 => write!(f, "0.3"),
            LoD::LoD1 => write!(f, "1"),
            LoD::LoD1_0 => write!(f, "1.0"),
            LoD::LoD1_1 => write!(f, "1.1"),
            LoD::LoD1_2 => write!(f, "1.2"),
            LoD::LoD1_3 => write!(f, "1.3"),
            LoD::LoD2 => write!(f, "2"),
            LoD::LoD2_0 => write!(f, "2.0"),
            LoD::LoD2_1 => write!(f, "2.1"),
            LoD::LoD2_2 => write!(f, "2.2"),
            LoD::LoD2_3 => write!(f, "2.3"),
            LoD::LoD3 => write!(f, "3"),
            LoD::LoD3_0 => write!(f, "3.0"),
            LoD::LoD3_1 => write!(f, "3.1"),
            LoD::LoD3_2 => write!(f, "3.2"),
            LoD::LoD3_3 => write!(f, "3.3"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cityjson::core::geometry::GeometryType;
    use crate::prelude::{
        BoundaryType, ImageType, MaterialTrait, QuantizedCoordinate, ResourcePool, SemanticTrait,
        TextureTrait,
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
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiPoint, BuilderMode::Regular);

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
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiPoint, BuilderMode::Regular);

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
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiPoint, BuilderMode::Regular);

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
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiPoint, BuilderMode::Regular);

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
        let mut builder = GeometryBuilder::new(
            &mut model,
            GeometryType::MultiLineString,
            BuilderMode::Regular,
        );

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
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiSurface, BuilderMode::Regular);

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
        let ring0 = builder.add_ring(&[p0, p1, p4]).expect("Failed to add ring");
        let surface0 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring0)
            .expect("Failed to add outer ring");

        // Surface 2: Square with semantic and texture
        let ring1 = builder
            .add_ring(&[p1, p2, p5, p6])
            .expect("Failed to add ring");
        let surface1 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring1)
            .expect("Failed to add outer ring");
        let ring2 = builder.add_ring(&[p0, p1, p2]).expect("Failed to add ring");
        builder
            .add_surface_inner_ring(ring2)
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
        let texture_ref =
            builder.set_texture_ring(Some(surface0), wall_texture, "theme-texture".to_string());

        // Create and assign semantic for the second surface
        let roof_semantic = Semantic::new(SemanticType::RoofSurface);
        let sem_ref = builder.set_semantic_surface(Some(surface1), roof_semantic);

        // Surface 3: Polygon with material
        let ring2 = builder
            .add_ring(&[p2, p3, p4, p8, p7])
            .expect("Failed to add ring");
        let surface2 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring2)
            .expect("Failed to add outer ring");

        // Create and assign material for the third surface
        let mut wall_material = OwnedMaterial::new("Wall".to_string());
        wall_material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
        wall_material.set_ambient_intensity(Some(0.5));
        let mat_ref = builder.set_material_surface(
            Some(surface2),
            wall_material,
            "material-theme".to_string(),
        );

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
        let (_theme_material, material_map) = materials.first().unwrap();
        let surface_materials = material_map.surfaces();

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
        let (_theme_texture, texture_map) = textures.first().unwrap();

        // Verify we have texture mappings
        assert!(
            texture_map.vertices().len() > 0,
            "No texture vertices found"
        );
        assert!(texture_map.rings().len() > 0, "No texture rings found");
        assert!(
            texture_map.ring_textures().len() > 0,
            "No ring textures found"
        );

        // Verify the texture references
        let texture_refs: Vec<ResourceId32> = texture_map
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

    #[test]
    fn test_solid() {
        let mut model = create_test_model();

        // Create a builder for Solid geometry
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::Solid, BuilderMode::Regular);

        // Add vertices for a simple cube
        let p0 = builder.add_point(QuantizedCoordinate::new(0, 0, 0)); // bottom-front-left
        let p1 = builder.add_point(QuantizedCoordinate::new(10, 0, 0)); // bottom-front-right
        let p2 = builder.add_point(QuantizedCoordinate::new(10, 10, 0)); // bottom-back-right
        let p3 = builder.add_point(QuantizedCoordinate::new(0, 10, 0)); // bottom-back-left
        let p4 = builder.add_point(QuantizedCoordinate::new(0, 0, 10)); // top-front-left
        let p5 = builder.add_point(QuantizedCoordinate::new(10, 0, 10)); // top-front-right
        let p6 = builder.add_point(QuantizedCoordinate::new(10, 10, 10)); // top-back-right
        let p7 = builder.add_point(QuantizedCoordinate::new(0, 10, 10)); // top-back-left

        // Define each surface (face) of the cube
        // Front face
        let ring0 = builder
            .add_ring(&[p0, p1, p5, p4, p0])
            .expect("Failed to create ring");
        let surface0 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring0)
            .expect("Failed to add front face");

        // Right face
        let ring1 = builder
            .add_ring(&[p1, p2, p6, p5, p1])
            .expect("Failed to create ring");
        let surface1 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring1)
            .expect("Failed to add right face");

        // Back face
        let ring2 = builder
            .add_ring(&[p2, p3, p7, p6, p2])
            .expect("Failed to create ring");
        let surface2 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring2)
            .expect("Failed to add back face");

        // Left face
        let ring3 = builder
            .add_ring(&[p3, p0, p4, p7, p3])
            .expect("Failed to create ring");
        let surface3 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring3)
            .expect("Failed to add left face");

        // Top face
        let ring4 = builder
            .add_ring(&[p4, p5, p6, p7, p4])
            .expect("Failed to create ring");
        let surface4 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring4)
            .expect("Failed to add top face");

        // Bottom face
        let ring5 = builder
            .add_ring(&[p0, p3, p2, p1, p0])
            .expect("Failed to create ring");
        let surface5 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring5)
            .expect("Failed to add bottom face");

        // Add semantics to faces
        let wall_semantic = Semantic::new(SemanticType::WallSurface);
        let roof_semantic = Semantic::new(SemanticType::RoofSurface);
        let floor_semantic = Semantic::new(SemanticType::FloorSurface);

        builder
            .set_semantic_surface(Some(surface0), wall_semantic.clone())
            .unwrap();
        builder
            .set_semantic_surface(Some(surface1), wall_semantic.clone())
            .unwrap();
        builder
            .set_semantic_surface(Some(surface2), wall_semantic.clone())
            .unwrap();
        builder
            .set_semantic_surface(Some(surface3), wall_semantic)
            .unwrap();
        let roof_sem_ref = builder.set_semantic_surface(Some(surface4), roof_semantic);
        let floor_sem_ref = builder.set_semantic_surface(Some(surface5), floor_semantic);

        // Add materials
        let mut wall_material = OwnedMaterial::new("Wall".to_string());
        wall_material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
        let mut roof_material = OwnedMaterial::new("Roof".to_string());
        roof_material.set_diffuse_color(Some([0.9, 0.1, 0.1]));

        let wall_mat_ref = builder.set_material_surface(
            Some(surface0),
            wall_material,
            "material-theme".to_string(),
        );
        let roof_mat_ref = builder.set_material_surface(
            Some(surface4),
            roof_material,
            "material-theme".to_string(),
        );

        // Create a shell from the surfaces
        builder
            .add_shell(&[surface0, surface1, surface2, surface3, surface4, surface5])
            .expect("Failed to add shell");

        // Set LoD
        builder = builder.with_lod(LoD::LoD1);

        // Build the geometry
        let geom_ref = builder.build().expect("Failed to build geometry");

        // Get the geometry from the model
        let geometry = model
            .geometries()
            .get(geom_ref)
            .expect("Failed to get geometry");

        // Check geometry type
        assert_eq!(geometry.type_geometry(), &GeometryType::Solid);
        assert_eq!(geometry.lod(), Some(&LoD::LoD1));

        // Get the boundary and check its type
        let boundary = geometry.boundaries().expect("No boundary found");
        assert_eq!(boundary.check_type(), BoundaryType::Solid);

        // Verify the nested representation matches what we created
        let nested = boundary
            .to_nested_solid()
            .expect("Failed to convert to nested representation");
        assert_eq!(nested.len(), 1); // One shell
        assert_eq!(nested[0].len(), 6); // Six surfaces in the shell

        // Verify semantics
        let semantics = geometry.semantics().expect("No semantics found");
        let surface_semantics = semantics.surfaces();
        assert_eq!(surface_semantics.len(), 6); // Should have entries for all surfaces

        // Verify the specific semantics
        assert_eq!(surface_semantics[4], Some(roof_sem_ref.clone().unwrap()));
        assert_eq!(surface_semantics[5], Some(floor_sem_ref.clone().unwrap()));

        // Verify materials
        let materials = geometry.materials().expect("No materials found");
        let (_theme_material, material_map) = materials.first().unwrap();
        let surface_materials = material_map.surfaces();
        assert_eq!(surface_materials.len(), 6); // Should have entries for all surfaces

        // Verify the material references
        assert_eq!(surface_materials[0], Some(wall_mat_ref.clone().unwrap()));
        assert_eq!(surface_materials[4], Some(roof_mat_ref.clone().unwrap()));

        // Verify the material objects
        let wall_material = model
            .get_material(wall_mat_ref.unwrap())
            .expect("Wall material not found");
        assert_eq!(wall_material.name(), "Wall");
        assert_eq!(wall_material.diffuse_color().unwrap(), &[0.8, 0.8, 0.8]);
    }

    #[test]
    fn test_multisolid() {
        let mut model = create_test_model();

        // Create a builder for MultiSolid geometry
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiSolid, BuilderMode::Regular);

        // Add vertices for first cube (small cube at origin)
        let p0 = builder.add_point(QuantizedCoordinate::new(0, 0, 0)); // small cube - bottom-front-left
        let p1 = builder.add_point(QuantizedCoordinate::new(5, 0, 0)); // small cube - bottom-front-right
        let p2 = builder.add_point(QuantizedCoordinate::new(5, 5, 0)); // small cube - bottom-back-right
        let p3 = builder.add_point(QuantizedCoordinate::new(0, 5, 0)); // small cube - bottom-back-left
        let p4 = builder.add_point(QuantizedCoordinate::new(0, 0, 5)); // small cube - top-front-left
        let p5 = builder.add_point(QuantizedCoordinate::new(5, 0, 5)); // small cube - top-front-right
        let p6 = builder.add_point(QuantizedCoordinate::new(5, 5, 5)); // small cube - top-back-right
        let p7 = builder.add_point(QuantizedCoordinate::new(0, 5, 5)); // small cube - top-back-left

        // Add vertices for second cube (larger cube offset from first)
        let p8 = builder.add_point(QuantizedCoordinate::new(10, 10, 0)); // large cube - bottom-front-left
        let p9 = builder.add_point(QuantizedCoordinate::new(20, 10, 0)); // large cube - bottom-front-right
        let p10 = builder.add_point(QuantizedCoordinate::new(20, 20, 0)); // large cube - bottom-back-right
        let p11 = builder.add_point(QuantizedCoordinate::new(10, 20, 0)); // large cube - bottom-back-left
        let p12 = builder.add_point(QuantizedCoordinate::new(10, 10, 10)); // large cube - top-front-left
        let p13 = builder.add_point(QuantizedCoordinate::new(20, 10, 10)); // large cube - top-front-right
        let p14 = builder.add_point(QuantizedCoordinate::new(20, 20, 10)); // large cube - top-back-right
        let p15 = builder.add_point(QuantizedCoordinate::new(10, 20, 10)); // large cube - top-back-left

        // Define surfaces for the first cube
        // Front face (cube 1)
        let ring0 = builder
            .add_ring(&[p0, p1, p5, p4, p0])
            .expect("Failed to create ring");
        let surface0 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring0)
            .expect("Failed to add front face of first cube");

        // Right face (cube 1)
        let ring1 = builder
            .add_ring(&[p1, p2, p6, p5, p1])
            .expect("Failed to create ring");
        let surface1 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring1)
            .expect("Failed to add right face of first cube");

        // Back face (cube 1)
        let ring2 = builder
            .add_ring(&[p2, p3, p7, p6, p2])
            .expect("Failed to create ring");
        let surface2 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring2)
            .expect("Failed to add back face of first cube");

        // Left face (cube 1)
        let _ring3 = builder
            .add_ring(&[p3, p0, p4, p7, p3])
            .expect("Failed to create ring");
        let surface3 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring2)
            .expect("Failed to add left face of first cube");

        // Top face (cube 1)
        let ring4 = builder
            .add_ring(&[p4, p5, p6, p7, p3])
            .expect("Failed to create ring");
        let surface4 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring4)
            .expect("Failed to add top face of first cube");

        // Bottom face (cube 1)
        let ring5 = builder
            .add_ring(&[p0, p3, p2, p1, p0])
            .expect("Failed to create ring");
        let surface5 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring5)
            .expect("Failed to add bottom face of first cube");

        // Define surfaces for the second cube
        // Front face (cube 2)
        let ring6 = builder
            .add_ring(&[p8, p9, p13, p12, p8])
            .expect("Failed to create ring");
        let surface6 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring6)
            .expect("Failed to add front face of second cube");

        // Right face (cube 2)
        let ring7 = builder
            .add_ring(&[p9, p10, p14, p13, p9])
            .expect("Failed to create ring");
        let surface7 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring7)
            .expect("Failed to add right face of second cube");

        // Back face (cube 2)
        let ring8 = builder
            .add_ring(&[p10, p11, p15, p14, p10])
            .expect("Failed to create ring");
        let surface8 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring8)
            .expect("Failed to add back face of second cube");

        // Left face (cube 2)
        let ring9 = builder
            .add_ring(&[p11, p8, p12, p15, p11])
            .expect("Failed to create ring");
        let surface9 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring9)
            .expect("Failed to add left face of second cube");

        // Top face (cube 2)
        let ring10 = builder
            .add_ring(&[p12, p13, p14, p15, p12])
            .expect("Failed to create ring");
        let surface10 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring10)
            .expect("Failed to add top face of second cube");

        // Bottom face (cube 2)
        let ring11 = builder
            .add_ring(&[p8, p11, p10, p9, p8])
            .expect("Failed to create ring");
        let surface11 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring11)
            .expect("Failed to add bottom face of second cube");

        // Create semantics for different types of surfaces
        let roof_semantic = Semantic::new(SemanticType::RoofSurface);
        let ground_semantic = Semantic::new(SemanticType::GroundSurface);

        // Add semantics to faces
        let roof_sem_ref1 = builder.set_semantic_surface(Some(surface4), roof_semantic.clone());
        let ground_sem_ref1 = builder.set_semantic_surface(Some(surface5), ground_semantic.clone());
        let roof_sem_ref2 = builder.set_semantic_surface(Some(surface10), roof_semantic);
        let ground_sem_ref2 = builder.set_semantic_surface(Some(surface11), ground_semantic);

        // For the walls, we'll use material instead of semantics
        let mut red_material = OwnedMaterial::new("RedWall".to_string());
        red_material.set_diffuse_color(Some([0.9, 0.1, 0.1]));
        let mut blue_material = OwnedMaterial::new("BlueWall".to_string());
        blue_material.set_diffuse_color(Some([0.1, 0.1, 0.9]));

        // Apply materials to some surfaces
        let red_mat_ref = builder.set_material_surface(
            Some(surface0),
            red_material,
            "material-theme".to_string(),
        );
        let blue_mat_ref = builder.set_material_surface(
            Some(surface6),
            blue_material,
            "material-theme".to_string(),
        );

        // Create shells for each cube
        builder
            .add_shell(&[surface0, surface1, surface2, surface3, surface4, surface5])
            .expect("Failed to add shell for first cube");

        builder
            .add_shell(&[surface6, surface7, surface8, surface9, surface10, surface11])
            .expect("Failed to add shell for second cube");

        // Create solids from shells
        let _solid0 = builder.start_solid();
        builder
            .add_solid_outer_shell(0)
            .expect("Failed to add outer shell to solid 0");

        let _solid1 = builder.start_solid();
        builder
            .add_solid_outer_shell(1)
            .expect("Failed to add outer shell to solid 1");

        // Set LoD
        builder = builder.with_lod(LoD::LoD1);

        // Build the geometry
        let geom_ref = builder.build().expect("Failed to build geometry");

        // Get the geometry from the model
        let geometry = model
            .geometries()
            .get(geom_ref)
            .expect("Failed to get geometry");

        // Check geometry type
        assert_eq!(geometry.type_geometry(), &GeometryType::MultiSolid);
        assert_eq!(geometry.lod(), Some(&LoD::LoD1));

        // Get the boundary and check its type
        let boundary = geometry.boundaries().expect("No boundary found");
        assert_eq!(boundary.check_type(), BoundaryType::MultiOrCompositeSolid);

        // Verify the nested representation matches what we created
        let nested = boundary
            .to_nested_multi_or_composite_solid()
            .expect("Failed to convert to nested");

        // We should have 2 solids
        assert_eq!(nested.len(), 2);

        // Each solid should have 1 shell
        assert_eq!(nested[0].len(), 1);
        assert_eq!(nested[1].len(), 1);

        // Each shell should have 6 surfaces
        assert_eq!(nested[0][0].len(), 6);
        assert_eq!(nested[1][0].len(), 6);

        // Verify semantics
        let semantics = geometry.semantics().expect("No semantics found");
        let surface_semantics = semantics.surfaces();
        assert_eq!(surface_semantics.len(), 12); // Should have entries for all surfaces

        // Verify specific semantics
        assert_eq!(surface_semantics[4], Some(roof_sem_ref1.clone().unwrap()));
        assert_eq!(surface_semantics[5], Some(ground_sem_ref1.clone().unwrap()));
        assert_eq!(surface_semantics[10], Some(roof_sem_ref2.clone().unwrap()));
        assert_eq!(
            surface_semantics[11],
            Some(ground_sem_ref2.clone().unwrap())
        );

        // Verify materials
        let materials = geometry.materials().expect("No materials found");
        let (_theme_material, material_map) = materials.first().unwrap();
        let surface_materials = material_map.surfaces();
        assert_eq!(surface_materials.len(), 12); // Should have entries for all surfaces

        // Verify the material references
        assert_eq!(surface_materials[0], Some(red_mat_ref.clone().unwrap()));
        assert_eq!(surface_materials[6], Some(blue_mat_ref.clone().unwrap()));

        // Verify the material objects
        let red_material = model
            .get_material(red_mat_ref.unwrap())
            .expect("Red material not found");
        assert_eq!(red_material.name(), "RedWall");
        assert_eq!(red_material.diffuse_color().unwrap(), &[0.9, 0.1, 0.1]);

        let blue_material = model
            .get_material(blue_mat_ref.unwrap())
            .expect("Blue material not found");
        assert_eq!(blue_material.name(), "BlueWall");
        assert_eq!(blue_material.diffuse_color().unwrap(), &[0.1, 0.1, 0.9]);
    }

    #[test]
    fn test_geometry_template_and_instance() {
        let mut model = create_test_model();

        // PART 1: Create a template geometry (MultiLineString with semantics)
        // ------------------------------------------------------------------

        // Create a builder in Template mode for template creation
        let mut template_builder = GeometryBuilder::new(
            &mut model,
            GeometryType::MultiLineString,
            BuilderMode::Template,
        );

        // Add template vertices using RealWorldCoordinate (in local coordinate system)
        let tp0 = template_builder.add_template_point(RealWorldCoordinate::new(0.0, 0.0, 0.0));
        let tp1 = template_builder.add_template_point(RealWorldCoordinate::new(1.0, 0.0, 0.0));
        let tp2 = template_builder.add_template_point(RealWorldCoordinate::new(1.0, 1.0, 0.0));
        let tp3 = template_builder.add_template_point(RealWorldCoordinate::new(0.0, 1.0, 0.0));
        let tp4 = template_builder.add_template_point(RealWorldCoordinate::new(2.0, 0.0, 0.0));
        let tp5 = template_builder.add_template_point(RealWorldCoordinate::new(2.0, 2.0, 0.0));

        // Create three linestrings in our template
        // First linestring: square/rectangle
        template_builder
            .add_linestring(&[tp0, tp1, tp2, tp3, tp0])
            .expect("Failed to add first linestring to template");

        // Second linestring: diagonal
        let ls2 = template_builder
            .add_linestring(&[tp0, tp2])
            .expect("Failed to add second linestring to template");

        // Third linestring: another line
        template_builder
            .add_linestring(&[tp1, tp4, tp5])
            .expect("Failed to add third linestring to template");

        // Create semantic for the second linestring
        let sem = Semantic::new(SemanticType::TransportationMarking);

        // Set semantic for the second linestring (the diagonal)
        let sem_ref = template_builder
            .set_semantic_linestring(Some(ls2), sem)
            .expect("Failed to set semantic for template linestring");

        // Set LoD for the template
        template_builder = template_builder.with_lod(LoD::LoD2);

        // Build the template geometry - this adds it to the template_geometries pool
        let template_ref = template_builder
            .build()
            .expect("Failed to build template geometry");

        // Verify template was created correctly and placed in the template pool
        assert!(
            model.template_geometries().get(template_ref).is_some(),
            "Template geometry not found in template pool"
        );

        // Get the template from the pool for further verification
        let template = model
            .template_geometries()
            .get(template_ref)
            .expect("Failed to get template geometry");

        // Verify template properties
        assert_eq!(template.type_geometry(), &GeometryType::MultiLineString);
        assert_eq!(template.lod(), Some(&LoD::LoD2));

        // Check that template vertices were added to template_vertices pool
        assert_eq!(
            model.template_vertices().len(),
            6,
            "Expected 6 vertices in template_vertices pool"
        );

        // Verify template semantics
        let semantics = template
            .semantics()
            .expect("No semantics found in template");
        let linestring_semantics = semantics.linestrings();

        // Verify linestrings have semantics applied correctly
        assert_eq!(linestring_semantics.len(), 3);
        assert!(linestring_semantics[0].is_none());
        assert_eq!(linestring_semantics[1], Some(sem_ref));
        assert!(linestring_semantics[2].is_none());

        // PART 2: Create a GeometryInstance that references this template
        // --------------------------------------------------------------

        // Add a reference point in the main vertex pool (not template vertices)
        // This is where the template will be positioned in the city model
        let ref_point_idx = model
            .add_vertex(QuantizedCoordinate::new(100, 200, 50))
            .expect("Failed to add reference point");

        // Create a GeometryInstance builder in Regular mode
        let mut instance_builder = GeometryBuilder::new(
            &mut model,
            GeometryType::GeometryInstance,
            BuilderMode::Regular,
        );

        // Add the reference point vertex (anchor point for template placement)
        let _instance_point = instance_builder.add_vertex(ref_point_idx);

        // Set the template reference - this tells the instance which template to use
        instance_builder = instance_builder
            .with_template(template_ref)
            .expect("Failed to set template boundaries");

        // Set the transformation matrix
        // This defines how the template is transformed at the reference point:
        // - Scale by 2 in all dimensions (first three diagonal elements)
        // - No rotation (zeros in off-diagonal elements)
        // - No additional translation beyond reference point (last row)
        instance_builder = instance_builder
            .with_transformation_matrix([
                2.0, 0.0, 0.0, 0.0, // Scale x by 2
                0.0, 2.0, 0.0, 0.0, // Scale y by 2
                0.0, 0.0, 2.0, 0.0, // Scale z by 2
                0.0, 0.0, 0.0, 1.0, // No additional translation (uses reference point)
            ])
            .expect("Failed to set transformation matrix");

        // Build the geometry instance - this adds it to the regular geometries pool
        let instance_ref = instance_builder
            .build()
            .expect("Failed to build geometry instance");

        // Get the geometry instance for verification
        let instance = model
            .geometries()
            .get(instance_ref)
            .expect("Failed to get geometry instance");

        // Verify the instance properties
        assert_eq!(instance.type_geometry(), &GeometryType::GeometryInstance);

        // Verify template reference is correct
        assert_eq!(instance.instance_template(), Some(&template_ref));

        // Verify transformation matrix is stored correctly
        let expected_matrix = [
            2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        assert_eq!(
            instance.instance_transformation_matrix(),
            Some(&expected_matrix)
        );

        // Verify that the instance has a boundary with just the reference point
        assert_eq!(instance.instance_reference_point(), Some(&ref_point_idx));

        // Make sure the instance is in the regular geometries pool, not the template pool
        assert!(
            model.geometries().get(instance_ref).is_some(),
            "Geometry instance not found in regular geometry pool"
        );
    }
}
