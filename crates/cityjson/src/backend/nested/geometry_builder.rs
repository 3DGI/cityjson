//! GeometryBuilder for the nested backend.
//!
//! This builder provides a fluent API for constructing geometries with nested boundary structures.

use crate::Error;
use crate::backend::nested::appearance::{MaterialValues, TextureValues};
use crate::backend::nested::boundary::*;
use crate::backend::nested::citymodel::CityModel;
use crate::backend::nested::geometry::Geometry;
use crate::backend::nested::semantics::{Semantic, SemanticValues, Semantics};
use crate::prelude::{
    GeometryType, LoD, QuantizedCoordinate, RealWorldCoordinate, StringStorage, UVCoordinate,
    VertexIndex, VertexIndex32,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderMode {
    Regular,
    Template,
}

/// Represents a vertex that can be either an index or a direct coordinate.
#[derive(Debug, Clone)]
enum VertexOrPoint {
    Index(VertexIndex32),
    Point(QuantizedCoordinate),
}

#[derive(Debug, Clone)]
enum TemplateVertexOrPoint {
    Index(VertexIndex32),
    Point(RealWorldCoordinate),
}

/// Tracks a surface being constructed.
#[derive(Debug, Clone)]
struct SurfaceInProgress {
    outer_ring: Option<usize>,
    inner_rings: Vec<usize>,
}

/// Tracks a solid being constructed.
#[derive(Debug, Clone)]
struct SolidInProgress {
    outer_shell: Option<usize>,
    inner_shells: Vec<usize>,
}

pub struct GeometryBuilder<'a, SS: StringStorage> {
    model: &'a mut CityModel<SS>,
    type_geometry: GeometryType,
    builder_mode: BuilderMode,
    lod: Option<LoD>,
    template_geometry: Option<usize>,
    transformation_matrix: Option<[f64; 16]>,

    // Vertex tracking
    vertices: Vec<VertexOrPoint>,
    template_vertices: Vec<TemplateVertexOrPoint>,

    // Boundary construction
    rings: Vec<Vec<usize>>,           // indices into vertices
    surfaces: Vec<SurfaceInProgress>, // surfaces with their rings
    shells: Vec<Vec<usize>>,          // shells with their surfaces
    solids: Vec<SolidInProgress>,     // solids with their shells

    // Active element tracking
    active_surface: Option<usize>,
    active_solid: Option<usize>,

    // Semantic storage
    point_semantics: HashMap<usize, Semantic<SS>>,
    linestring_semantics: HashMap<usize, Semantic<SS>>,
    surface_semantics: HashMap<usize, Semantic<SS>>,

    // Material storage: theme -> [(surface_idx, material_idx)]
    surface_materials: Vec<(String, Vec<(usize, usize)>)>,

    // Texture storage: theme -> [(ring_idx, texture_idx)]
    ring_textures: Vec<(String, Vec<(usize, usize)>)>,

    // UV coordinates
    uv_coordinates: Vec<UVCoordinate>,
    vertex_uv_mapping: HashMap<usize, usize>,
}

impl<'a, SS: StringStorage> GeometryBuilder<'a, SS> {
    // ========== Constructor ==========

    pub fn new(
        model: &'a mut CityModel<SS>,
        type_geometry: GeometryType,
        builder_mode: BuilderMode,
    ) -> Self {
        Self {
            model,
            type_geometry,
            builder_mode,
            lod: None,
            template_geometry: None,
            transformation_matrix: None,
            vertices: Vec::new(),
            template_vertices: Vec::new(),
            rings: Vec::new(),
            surfaces: Vec::new(),
            shells: Vec::new(),
            solids: Vec::new(),
            active_surface: None,
            active_solid: None,
            point_semantics: HashMap::new(),
            linestring_semantics: HashMap::new(),
            surface_semantics: HashMap::new(),
            surface_materials: Vec::new(),
            ring_textures: Vec::new(),
            uv_coordinates: Vec::new(),
            vertex_uv_mapping: HashMap::new(),
        }
    }

    // ========== Configuration ==========

    pub fn with_lod(mut self, lod: LoD) -> Self {
        self.lod = Some(lod);
        self
    }

    pub fn with_template(mut self, template_idx: usize) -> Result<Self, Error> {
        self.template_geometry = Some(template_idx);
        Ok(self)
    }

