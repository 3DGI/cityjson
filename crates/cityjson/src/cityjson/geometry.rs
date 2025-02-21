use crate::cityjson::citymodel::GenericCityModel;
use crate::cityjson::coordinate::RealWorldCoordinate;
use crate::cityjson::geometry::boundary::{Boundary, BoundaryCounter};
use crate::cityjson::geometry::semantic::SemanticType;
use crate::cityjson::index::{VertexIndex, VertexRef};
use crate::errors;
use crate::errors::Error;
use crate::resources::mapping::{MaterialMap, SemanticMap, TextureMap};
use crate::resources::pool::{ResourcePool, ResourceRef};
use crate::resources::storage::StringStorage;
use std::collections::HashMap;

use crate::cityjson::geometry::material::Material;
use crate::cityjson::geometry::semantic::Semantic;
use crate::cityjson::geometry::texture::Texture;

pub mod boundary;
pub mod material;
pub mod semantic;
pub mod texture;

pub trait GeometryTrait<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
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
}

/// Represents a surface under construction with one outer ring and optional inner rings
#[derive(Default)]
struct SurfaceInProgress {
    outer_ring: Option<usize>,      // index to outer ring
    inner_rings: Vec<usize>,        // indices to inner rings
    semantic: Option<SemanticType>, // semantic type for the whole surface
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

pub struct GeometryBuilder<
    'a,
    VR: VertexRef,
    RR: ResourceRef,
    RPS: ResourcePool<Sem, RR>,
    RPM: ResourcePool<Mat, RR>,
    RPT: ResourcePool<Tex, RR>,
    SS: StringStorage,
    Geo: GeometryTrait<VR, RR, SS>,
    Mat: Material<SS>,
    Sem: Semantic<RR, SS>,
    Tex: Texture<SS>,
> {
    model: &'a mut GenericCityModel<VR, RR, RPS, RPM, RPT, SS, Geo, Mat, Sem, Tex>,
    type_geometry: GeometryType,
    lod: Option<LoD>,
    vertices: Vec<RealWorldCoordinate>,
    rings: Vec<Vec<usize>>,           // indices into vertices
    surfaces: Vec<SurfaceInProgress>, // surfaces with their rings
    shells: Vec<ShellInProgress>,     // shells with their surfaces
    solids: Vec<SolidInProgress>,     // solids with their shells
    // Current element tracking
    current_linestring: Option<usize>, // current linestring being built
    current_surface: Option<usize>,    // current surface being built
    current_shell: Option<usize>,      // current shell being built
    current_solid: Option<usize>,      // current solid being built
    // Semantic storage
    point_semantics: HashMap<usize, RR>,
    linestring_semantics: HashMap<usize, RR>,
    surface_semantics: HashMap<usize, RR>,
    // Material storage
    surface_materials: HashMap<usize, RR>,
    // Texture storage
    surface_textures: HashMap<usize, RR>,
}

impl<'a, VR, RR, RPS, RPM, RPT, SS, Geo, Mat, Sem, Tex>
    GeometryBuilder<'a, VR, RR, RPS, RPM, RPT, SS, Geo, Mat, Sem, Tex>
where
    VR: VertexRef,
    RR: ResourceRef,
    RPS: ResourcePool<Sem, RR>,
    RPM: ResourcePool<Mat, RR>,
    RPT: ResourcePool<Tex, RR>,
    SS: StringStorage,
    Geo: GeometryTrait<VR, RR, SS>,
    Mat: Material<SS>,
    Sem: Semantic<RR, SS>,
    Tex: Texture<SS>,
{
    pub fn new(
        model: &'a mut GenericCityModel<VR, RR, RPS, RPM, RPT, SS, Geo, Mat, Sem, Tex>,
        type_geometry: GeometryType,
    ) -> Self {
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

    /// Adds a new vertex to the CityModel
    ///
    /// Returns the index of the new vertex.
    pub fn add_vertex(&mut self, x: f64, y: f64, z: f64) -> usize {
        self.vertices.push(RealWorldCoordinate::new(x, y, z));
        self.vertices.len() - 1
    }

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
    pub fn start_surface(&mut self, semantic: Option<SemanticType>) -> usize {
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
    pub fn add_point_with_semantic(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        semantic: Option<Sem>,
    ) -> usize {
        let point_idx = self.add_vertex(x, y, z);
        if let Some(semantic) = semantic {
            let sem_id = self.model.add_semantic(semantic);
            self.point_semantics.insert(point_idx, sem_id);
        }
        point_idx
    }

    // LineString semantics
    pub fn set_linestring_semantic(&mut self, semantic: Sem) -> errors::Result<()> {
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
    pub fn set_surface_semantic(&mut self, semantic: Sem) -> errors::Result<()> {
        let surface_idx = self
            .current_surface
            .ok_or_else(|| Error::NoCurrentElement {
                element_type: "surface".to_string(),
            })?;
        let sem_id = self.model.add_semantic(semantic);
        self.surface_semantics.insert(surface_idx, sem_id);
        Ok(())
    }

    pub fn set_surface_material(&mut self, material: Mat) -> errors::Result<()> {
        let surface_idx = self
            .current_surface
            .ok_or_else(|| Error::NoCurrentElement {
                element_type: "surface".to_string(),
            })?;
        let mat_id = self.model.add_material(material);
        self.surface_materials.insert(surface_idx, mat_id);
        Ok(())
    }

    pub fn set_surface_texture(&mut self, texture: Tex) -> errors::Result<()> {
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
    pub fn build(self) -> errors::Result<()> {
        // Validate structure before building
        self.validate_structure()?;

        // Add all vertices to the model and get their indices
        let vertex_indices: Vec<VertexIndex<VR>> = self
            .vertices
            .into_iter()
            .map(|v| self.model.add_vertex(v))
            .collect::<errors::Result<_>>()?;

        // Create boundary structure
        let mut boundary = Boundary::new();
        let mut counter = BoundaryCounter::default();

        // Create semantic mappings
        let mut semantic_map = SemanticMap::<VR, RR>::default();
        // Create material mappings
        let mut material_map = MaterialMap::<VR, RR>::default();
        // Create texture mappings
        let texture_map = TextureMap::<VR, RR>::default();

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
        let geometry = Geo::new(
            self.type_geometry,
            self.lod,
            Some(boundary),
            Some(semantic_map),
            Some(material_map),
            Some(texture_map),
            None,
            None,
        );

        self.model.add_geometry(geometry);
        Ok(())
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
                if self.solids.is_empty() {
                    return Err(Error::InvalidGeometryType {
                        expected: "solid geometry".to_string(),
                        found: "empty geometry".to_string(),
                    });
                }
            }
            GeometryType::Solid => {
                if self.solids.len() != 1 {
                    return Err(Error::InvalidGeometryType {
                        expected: "single solid".to_string(),
                        found: format!("{} solids", self.solids.len()),
                    });
                }
            }
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                if self.surfaces.is_empty() {
                    return Err(Error::InvalidGeometryType {
                        expected: "surface geometry".to_string(),
                        found: "empty geometry".to_string(),
                    });
                }
                if !self.shells.is_empty() || !self.solids.is_empty() {
                    return Err(Error::InvalidGeometryType {
                        expected: "surface geometry".to_string(),
                        found: "geometry with shells or solids".to_string(),
                    });
                }
            }
            _ => {}
        }

        Ok(())
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
