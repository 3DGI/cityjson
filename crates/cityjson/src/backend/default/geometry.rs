//#![doc = include_str!("../../docs/boundary_guide.md")]

use crate::cityjson::core::boundary::BoundaryCounter;
use crate::cityjson::core::vertex::VertexRef;
use crate::cityjson::traits::coordinate::Coordinate;
use crate::error::{Error, Result};
use crate::prelude::{
    Boundary, RealWorldCoordinate, StringStorage, UVCoordinate, VertexIndex, Vertices,
};
use crate::resources::mapping::SemanticOrMaterialMap;
use crate::resources::mapping::textures::TextureMapCore;
use crate::resources::pool::ResourceRef;
use std::collections::HashMap;
use std::str::FromStr;

// Type aliases to simplify complex type signatures
type ThemedMaterialMappings<RR> = Vec<(usize, RR)>;
type ThemedTextureMappings<RR> = Vec<(usize, RR)>;
type ThemedMaterials<VR, RR, SS> = Vec<(SS, SemanticOrMaterialMap<VR, RR>)>;
type ThemedTextures<VR, RR, SS> = Vec<(SS, TextureMapCore<VR, RR>)>;

// Local trait to decouple GeometryBuilder from any global CityModel traits.
pub trait GeometryModelOps<VR, RR, C, Semantic, Material, Texture, Geometry, SS>
where
    VR: VertexRef,
    RR: ResourceRef,
    C: Coordinate,
    SS: StringStorage,
{
    /// Add a semantic resource.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when the semantic pool cannot accept
    /// additional entries.
    fn add_semantic(&mut self, semantic: Semantic) -> Result<RR>;
    /// Return an existing semantic resource reference or insert a new one.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when inserting a new semantic would
    /// exceed pool capacity.
    fn get_or_insert_semantic(&mut self, semantic: Semantic) -> Result<RR>;
    /// Add a material resource.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when the material pool cannot accept
    /// additional entries.
    fn add_material(&mut self, material: Material) -> Result<RR>;
    /// Return an existing material resource reference or insert a new one.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when inserting a new material would
    /// exceed pool capacity.
    fn get_or_insert_material(&mut self, material: Material) -> Result<RR>;
    /// Add a texture resource.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when the texture pool cannot accept
    /// additional entries.
    fn add_texture(&mut self, texture: Texture) -> Result<RR>;
    /// Return an existing texture resource reference or insert a new one.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when inserting a new texture would
    /// exceed pool capacity.
    fn get_or_insert_texture(&mut self, texture: Texture) -> Result<RR>;
    /// Add a UV coordinate vertex.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::VerticesContainerFull`] when the UV vertex container is
    /// at capacity for `VR`.
    fn add_uv_coordinate(&mut self, uvcoordinate: UVCoordinate) -> Result<VertexIndex<VR>>;

    /// Add a regular geometry resource.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when the geometry pool cannot accept
    /// additional entries.
    fn add_geometry(&mut self, geometry: Geometry) -> Result<RR>;
    /// Add a template geometry resource.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when the template geometry pool cannot
    /// accept additional entries.
    fn add_template_geometry(&mut self, geometry: Geometry) -> Result<RR>;

    /// Add a regular coordinate vertex.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::VerticesContainerFull`] when the vertex container is at
    /// capacity for `VR`.
    fn add_vertex(&mut self, coordinate: C) -> Result<VertexIndex<VR>>;
    fn vertices_mut(&mut self) -> &mut Vertices<VR, C>;

    /// Add a template coordinate vertex.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::VerticesContainerFull`] when the template-vertex
    /// container is at capacity for `VR`.
    fn add_template_vertex(&mut self, coordinate: RealWorldCoordinate) -> Result<VertexIndex<VR>>;
    fn template_vertices_mut(&mut self) -> &mut Vertices<VR, RealWorldCoordinate>;
}

// Trait for geometry construction
pub(crate) trait GeometryConstructor<VR: VertexRef, RR: crate::resources::pool::ResourceRef, SS> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        type_geometry: GeometryType,
        lod: Option<LoD>,
        boundaries: Option<Boundary<VR>>,
        semantics: Option<SemanticOrMaterialMap<VR, RR>>,
        materials: Option<Vec<(SS, SemanticOrMaterialMap<VR, RR>)>>,
        textures: Option<Vec<(SS, TextureMapCore<VR, RR>)>>,
        instance_template: Option<RR>,
        instance_reference_point: Option<VertexIndex<VR>>,
        instance_transformation_matrix: Option<[f64; 16]>,
    ) -> Self;
}

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

struct GeometryBuildState<VR: VertexRef, RR: ResourceRef, S> {
    semantic_map_option: Option<SemanticOrMaterialMap<VR, RR>>,
    material_map_option: Option<ThemedMaterials<VR, RR, S>>,
    instance_reference_point: Option<VertexIndex<VR>>,
}

/// Controls the [`GeometryBuilder`] to build a regular geometry or a geometry template.
#[non_exhaustive]
pub enum BuilderMode {
    /// Build a regular geometry
    Regular,
    /// Build a geometry template
    Template,
}

/// Geometry builder.
///
/// The `GeometryBuilder` is generic over the `CityModel` and Coordinate type, thus it can
/// build a `CityModel` with either real-world coordinates or quantized coordinates,
/// for all supported `CityJSON` versions.
pub struct GeometryBuilder<'a, VR, RR, C, Semantic, Material, Texture, Geometry, M, SS>
where
    VR: VertexRef,
    RR: ResourceRef,
    C: Coordinate,
    SS: StringStorage,
{
    model: &'a mut M,
    type_geometry: GeometryType,
    builder_mode: BuilderMode,
    lod: Option<LoD>,
    template_geometry: Option<RR>,
    transformation_matrix: Option<[f64; 16]>,
    template_vertices: Vec<TemplateVertexOrPoint<VR>>,
    vertices: Vec<VertexOrPoint<VR, C>>,
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
    point_semantics: HashMap<usize, RR>,
    linestring_semantics: HashMap<usize, RR>,
    surface_semantics: HashMap<usize, RR>,
    // Material storage with themes as [(theme, [(surface idx, material ref)])]
    surface_materials: Vec<(SS::String, ThemedMaterialMappings<RR>)>,
    // Maps ring index to texture reference
    ring_textures: Vec<(SS::String, ThemedTextureMappings<RR>)>,
    _phantom_semantic: std::marker::PhantomData<Semantic>,
    _phantom_material: std::marker::PhantomData<Material>,
    _phantom_texture: std::marker::PhantomData<Texture>,
    _phantom_geometry: std::marker::PhantomData<Geometry>,
}

impl<'a, VR, RR, C, Semantic, Material, Texture, Geometry, M, SS>
    GeometryBuilder<'a, VR, RR, C, Semantic, Material, Texture, Geometry, M, SS>
