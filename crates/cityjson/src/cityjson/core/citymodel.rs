//! # CityModel Core
//!
//! Core implementation of CityModel that is shared across different CityJSON versions.
//!
//! This module provides the `CityModelCore` type which contains the common data structures
//! used by all CityJSON versions. Version-specific implementations wrap this core type
//! and provide version-specific behavior through macros.

use crate::cityjson::core::attributes::Attributes;
use crate::cityjson::core::coordinate::{UVCoordinate, Vertices};
use crate::cityjson::core::vertex::{VertexIndex, VertexRef};
use crate::cityjson::traits::coordinate::Coordinate;
use crate::prelude::{RealWorldCoordinate, Result};
use crate::resources::pool::{DefaultResourcePool, ResourcePool, ResourceRef};
use crate::resources::storage::StringStorage;
use crate::{CityJSONVersion, CityModelType};

/// Core CityModel structure that is shared across all CityJSON versions.
///
/// This type is generic over:
/// - `C`: The coordinate type (FlexibleCoordinate or QuantizedCoordinate)
/// - `VR`: The vertex reference type
/// - `RR`: The resource reference type
/// - `SS`: The string storage type
/// - `Semantic`: The semantic type for this version
/// - `Material`: The material type for this version
/// - `Texture`: The texture type for this version
/// - `Geometry`: The geometry type for this version
/// - `Metadata`: The metadata type for this version
/// - `Transform`: The transform type for this version
/// - `Extensions`: The extensions type for this version
/// - `CityObjects`: The city objects collection type for this version
#[derive(Debug, Clone)]
pub struct CityModelCore<
    C: Coordinate,
    VR: VertexRef,
    RR: ResourceRef,
    SS: StringStorage,
    Semantic,
    Material,
    Texture,
    Geometry,
    Metadata,
    Transform,
    Extensions,
    CityObjects,
> {
    /// CityModel type
    type_citymodel: CityModelType,
    /// CityJSON version
    version: Option<CityJSONVersion>,
    /// CityJSON Extension declarations
    extensions: Option<Extensions>,
    /// Extra root properties for the CityModel
    extra: Option<Attributes<SS, RR>>,
    /// CityModel metadata
    metadata: Option<Metadata>,
    /// Collection of CityObjects
    cityobjects: CityObjects,
    /// The transform object
    transform: Option<Transform>,
    /// Pool of vertex coordinates
    vertices: Vertices<VR, C>,
    /// Pool of geometries
    geometries: DefaultResourcePool<Geometry, RR>,
    /// Pool of vertex coordinates used by the geometry templates in template_geometries
    template_vertices: Vertices<VR, RealWorldCoordinate>,
    /// Pool of geometry templates
    template_geometries: DefaultResourcePool<Geometry, RR>,
    /// Pool of semantic objects
    semantics: DefaultResourcePool<Semantic, RR>,
    /// Pool of material objects
    materials: DefaultResourcePool<Material, RR>,
    /// Pool of texture objects
    textures: DefaultResourcePool<Texture, RR>,
    /// Pool of vertex textures (UV coordinates)
    vertices_texture: Vertices<VR, UVCoordinate>,
    /// Default theme material reference
    default_theme_material: Option<RR>,
    /// Default theme texture reference
    default_theme_texture: Option<RR>,
}

impl<
    C: Coordinate,
    VR: VertexRef,
    RR: ResourceRef,
    SS: StringStorage,
    Semantic,
    Material,
    Texture,
    Geometry,
    Metadata,
    Transform,
    Extensions,
    CityObjects,
>
    CityModelCore<
        C,
        VR,
        RR,
        SS,
        Semantic,
        Material,
        Texture,
        Geometry,
        Metadata,
        Transform,
        Extensions,
        CityObjects,
    >