    pub fn with_transformation_matrix(mut self, matrix: [f64; 16]) -> Self {
        self.transformation_matrix = Some(matrix);
        self
    }

    // ========== Vertex Operations ==========

    pub fn add_vertex(&mut self, index: VertexIndex<u32>) -> Result<&mut Self, Error> {
        self.vertices.push(VertexOrPoint::Index(index.into()));
        Ok(self)
    }

    pub fn add_point(&mut self, coordinate: QuantizedCoordinate) -> Result<&mut Self, Error> {
        self.vertices.push(VertexOrPoint::Point(coordinate));
        Ok(self)
    }

    pub fn add_template_vertex(&mut self, index: VertexIndex<u32>) -> Result<&mut Self, Error> {
        self.template_vertices
            .push(TemplateVertexOrPoint::Index(index.into()));
        Ok(self)
    }

    pub fn add_template_point(
        &mut self,
        coordinate: RealWorldCoordinate,
    ) -> Result<&mut Self, Error> {
        self.template_vertices
            .push(TemplateVertexOrPoint::Point(coordinate));
        Ok(self)
    }

    // ========== Ring Operations ==========

    pub fn add_ring(&mut self, vertex_indices: &[usize]) -> Result<usize, Error> {
        if vertex_indices.is_empty() {
            return Err(Error::InvalidGeometry("Ring cannot be empty".to_string()));
        }
        let ring_idx = self.rings.len();
        self.rings.push(vertex_indices.to_vec());
        Ok(ring_idx)
    }

    pub fn start_ring(&mut self) -> Result<&mut Self, Error> {
        // Ring is implicitly started when adding vertices
        Ok(self)
    }

    pub fn end_ring(&mut self) -> Result<usize, Error> {
        // Collect all current vertices as a ring
        let vertex_indices: Vec<usize> = (0..self.vertices.len()).collect();
        let ring_idx = self.add_ring(&vertex_indices)?;
        Ok(ring_idx)
    }

    // ========== Surface Operations ==========

    pub fn start_surface(&mut self) -> Result<&mut Self, Error> {
        let surface_idx = self.surfaces.len();
        self.surfaces.push(SurfaceInProgress {
            outer_ring: None,
            inner_rings: Vec::new(),
        });
        self.active_surface = Some(surface_idx);
        Ok(self)
    }

    pub fn add_surface_outer_ring(&mut self, ring_idx: usize) -> Result<&mut Self, Error> {
        let surface_idx = self
            .active_surface
            .ok_or_else(|| Error::InvalidGeometry("No active surface".to_string()))?;

        if ring_idx >= self.rings.len() {
            return Err(Error::InvalidGeometry(format!(
                "Ring index {} out of bounds",
                ring_idx
            )));
        }

        self.surfaces[surface_idx].outer_ring = Some(ring_idx);
        Ok(self)
    }

    pub fn add_surface_inner_ring(&mut self, ring_idx: usize) -> Result<&mut Self, Error> {
        let surface_idx = self
            .active_surface
            .ok_or_else(|| Error::InvalidGeometry("No active surface".to_string()))?;

        if ring_idx >= self.rings.len() {
            return Err(Error::InvalidGeometry(format!(
                "Ring index {} out of bounds",
                ring_idx
            )));
        }

        self.surfaces[surface_idx].inner_rings.push(ring_idx);
        Ok(self)
    }

    pub fn end_surface(&mut self) -> Result<usize, Error> {
        let surface_idx = self
            .active_surface
            .ok_or_else(|| Error::InvalidGeometry("No active surface".to_string()))?;

        if self.surfaces[surface_idx].outer_ring.is_none() {
            return Err(Error::InvalidGeometry(
                "Surface must have an outer ring".to_string(),
            ));
        }

        self.active_surface = None;
        Ok(surface_idx)
    }

    // ========== Shell Operations ==========

    pub fn start_shell(&mut self) -> Result<&mut Self, Error> {
        self.shells.push(Vec::new());
        Ok(self)
    }

    pub fn add_shell_surface(&mut self, surface_idx: usize) -> Result<&mut Self, Error> {
        if self.shells.is_empty() {
            return Err(Error::InvalidGeometry("No active shell".to_string()));
        }

        if surface_idx >= self.surfaces.len() {
            return Err(Error::InvalidGeometry(format!(
                "Surface index {} out of bounds",
                surface_idx
            )));
        }

        let shell_idx = self.shells.len() - 1;
        self.shells[shell_idx].push(surface_idx);
        Ok(self)
    }