where
    VR: VertexRef,
    RR: ResourceRef,
    C: Coordinate,
    SS: StringStorage,
{
    /// Instantiates a new `GeometryBuilder`.
    ///
    /// # Parameters
    /// * `model` - A `CityModel` instance.
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
            vertex_uv_mapping: HashMap::default(),
            rings: Vec::new(),
            surfaces: Vec::new(),
            shells: Vec::new(),
            solids: Vec::new(),
            active_surface: None,
            active_solid: None,
            point_semantics: HashMap::default(),
            linestring_semantics: HashMap::default(),
            surface_semantics: HashMap::default(),
            surface_materials: Vec::default(),
            ring_textures: Vec::default(),
            _phantom_semantic: std::marker::PhantomData,
            _phantom_material: std::marker::PhantomData,
            _phantom_texture: std::marker::PhantomData,
            _phantom_geometry: std::marker::PhantomData,
        }
    }

    /// Set the Level of Detail on the Geometry.
    #[must_use]
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
    /// * [`Error::InvalidGeometryType`] if geometry is not a `GeometryInstance`.
    pub fn with_template(mut self, template_ref: RR) -> Result<Self> {
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
    /// Self-for method chaining
    ///
    /// # Errors
    /// * [`Error::InvalidGeometryType`] if geometry is not a `GeometryInstance`.
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

    #[must_use]
    pub fn with_reference_point(mut self, point: C) -> Self {
        self.add_point(point);
        self
    }

    #[must_use]
    pub fn with_reference_vertex(mut self, vertex: VertexIndex<VR>) -> Self {
        self.add_vertex(vertex);
        self
    }

    /// Add a new point to the boundary by providing its coordinates. The point will be
    /// added as a new vertex to the vertex pool. Use this method when adding completely
    /// new vertices to the `CityModel` and the Boundary. Can be used interchangeably
    /// with [`GeometryBuilder::add_vertex`] for building a Boundary.
    ///
    /// # Returns
    ///
    /// The index of the added vertex in the boundary.
    pub fn add_point(&mut self, point: C) -> usize {
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
    /// used interchangeably with [`GeometryBuilder::add_point`] for building a Boundary.
    ///
    /// # Returns
    ///
    /// The index of the added vertex in the boundary.
    pub fn add_vertex(&mut self, vertex: VertexIndex<VR>) -> usize {
        self.vertices.push(VertexOrPoint::Vertex(vertex));
        self.vertices.len().saturating_sub(1)
    }

    pub fn add_template_vertex(&mut self, vertex: VertexIndex<VR>) -> usize {
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
    /// - `vertex_idx`: Index of the target vertex, as returned from [`GeometryBuilder::add_point`] or
    ///   [`GeometryBuilder::add_vertex`].
    /// - `uv_idx`: Index of the corresponding UV coordinate, as returned by
    ///   [`GeometryBuilder::add_uv_coordinate`].
    pub fn map_vertex_to_uv(&mut self, vertex_idx: usize, uv_idx: usize) {
        self.vertex_uv_mapping.insert(vertex_idx, uv_idx);
    }

    /// Add a `LineString` to the boundary by providing its vertex indices in the boundary.
    /// The indices are returned by the [`GeometryBuilder::add_point`] or [`GeometryBuilder::add_vertex`] methods.
    ///
    /// # Errors
    /// * `InvalidLineString` - If less than two vertices have been provided
    ///
    /// # Returns
    ///
    /// The index of the added `LineString` in the boundary.
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
    /// The indices are returned by the [`GeometryBuilder::add_point`] or [`GeometryBuilder::add_vertex`] methods.
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
    ///   value returned by the [`GeometryBuilder::add_point`] or [`GeometryBuilder::add_vertex`] methods. If
    ///   `None`, the Semantic is added to the last vertex in the `GeometryBuilder`.
    /// * `semantic` - The semantic instance to add to the Point.
    ///
    /// # Returns
    ///
    /// The reference to the Semantic in the resource pool of the `model`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ResourcePoolFull`] when inserting a new semantic exceeds pool capacity.
    /// Returns [`Error::InvalidReference`] when `index` is out of range.
    pub fn set_semantic_point(
        &mut self,
        index: Option<usize>,
        semantic: Semantic,
        deduplicate: bool,
    ) -> Result<RR>
    where
        M: GeometryModelOps<VR, RR, C, Semantic, Material, Texture, Geometry, SS>,
    {
        let semantic_ref = if deduplicate {
            self.model.get_or_insert_semantic(semantic)
        } else {
            self.model.add_semantic(semantic)
        }?;
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

    /// Set the Semantic on a `LineString`.
    /// A `LineString` can only have one semantic value. The Semantic is directly added to the
    /// `model`.
    ///
    /// # Parameters
    ///
    /// * `index` - The index of the `LineString` that will get the semantic. The index is the
    ///   value returned by the [`GeometryBuilder::add_linestring`] or [`GeometryBuilder::add_ring`] methods. If
    ///   `None`, the Semantic is added to the last `LineString` in the `GeometryBuilder`.
    /// * `semantic` - The semantic instance to add to the `LineString`.
    ///
    /// # Returns
    ///
    /// The reference to the Semantic in the resource pool of the `model`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ResourcePoolFull`] when inserting a new semantic exceeds pool capacity.
    /// Returns [`Error::InvalidReference`] when `index` is out of range.
    pub fn set_semantic_linestring(
        &mut self,
        index: Option<usize>,
        semantic: Semantic,
        deduplicate: bool,
    ) -> Result<RR>
    where
        M: GeometryModelOps<VR, RR, C, Semantic, Material, Texture, Geometry, SS>,
    {
        let semantic_ref = if deduplicate {
            self.model.get_or_insert_semantic(semantic)
        } else {
            self.model.add_semantic(semantic)
        }?;
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
    /// * `index` - The index of the surface that will get the semantic. If
    ///   `None`, the Semantic is added to the last surface in the `GeometryBuilder`.
    /// * `semantic` - The Semantic instance to add to the surface.
    ///
    /// # Returns
    ///
    /// The reference to the Semantic in the resource pool of the `model`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ResourcePoolFull`] when inserting a new semantic exceeds pool capacity.
    /// Returns [`Error::InvalidReference`] when `index` is out of range.
    pub fn set_semantic_surface(
        &mut self,
        index: Option<usize>,
        semantic: Semantic,
        deduplicate: bool,
    ) -> Result<RR>
    where
        M: GeometryModelOps<VR, RR, C, Semantic, Material, Texture, Geometry, SS>,
    {
        let semantic_ref = if deduplicate {
            self.model.get_or_insert_semantic(semantic)
        } else {
            self.model.add_semantic(semantic)
        }?;
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
    /// * `index` - The index of the surface that will get the material. If
    ///   `None`, the Material is added to the last surface in the `GeometryBuilder`.
    /// * `material` - The Material instance to add to the surface.
    /// * `theme` - The theme of the material.
    ///
    /// # Returns
    ///
    /// The reference to the Material in the resource pool of the `model`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ResourcePoolFull`] when inserting a new material exceeds pool capacity.
    /// Returns [`Error::InvalidReference`] when `index` is out of range.
    pub fn set_material_surface(
        &mut self,
        index: Option<usize>,
        material: Material,
        theme: SS::String,
        deduplicate: bool,
    ) -> Result<RR>
    where
        M: GeometryModelOps<VR, RR, C, Semantic, Material, Texture, Geometry, SS>,
    {
        let material_ref = if deduplicate {
            self.model.get_or_insert_material(material)
        } else {
            self.model.add_material(material)
        }?;
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
    ///   value returned by the [`GeometryBuilder::add_ring`] method. If
    ///   `None`, the Texture is added to the last ring in the `GeometryBuilder`.
    /// * `texture` - The Texture instance to add to the ring.
    /// * `theme` - The theme of the texture.
    ///
    /// # Returns
    ///
    /// The reference to the Texture in the resource pool of the `model`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ResourcePoolFull`] when inserting a new texture exceeds pool capacity.
    /// Returns [`Error::InvalidReference`] when `index` is out of range.
    pub fn set_texture_ring(
        &mut self,
        index: Option<usize>,
        texture: Texture,
        theme: SS::String,
        deduplicate: bool,
    ) -> Result<RR>
    where
        M: GeometryModelOps<VR, RR, C, Semantic, Material, Texture, Geometry, SS>,
    {
        let texture_ref = if deduplicate {
            self.model.get_or_insert_texture(texture)
        } else {
            self.model.add_texture(texture)
        }?;
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
    #[allow(private_bounds)]
    pub fn build(self) -> Result<RR>
    where
        M: GeometryModelOps<VR, RR, C, Semantic, Material, Texture, Geometry, SS>,
        Geometry: GeometryConstructor<VR, RR, SS::String>,
    {
        let mut builder = self;
        builder.validate_structure()?;

        let (vertices_capacity, rings_capacity, surfaces_capacity) = builder.boundary_capacities();
        let mut boundary = Boundary::with_capacity(
            vertices_capacity,
            rings_capacity,
            surfaces_capacity,
            builder.shells.len(),
            builder.solids.len(),
        );

        builder.reserve_new_vertices()?;
        let nr_builder_vertices = builder.vertices.len();
        let vertex_indices = builder.resolve_vertex_indices()?;

        let build_state =
            builder.populate_boundary_and_maps(&mut boundary, &vertex_indices, nr_builder_vertices);
        let texture_map_option = builder.build_texture_maps(&boundary)?;
        let boundary_option =
            (builder.type_geometry != GeometryType::GeometryInstance).then_some(boundary);

        let geometry = Geometry::new(
            builder.type_geometry,
            builder.lod,
            boundary_option,
            build_state.semantic_map_option,
            build_state.material_map_option,
            texture_map_option,
            builder.template_geometry,
            build_state.instance_reference_point,
            builder.transformation_matrix,
        );

        match builder.builder_mode {
            BuilderMode::Regular => builder.model.add_geometry(geometry),
            BuilderMode::Template => builder.model.add_template_geometry(geometry),
        }
    }

    fn boundary_capacities(&self) -> (usize, usize, usize) {
        match self.type_geometry {
            GeometryType::MultiPoint => (self.vertices.len(), 0, 0),
            GeometryType::MultiLineString => (
                self.rings.iter().map(std::vec::Vec::len).sum(),
                self.rings.len(),
                0,
            ),
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                let rings_capacity = self
                    .surfaces
                    .iter()
                    .map(|surface| surface.outer_ring.map_or(0, |_| 1) + surface.inner_rings.len())
                    .sum();
                let vertices_capacity = self.rings.iter().map(std::vec::Vec::len).sum();
                (vertices_capacity, rings_capacity, self.surfaces.len())
            }
            GeometryType::GeometryInstance => (1, 0, 0),
            _ => (0, 0, 0),
        }
    }

    fn reserve_new_vertices(&mut self) -> Result<()>
    where
        M: GeometryModelOps<VR, RR, C, Semantic, Material, Texture, Geometry, SS>,
    {
        let cnt_new_vertices = self
            .vertices
            .iter()
            .filter(|vertex| matches!(vertex, VertexOrPoint::Point(_)))
            .count();
        if cnt_new_vertices > 0 {
            self.model.vertices_mut().reserve(cnt_new_vertices)?;
        }

        let cnt_new_template_vertices = self
            .template_vertices
            .iter()
            .filter(|vertex| matches!(vertex, TemplateVertexOrPoint::Point(_)))
            .count();
        if cnt_new_template_vertices > 0 {
            self.model
                .template_vertices_mut()
                .reserve(cnt_new_template_vertices)?;
        }

        Ok(())
    }

    fn resolve_vertex_indices(&mut self) -> Result<Vec<VertexIndex<VR>>>
    where
        M: GeometryModelOps<VR, RR, C, Semantic, Material, Texture, Geometry, SS>,
    {
        match self.builder_mode {
            BuilderMode::Regular => std::mem::take(&mut self.vertices)
                .into_iter()
                .map(|vertex| match vertex {
                    VertexOrPoint::Vertex(idx) => Ok(idx),
                    VertexOrPoint::Point(point) => self.model.add_vertex(point),
                })
                .collect(),
            BuilderMode::Template => std::mem::take(&mut self.template_vertices)
                .into_iter()
                .map(|vertex| match vertex {
                    TemplateVertexOrPoint::Vertex(idx) => Ok(idx),
                    TemplateVertexOrPoint::Point(point) => self.model.add_template_vertex(point),
                })
                .collect(),
        }
    }

    fn append_ring_vertices(
        &self,
        ring_idx: usize,
        boundary: &mut Boundary<VR>,
        vertex_indices: &[VertexIndex<VR>],
        counter: &mut BoundaryCounter<VR>,
    ) {
        boundary.rings.push(counter.vertex_offset());
        for &vertex_idx in &self.rings[ring_idx] {
            boundary.vertices.push(vertex_indices[vertex_idx]);
            counter.increment_vertex_idx();
        }
        counter.increment_ring_idx();
    }

    fn append_multisurface_surface(
        &self,
        surface: &SurfaceInProgress,
        boundary: &mut Boundary<VR>,
        vertex_indices: &[VertexIndex<VR>],
        counter: &mut BoundaryCounter<VR>,
    ) {
        if let Some(outer_ring_idx) = surface.outer_ring {
            boundary.surfaces.push(counter.ring_offset());
            self.append_ring_vertices(outer_ring_idx, boundary, vertex_indices, counter);
            for &inner_ring_idx in &surface.inner_rings {
                self.append_ring_vertices(inner_ring_idx, boundary, vertex_indices, counter);
            }
        }
    }

    fn append_surface_from_shell(
        &self,
        surface_idx: usize,
        boundary: &mut Boundary<VR>,
        vertex_indices: &[VertexIndex<VR>],
        counter: &mut BoundaryCounter<VR>,
    ) {
        if surface_idx < self.surfaces.len() {
            boundary.surfaces.push(counter.ring_offset());
            if let Some(outer_ring_idx) = self.surfaces[surface_idx].outer_ring {
                self.append_ring_vertices(outer_ring_idx, boundary, vertex_indices, counter);
                for &inner_ring_idx in &self.surfaces[surface_idx].inner_rings {
                    self.append_ring_vertices(inner_ring_idx, boundary, vertex_indices, counter);
                }
            }
            counter.increment_surface_idx();
        }
    }

    fn append_shell_surfaces(
        &self,
        shell_idx: usize,
        boundary: &mut Boundary<VR>,
        vertex_indices: &[VertexIndex<VR>],
        counter: &mut BoundaryCounter<VR>,
    ) {
        if shell_idx < self.shells.len() {
            for &surface_idx in &self.shells[shell_idx] {
                self.append_surface_from_shell(surface_idx, boundary, vertex_indices, counter);
            }
        }
    }

    fn append_multisolid_shells(
        &self,
        boundary: &mut Boundary<VR>,
        vertex_indices: &[VertexIndex<VR>],
        counter: &mut BoundaryCounter<VR>,
    ) {
        for solid in &self.solids {
            if let Some(outer_shell_idx) = solid.outer_shell {
                boundary.solids.push(counter.shell_offset());
                if outer_shell_idx < self.shells.len() {
                    boundary.shells.push(counter.surface_offset());
                    self.append_shell_surfaces(outer_shell_idx, boundary, vertex_indices, counter);
                    counter.increment_shell_idx();
                }

                for &inner_shell_idx in &solid.inner_shells {
                    if inner_shell_idx < self.shells.len() {
                        boundary.shells.push(counter.surface_offset());
                        self.append_shell_surfaces(
                            inner_shell_idx,
                            boundary,
                            vertex_indices,
                            counter,
                        );
                        counter.increment_shell_idx();
                    }
                }
                counter.increment_solid_idx();
            }
        }
    }

    fn populate_boundary_and_maps(
        &self,
        boundary: &mut Boundary<VR>,
        vertex_indices: &[VertexIndex<VR>],
        nr_builder_vertices: usize,
    ) -> GeometryBuildState<VR, RR, SS::String> {
        let mut counter = BoundaryCounter::<VR>::default();
        let mut semantic_map_option = None;
        let mut material_map_option = None;
        let mut instance_reference_point = None;

        match self.type_geometry {
            GeometryType::GeometryInstance => {
                instance_reference_point = Some(vertex_indices[0]);
            }
            GeometryType::MultiPoint => {
                boundary.vertices = vertex_indices.to_vec();
                semantic_map_option = build_semantic_map::<VR, RR>(
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
                    counter.increment_ring_idx();
                }
                semantic_map_option = build_semantic_map::<VR, RR>(
                    &self.type_geometry,
                    &self.linestring_semantics,
                    self.rings.len(),
                );
            }
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                for surface in &self.surfaces {
                    self.append_multisurface_surface(
                        surface,
                        boundary,
                        vertex_indices,
                        &mut counter,
                    );
                }
                semantic_map_option = build_semantic_map::<VR, RR>(
                    &self.type_geometry,
                    &self.surface_semantics,
                    self.surfaces.len(),
                );
                material_map_option =
                    build_material_map::<VR, RR, SS>(&self.surface_materials, &self.surfaces);
            }
            GeometryType::Solid => {
                boundary.shells.push(counter.surface_offset());
                self.append_shell_surfaces(0, boundary, vertex_indices, &mut counter);
                semantic_map_option = build_semantic_map::<VR, RR>(
                    &self.type_geometry,
                    &self.surface_semantics,
                    self.surfaces.len(),
                );
                material_map_option =
                    build_material_map::<VR, RR, SS>(&self.surface_materials, &self.surfaces);
            }
            GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                self.append_multisolid_shells(boundary, vertex_indices, &mut counter);
                semantic_map_option = build_semantic_map::<VR, RR>(
                    &self.type_geometry,
                    &self.surface_semantics,
                    self.surfaces.len(),
                );
                material_map_option =
                    build_material_map::<VR, RR, SS>(&self.surface_materials, &self.surfaces);
            }
        }

        GeometryBuildState {
            semantic_map_option,
            material_map_option,
            instance_reference_point,
        }
    }

    fn build_texture_maps(
        &mut self,
        boundary: &Boundary<VR>,
    ) -> Result<Option<ThemedTextures<VR, RR, SS::String>>>
    where
        M: GeometryModelOps<VR, RR, C, Semantic, Material, Texture, Geometry, SS>,
    {
        if self.ring_textures.is_empty() && self.vertex_uv_mapping.is_empty() {
            return Ok(None);
        }

        let texture_map_option = Some(build_texture_map::<VR, RR, SS>(
            boundary,
            &self.ring_textures,
            &self.vertex_uv_mapping,
        ));

        for uv in std::mem::take(&mut self.uv_coordinates) {
            self.model.add_uv_coordinate(uv)?;
        }

        Ok(texture_map_option)
    }

    fn validate_structure(&self) -> Result<()> {
        self.validate_geometry_shape()?;
        self.validate_outer_elements()
    }

    fn validate_geometry_shape(&self) -> Result<()> {
        let (vertices_empty, template_str) = self.builder_vertices_state();

        match self.type_geometry {
            GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                if self.solids.is_empty()
                    || self.shells.is_empty()
                    || self.surfaces.is_empty()
                    || self.rings.is_empty()
                    || vertices_empty
                {
                    return Err(self.invalid_geometry_type_error(format!(
                        "multi- or composite solid geometry {template_str}"
                    )));
                }
            }
            GeometryType::Solid => {
                if !self.solids.is_empty()
                    || self.shells.is_empty()
                    || self.surfaces.is_empty()
                    || self.rings.is_empty()
                    || vertices_empty
                {
                    return Err(self.invalid_geometry_type_error(format!(
                        "single solid geometry {template_str}"
                    )));
                }
            }
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                if !self.solids.is_empty()
                    || !self.shells.is_empty()
                    || self.surfaces.is_empty()
                    || self.rings.is_empty()
                    || vertices_empty
                {
                    return Err(self.invalid_geometry_type_error(format!(
                        "multi- or composite surface geometry {template_str}"
                    )));
                }
            }
            GeometryType::MultiLineString => {
                if !self.solids.is_empty()
                    || !self.shells.is_empty()
                    || !self.surfaces.is_empty()
                    || self.rings.is_empty()
                    || vertices_empty
                {
                    return Err(self.invalid_geometry_type_error(format!(
                        "multi linestring geometry {template_str}"
                    )));
                }
            }
            GeometryType::MultiPoint => {
                if !self.solids.is_empty()
                    || !self.shells.is_empty()
                    || !self.surfaces.is_empty()
                    || !self.rings.is_empty()
                    || vertices_empty
                {
                    return Err(self.invalid_geometry_type_error(format!(
                        "multi point geometry {template_str}"
                    )));
                }
            }
            GeometryType::GeometryInstance => {
                self.validate_geometry_instance_shape()?;
            }
        }

        Ok(())
    }

    fn builder_vertices_state(&self) -> (bool, &'static str) {
        match self.builder_mode {
            BuilderMode::Regular => (self.vertices.is_empty(), ""),
            BuilderMode::Template => (self.template_vertices.is_empty(), "template"),
        }
    }

    fn invalid_geometry_type_error(&self, expected: String) -> Error {
        Error::InvalidGeometryType {
            expected,
            found: self.format_counts(),
        }
    }

    fn validate_geometry_instance_shape(&self) -> Result<()> {
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
            return Err(Error::IncompleteGeometry(
                "GeometryInstance must have a boundary with only a single vertex, which is the reference point for the template transformations"
                    .to_string(),
            ));
        }

        Ok(())
    }

    fn validate_outer_elements(&self) -> Result<()> {
        for (i, surface) in self.surfaces.iter().enumerate() {
            if surface.outer_ring.is_none() {
                return Err(Error::IncompleteGeometry(format!(
                    "Surface {i} missing outer ring"
                )));
            }
        }

        for (i, solid) in self.solids.iter().enumerate() {
            if solid.outer_shell.is_none() {
                return Err(Error::IncompleteGeometry(format!(
                    "Solid {i} missing outer shell"
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

fn build_semantic_map<VR: VertexRef, RR: ResourceRef>(
    type_geometry: &GeometryType,
    builder_semantics: &HashMap<usize, RR>,
    nr_primitives: usize,
) -> Option<SemanticOrMaterialMap<VR, RR>> {
    match type_geometry {
        GeometryType::GeometryInstance => None,
        GeometryType::MultiPoint => {
            if builder_semantics.is_empty() {
                None
            } else {
                let mut semantic_map = SemanticOrMaterialMap::<VR, RR>::new();
                for i in 0..nr_primitives {
                    semantic_map.add_point(builder_semantics.get(&i).copied());
                }
                Some(semantic_map)
            }
        }
        GeometryType::MultiLineString => {
            if builder_semantics.is_empty() {
                None
            } else {
                let mut semantic_map = SemanticOrMaterialMap::<VR, RR>::new();
                for i in 0..nr_primitives {
                    semantic_map.add_linestring(builder_semantics.get(&i).copied());
                }
                Some(semantic_map)
            }
        }
        _ => {
            // Handle semantics, materials and textures for surfaces
            if builder_semantics.is_empty() {
                None
            } else {
                let mut semantic_map = SemanticOrMaterialMap::<VR, RR>::new();
                for i in 0..nr_primitives {
                    semantic_map.add_surface(builder_semantics.get(&i).copied());
                }
                Some(semantic_map)
            }
        }
    }
}

fn build_material_map<VR: VertexRef, RR: ResourceRef, SS: StringStorage>(
    surface_materials: &[(SS::String, ThemedMaterialMappings<RR>)],
    surfaces: &[SurfaceInProgress],
) -> Option<ThemedMaterials<VR, RR, SS::String>> {
    if surface_materials.is_empty() {
        None
    } else {
        // Create a vector to hold all theme/materialmap pairs
        let mut themed_materials = Vec::with_capacity(surface_materials.len());

        // For each theme, create a MaterialMap
        for (theme_name, surface_mappings) in surface_materials {
            // We need to ensure the materials vector has entries for all surfaces
            // by creating an array of the right size with all None values
            let mut material_map = SemanticOrMaterialMap::<VR, RR>::new();
            for _ in 0..surfaces.len() {
                material_map.add_surface(None);
            }

            // Now fill in the materials that are defined for this theme
            for (surface_idx, material_ref) in surface_mappings {
                if *surface_idx < surfaces.len() {
                    material_map.surfaces[*surface_idx] = Some(*material_ref);
                }
            }

            // Add this theme and its material map to our collection
            themed_materials.push((theme_name.clone(), material_map));
        }

        if themed_materials.is_empty() {
            None
        } else {
            Some(themed_materials)
        }
    }
}

fn build_texture_map<VR: VertexRef, RR: ResourceRef, SS: StringStorage>(
    boundary: &Boundary<VR>,
    ring_textures: &[(SS::String, ThemedTextureMappings<RR>)],
    vertex_uv_mapping: &HashMap<usize, usize>,
) -> ThemedTextures<VR, RR, SS::String> {
    let mut themed_texture_maps = Vec::new();

    for (theme_name, ring_mappings) in ring_textures {
        let mut texture_map = TextureMapCore::<VR, RR>::default();

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
            if *vertex_idx < texture_map.vertices_mut().len()
                && let Ok(uv_vertex_idx) = VertexIndex::<VR>::try_from(*uv_idx)
            {
                texture_map.vertices_mut()[*vertex_idx] = Some(uv_vertex_idx);
            }
        }

        // Only add the texture map if it has at least one ring with texture
        if !texture_map.rings().is_empty() {
            themed_texture_maps.push((theme_name.clone(), texture_map));
        }
    }

    themed_texture_maps
}

#[repr(C)]
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[non_exhaustive]
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
        write!(f, "{self:?}")
    }
}

impl FromStr for GeometryType {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "MultiPoint" => Ok(GeometryType::MultiPoint),
            "MultiLineString" => Ok(GeometryType::MultiLineString),
            "MultiSurface" => Ok(GeometryType::MultiSurface),
            "CompositeSurface" => Ok(GeometryType::CompositeSurface),
            "Solid" => Ok(GeometryType::Solid),
            "MultiSolid" => Ok(GeometryType::MultiSolid),
            "CompositeSolid" => Ok(GeometryType::CompositeSolid),
            "GeometryInstance" => Ok(GeometryType::GeometryInstance),
            &_ => Err(Error::InvalidGeometryType {
                expected: "one of MultiPoint, MultiLineString, MultiSurface, CompositeSurface, Solid, MultiSolid, CompositeSolid, GeometryInstance"
                    .to_string(),
                found: s.to_string(),
            }),
        }
    }
}

/// Level of Detail (`LoD`) for the geometry
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
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
    use crate::CityModelType;
    use crate::cityjson::core::geometry::GeometryType;
    use crate::prelude::{BoundaryType, ImageType, QuantizedCoordinate};
    use crate::resources::handles::{
        GeometryRef, MaterialRef, SemanticRef, TemplateGeometryRef, TextureRef,
    };
    use crate::resources::pool::ResourceId32;
    use crate::resources::storage::OwnedStringStorage;
    use crate::v2_0::RGB;
    use crate::v2_0::{CityModel, OwnedMaterial, OwnedTexture, Semantic, SemanticType};

    // Test helper to create a new model
    fn create_test_model() -> CityModel<u32, OwnedStringStorage> {
        CityModel::new(CityModelType::CityJSON)
    }

    const FLOAT_EPSILON: f64 = 1.0e-12;

    fn assert_f64_eq(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= FLOAT_EPSILON,
            "expected {expected}, got {actual} (epsilon {FLOAT_EPSILON})"
        );
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
            .get_geometry(GeometryRef::from_raw(geom_ref))
            .expect("Failed to get geometry");

        // Check geometry type
        assert_eq!(geometry.type_geometry(), &GeometryType::MultiPoint);

        // Get the boundary and convert to nested representation
        let boundary = geometry.boundaries().expect("No boundary found");
        let nested = boundary
            .to_nested_multi_point()
            .expect("Failed to convert to nested");

        // Verify the nested representation (should have 3 points)
        assert_eq!(model.vertices().len(), 3);
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
            .get_geometry(GeometryRef::from_raw(geom_ref))
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
        assert_eq!(model.vertices().len(), 3);
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
            .get_geometry(GeometryRef::from_raw(geom_ref))
            .expect("Failed to get geometry");

        // Check geometry type
        assert_eq!(geometry.type_geometry(), &GeometryType::MultiPoint);

        // Get the boundary and convert to nested representation
        let boundary = geometry.boundaries().expect("No boundary found");
        let nested = boundary
            .to_nested_multi_point()
            .expect("Failed to convert to nested");

        // Verify the nested representation (should have 3 points)
        assert_eq!(model.vertices().len(), 4);
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
        let first_semantic_ref = builder.set_semantic_point(Some(p0), sem0, false);
        let second_semantic_ref = builder.set_semantic_point(Some(p2), sem1, false);

        // Build the geometry
        let geom_ref = builder.build().expect("Failed to build geometry");

        // Get the geometry from the model
        let geometry = model
            .get_geometry(GeometryRef::from_raw(geom_ref))
            .expect("Failed to get geometry");

        // Check geometry type
        assert_eq!(geometry.type_geometry(), &GeometryType::MultiPoint);

        // Get the boundary and convert to nested representation
        let boundary = geometry.boundaries().expect("No boundary found");
        let nested = boundary
            .to_nested_multi_point()
            .expect("Failed to convert to nested");

        // Verify the nested representation (should have 3 points)
        assert_eq!(model.vertices().len(), 3);
        assert_eq!(nested, vec![0, 1, 2]);

        // Check semantics
        let semantic_map = geometry.semantics().expect("No semantics found");
        let point_semantics = semantic_map.points();

        // Verify points have semantics applied correctly
        assert_eq!(point_semantics.len(), 3);

        // Verify the semantic references are the ones we set
        let referenced_semantics: Vec<SemanticRef> = point_semantics
            .iter()
            .filter_map(|s| s.as_ref())
            .copied()
            .collect();
        assert!(referenced_semantics.contains(&SemanticRef::from_raw(
            *first_semantic_ref.as_ref().unwrap()
        )));
        assert!(referenced_semantics.contains(&SemanticRef::from_raw(
            *second_semantic_ref.as_ref().unwrap()
        )));

        // Verify the semantics themselves
        let first_semantic = model
            .get_semantic(SemanticRef::from_raw(first_semantic_ref.unwrap()))
            .expect("Semantic 0 not found");
        assert_eq!(
            first_semantic.type_semantic(),
            &SemanticType::TransportationHole
        );

        let second_semantic = model
            .get_semantic(SemanticRef::from_raw(second_semantic_ref.unwrap()))
            .expect("Semantic 1 not found");
        assert_eq!(
            second_semantic.type_semantic(),
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
        let sem_ref = builder.set_semantic_linestring(Some(ls2), sem, false);

        // Build the geometry
        let geom_ref = builder.build().expect("Failed to build geometry");

        // Get the geometry from the model
        let geometry = model
            .get_geometry(GeometryRef::from_raw(geom_ref))
            .expect("Failed to get geometry");

        // Check geometry type
        assert_eq!(geometry.type_geometry(), &GeometryType::MultiLineString);

        // Get the boundary and convert to nested representation
        let boundary = geometry.boundaries().expect("No boundary found");
        let nested = boundary
            .to_nested_multi_linestring()
            .expect("Failed to convert to nested");

        // Verify the nested representation
        assert_eq!(model.vertices().len(), 6);
        assert_eq!(nested, vec![vec![0, 1, 2, 3, 4], vec![0, 2], vec![1, 4, 5]]);

        // Check semantics
        let semantics = geometry.semantics().expect("No semantics found");
        let linestring_semantics = semantics.linestrings();

        // Verify linestrings have semantics applied correctly
        assert_eq!(linestring_semantics.len(), 3); // Should have entries for all linestrings

        // Only the second linestring should have a semantic
        assert!(linestring_semantics[0].is_none());
        assert_eq!(
            linestring_semantics[1],
            Some(SemanticRef::from_raw(sem_ref.clone().unwrap()))
        );
        assert!(linestring_semantics[2].is_none());

        // Verify the semantic itself
        let semantic = model
            .get_semantic(SemanticRef::from_raw(sem_ref.unwrap()))
            .expect("Semantic not found");
        assert_eq!(
            semantic.type_semantic(),
            &SemanticType::TransportationMarking
        );
    }

    fn build_multisurface_test_geometry(
        model: &mut CityModel<u32, OwnedStringStorage>,
    ) -> (ResourceId32, ResourceId32, ResourceId32, ResourceId32) {
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

        let mut builder =
            GeometryBuilder::new(model, GeometryType::MultiSurface, BuilderMode::Regular);
        let p0 = builder.add_vertex(v0);
        let p1 = builder.add_vertex(v1);
        let p2 = builder.add_vertex(v2);
        let p3 = builder.add_vertex(v3);
        let p4 = builder.add_point(QuantizedCoordinate::new(5, 15, 0));
        let p5 = builder.add_point(QuantizedCoordinate::new(15, 5, 0));
        let p6 = builder.add_point(QuantizedCoordinate::new(20, 0, 0));
        let p7 = builder.add_point(QuantizedCoordinate::new(20, 10, 0));
        let p8 = builder.add_point(QuantizedCoordinate::new(15, 15, 0));

        let ring0 = builder.add_ring(&[p0, p1, p4]).expect("Failed to add ring");
        let surface0 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring0)
            .expect("Failed to add outer ring");

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

        let uv0 = builder.add_uv_coordinate(0.0, 0.0);
        let uv1 = builder.add_uv_coordinate(1.0, 0.0);
        let uv2 = builder.add_uv_coordinate(1.0, 1.0);
        let uv3 = builder.add_uv_coordinate(0.0, 1.0);
        builder.map_vertex_to_uv(p1, uv0);
        builder.map_vertex_to_uv(p2, uv1);
        builder.map_vertex_to_uv(p5, uv2);
        builder.map_vertex_to_uv(p6, uv3);

        let wall_texture = OwnedTexture::new("facade.jpg".to_string(), ImageType::Jpg);
        let texture_ref = builder
            .set_texture_ring(
                Some(surface0),
                wall_texture,
                "theme-texture".to_string(),
                true,
            )
            .expect("Failed to set texture on ring");
        let roof_semantic = Semantic::new(SemanticType::RoofSurface);
        let sem_ref = builder
            .set_semantic_surface(Some(surface1), roof_semantic, false)
            .expect("Failed to set semantic on surface");

        let ring3 = builder
            .add_ring(&[p2, p3, p4, p8, p7])
            .expect("Failed to add ring");
        let surface2 = builder.start_surface();
        builder
            .add_surface_outer_ring(ring3)
            .expect("Failed to add outer ring");

        let mut wall_material = OwnedMaterial::new("Wall".to_string());
        wall_material.set_diffuse_color(Some([0.8, 0.8, 0.8].into()));
        wall_material.set_ambient_intensity(Some(0.5));
        let mat_ref = builder
            .set_material_surface(
                Some(surface2),
                wall_material,
                "material-theme".to_string(),
                true,
            )
            .expect("Failed to set material on surface");
        let geom_ref = builder.build().expect("Failed to build geometry");

        (geom_ref, sem_ref, mat_ref, texture_ref)
    }

    fn assert_multisurface_test_geometry(
        model: &CityModel<u32, OwnedStringStorage>,
        geom_ref: ResourceId32,
        sem_ref: ResourceId32,
        mat_ref: ResourceId32,
        texture_ref: ResourceId32,
    ) {
        let geometry = model
            .get_geometry(GeometryRef::from_raw(geom_ref))
            .expect("Failed to get geometry");
        assert_eq!(geometry.type_geometry(), &GeometryType::MultiSurface);

        let boundary = geometry.boundaries().expect("No boundary found");
        let nested = boundary
            .to_nested_multi_or_composite_surface()
            .expect("Failed to convert to nested");
        let nested_expected = vec![
            vec![vec![0, 1, 4]],
            vec![vec![1, 2, 5, 6], vec![0, 1, 2]],
            vec![vec![2, 3, 4, 8, 7]],
        ];
        assert_eq!(model.vertices().len(), 9);
        assert_eq!(nested, nested_expected);

        let semantics = geometry.semantics().expect("No semantics found");
        let surface_semantics = semantics.surfaces();
        assert_eq!(surface_semantics.len(), 3);
        assert!(surface_semantics[0].is_none());
        assert_eq!(surface_semantics[1], Some(SemanticRef::from_raw(sem_ref)));
        assert!(surface_semantics[2].is_none());

        let semantic = model
            .get_semantic(SemanticRef::from_raw(sem_ref))
            .expect("Semantic not found");
        assert_eq!(semantic.type_semantic(), &SemanticType::RoofSurface);

        let materials = geometry.materials().expect("No materials found");
        let (_theme_material, material_map) = materials.first().expect("Missing material map");
        let surface_materials = material_map.surfaces();
        assert_eq!(surface_materials.len(), 3);
        assert!(surface_materials[0].is_none());
        assert!(surface_materials[1].is_none());
        assert_eq!(surface_materials[2], Some(MaterialRef::from_raw(mat_ref)));

        let material = model
            .get_material(MaterialRef::from_raw(mat_ref))
            .expect("Material not found");
        assert_eq!(material.name(), "Wall");
        assert!(material.diffuse_color().is_some());
        assert_f64_eq(f64::from(material.ambient_intensity().unwrap()), 0.5);

        let texture_sets = geometry.textures().expect("No textures found");
        let (_theme_texture, texture_map) = texture_sets.first().expect("Missing texture map");
        assert!(
            !texture_map.vertices().is_empty(),
            "No texture vertices found"
        );
        assert!(!texture_map.rings().is_empty(), "No texture rings found");
        assert!(
            !texture_map.ring_textures().is_empty(),
            "No ring textures found"
        );

        let texture_refs: Vec<TextureRef> = texture_map
            .ring_textures()
            .iter()
            .filter_map(|texture| texture.as_ref())
            .copied()
            .collect();
        assert!(
            texture_refs.contains(&TextureRef::from_raw(texture_ref)),
            "Texture reference not found"
        );

        let facade_texture = model
            .get_texture(TextureRef::from_raw(texture_ref))
            .expect("Texture not found");
        assert_eq!(facade_texture.image(), "facade.jpg");
        assert_eq!(facade_texture.image_type(), &ImageType::Jpg);
    }

    #[test]
    fn test_multisurface() {
        let mut model = create_test_model();
        let (geom_ref, sem_ref, mat_ref, texture_ref) =
            build_multisurface_test_geometry(&mut model);
        assert_multisurface_test_geometry(&model, geom_ref, sem_ref, mat_ref, texture_ref);
    }

    fn build_solid_test_geometry(
        model: &mut CityModel<u32, OwnedStringStorage>,
    ) -> (
        ResourceId32,
        ResourceId32,
        ResourceId32,
        ResourceId32,
        ResourceId32,
    ) {
        let mut builder = GeometryBuilder::new(model, GeometryType::Solid, BuilderMode::Regular);
        let cube_points = [
            builder.add_point(QuantizedCoordinate::new(0, 0, 0)),
            builder.add_point(QuantizedCoordinate::new(10, 0, 0)),
            builder.add_point(QuantizedCoordinate::new(10, 10, 0)),
            builder.add_point(QuantizedCoordinate::new(0, 10, 0)),
            builder.add_point(QuantizedCoordinate::new(0, 0, 10)),
            builder.add_point(QuantizedCoordinate::new(10, 0, 10)),
            builder.add_point(QuantizedCoordinate::new(10, 10, 10)),
            builder.add_point(QuantizedCoordinate::new(0, 10, 10)),
        ];
        let cube_rings = [
            [0, 1, 5, 4, 0],
            [1, 2, 6, 5, 1],
            [2, 3, 7, 6, 2],
            [3, 0, 4, 7, 3],
            [4, 5, 6, 7, 4],
            [0, 3, 2, 1, 0],
        ];

        let mut surfaces = [0_usize; 6];
        for (surface_idx, ring_definition) in cube_rings.iter().enumerate() {
            let ring_vertices = ring_definition.map(|point_idx| cube_points[point_idx]);
            let ring = builder
                .add_ring(&ring_vertices)
                .expect("Failed to create ring");
            let surface = builder.start_surface();
            builder
                .add_surface_outer_ring(ring)
                .expect("Failed to add surface");
            surfaces[surface_idx] = surface;
        }

        let wall_semantic = Semantic::new(SemanticType::WallSurface);
        let roof_semantic = Semantic::new(SemanticType::RoofSurface);
        let floor_semantic = Semantic::new(SemanticType::FloorSurface);
        for &surface in &surfaces[..4] {
            builder
                .set_semantic_surface(Some(surface), wall_semantic.clone(), false)
                .expect("Failed to set wall semantic");
        }
        let roof_sem_ref = builder
            .set_semantic_surface(Some(surfaces[4]), roof_semantic, false)
            .expect("Failed to set roof semantic");
        let floor_sem_ref = builder
            .set_semantic_surface(Some(surfaces[5]), floor_semantic, false)
            .expect("Failed to set floor semantic");

        let mut wall_material = OwnedMaterial::new("Wall".to_string());
        wall_material.set_diffuse_color(Some([0.8, 0.8, 0.8].into()));
        let mut roof_material = OwnedMaterial::new("Roof".to_string());
        roof_material.set_diffuse_color(Some([0.9, 0.1, 0.1].into()));
        let wall_mat_ref = builder
            .set_material_surface(
                Some(surfaces[0]),
                wall_material,
                "material-theme".to_string(),
                true,
            )
            .expect("Failed to set wall material");
        let roof_mat_ref = builder
            .set_material_surface(
                Some(surfaces[4]),
                roof_material,
                "material-theme".to_string(),
                true,
            )
            .expect("Failed to set roof material");

        builder.add_shell(&surfaces).expect("Failed to add shell");
        builder = builder.with_lod(LoD::LoD1);
        let geom_ref = builder.build().expect("Failed to build geometry");

        (
            geom_ref,
            roof_sem_ref,
            floor_sem_ref,
            wall_mat_ref,
            roof_mat_ref,
        )
    }

    fn assert_solid_test_geometry(
        model: &CityModel<u32, OwnedStringStorage>,
        geom_ref: ResourceId32,
        roof_sem_ref: ResourceId32,
        floor_sem_ref: ResourceId32,
        wall_mat_ref: ResourceId32,
        roof_mat_ref: ResourceId32,
    ) {
        let geometry = model
            .get_geometry(GeometryRef::from_raw(geom_ref))
            .expect("Failed to get geometry");
        assert_eq!(geometry.type_geometry(), &GeometryType::Solid);
        assert_eq!(geometry.lod(), Some(&LoD::LoD1));

        let boundary = geometry.boundaries().expect("No boundary found");
        assert_eq!(boundary.check_type(), BoundaryType::Solid);
        let nested = boundary
            .to_nested_solid()
            .expect("Failed to convert to nested representation");
        assert_eq!(nested.len(), 1);
        assert_eq!(nested[0].len(), 6);

        let semantics = geometry.semantics().expect("No semantics found");
        let surface_semantics = semantics.surfaces();
        assert_eq!(surface_semantics.len(), 6);
        assert_eq!(
            surface_semantics[4],
            Some(SemanticRef::from_raw(roof_sem_ref))
        );
        assert_eq!(
            surface_semantics[5],
            Some(SemanticRef::from_raw(floor_sem_ref))
        );

        let materials = geometry.materials().expect("No materials found");
        let (_theme_material, material_map) = materials.first().expect("Missing material map");
        let surface_materials = material_map.surfaces();
        assert_eq!(surface_materials.len(), 6);
        assert_eq!(
            surface_materials[0],
            Some(MaterialRef::from_raw(wall_mat_ref))
        );
        assert_eq!(
            surface_materials[4],
            Some(MaterialRef::from_raw(roof_mat_ref))
        );

        let wall_material = model
            .get_material(MaterialRef::from_raw(wall_mat_ref))
            .expect("Wall material not found");
        assert_eq!(wall_material.name(), "Wall");
        assert_eq!(
            wall_material
                .diffuse_color()
                .expect("Missing diffuse color"),
            RGB::from([0.8, 0.8, 0.8])
        );
    }

    #[test]
    fn test_solid() {
        let mut model = create_test_model();
        let (geom_ref, roof_sem_ref, floor_sem_ref, wall_mat_ref, roof_mat_ref) =
            build_solid_test_geometry(&mut model);
        assert_solid_test_geometry(
            &model,
            geom_ref,
            roof_sem_ref,
            floor_sem_ref,
            wall_mat_ref,
            roof_mat_ref,
        );
    }

    const MULTISOLID_CUBE1_COORDS: [(i64, i64, i64); 8] = [
        (0, 0, 0),
        (5, 0, 0),
        (5, 5, 0),
        (0, 5, 0),
        (0, 0, 5),
        (5, 0, 5),
        (5, 5, 5),
        (0, 5, 5),
    ];
    const MULTISOLID_CUBE2_COORDS: [(i64, i64, i64); 8] = [
        (10, 10, 0),
        (20, 10, 0),
        (20, 20, 0),
        (10, 20, 0),
        (10, 10, 10),
        (20, 10, 10),
        (20, 20, 10),
        (10, 20, 10),
    ];
    const MULTISOLID_CUBE1_RING_DEFINITIONS: [[usize; 5]; 6] = [
        [0, 1, 5, 4, 0],
        [1, 2, 6, 5, 1],
        [2, 3, 7, 6, 2],
        [3, 0, 4, 7, 3],
        [4, 5, 6, 7, 3],
        [0, 3, 2, 1, 0],
    ];
    const MULTISOLID_CUBE2_RING_DEFINITIONS: [[usize; 5]; 6] = [
        [0, 1, 5, 4, 0],
        [1, 2, 6, 5, 1],
        [2, 3, 7, 6, 2],
        [3, 0, 4, 7, 3],
        [4, 5, 6, 7, 4],
        [0, 3, 2, 1, 0],
    ];
    const MULTISOLID_CUBE1_SURFACE_TO_RING: [usize; 6] = [0, 1, 2, 2, 4, 5];
    const MULTISOLID_CUBE2_SURFACE_TO_RING: [usize; 6] = [0, 1, 2, 3, 4, 5];
    type MultiSolidRefs = [ResourceId32; 6];

    fn build_multisolid_test_geometry(
        model: &mut CityModel<u32, OwnedStringStorage>,
    ) -> (ResourceId32, MultiSolidRefs) {
        let mut builder =
            GeometryBuilder::new(model, GeometryType::MultiSolid, BuilderMode::Regular);

        let cube1_points = MULTISOLID_CUBE1_COORDS
            .map(|(x, y, z)| builder.add_point(QuantizedCoordinate::new(x, y, z)));
        let cube2_points = MULTISOLID_CUBE2_COORDS
            .map(|(x, y, z)| builder.add_point(QuantizedCoordinate::new(x, y, z)));

        let mut add_cube_surfaces = |points: [usize; 8],
                                     ring_definitions: [[usize; 5]; 6],
                                     surface_to_ring: [usize; 6]|
         -> [usize; 6] {
            let mut rings = [0_usize; 6];
            for (ring_idx, ring_definition) in ring_definitions.iter().enumerate() {
                let ring_points = ring_definition.map(|point_idx| points[point_idx]);
                rings[ring_idx] = builder
                    .add_ring(&ring_points)
                    .expect("Failed to create ring");
            }

            let mut surfaces = [0_usize; 6];
            for (surface_idx, ring_idx) in surface_to_ring.iter().copied().enumerate() {
                let surface = builder.start_surface();
                builder
                    .add_surface_outer_ring(rings[ring_idx])
                    .expect("Failed to add surface");
                surfaces[surface_idx] = surface;
            }

            surfaces
        };

        let cube1_surfaces = add_cube_surfaces(
            cube1_points,
            MULTISOLID_CUBE1_RING_DEFINITIONS,
            MULTISOLID_CUBE1_SURFACE_TO_RING,
        );
        let cube2_surfaces = add_cube_surfaces(
            cube2_points,
            MULTISOLID_CUBE2_RING_DEFINITIONS,
            MULTISOLID_CUBE2_SURFACE_TO_RING,
        );

        let roof_semantic = Semantic::new(SemanticType::RoofSurface);
        let ground_semantic = Semantic::new(SemanticType::GroundSurface);
        let roof_sem_ref1 = builder
            .set_semantic_surface(Some(cube1_surfaces[4]), roof_semantic.clone(), false)
            .expect("Failed to set semantic");
        let ground_sem_ref1 = builder
            .set_semantic_surface(Some(cube1_surfaces[5]), ground_semantic.clone(), false)
            .expect("Failed to set semantic");
        let roof_sem_ref2 = builder
            .set_semantic_surface(Some(cube2_surfaces[4]), roof_semantic, false)
            .expect("Failed to set semantic");
        let ground_sem_ref2 = builder
            .set_semantic_surface(Some(cube2_surfaces[5]), ground_semantic, false)
            .expect("Failed to set semantic");

        let mut red_material = OwnedMaterial::new("RedWall".to_string());
        red_material.set_diffuse_color(Some([0.9, 0.1, 0.1].into()));
        let mut blue_material = OwnedMaterial::new("BlueWall".to_string());
        blue_material.set_diffuse_color(Some([0.1, 0.1, 0.9].into()));
        let red_mat_ref = builder
            .set_material_surface(
                Some(cube1_surfaces[0]),
                red_material,
                "material-theme".to_string(),
                true,
            )
            .expect("Failed to set material");
        let blue_mat_ref = builder
            .set_material_surface(
                Some(cube2_surfaces[0]),
                blue_material,
                "material-theme".to_string(),
                true,
            )
            .expect("Failed to set material");

        builder
            .add_shell(&cube1_surfaces)
            .expect("Failed to add first shell");
        builder
            .add_shell(&cube2_surfaces)
            .expect("Failed to add second shell");
        builder.start_solid();
        builder
            .add_solid_outer_shell(0)
            .expect("Failed to add shell to solid");
        builder.start_solid();
        builder
            .add_solid_outer_shell(1)
            .expect("Failed to add shell to solid");
        builder = builder.with_lod(LoD::LoD1);
        let geom_ref = builder.build().expect("Failed to build geometry");

        (
            geom_ref,
            [
                roof_sem_ref1,
                ground_sem_ref1,
                roof_sem_ref2,
                ground_sem_ref2,
                red_mat_ref,
                blue_mat_ref,
            ],
        )
    }

    fn assert_multisolid_test_geometry(
        model: &CityModel<u32, OwnedStringStorage>,
        geom_ref: ResourceId32,
        refs: MultiSolidRefs,
    ) {
        let [
            roof_sem_ref1,
            ground_sem_ref1,
            roof_sem_ref2,
            ground_sem_ref2,
            red_mat_ref,
            blue_mat_ref,
        ] = refs;
        let geometry = model
            .get_geometry(GeometryRef::from_raw(geom_ref))
            .expect("Failed to get geometry");
        assert_eq!(geometry.type_geometry(), &GeometryType::MultiSolid);
        assert_eq!(geometry.lod(), Some(&LoD::LoD1));

        let boundary = geometry.boundaries().expect("No boundary found");
        assert_eq!(boundary.check_type(), BoundaryType::MultiOrCompositeSolid);
        let nested = boundary
            .to_nested_multi_or_composite_solid()
            .expect("Failed to convert to nested");
        assert_eq!(nested.len(), 2);
        assert_eq!(nested[0].len(), 1);
        assert_eq!(nested[1].len(), 1);
        assert_eq!(nested[0][0].len(), 6);
        assert_eq!(nested[1][0].len(), 6);

        let semantics = geometry.semantics().expect("No semantics found");
        let surface_semantics = semantics.surfaces();
        assert_eq!(surface_semantics.len(), 12);
        assert_eq!(
            surface_semantics[4],
            Some(SemanticRef::from_raw(roof_sem_ref1))
        );
        assert_eq!(
            surface_semantics[5],
            Some(SemanticRef::from_raw(ground_sem_ref1))
        );
        assert_eq!(
            surface_semantics[10],
            Some(SemanticRef::from_raw(roof_sem_ref2))
        );
        assert_eq!(
            surface_semantics[11],
            Some(SemanticRef::from_raw(ground_sem_ref2))
        );

        let materials = geometry.materials().expect("No materials found");
        let (_theme_material, material_map) = materials.first().expect("Missing material map");
        let surface_materials = material_map.surfaces();
        assert_eq!(surface_materials.len(), 12);
        assert_eq!(
            surface_materials[0],
            Some(MaterialRef::from_raw(red_mat_ref))
        );
        assert_eq!(
            surface_materials[6],
            Some(MaterialRef::from_raw(blue_mat_ref))
        );

        let red_material = model
            .get_material(MaterialRef::from_raw(red_mat_ref))
            .expect("Red material not found");
        assert_eq!(red_material.name(), "RedWall");
        assert_eq!(
            red_material.diffuse_color().expect("Missing diffuse color"),
            RGB::from([0.9, 0.1, 0.1])
        );

        let blue_material = model
            .get_material(MaterialRef::from_raw(blue_mat_ref))
            .expect("Blue material not found");
        assert_eq!(blue_material.name(), "BlueWall");
        assert_eq!(
            blue_material
                .diffuse_color()
                .expect("Missing diffuse color"),
            RGB::from([0.1, 0.1, 0.9])
        );
    }

    #[test]
    fn test_multisolid() {
        let mut model = create_test_model();
        let (geom_ref, refs) = build_multisolid_test_geometry(&mut model);
        assert_multisolid_test_geometry(&model, geom_ref, refs);
    }

    fn build_template_geometry_for_instance(
        model: &mut CityModel<u32, OwnedStringStorage>,
    ) -> (ResourceId32, ResourceId32) {
        let mut template_builder =
            GeometryBuilder::new(model, GeometryType::MultiLineString, BuilderMode::Template);
        let template_points = [
            template_builder.add_template_point(RealWorldCoordinate::new(0.0, 0.0, 0.0)),
            template_builder.add_template_point(RealWorldCoordinate::new(1.0, 0.0, 0.0)),
            template_builder.add_template_point(RealWorldCoordinate::new(1.0, 1.0, 0.0)),
            template_builder.add_template_point(RealWorldCoordinate::new(0.0, 1.0, 0.0)),
            template_builder.add_template_point(RealWorldCoordinate::new(2.0, 0.0, 0.0)),
            template_builder.add_template_point(RealWorldCoordinate::new(2.0, 2.0, 0.0)),
        ];

        template_builder
            .add_linestring(&[
                template_points[0],
                template_points[1],
                template_points[2],
                template_points[3],
                template_points[0],
            ])
            .expect("Failed to add first linestring to template");
        let diagonal_linestring = template_builder
            .add_linestring(&[template_points[0], template_points[2]])
            .expect("Failed to add second linestring to template");
        template_builder
            .add_linestring(&[template_points[1], template_points[4], template_points[5]])
            .expect("Failed to add third linestring to template");

        let sem_ref = template_builder
            .set_semantic_linestring(
                Some(diagonal_linestring),
                Semantic::new(SemanticType::TransportationMarking),
                false,
            )
            .expect("Failed to set semantic for template linestring");
        template_builder = template_builder.with_lod(LoD::LoD2);
        let template_ref = template_builder
            .build()
            .expect("Failed to build template geometry");

        (template_ref, sem_ref)
    }

    fn assert_template_geometry_for_instance(
        model: &CityModel<u32, OwnedStringStorage>,
        template_ref: ResourceId32,
        sem_ref: ResourceId32,
    ) {
        assert!(
            model
                .get_template_geometry(TemplateGeometryRef::from_raw(template_ref))
                .is_some(),
            "Template geometry not found in template pool"
        );

        let template = model
            .get_template_geometry(TemplateGeometryRef::from_raw(template_ref))
            .expect("Failed to get template geometry");
        assert_eq!(template.type_geometry(), &GeometryType::MultiLineString);
        assert_eq!(template.lod(), Some(&LoD::LoD2));
        assert_eq!(
            model.template_vertices().len(),
            6,
            "Expected 6 vertices in template_vertices pool"
        );

        let semantics = template
            .semantics()
            .expect("No semantics found in template");
        let linestring_semantics = semantics.linestrings();
        assert_eq!(linestring_semantics.len(), 3);
        assert!(linestring_semantics[0].is_none());
        assert_eq!(
            linestring_semantics[1],
            Some(SemanticRef::from_raw(sem_ref))
        );
        assert!(linestring_semantics[2].is_none());
    }

    fn build_geometry_instance_from_template(
        model: &mut CityModel<u32, OwnedStringStorage>,
        template_ref: ResourceId32,
    ) -> (ResourceId32, VertexIndex<u32>) {
        let ref_point_idx = model
            .add_vertex(QuantizedCoordinate::new(100, 200, 50))
            .expect("Failed to add reference point");
        let mut instance_builder =
            GeometryBuilder::new(model, GeometryType::GeometryInstance, BuilderMode::Regular);
        instance_builder.add_vertex(ref_point_idx);
        instance_builder = instance_builder
            .with_template(template_ref)
            .expect("Failed to set template boundaries");
        instance_builder = instance_builder
            .with_transformation_matrix([
                2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ])
            .expect("Failed to set transformation matrix");
        let instance_ref = instance_builder
            .build()
            .expect("Failed to build geometry instance");

        (instance_ref, ref_point_idx)
    }

    fn assert_geometry_instance_from_template(
        model: &CityModel<u32, OwnedStringStorage>,
        instance_ref: ResourceId32,
        template_ref: ResourceId32,
        ref_point_idx: VertexIndex<u32>,
    ) {
        let instance = model
            .get_geometry(GeometryRef::from_raw(instance_ref))
            .expect("Failed to get geometry instance");
        assert_eq!(instance.type_geometry(), &GeometryType::GeometryInstance);
        assert_eq!(
            instance.instance_template(),
            Some(TemplateGeometryRef::from_raw(template_ref))
        );

        let expected_matrix = [
            2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        assert_eq!(
            instance.instance_transformation_matrix(),
            Some(&expected_matrix)
        );
        assert_eq!(instance.instance_reference_point(), Some(&ref_point_idx));
        assert!(
            model
                .get_geometry(GeometryRef::from_raw(instance_ref))
                .is_some(),
            "Geometry instance not found in regular geometry pool"
        );
    }

    #[test]
    fn test_geometry_template_and_instance() {
        let mut model = create_test_model();
        let (template_ref, sem_ref) = build_template_geometry_for_instance(&mut model);
        assert_template_geometry_for_instance(&model, template_ref, sem_ref);
        let (instance_ref, ref_point_idx) =
            build_geometry_instance_from_template(&mut model, template_ref);
        assert_geometry_instance_from_template(&model, instance_ref, template_ref, ref_point_idx);
    }
}
