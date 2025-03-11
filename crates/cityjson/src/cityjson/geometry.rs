#![doc = include_str!("../../docs/boundary_guide.md")]

use crate::cityjson::citymodel::{CityModelTrait, CityModelTypes};
use crate::cityjson::geometry::boundary::{Boundary, BoundaryCounter};
use crate::cityjson::geometry::semantic::SemanticTypeTrait;
use crate::cityjson::vertex::{VertexIndex, VertexRef};
use crate::errors;
use crate::errors::Error;
use crate::resources::mapping::{MaterialMap, SemanticMap, TextureMap};
use crate::resources::pool::ResourceRef;
use std::collections::HashMap;

pub mod boundary;
pub mod semantic;

pub trait GeometryTrait<VR: VertexRef, RR: ResourceRef> {
    /// Create a new geometry given the parts
    fn new(
        type_geometry: GeometryType,
        lod: Option<LoD>,
        boundaries: Option<Boundary<VR>>,
        semantics: Option<SemanticMap<VR, RR>>,
        material: Option<MaterialMap<VR, RR>>,
        texture: Option<TextureMap<VR, RR>>,
        template_boundaries: Option<usize>,
        template_transformation_matrix: Option<[f64; 16]>,
    ) -> Self;

    /// Returns the geometry type
    fn type_geometry(&self) -> &GeometryType;

    /// Returns the level of detail
    fn lod(&self) -> Option<&LoD>;

    /// Returns the geometry boundaries
    fn boundaries(&self) -> Option<&Boundary<VR>>;

    /// Returns the semantic mapping
    fn semantics(&self) -> Option<&SemanticMap<VR, RR>>;

    /// Returns the material mapping
    fn materials(&self) -> Option<&MaterialMap<VR, RR>>;

    /// Returns the texture mapping
    fn textures(&self) -> Option<&TextureMap<VR, RR>>;

    /// Returns the template boundaries index, if any
    fn template_boundaries(&self) -> Option<&usize>;

    /// Returns the template transformation matrix, if any
    fn template_transformation_matrix(&self) -> Option<&[f64; 16]>;
}