    pub fn end_shell(&mut self) -> Result<usize, Error> {
        if self.shells.is_empty() {
            return Err(Error::InvalidGeometry("No active shell".to_string()));
        }

        let shell_idx = self.shells.len() - 1;
        if self.shells[shell_idx].is_empty() {
            return Err(Error::InvalidGeometry(
                "Shell must have at least one surface".to_string(),
            ));
        }

        Ok(shell_idx)
    }

    // ========== Solid Operations ==========

    pub fn start_solid(&mut self) -> Result<&mut Self, Error> {
        let solid_idx = self.solids.len();
        self.solids.push(SolidInProgress {
            outer_shell: None,
            inner_shells: Vec::new(),
        });
        self.active_solid = Some(solid_idx);
        Ok(self)
    }

    pub fn add_solid_outer_shell(&mut self, shell_idx: usize) -> Result<&mut Self, Error> {
        let solid_idx = self
            .active_solid
            .ok_or_else(|| Error::InvalidGeometry("No active solid".to_string()))?;

        if shell_idx >= self.shells.len() {
            return Err(Error::InvalidGeometry(format!(
                "Shell index {} out of bounds",
                shell_idx
            )));
        }

        self.solids[solid_idx].outer_shell = Some(shell_idx);
        Ok(self)
    }

    pub fn add_solid_inner_shell(&mut self, shell_idx: usize) -> Result<&mut Self, Error> {
        let solid_idx = self
            .active_solid
            .ok_or_else(|| Error::InvalidGeometry("No active solid".to_string()))?;

        if shell_idx >= self.shells.len() {
            return Err(Error::InvalidGeometry(format!(
                "Shell index {} out of bounds",
                shell_idx
            )));
        }

        self.solids[solid_idx].inner_shells.push(shell_idx);
        Ok(self)
    }

    pub fn end_solid(&mut self) -> Result<usize, Error> {
        let solid_idx = self
            .active_solid
            .ok_or_else(|| Error::InvalidGeometry("No active solid".to_string()))?;

        if self.solids[solid_idx].outer_shell.is_none() {
            return Err(Error::InvalidGeometry(
                "Solid must have an outer shell".to_string(),
            ));
        }

        self.active_solid = None;
        Ok(solid_idx)
    }

    // ========== Semantics Operations ==========

    pub fn set_semantic_point(
        &mut self,
        point_idx: usize,
        semantic: Semantic<SS>,
    ) -> Result<&mut Self, Error> {
        self.point_semantics.insert(point_idx, semantic);
        Ok(self)
    }

    pub fn set_semantic_linestring(
        &mut self,
        linestring_idx: usize,
        semantic: Semantic<SS>,
    ) -> Result<&mut Self, Error> {
        self.linestring_semantics.insert(linestring_idx, semantic);
        Ok(self)
    }

    pub fn set_semantic_surface(
        &mut self,
        surface_idx: usize,
        semantic: Semantic<SS>,
        _is_roof: bool,
    ) -> Result<&mut Self, Error> {
        self.surface_semantics.insert(surface_idx, semantic);
        Ok(self)
    }

    // ========== Material Operations ==========

    pub fn set_material_surface(
        &mut self,
        theme: String,
        surface_idx: usize,
        material_idx: usize,
    ) -> Result<&mut Self, Error> {
        // Find or create theme entry
        if let Some((_theme, mappings)) =
            self.surface_materials.iter_mut().find(|(t, _)| t == &theme)
        {
            mappings.push((surface_idx, material_idx));
        } else {
            self.surface_materials
                .push((theme, vec![(surface_idx, material_idx)]));
        }
        Ok(self)
    }

    // ========== Texture Operations ==========

    pub fn set_texture_ring(
        &mut self,
        theme: String,
        ring_idx: usize,
        texture_idx: usize,
    ) -> Result<&mut Self, Error> {
        // Find or create theme entry
        if let Some((_theme, mappings)) = self.ring_textures.iter_mut().find(|(t, _)| t == &theme) {
            mappings.push((ring_idx, texture_idx));
        } else {
            self.ring_textures
                .push((theme, vec![(ring_idx, texture_idx)]));
        }
        Ok(self)
    }

