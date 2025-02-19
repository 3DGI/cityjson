//! # Geometry
//!
//! Represents a [Geometry object](https://www.cityjson.org/specs/1.1.3/#geometry-objects).
use crate::common::boundary::{Boundary, BoundaryCounter};
use crate::common::coordinate::RealWorldCoordinate;
use crate::common::index::{VertexIndex, VertexIndices, VertexRef};
use crate::common::storage::StringStorage;
use crate::common::{GeometryType, LoD};
use crate::errors::{Error, Result};
use crate::resources::mapping::{MaterialMap, SemanticMap, TextureMap};
use crate::resources::pool::{ResourcePool, ResourceRef};
use crate::common::citymodel::GenericCityModel;
use crate::v1_1::material::Material;
use crate::v1_1::semantic::Semantic;
use crate::v1_1::texture::Texture;
use std::collections::HashMap;
use crate::common::semantic::SemanticType;

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Geometry<VR: VertexRef, RR: ResourceRef> {
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary<VR>>,
    semantics: Option<SemanticMap<VR, RR>>,
    material: Option<MaterialMap<VR, RR>>,
    texture: Option<TextureMap<VR, RR>>,
    template_boundaries: Option<usize>,
    template_transformation_matrix: Option<[f64; 16]>,
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
    RPS: ResourcePool<Semantic<VR, S>, RR>,
    RPM: ResourcePool<Material<S>, RR>,
    RPT: ResourcePool<Texture<S>, RR>,
    S: StringStorage,
> {
    model: &'a mut GenericCityModel<VR, RR, RPS, RPM, RPT, S>,
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

impl<'a, VR, RR, RPS, RPM, RPT, S> GeometryBuilder<'a, VR, RR, RPS, RPM, RPT, S>
where
    VR: VertexRef,
    RR: ResourceRef,
    RPS: ResourcePool<Semantic<VR, S>, RR>,
    RPM: ResourcePool<Material<S>, RR>,
    RPT: ResourcePool<Texture<S>, RR>,
    S: StringStorage,
{
    pub fn new(
        model: &'a mut GenericCityModel<VR, RR, RPS, RPM, RPT, S>,
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
    pub fn add_ring(&mut self, vertices: &[usize]) -> Result<usize> {
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
    pub fn set_surface_outer_ring(&mut self, vertices: &[usize]) -> Result<()> {
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
    pub fn add_surface_inner_ring(&mut self, vertices: &[usize]) -> Result<()> {
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
    pub fn add_shell_outer_surface(&mut self, surface_idx: usize) -> Result<()> {
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
    pub fn add_shell_inner_surface(&mut self, surface_idx: usize) -> Result<()> {
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
    pub fn set_solid_outer_shell(&mut self, shell_idx: usize) -> Result<()> {
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
    pub fn add_solid_inner_shell(&mut self, shell_idx: usize) -> Result<()> {
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
        semantic: Option<Semantic<VR, S>>,
    ) -> usize {
        let point_idx = self.add_vertex(x, y, z);
        if let Some(semantic) = semantic {
            let sem_id = self.model.add_semantic(semantic);
            self.point_semantics.insert(point_idx, sem_id);
        }
        point_idx
    }

    // LineString semantics
    pub fn set_linestring_semantic(&mut self, semantic: Semantic<VR, S>) -> Result<()> {
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
    pub fn set_surface_semantic(&mut self, semantic: Semantic<VR, S>) -> Result<()> {
        let surface_idx = self
            .current_surface
            .ok_or_else(|| Error::NoCurrentElement {
                element_type: "surface".to_string(),
            })?;
        let sem_id = self.model.add_semantic(semantic);
        self.surface_semantics.insert(surface_idx, sem_id);
        Ok(())
    }

    pub fn set_surface_material(&mut self, material: Material<S>) -> Result<()> {
        let surface_idx = self
            .current_surface
            .ok_or_else(|| Error::NoCurrentElement {
                element_type: "surface".to_string(),
            })?;
        let mat_id = self.model.add_material(material);
        self.surface_materials.insert(surface_idx, mat_id);
        Ok(())
    }

    pub fn set_surface_texture(&mut self, texture: Texture<S>) -> Result<()> {
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
    pub fn build(self) -> Result<()> {
        // Validate structure before building
        self.validate_structure()?;

        // Add all vertices to the model and get their indices
        let vertex_indices: Vec<VertexIndex<VR>> = self
            .vertices
            .into_iter()
            .map(|v| self.model.add_vertex(v))
            .collect::<Result<_>>()?;

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
                boundary.vertices = VertexIndices::from_iter(vertex_indices.clone());

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
                boundary.vertices = VertexIndices::from_iter(vertex_list);
                boundary.rings = VertexIndices::from_iter(ring_indices);

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
                boundary.vertices = VertexIndices::from_iter(vertex_list);
                boundary.rings = VertexIndices::from_iter(ring_indices.clone());

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
                boundary.surfaces = VertexIndices::from_iter(surface_indices);

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
                    boundary.shells = VertexIndices::from_iter(shell_indices);
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
                    boundary.solids = VertexIndices::from_iter(solid_indices);
                }
            }
        }

        // Create the geometry
        let geometry = Geometry {
            type_geometry: self.type_geometry,
            lod: self.lod,
            boundaries: Some(boundary),
            semantics: Some(semantic_map),
            material: Some(material_map),
            texture: Some(texture_map),
            template_boundaries: None,
            template_transformation_matrix: None,
        };

        self.model.add_geometry(geometry);
        Ok(())
    }

    fn validate_structure(&self) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::attributes::AttributeValue;
    use crate::common::boundary::nested::BoundaryNestedMultiOrCompositeSolid32;
    use crate::common::storage::OwnedStringStorage;
    use crate::v1_1::citymodel::CityModel;

    #[test]
    fn test_multipoint_with_semantics() -> Result<()> {
        // Create a city model
        let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new();
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiPoint).with_lod(LoD::LoD2);

        // Create vertices with semantics
        // First point - TransportationMarking
        let v0 = builder.add_point_with_semantic(
            0.0,
            0.0,
            0.0,
            Some(Semantic::new(SemanticType::TransportationMarking)),
        );

        // Second point - no semantic
        let _v1 = builder.add_vertex(1.0, 0.0, 0.0);

        // Third point - TransportationHole with diameter attribute
        let mut hole_semantic = Semantic::new(SemanticType::TransportationHole);
        let mut attrs = hole_semantic.attributes_mut();
        attrs.insert("diameter".to_string(), AttributeValue::Float(1.5));
        let v2 = builder.add_point_with_semantic(2.0, 0.0, 0.0, Some(hole_semantic));

        // Fourth point - no semantic
        let _v3 = builder.add_vertex(3.0, 0.0, 0.0);

        // Fifth point - TransportationMarking
        let v4 = builder.add_point_with_semantic(
            4.0,
            0.0,
            0.0,
            Some(Semantic::new(SemanticType::TransportationMarking)),
        );

        // Build the geometry
        builder.build()?;

        // Get the built geometry and test filtering
        let geometry = &model.geometries[0];

        Ok(())
    }

    #[test]
    fn test_build_complex_multisolid() -> Result<()> {
        let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new();
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiSolid).with_lod(LoD::LoD2);

        // Create vertices for a cube with an inner cube
        // Outer cube vertices (0-7)
        let v0 = builder.add_vertex(0.0, 0.0, 0.0);
        let v1 = builder.add_vertex(2.0, 0.0, 0.0);
        let v2 = builder.add_vertex(2.0, 2.0, 0.0);
        let v3 = builder.add_vertex(0.0, 2.0, 0.0);
        let v4 = builder.add_vertex(0.0, 0.0, 2.0);
        let v5 = builder.add_vertex(2.0, 0.0, 2.0);
        let v6 = builder.add_vertex(2.0, 2.0, 2.0);
        let v7 = builder.add_vertex(0.0, 2.0, 2.0);

        // Inner cube vertices (8-15) - smaller cube inside
        let v8 = builder.add_vertex(0.5, 0.5, 0.5);
        let v9 = builder.add_vertex(1.5, 0.5, 0.5);
        let v10 = builder.add_vertex(1.5, 1.5, 0.5);
        let v11 = builder.add_vertex(0.5, 1.5, 0.5);
        let v12 = builder.add_vertex(0.5, 0.5, 1.5);
        let v13 = builder.add_vertex(1.5, 0.5, 1.5);
        let v14 = builder.add_vertex(1.5, 1.5, 1.5);
        let v15 = builder.add_vertex(0.5, 1.5, 1.5);

        // Window vertices for inner rings in front and right faces
        let w0 = builder.add_vertex(0.7, 0.0, 0.7);
        let w1 = builder.add_vertex(1.3, 0.0, 0.7);
        let w2 = builder.add_vertex(1.3, 0.0, 1.3);
        let w3 = builder.add_vertex(0.7, 0.0, 1.3);

        let w4 = builder.add_vertex(2.0, 0.7, 0.7);
        let w5 = builder.add_vertex(2.0, 1.3, 0.7);
        let w6 = builder.add_vertex(2.0, 1.3, 1.3);
        let w7 = builder.add_vertex(2.0, 0.7, 1.3);

        // Start building the outer shell
        let _solid_idx = builder.start_solid();
        let outer_shell_idx = builder.start_shell();

        // Bottom face (GroundSurface)
        let bottom_idx = builder.start_surface(Some(SemanticType::GroundSurface));
        builder.set_surface_outer_ring(&[v0, v3, v2, v1])?;
        builder.add_shell_outer_surface(bottom_idx)?;

        // Top face (RoofSurface)
        let top_idx = builder.start_surface(Some(SemanticType::RoofSurface));
        builder.set_surface_outer_ring(&[v4, v5, v6, v7])?;
        builder.add_shell_outer_surface(top_idx)?;

        // Front face with a window (WallSurface)
        let front_idx = builder.start_surface(Some(SemanticType::WallSurface));
        builder.set_surface_outer_ring(&[v0, v1, v5, v4])?;
        builder.add_surface_inner_ring(&[w0, w1, w2, w3])?; // Window hole
        builder.add_shell_outer_surface(front_idx)?;

        // Right face with a window (WallSurface)
        let right_idx = builder.start_surface(Some(SemanticType::WallSurface));
        builder.set_surface_outer_ring(&[v1, v2, v6, v5])?;
        builder.add_surface_inner_ring(&[w4, w5, w6, w7])?; // Window hole
        builder.add_shell_outer_surface(right_idx)?;

        // Back face (WallSurface)
        let back_idx = builder.start_surface(Some(SemanticType::WallSurface));
        builder.set_surface_outer_ring(&[v2, v3, v7, v6])?;
        builder.add_shell_outer_surface(back_idx)?;

        // Left face (no semantics)
        let left_idx = builder.start_surface(None);
        builder.set_surface_outer_ring(&[v3, v0, v4, v7])?;
        builder.add_shell_outer_surface(left_idx)?;

        // Set the outer shell
        builder.set_solid_outer_shell(outer_shell_idx)?;

        // Start building the inner shell (void)
        let inner_shell_idx = builder.start_shell();

        // Create the faces of the inner cube (all surfaces of inner shell)
        // Bottom
        let inner_bottom_idx = builder.start_surface(Some(SemanticType::InteriorWallSurface));
        builder.set_surface_outer_ring(&[v8, v11, v10, v9])?;
        builder.add_shell_outer_surface(inner_bottom_idx)?;

        // Top
        let inner_top_idx = builder.start_surface(Some(SemanticType::InteriorWallSurface));
        builder.set_surface_outer_ring(&[v12, v13, v14, v15])?;
        builder.add_shell_outer_surface(inner_top_idx)?;

        // Four sides
        let inner_sides = [
            &[v8, v9, v13, v12],   // Front
            &[v9, v10, v14, v13],  // Right
            &[v10, v11, v15, v14], // Back
            &[v11, v8, v12, v15],  // Left
        ];

        for vertices in inner_sides {
            let inner_side_idx = builder.start_surface(Some(SemanticType::InteriorWallSurface));
            builder.set_surface_outer_ring(vertices)?;
            builder.add_shell_outer_surface(inner_side_idx)?;
        }

        // Add the inner shell to the solid
        builder.add_solid_inner_shell(inner_shell_idx)?;

        // Add vertices for a tetrahedron (second solid)
        let t0 = builder.add_vertex(4.0, 0.0, 0.0); // base point 1
        let t1 = builder.add_vertex(5.0, 0.0, 0.0); // base point 2
        let t2 = builder.add_vertex(4.5, 1.0, 0.0); // base point 3
        let t3 = builder.add_vertex(4.5, 0.5, 1.0); // apex

        // Start building the second solid (tetrahedron)
        let _second_solid_idx = builder.start_solid();
        let tetra_shell_idx = builder.start_shell();

        // Create the four triangular faces of the tetrahedron
        // Base face
        let tetra_base_idx = builder.start_surface(Some(SemanticType::GroundSurface));
        builder.set_surface_outer_ring(&[t0, t1, t2])?;
        builder.add_shell_outer_surface(tetra_base_idx)?;

        // Three side faces
        let side_vertices = [
            &[t0, t2, t3], // First side
            &[t1, t3, t2], // Second side
            &[t0, t3, t1], // Third side
        ];

        for vertices in side_vertices {
            let side_idx = builder.start_surface(Some(SemanticType::WallSurface));
            builder.set_surface_outer_ring(vertices)?;
            builder.add_shell_outer_surface(side_idx)?;
        }

        // Set the outer shell for the tetrahedron
        builder.set_solid_outer_shell(tetra_shell_idx)?;

        // Build the geometry
        builder.build()?;

        // Get the boundary from the built geometry
        let geometry = model.geometries.first().unwrap();
        let boundary = geometry.boundaries.as_ref().unwrap();

        // Convert to nested representation and compare
        let nested = boundary.to_nested_multi_or_composite_solid()?;

        // Expected nested structure:
        let expected: BoundaryNestedMultiOrCompositeSolid32 = vec![
            // First solid (outer cube with windows and inner cube)
            vec![
                // Outer shell
                vec![
                    // Bottom face (ground)
                    vec![vec![0, 3, 2, 1]],
                    // Top face (roof)
                    vec![vec![4, 5, 6, 7]],
                    // Front face with a window
                    vec![
                        vec![0, 1, 5, 4],     // Outer ring
                        vec![16, 17, 18, 19], // Window hole
                    ],
                    // Right face with a window
                    vec![
                        vec![1, 2, 6, 5],     // Outer ring
                        vec![20, 21, 22, 23], // Window hole
                    ],
                    // Back face
                    vec![vec![2, 3, 7, 6]],
                    // Left face
                    vec![vec![3, 0, 4, 7]],
                ],
                // Inner shell (void cube)
                vec![
                    // Bottom face
                    vec![vec![8, 11, 10, 9]],
                    // Top face
                    vec![vec![12, 13, 14, 15]],
                    // Front face
                    vec![vec![8, 9, 13, 12]],
                    // Right face
                    vec![vec![9, 10, 14, 13]],
                    // Back face
                    vec![vec![10, 11, 15, 14]],
                    // Left face
                    vec![vec![11, 8, 12, 15]],
                ],
            ],
            // Second solid (tetrahedron)
            vec![
                // Outer shell
                vec![
                    // Base face
                    vec![vec![24, 25, 26]],
                    // Three side faces
                    vec![vec![24, 26, 27]],
                    vec![vec![25, 27, 26]],
                    vec![vec![24, 27, 25]],
                ],
            ],
        ];

        assert_eq!(nested, expected);
        Ok(())
    }
}