/// Represents a surface under construction with one outer ring and optional inner rings
#[derive(Default)]
struct SurfaceInProgress<SemType: SemanticTypeTrait> {
    outer_ring: Option<usize>, // index to outer ring
    inner_rings: Vec<usize>,   // indices to inner rings
    semantic: Option<SemType>, // semantic type for the whole surface
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

pub struct GeometryBuilder<'a, V: CityModelTypes, M: CityModelTrait<V>> {
    model: &'a mut M,
    type_geometry: GeometryType,
    lod: Option<LoD>,
    vertices: Vec<V::CoordinateType>, // todo: generalize to Coordinate
    rings: Vec<Vec<usize>>,             // indices into vertices
    surfaces: Vec<SurfaceInProgress<V::SemType>>, // surfaces with their rings
    shells: Vec<ShellInProgress>,       // shells with their surfaces
    solids: Vec<SolidInProgress>,       // solids with their shells
    // Current element tracking
    current_linestring: Option<usize>, // current linestring being built
    current_surface: Option<usize>,    // current surface being built
    current_shell: Option<usize>,      // current shell being built
    current_solid: Option<usize>,      // current solid being built
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
    pub fn new(model: &'a mut M, type_geometry: GeometryType) -> Self {
        Self {
            model,
            type_geometry,
            lod: None,
            vertices: Vec::new(),
            rings: Vec::new(),
            surfaces: Vec::new(),
            shells: Vec::new(),
            solids: Vec::new(),
            current_linestring: None,
            current_surface: None,
            current_shell: None,
            current_solid: None,
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

    /// Adds a new vertex to the CityModel.
    ///
    /// Returns the index of the new vertex.
    // pub fn add_vertex(&mut self, x: i64, y: i64, z: i64) -> usize {
    //     self.vertices.push(QuantizedCoordinate::new(x, y, z));
    //     self.vertices.len() - 1
    // }

    /// Adds a new ring to the geometry.
    ///
    /// # Errors
    ///
    /// Returns `InvalidRing` if:
    /// - The ring has fewer than three vertices
    /// - The vertices do not form a valid ring (first != last)
    /// - Any vertex index is out of bounds
    pub fn add_ring(&mut self, vertices: &[usize]) -> errors::Result<usize> {
        if vertices.len() < 3 {
            return Err(Error::InvalidRing {
                reason: "Ring must have at least 3 vertices".to_string(),
                vertex_count: vertices.len(),
            });
        }
        self.rings.push(vertices.to_vec());
        Ok(self.rings.len() - 1)
    }

    /// Starts a new surface with an optional semantic type.
    ///
    /// Returns the index of the new surface.
    pub fn start_surface(&mut self, semantic: Option<V::SemType>) -> usize {
        let idx = self.surfaces.len();
        self.surfaces.push(SurfaceInProgress::default());
        self.surfaces[idx].semantic = semantic;
        self.current_surface = Some(idx);
        idx
    }

    /// Sets the outer ring for the current surface.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No surface is currently being built (`NoCurrentElement`)
    /// - The ring is invalid (`InvalidRing`)
    /// - An outer ring is already set (`InvalidGeometry`)
    pub fn set_surface_outer_ring(&mut self, vertices: &[usize]) -> errors::Result<()> {
        let surface_idx = self
            .current_surface
            .ok_or_else(|| Error::NoCurrentElement {
                element_type: "surface".to_string(),
            })?;
        let ring_idx = self.add_ring(vertices)?;
        self.surfaces[surface_idx].outer_ring = Some(ring_idx);
        Ok(())
    }

    /// Adds an inner ring (hole) to the current surface.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No surface is currently being built (`NoCurrentElement`)
    /// - The current surface has no outer ring (`MissingOuterElement`)
    /// - The ring is invalid (`InvalidRing`)
    /// - The ring is not contained within the outer ring (`InvalidGeometry`)
    pub fn add_surface_inner_ring(&mut self, vertices: &[usize]) -> errors::Result<()> {
        let surface_idx = self
            .current_surface
            .ok_or_else(|| Error::NoCurrentElement {
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

    /// Starts a new shell.
    ///
    /// Returns the index of the new shell.
    pub fn start_shell(&mut self) -> usize {
        let idx = self.shells.len();
        self.shells.push(ShellInProgress::default());
        self.current_shell = Some(idx);
        idx
    }

    /// Adds an outer surface to the current shell.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No shell is currently being built (`NoCurrentElement`)
    /// - The surface index is invalid (`InvalidReference`)
    /// - The surface is already part of another shell (`InvalidGeometry`)
    pub fn add_shell_outer_surface(&mut self, surface_idx: usize) -> errors::Result<()> {
        let shell_idx = self.current_shell.ok_or_else(|| Error::NoCurrentElement {
            element_type: "shell".to_string(),
        })?;

        if surface_idx >= self.surfaces.len() {
            return Err(Error::InvalidReference {
                element_type: "surface".to_string(),
                index: surface_idx,
                max_index: self.surfaces.len().saturating_sub(1),
            });
        }

        self.shells[shell_idx].outer_surfaces.push(surface_idx);
        Ok(())
    }

    /// Adds an inner surface to the current shell.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No shell is currently being built (`NoCurrentElement`)
    /// - The surface index is invalid (`InvalidReference`)
    /// - The shell has no outer surfaces (`MissingOuterElement`)
    /// - The surface is already part of another shell (`InvalidGeometry`)
    pub fn add_shell_inner_surface(&mut self, surface_idx: usize) -> errors::Result<()> {
        let shell_idx = self.current_shell.ok_or_else(|| Error::NoCurrentElement {
            element_type: "shell".to_string(),
        })?;

        if surface_idx >= self.surfaces.len() {
            return Err(Error::InvalidReference {
                element_type: "surface".to_string(),
                index: surface_idx,
                max_index: self.surfaces.len().saturating_sub(1),
            });
        }

        if self.shells[shell_idx].outer_surfaces.is_empty() {
            return Err(Error::MissingOuterElement {
                context: "Cannot add inner surface before outer surfaces".to_string(),
            });
        }

        self.shells[shell_idx].inner_surfaces.push(surface_idx);
        Ok(())
    }

    /// Starts a new solid.
    ///
    /// Returns the index of the new solid.
    pub fn start_solid(&mut self) -> usize {
        let idx = self.solids.len();
        self.solids.push(SolidInProgress::default());
        self.current_solid = Some(idx);
        idx
    }

    /// Sets the outer shell for the current solid.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No solid is currently being built (`NoCurrentElement`)
    /// - The shell index is invalid (`InvalidReference`)
    /// - An outer shell is already set (`InvalidGeometry`)
    /// - The shell is already part of another solid (`InvalidGeometry`)
    pub fn set_solid_outer_shell(&mut self, shell_idx: usize) -> errors::Result<()> {
        let solid_idx = self.current_solid.ok_or_else(|| Error::NoCurrentElement {
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

    /// Adds an inner shell to the current solid.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No solid is currently being built (`NoCurrentElement`)
    /// - The shell index is invalid (`InvalidReference`)
    /// - The solid has no outer shell (`MissingOuterElement`)
    /// - The shell is already part of another solid (`InvalidGeometry`)
    pub fn add_solid_inner_shell(&mut self, shell_idx: usize) -> errors::Result<()> {
        let solid_idx = self.current_solid.ok_or_else(|| Error::NoCurrentElement {
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

    // Point semantics
    // pub fn add_point_with_semantic(
    //     &mut self,
    //     x: i64,
    //     y: i64,
    //     z: i64,
    //     semantic: Option<V::Semantic>,
    // ) -> usize {
    //     let point_idx = self.add_vertex(x, y, z);
    //     if let Some(semantic) = semantic {
    //         let sem_id = self.model.add_semantic(semantic);
    //         self.point_semantics.insert(point_idx, sem_id);
    //     }
    //     point_idx
    // }

    // LineString semantics
    pub fn set_linestring_semantic(&mut self, semantic: V::Semantic) -> errors::Result<()> {
        let line_idx = self
            .current_linestring
            .ok_or_else(|| Error::NoCurrentElement {
                element_type: "linestring".to_string(),
            })?;
        let sem_id = self.model.add_semantic(semantic);
        self.linestring_semantics.insert(line_idx, sem_id);
        Ok(())
    }

    // Surface semantics
    pub fn set_surface_semantic(&mut self, semantic: V::Semantic) -> errors::Result<()> {
        let surface_idx = self
            .current_surface
            .ok_or_else(|| Error::NoCurrentElement {
                element_type: "surface".to_string(),
            })?;
        let sem_id = self.model.add_semantic(semantic);
        self.surface_semantics.insert(surface_idx, sem_id);
        Ok(())
    }

    pub fn set_surface_material(&mut self, material: V::Material) -> errors::Result<()> {
        let surface_idx = self
            .current_surface
            .ok_or_else(|| Error::NoCurrentElement {
                element_type: "surface".to_string(),
            })?;
        let mat_id = self.model.add_material(material);
        self.surface_materials.insert(surface_idx, mat_id);
        Ok(())
    }

    pub fn set_surface_texture(&mut self, texture: V::Texture) -> errors::Result<()> {
        let surface_idx = self
            .current_surface
            .ok_or_else(|| Error::NoCurrentElement {
                element_type: "surface".to_string(),
            })?;
        let tex_id = self.model.add_texture(texture);
        self.surface_textures.insert(surface_idx, tex_id);
        Ok(())
    }

    /// Builds the geometry and adds it to the model.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Any surface is missing an outer ring (`IncompleteGeometry`)
    /// - Any shell has no outer surfaces (`IncompleteGeometry`)
    /// - Any solid is missing an outer shell (`IncompleteGeometry`)
    /// - The geometry type doesn't match the structure (`InvalidGeometryType`)
    /// - Any vertex indices are invalid (`InvalidReference`)
    /// - Any referenced elements are invalid (`InvalidReference`)
    /// - The resulting geometry would be invalid (`InvalidGeometry`)
    pub fn build(self) -> errors::Result<V::ResourceRef> {
        // Validate structure before building
        self.validate_structure()?;

        // Add all vertices to the model and get their indices
        let vertex_indices: Vec<VertexIndex<V::VertexRef>> = self
            .vertices
            .into_iter()
            .map(|v| self.model.add_vertex(v))
            .collect::<errors::Result<_>>()?;

        // Create boundary structure
        let mut boundary = Boundary::new();
        let mut counter = BoundaryCounter::default();

        // Create semantic mappings
        let mut semantic_map = SemanticMap::<V::VertexRef, V::ResourceRef>::default();
        // Create material mappings
        let mut material_map = MaterialMap::<V::VertexRef, V::ResourceRef>::default();
        // Create texture mappings
        let texture_map = TextureMap::<V::VertexRef, V::ResourceRef>::default();

        match self.type_geometry {
            GeometryType::MultiPoint => {
                // Set vertex indices directly for multipoint
                boundary.vertices = vertex_indices.clone();

                // Create point semantics mapping
                if !self.point_semantics.is_empty() {
                    // todo: what about None cases?
                    semantic_map.points = self
                        .point_semantics
                        .into_values()
                        .map(|v| Some(v))
                        .collect();
                }
            }
            GeometryType::MultiLineString => {
                // Process vertices for linestrings
                let mut vertex_list = Vec::new();
                let mut ring_indices = Vec::new();

                for linestring in &self.rings {
                    ring_indices.push(counter.vertex_offset());
                    for &vertex_idx in linestring {
                        vertex_list.push(vertex_indices[vertex_idx]);
                        counter.increment_vertex_idx();
                    }
                }
                boundary.vertices = vertex_list;
                boundary.rings = ring_indices;

                // Create linestring semantics mapping
                if !self.linestring_semantics.is_empty() {
                    semantic_map.linestrings = self
                        .linestring_semantics
                        .into_values()
                        .map(|v| Some(v))
                        .collect();
                }
            }
            _ => {
                // Process vertices and rings for surfaces/solids
                let mut vertex_list = Vec::new();
                let mut ring_indices = Vec::new();

                for ring in &self.rings {
                    ring_indices.push(counter.vertex_offset());
                    for &vertex_idx in ring {
                        vertex_list.push(vertex_indices[vertex_idx]);
                        counter.increment_vertex_idx();
                    }
                }
                boundary.vertices = vertex_list;
                boundary.rings = ring_indices.clone();

                // Process surfaces with their rings
                let mut surface_indices = Vec::new();
                for surface in &self.surfaces {
                    if let Some(outer_ring) = surface.outer_ring {
                        // Start of this surface's rings
                        surface_indices.push(counter.ring_offset());

                        // Add outer ring
                        ring_indices.push(VertexIndex::try_from(outer_ring)?);
                        counter.increment_ring_idx();

                        // Add inner rings if any
                        for &inner_ring in &surface.inner_rings {
                            ring_indices.push(VertexIndex::try_from(inner_ring)?);
                            counter.increment_ring_idx();
                        }
                    }
                }
                boundary.surfaces = surface_indices;

                // Create surface semantics mapping
                if !self.surface_semantics.is_empty() {
                    semantic_map.surfaces = self
                        .surface_semantics
                        .into_values()
                        .map(|v| Some(v))
                        .collect();
                }

                // Add surface materials
                if !self.surface_materials.is_empty() {
                    material_map.surfaces = self
                        .surface_materials
                        .into_values()
                        .map(|v| Some(v))
                        .collect();
                }

                // Process shells with their surfaces
                let mut shell_indices = Vec::new();
                for shell in &self.shells {
                    shell_indices.push(counter.surface_offset());

                    // Account for all surfaces in this shell
                    for _ in 0..shell.outer_surfaces.len() + shell.inner_surfaces.len() {
                        counter.increment_surface_idx();
                    }
                }
                if !shell_indices.is_empty() {
                    boundary.shells = shell_indices;
                }

                // Process solids with their shells
                let mut solid_indices = Vec::new();
                for solid in &self.solids {
                    if let Some(_) = solid.outer_shell {
                        solid_indices.push(counter.shell_offset());
                        counter.increment_shell_idx(); // Outer shell

                        // Account for inner shells
                        for _ in &solid.inner_shells {
                            counter.increment_shell_idx();
                        }
                    }
                }
                if !solid_indices.is_empty() {
                    boundary.solids = solid_indices;
                }
            }
        }

        // Create the geometry
        let geometry = V::Geometry::new(
            self.type_geometry,
            self.lod,
            Some(boundary),
            Some(semantic_map),
            Some(material_map),
            Some(texture_map),
            None,
            None,
        );

        Ok(self.model.add_geometry(geometry))
    }

    fn validate_structure(&self) -> errors::Result<()> {
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

        // Verify geometry type matches structure
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
            }
            GeometryType::GeometryInstance => {
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