    pub fn add_uv_to_vertex(
        &mut self,
        vertex_idx: usize,
        uv: UVCoordinate,
    ) -> Result<&mut Self, Error> {
        let uv_idx = self.uv_coordinates.len();
        self.uv_coordinates.push(uv);
        self.vertex_uv_mapping.insert(vertex_idx, uv_idx);
        Ok(self)
    }

    // ========== Helper Methods ==========

    fn resolve_vertex(&self, v: &VertexOrPoint) -> Result<VertexIndex32, Error> {
        match v {
            VertexOrPoint::Index(idx) => Ok(*idx),
            VertexOrPoint::Point(_) => Err(Error::InvalidGeometry(
                "Cannot resolve point coordinate without model context".to_string(),
            )),
        }
    }

    fn build_ring(&self, ring_idx: usize) -> Result<Vec<VertexIndex32>, Error> {
        let ring = &self.rings[ring_idx];
        ring.iter()
            .map(|&idx| {
                if idx < self.vertices.len() {
                    self.resolve_vertex(&self.vertices[idx])
                } else {
                    Err(Error::InvalidGeometry(format!(
                        "Vertex index {} out of bounds",
                        idx
                    )))
                }
            })
            .collect()
    }

    fn build_surface(&self, surface_idx: usize) -> Result<Vec<Vec<VertexIndex32>>, Error> {
        let surface = &self.surfaces[surface_idx];
        let mut result = Vec::new();

        // Outer ring
        if let Some(outer_idx) = surface.outer_ring {
            result.push(self.build_ring(outer_idx)?);
        }

        // Inner rings
        for &inner_idx in &surface.inner_rings {
            result.push(self.build_ring(inner_idx)?);
        }

        Ok(result)
    }

    fn build_shell(&self, shell_idx: usize) -> Result<Vec<Vec<Vec<VertexIndex32>>>, Error> {
        self.shells[shell_idx]
            .iter()
            .map(|&surface_idx| self.build_surface(surface_idx))
            .collect()
    }

    // ========== Build Method ==========