where
    CityObjects: Default,
{
    /// Create a new CityModelCore with the given type and version
    pub fn new(type_citymodel: CityModelType, version: Option<CityJSONVersion>) -> Self {
        Self {
            type_citymodel,
            version,
            extensions: None,
            extra: None,
            metadata: None,
            cityobjects: CityObjects::default(),
            transform: None,
            vertices: Vertices::new(),
            geometries: DefaultResourcePool::new_pool(),
            template_vertices: Vertices::new(),
            template_geometries: DefaultResourcePool::new_pool(),
            semantics: DefaultResourcePool::new_pool(),
            materials: DefaultResourcePool::new_pool(),
            textures: DefaultResourcePool::new_pool(),
            vertices_texture: Vertices::new(),
            default_theme_material: None,
            default_theme_texture: None,
        }
    }

    /// Create a new CityModelCore with specified capacities
    #[allow(clippy::too_many_arguments)]
    pub fn with_capacity(
        type_citymodel: CityModelType,
        version: Option<CityJSONVersion>,
        cityobjects_capacity: usize,
        vertex_capacity: usize,
        semantic_capacity: usize,
        material_capacity: usize,
        texture_capacity: usize,
        geometry_capacity: usize,
        create_cityobjects: impl FnOnce(usize) -> CityObjects,
    ) -> Self {
        Self {
            type_citymodel,
            version,
            extensions: None,
            extra: None,
            metadata: None,
            cityobjects: create_cityobjects(cityobjects_capacity),
            transform: None,
            vertices: Vertices::with_capacity(vertex_capacity),
            geometries: DefaultResourcePool::with_capacity(geometry_capacity),
            template_vertices: Vertices::new(),
            template_geometries: DefaultResourcePool::new(),
            semantics: DefaultResourcePool::with_capacity(semantic_capacity),
            materials: DefaultResourcePool::with_capacity(material_capacity),
            textures: DefaultResourcePool::with_capacity(texture_capacity),
            vertices_texture: Vertices::new(),
            default_theme_material: None,
            default_theme_texture: None,
        }
    }

    // Semantic methods
    pub fn add_semantic(&mut self, semantic: Semantic) -> RR {
        self.semantics.add(semantic)
    }

    pub fn get_semantic(&self, id: RR) -> Option<&Semantic> {
        self.semantics.get(id)
    }

    pub fn get_semantic_mut(&mut self, id: RR) -> Option<&mut Semantic> {
        self.semantics.get_mut(id)
    }

    pub fn get_or_insert_semantic(&mut self, semantic: Semantic) -> RR
    where
        Semantic: PartialEq,
    {
        if let Some(existing_id) = self.semantics.find(&semantic) {
            return existing_id;
        }
        self.semantics.add(semantic)
    }

    pub fn semantics(&self) -> &DefaultResourcePool<Semantic, RR> {
        &self.semantics
    }

    pub fn semantics_mut(&mut self) -> &mut DefaultResourcePool<Semantic, RR> {
        &mut self.semantics
    }

    // Material methods
    pub fn add_material(&mut self, material: Material) -> RR {
        self.materials.add(material)
    }

    pub fn get_material(&self, id: RR) -> Option<&Material> {
        self.materials.get(id)
    }

    pub fn get_material_mut(&mut self, id: RR) -> Option<&mut Material> {
        self.materials.get_mut(id)
    }

    pub fn get_or_insert_material(&mut self, material: Material) -> RR
    where
        Material: PartialEq,
    {
        if let Some(existing_id) = self.materials.find(&material) {
            return existing_id;
        }
        self.materials.add(material)
    }

    pub fn materials(&self) -> &DefaultResourcePool<Material, RR> {
        &self.materials
    }

    pub fn materials_mut(&mut self) -> &mut DefaultResourcePool<Material, RR> {
        &mut self.materials
    }

    // Texture methods
    pub fn add_texture(&mut self, texture: Texture) -> RR {
        self.textures.add(texture)
    }

    pub fn get_texture(&self, id: RR) -> Option<&Texture> {
        self.textures.get(id)
    }

    pub fn get_texture_mut(&mut self, id: RR) -> Option<&mut Texture> {
        self.textures.get_mut(id)
    }

    pub fn get_or_insert_texture(&mut self, texture: Texture) -> RR
    where
        Texture: PartialEq,
    {
        if let Some(existing_id) = self.textures.find(&texture) {
            return existing_id;
        }
        self.textures.add(texture)
    }

    pub fn textures(&self) -> &DefaultResourcePool<Texture, RR> {
        &self.textures
    }

    pub fn textures_mut(&mut self) -> &mut DefaultResourcePool<Texture, RR> {
        &mut self.textures
    }

    // Geometry methods
    pub fn add_geometry(&mut self, geometry: Geometry) -> RR {
        self.geometries.add(geometry)
    }

    pub fn geometries(&self) -> &DefaultResourcePool<Geometry, RR> {
        &self.geometries
    }

    pub fn geometries_mut(&mut self) -> &mut DefaultResourcePool<Geometry, RR> {
        &mut self.geometries
    }

    pub fn clear_geometries(&mut self) {
        self.geometries.clear();
    }

    // Vertex methods
    pub fn vertices(&self) -> &Vertices<VR, C> {
        &self.vertices
    }

    pub fn vertices_mut(&mut self) -> &mut Vertices<VR, C> {
        &mut self.vertices
    }

    pub fn clear_vertices(&mut self) {
        self.vertices.clear();
    }

    pub fn add_vertex(&mut self, coordinate: C) -> Result<VertexIndex<VR>> {
        self.vertices.push(coordinate)
    }

    pub fn get_vertex(&self, index: VertexIndex<VR>) -> Option<&C> {
        self.vertices.get(index)
    }

    // Metadata methods
    pub fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    pub fn metadata_mut(&mut self) -> &mut Metadata
    where
        Metadata: Default,
    {
        if self.metadata.is_none() {
            self.metadata = Some(Metadata::default());
        }
        self.metadata.as_mut().unwrap()
    }

    // Extra methods
    pub fn extra(&self) -> Option<&Attributes<SS, RR>> {
        self.extra.as_ref()
    }

    pub fn extra_mut(&mut self) -> &mut Attributes<SS, RR> {
        if self.extra.is_none() {
            self.extra = Some(Attributes::new());
        }
        self.extra.as_mut().unwrap()
    }

    // Transform methods
    pub fn transform(&self) -> Option<&Transform> {
        self.transform.as_ref()
    }

    pub fn transform_mut(&mut self) -> &mut Transform
    where
        Transform: Default,
    {
        if self.transform.is_none() {
            self.transform = Some(Transform::default());
        }
        self.transform.as_mut().unwrap()
    }

    // Extensions methods
    pub fn extensions(&self) -> Option<&Extensions> {
        self.extensions.as_ref()
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions
    where
        Extensions: Default,
    {
        if self.extensions.is_none() {
            self.extensions = Some(Extensions::default());
        }
        self.extensions.as_mut().unwrap()
    }

    // CityObjects methods
    pub fn cityobjects(&self) -> &CityObjects {
        &self.cityobjects
    }

    pub fn cityobjects_mut(&mut self) -> &mut CityObjects {
        &mut self.cityobjects
    }

    // UV coordinate methods
    pub fn add_uv_coordinate(&mut self, uvcoordinate: UVCoordinate) -> Result<VertexIndex<VR>> {
        self.vertices_texture.push(uvcoordinate)
    }

    pub fn get_uv_coordinate(&self, index: VertexIndex<VR>) -> Option<&UVCoordinate> {
        self.vertices_texture.get(index)
    }

    pub fn vertices_texture(&self) -> &Vertices<VR, UVCoordinate> {
        &self.vertices_texture
    }

    pub fn vertices_texture_mut(&mut self) -> &mut Vertices<VR, UVCoordinate> {
        &mut self.vertices_texture
    }

    // Template vertex methods
    pub fn add_template_vertex(
        &mut self,
        coordinate: RealWorldCoordinate,
    ) -> Result<VertexIndex<VR>> {
        self.template_vertices.push(coordinate)
    }

    pub fn get_template_vertex(&self, index: VertexIndex<VR>) -> Option<&RealWorldCoordinate> {
        self.template_vertices.get(index)
    }

    pub fn template_vertices(&self) -> &Vertices<VR, RealWorldCoordinate> {
        &self.template_vertices
    }

    pub fn template_vertices_mut(&mut self) -> &mut Vertices<VR, RealWorldCoordinate> {
        &mut self.template_vertices
    }

    pub fn clear_template_vertices(&mut self) {
        self.template_vertices.clear();
    }

    // Template geometry methods
    pub fn add_template_geometry(&mut self, geometry: Geometry) -> RR {
        self.template_geometries.add(geometry)
    }

    pub fn template_geometries(&self) -> &DefaultResourcePool<Geometry, RR> {
        &self.template_geometries
    }

    pub fn template_geometries_mut(&mut self) -> &mut DefaultResourcePool<Geometry, RR> {
        &mut self.template_geometries
    }

    // Type and version methods
    pub fn type_citymodel(&self) -> CityModelType {
        self.type_citymodel
    }

    pub fn version(&self) -> Option<CityJSONVersion> {
        self.version
    }

    // Appearance theme methods
    pub fn default_theme_material(&self) -> Option<RR> {
        self.default_theme_material
    }

    pub fn set_default_theme_material(&mut self, material_ref: Option<RR>) {
        self.default_theme_material = material_ref;
    }

    pub fn default_theme_texture(&self) -> Option<RR> {
        self.default_theme_texture
    }

    pub fn set_default_theme_texture(&mut self, texture_ref: Option<RR>) {
        self.default_theme_texture = texture_ref;
    }
}