    pub fn build(self) -> Result<Geometry<SS>, Error> {
        // Build nested boundaries based on geometry type
        let boundaries = match self.type_geometry {
            GeometryType::MultiPoint => {
                let points: Result<Vec<VertexIndex32>, Error> = self
                    .vertices
                    .iter()
                    .map(|v| self.resolve_vertex(v))
                    .collect();
                Some(Boundary::MultiPoint(points?))
            }

            GeometryType::MultiLineString => {
                let linestrings: Result<Vec<Vec<VertexIndex32>>, Error> = self
                    .rings
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| self.build_ring(idx))
                    .collect();
                Some(Boundary::MultiLineString(linestrings?))
            }

            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                let surfaces: Result<Vec<Vec<Vec<VertexIndex32>>>, Error> = self
                    .surfaces
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| self.build_surface(idx))
                    .collect();
                let boundary_surfaces = surfaces?;
                if self.type_geometry == GeometryType::MultiSurface {
                    Some(Boundary::MultiSurface(boundary_surfaces))
                } else {
                    Some(Boundary::CompositeSurface(boundary_surfaces))
                }
            }

            GeometryType::Solid => {
                let shells: Result<Vec<Vec<Vec<Vec<VertexIndex32>>>>, Error> = self
                    .shells
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| self.build_shell(idx))
                    .collect();
                Some(Boundary::Solid(shells?))
            }

            GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                let mut solids_boundaries: Vec<Vec<Vec<Vec<Vec<VertexIndex32>>>>> = Vec::new();

                for solid in &self.solids {
                    let mut solid_shells = Vec::new();

                    // Outer shell
                    if let Some(outer_idx) = solid.outer_shell {
                        solid_shells.push(self.build_shell(outer_idx)?);
                    }

                    // Inner shells
                    for &inner_idx in &solid.inner_shells {
                        solid_shells.push(self.build_shell(inner_idx)?);
                    }

                    solids_boundaries.push(solid_shells);
                }

                if self.type_geometry == GeometryType::MultiSolid {
                    Some(Boundary::MultiSolid(solids_boundaries))
                } else {
                    Some(Boundary::CompositeSolid(solids_boundaries))
                }
            }

            GeometryType::GeometryInstance => None,
        };

        // Build semantics
        let semantics = if !self.surface_semantics.is_empty() {
            Some(self.build_nested_semantics()?)
        } else {
            None
        };

        // Build materials
        let materials = if !self.surface_materials.is_empty() {
            Some(self.build_nested_materials()?)
        } else {
            None
        };

        // Build textures
        let textures = if !self.ring_textures.is_empty() {
            Some(self.build_nested_textures()?)
        } else {
            None
        };

        // Construct geometry
        Ok(Geometry::new(
            self.type_geometry,
            self.lod,
            boundaries,
            semantics,
            materials,
            textures,
            self.template_geometry,
            self.transformation_matrix
                .map(|_| RealWorldCoordinate::new(0.0, 0.0, 0.0)), // TODO: compute reference point
            self.transformation_matrix,
        ))
    }

    fn build_nested_semantics(&self) -> Result<Semantics<SS>, Error> {
        // Collect all unique semantics
        let mut surfaces = Vec::new();
        let mut semantic_index_map: HashMap<String, usize> = HashMap::new();

        for (_, semantic) in &self.surface_semantics {
            let key = format!("{:?}", semantic.type_semantic());
            if !semantic_index_map.contains_key(&key) {
                let idx = surfaces.len();
                surfaces.push(semantic.clone());
                semantic_index_map.insert(key, idx);
            }
        }

        // Build SemanticValues based on geometry type
        let values = match self.type_geometry {
            GeometryType::MultiPoint
            | GeometryType::MultiLineString
            | GeometryType::MultiSurface
            | GeometryType::CompositeSurface => {
                let mut surface_values = Vec::new();
                for i in 0..self.surfaces.len() {
                    let semantic_idx = self.surface_semantics.get(&i).and_then(|s| {
                        let key = format!("{:?}", s.type_semantic());
                        semantic_index_map.get(&key).copied()
                    });
                    surface_values.push(semantic_idx);
                }
                SemanticValues::PointOrLineStringOrSurface(surface_values)
            }

            GeometryType::Solid => {
                let mut solid_values = Vec::new();
                for shell_surfaces in &self.shells {
                    let mut shell_values = Vec::new();
                    for &surface_idx in shell_surfaces {
                        let semantic_idx = self.surface_semantics.get(&surface_idx).and_then(|s| {
                            let key = format!("{:?}", s.type_semantic());
                            semantic_index_map.get(&key).copied()
                        });
                        shell_values.push(semantic_idx);
                    }
                    solid_values.push(shell_values);
                }
                SemanticValues::Solid(solid_values)
            }

            GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                let mut multisolid_values = Vec::new();
                for solid in &self.solids {
                    let mut solid_values = Vec::new();

                    // Process outer shell
                    if let Some(outer_idx) = solid.outer_shell {
                        let mut shell_values = Vec::new();
                        for &surface_idx in &self.shells[outer_idx] {
                            let semantic_idx =
                                self.surface_semantics.get(&surface_idx).and_then(|s| {
                                    let key = format!("{:?}", s.type_semantic());
                                    semantic_index_map.get(&key).copied()
                                });
                            shell_values.push(semantic_idx);
                        }
                        solid_values.push(shell_values);
                    }

                    // Process inner shells
                    for &inner_idx in &solid.inner_shells {
                        let mut shell_values = Vec::new();
                        for &surface_idx in &self.shells[inner_idx] {
                            let semantic_idx =
                                self.surface_semantics.get(&surface_idx).and_then(|s| {
                                    let key = format!("{:?}", s.type_semantic());
                                    semantic_index_map.get(&key).copied()
                                });
                            shell_values.push(semantic_idx);
                        }
                        solid_values.push(shell_values);
                    }

                    multisolid_values.push(solid_values);
                }
                SemanticValues::MultiSolid(multisolid_values)
            }

            GeometryType::GeometryInstance => {
                return Err(Error::InvalidGeometry(
                    "GeometryInstance cannot have semantics".to_string(),
                ));
            }
        };

        Ok(Semantics::new(surfaces, values))
    }

    fn build_nested_materials(&self) -> Result<HashMap<String, MaterialValues>, Error> {
        let mut result = HashMap::new();

        for (theme, mappings) in &self.surface_materials {
            let values = match self.type_geometry {
                GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                    let mut surface_values = Vec::new();
                    for i in 0..self.surfaces.len() {
                        let material_idx = mappings
                            .iter()
                            .find(|(idx, _)| *idx == i)
                            .map(|(_, mat_idx)| *mat_idx);
                        surface_values.push(material_idx);
                    }
                    MaterialValues::PointOrLineStringOrSurface(surface_values)
                }

                GeometryType::Solid => {
                    let mut solid_values = Vec::new();
                    for shell_surfaces in &self.shells {
                        let mut shell_values = Vec::new();
                        for &surface_idx in shell_surfaces {
                            let material_idx = mappings
                                .iter()
                                .find(|(idx, _)| *idx == surface_idx)
                                .map(|(_, mat_idx)| *mat_idx);
                            shell_values.push(material_idx);
                        }
                        solid_values.push(shell_values);
                    }
                    MaterialValues::Solid(solid_values)
                }

                GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                    let mut multisolid_values = Vec::new();
                    for solid in &self.solids {
                        let mut solid_values = Vec::new();

                        if let Some(outer_idx) = solid.outer_shell {
                            let mut shell_values = Vec::new();
                            for &surface_idx in &self.shells[outer_idx] {
                                let material_idx = mappings
                                    .iter()
                                    .find(|(idx, _)| *idx == surface_idx)
                                    .map(|(_, mat_idx)| *mat_idx);
                                shell_values.push(material_idx);
                            }
                            solid_values.push(shell_values);
                        }

                        for &inner_idx in &solid.inner_shells {
                            let mut shell_values = Vec::new();
                            for &surface_idx in &self.shells[inner_idx] {
                                let material_idx = mappings
                                    .iter()
                                    .find(|(idx, _)| *idx == surface_idx)
                                    .map(|(_, mat_idx)| *mat_idx);
                                shell_values.push(material_idx);
                            }
                            solid_values.push(shell_values);
                        }

                        multisolid_values.push(solid_values);
                    }
                    MaterialValues::MultiSolid(multisolid_values)
                }

                _ => {
                    return Err(Error::InvalidGeometry(format!(
                        "Materials not supported for geometry type {:?}",
                        self.type_geometry
                    )));
                }
            };

            result.insert(theme.clone(), values);
        }

        Ok(result)
    }

    fn build_nested_textures(&self) -> Result<HashMap<String, TextureValues>, Error> {
        let mut result = HashMap::new();

        for (theme, mappings) in &self.ring_textures {
            let values = match self.type_geometry {
                GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                    let mut surface_values = Vec::new();
                    for surface in &self.surfaces {
                        let mut surface_ring_values = Vec::new();

                        // Outer ring
                        if let Some(outer_idx) = surface.outer_ring {
                            let texture_idx = mappings
                                .iter()
                                .find(|(idx, _)| *idx == outer_idx)
                                .map(|(_, tex_idx)| *tex_idx);
                            surface_ring_values.push(texture_idx);
                        }

                        // Inner rings
                        for &inner_idx in &surface.inner_rings {
                            let texture_idx = mappings
                                .iter()
                                .find(|(idx, _)| *idx == inner_idx)
                                .map(|(_, tex_idx)| *tex_idx);
                            surface_ring_values.push(texture_idx);
                        }

                        surface_values.push(surface_ring_values);
                    }
                    TextureValues::PointOrLineStringOrSurface(surface_values)
                }

                GeometryType::Solid => {
                    let mut solid_values = Vec::new();
                    for shell_surfaces in &self.shells {
                        let mut shell_values = Vec::new();
                        for &surface_idx in shell_surfaces {
                            let surface = &self.surfaces[surface_idx];
                            let mut surface_ring_values = Vec::new();

                            // Outer ring
                            if let Some(outer_idx) = surface.outer_ring {
                                let texture_idx = mappings
                                    .iter()
                                    .find(|(idx, _)| *idx == outer_idx)
                                    .map(|(_, tex_idx)| *tex_idx);
                                surface_ring_values.push(texture_idx);
                            }

                            // Inner rings
                            for &inner_idx in &surface.inner_rings {
                                let texture_idx = mappings
                                    .iter()
                                    .find(|(idx, _)| *idx == inner_idx)
                                    .map(|(_, tex_idx)| *tex_idx);
                                surface_ring_values.push(texture_idx);
                            }

                            shell_values.push(surface_ring_values);
                        }
                        solid_values.push(shell_values);
                    }
                    TextureValues::Solid(solid_values)
                }

                _ => {
                    return Err(Error::InvalidGeometry(format!(
                        "Textures not supported for geometry type {:?}",
                        self.type_geometry
                    )));
                }
            };

            result.insert(theme.clone(), values);
        }

        Ok(result)
    }
}
