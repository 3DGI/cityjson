//! CityModel type for the nested backend.
//!

use crate::Error;
use crate::error::Result;
use crate::backend::nested::appearance::{Appearance, Material, Texture};
use crate::backend::nested::attributes::Attributes as NestedAttributes;
use crate::backend::nested::cityobject::{CityObject, CityObjects};
use crate::backend::nested::coordinate::Vertices as NestedVertices;
use crate::backend::nested::geometry::{Geometry, GeometryTemplates};
use crate::backend::nested::metadata::Metadata;
use crate::prelude::{
    CityJSONVersion, CityModelType, QuantizedCoordinate, RealWorldCoordinate as NestedRealWorldCoordinate,
    StringStorage, UVCoordinate as NestedUVCoordinate, VertexIndex as NestedVertexIndex,
};
use crate::v2_0::extension::Extensions;
use crate::v2_0::transform::Transform;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CityModel<SS: StringStorage, RR> {
    id: Option<SS::String>,
    type_cm: CityModelType,
    version: Option<CityJSONVersion>,
    transform: Option<Transform>,
    cityobjects: CityObjects<SS, RR>,
    metadata: Option<Metadata<SS, RR>>,
    appearance: Option<Appearance<SS>>,
    geometry_templates: Option<GeometryTemplates<SS, RR>>,
    extra: Option<NestedAttributes<SS, RR>>,
    extensions: Option<Extensions<SS>>,
    vertices: NestedVertices<u32, QuantizedCoordinate>,
    vertices_texture: NestedVertices<u32, NestedUVCoordinate>,
    vertices_template: NestedVertices<u32, NestedRealWorldCoordinate>,
}

impl<SS: StringStorage, RR> CityModel<SS, RR> {
    // ========== Constructors ==========

    pub fn new(type_citymodel: CityModelType) -> Self {
        Self {
            id: None,
            type_cm: type_citymodel,
            version: Some(CityJSONVersion::V2_0),
            transform: None,
            cityobjects: HashMap::new(),
            metadata: None,
            appearance: None,
            geometry_templates: None,
            extra: None,
            extensions: None,
            vertices: NestedVertices::new(),
            vertices_texture: NestedVertices::new(),
            vertices_template: NestedVertices::new(),
        }
    }

    pub fn with_capacity(
        type_citymodel: CityModelType,
        cityobjects_capacity: usize,
        vertex_capacity: usize,
        material_capacity: usize,
        texture_capacity: usize,
    ) -> Self {
        let appearance = Appearance {
            materials: Some(Vec::with_capacity(material_capacity)),
            textures: Some(Vec::with_capacity(texture_capacity)),
            ..Default::default()
        };

        Self {
            id: None,
            type_cm: type_citymodel,
            version: Some(CityJSONVersion::V2_0),
            transform: None,
            cityobjects: HashMap::with_capacity(cityobjects_capacity),
            metadata: None,
            appearance: Some(appearance),
            geometry_templates: None,
            extra: None,
            extensions: None,
            vertices: NestedVertices::with_capacity(vertex_capacity),
            vertices_texture: NestedVertices::new(),
            vertices_template: NestedVertices::new(),
        }
    }

    // ========== Vertex Management (Regular) ==========

    pub fn add_vertex(
        &mut self,
        coordinate: QuantizedCoordinate,
    ) -> Result<NestedVertexIndex<u32>> {
        self.vertices.push(coordinate)
    }

    pub fn get_vertex(&self, index: NestedVertexIndex<u32>) -> Option<&QuantizedCoordinate> {
        self.vertices.get(index)
    }

    pub fn vertices(&self) -> &NestedVertices<u32, QuantizedCoordinate> {
        &self.vertices
    }

    pub fn vertices_mut(&mut self) -> &mut NestedVertices<u32, QuantizedCoordinate> {
        &mut self.vertices
    }

    pub fn clear_vertices(&mut self) {
        self.vertices.clear();
    }

    // ========== UV Coordinate Management ==========

    pub fn add_uv_coordinate(
        &mut self,
        uvcoordinate: NestedUVCoordinate,
    ) -> Result<NestedVertexIndex<u32>> {
        self.vertices_texture.push(uvcoordinate)
    }

    pub fn get_uv_coordinate(&self, index: NestedVertexIndex<u32>) -> Option<&NestedUVCoordinate> {
        self.vertices_texture.get(index)
    }

    pub fn vertices_texture(&self) -> &NestedVertices<u32, NestedUVCoordinate> {
        &self.vertices_texture
    }

    pub fn vertices_texture_mut(&mut self) -> &mut NestedVertices<u32, NestedUVCoordinate> {
        &mut self.vertices_texture
    }

    // ========== Template Vertex Management ==========

    pub fn add_template_vertex(
        &mut self,
        coordinate: NestedRealWorldCoordinate,
    ) -> Result<NestedVertexIndex<u32>> {
        self.vertices_template.push(coordinate)
    }

    pub fn get_template_vertex(
        &self,
        index: NestedVertexIndex<u32>,
    ) -> Option<&NestedRealWorldCoordinate> {
        self.vertices_template.get(index)
    }

    pub fn template_vertices(&self) -> &NestedVertices<u32, NestedRealWorldCoordinate> {
        &self.vertices_template
    }

    pub fn template_vertices_mut(
        &mut self,
    ) -> &mut NestedVertices<u32, NestedRealWorldCoordinate> {
        &mut self.vertices_template
    }

    pub fn clear_template_vertices(&mut self) {
        self.vertices_template.clear();
    }

    // ========== Materials Management ==========

    pub fn add_material(&mut self, material: Material<SS>) -> usize {
        // Auto-initialize appearance if needed
        if self.appearance.is_none() {
            self.appearance = Some(Appearance::default());
        }

        let appearance = self.appearance.as_mut().unwrap();

        // Auto-initialize materials vector if needed
        if appearance.materials.is_none() {
            appearance.materials = Some(Vec::new());
        }

        let materials = appearance.materials.as_mut().unwrap();
        let idx = materials.len();
        materials.push(material);
        idx
    }

    pub fn get_material(&self, idx: usize) -> Option<&Material<SS>> {
        self.appearance.as_ref()?.materials.as_ref()?.get(idx)
    }

    pub fn get_material_mut(&mut self, idx: usize) -> Option<&mut Material<SS>> {
        self.appearance.as_mut()?.materials.as_mut()?.get_mut(idx)
    }

    pub fn find_material(&self, material: &Material<SS>) -> Option<usize> {
        self.appearance
            .as_ref()?
            .materials
            .as_ref()?
            .iter()
            .position(|m| m == material)
    }

    pub fn material_count(&self) -> usize {
        self.appearance
            .as_ref()
            .and_then(|a| a.materials.as_ref())
            .map(|m| m.len())
            .unwrap_or(0)
    }

    pub fn iter_materials(&self) -> impl Iterator<Item = (usize, &Material<SS>)> {
        self.appearance
            .as_ref()
            .and_then(|a| a.materials.as_ref())
            .into_iter()
            .flat_map(|materials| materials.iter().enumerate())
    }

    pub fn iter_materials_mut(&mut self) -> impl Iterator<Item = (usize, &mut Material<SS>)> {
        self.appearance
            .as_mut()
            .and_then(|a| a.materials.as_mut())
            .into_iter()
            .flat_map(|materials| materials.iter_mut().enumerate())
    }

    pub fn default_theme_material(&self) -> Option<&SS::String> {
        self.appearance.as_ref()?.default_theme_material.as_ref()
    }

    pub fn set_default_theme_material(&mut self, theme: Option<SS::String>) {
        if self.appearance.is_none() {
            self.appearance = Some(Appearance::default());
        }
        self.appearance.as_mut().unwrap().default_theme_material = theme;
    }

    // ========== Textures Management ==========

    pub fn add_texture(&mut self, texture: Texture<SS>) -> usize {
        // Auto-initialize appearance if needed
        if self.appearance.is_none() {
            self.appearance = Some(Appearance::default());
        }

        let appearance = self.appearance.as_mut().unwrap();

        // Auto-initialize textures vector if needed
        if appearance.textures.is_none() {
            appearance.textures = Some(Vec::new());
        }

        let textures = appearance.textures.as_mut().unwrap();
        let idx = textures.len();
        textures.push(texture);
        idx
    }

    pub fn get_texture(&self, idx: usize) -> Option<&Texture<SS>> {
        self.appearance.as_ref()?.textures.as_ref()?.get(idx)
    }

    pub fn get_texture_mut(&mut self, idx: usize) -> Option<&mut Texture<SS>> {
        self.appearance.as_mut()?.textures.as_mut()?.get_mut(idx)
    }

    pub fn find_texture(&self, texture: &Texture<SS>) -> Option<usize> {
        self.appearance
            .as_ref()?
            .textures
            .as_ref()?
            .iter()
            .position(|t| t == texture)
    }

    pub fn texture_count(&self) -> usize {
        self.appearance
            .as_ref()
            .and_then(|a| a.textures.as_ref())
            .map(|t| t.len())
            .unwrap_or(0)
    }

    pub fn iter_textures(&self) -> impl Iterator<Item = (usize, &Texture<SS>)> {
        self.appearance
            .as_ref()
            .and_then(|a| a.textures.as_ref())
            .into_iter()
            .flat_map(|textures| textures.iter().enumerate())
    }

    pub fn default_theme_texture(&self) -> Option<&SS::String> {
        self.appearance.as_ref()?.default_theme_texture.as_ref()
    }

    pub fn set_default_theme_texture(&mut self, theme: Option<SS::String>) {
        if self.appearance.is_none() {
            self.appearance = Some(Appearance::default());
        }
        self.appearance.as_mut().unwrap().default_theme_texture = theme;
    }

    // ========== Geometries Management ==========

    pub fn add_geometry_to_cityobject(
        &mut self,
        cityobject_id: &str,
        geometry: Geometry<SS, RR>,
    ) -> Result<usize> {
        let cityobject = self.cityobjects.get_mut(cityobject_id).ok_or_else(|| {
            Error::InvalidGeometry(format!("CityObject not found: {}", cityobject_id))
        })?;

        let geometries = cityobject.geometry_mut();
        let idx = geometries.len();
        geometries.push(geometry);
        Ok(idx)
    }

    pub fn get_geometry_from_cityobject(
        &self,
        cityobject_id: &str,
        geometry_idx: usize,
    ) -> Option<&Geometry<SS, RR>> {
        self.cityobjects
            .get(cityobject_id)?
            .geometry()?
            .get(geometry_idx)
    }

    pub fn add_template_geometry(&mut self, geometry: Geometry<SS, RR>) -> usize {
        if self.geometry_templates.is_none() {
            self.geometry_templates = Some(GeometryTemplates::default());
        }

        let templates = self.geometry_templates.as_mut().unwrap();
        let idx = templates.templates.len();
        templates.templates.push(geometry);
        idx
    }

    pub fn get_template_geometry(&self, idx: usize) -> Option<&Geometry<SS, RR>> {
        self.geometry_templates.as_ref()?.templates.get(idx)
    }

    pub fn get_template_geometry_mut(&mut self, idx: usize) -> Option<&mut Geometry<SS, RR>> {
        self.geometry_templates.as_mut()?.templates.get_mut(idx)
    }

    pub fn template_geometry_count(&self) -> usize {
        self.geometry_templates
            .as_ref()
            .map(|gt| gt.templates.len())
            .unwrap_or(0)
    }

    // ========== CityObjects Management ==========

    pub fn cityobjects(&self) -> &HashMap<String, CityObject<SS, RR>> {
        &self.cityobjects
    }

    pub fn cityobjects_mut(&mut self) -> &mut HashMap<String, CityObject<SS, RR>> {
        &mut self.cityobjects
    }

    pub fn add_cityobject(&mut self, id: String, cityobject: CityObject<SS, RR>) {
        self.cityobjects.insert(id, cityobject);
    }

    pub fn get_cityobject(&self, id: &str) -> Option<&CityObject<SS, RR>> {
        self.cityobjects.get(id)
    }

    pub fn get_cityobject_mut(&mut self, id: &str) -> Option<&mut CityObject<SS, RR>> {
        self.cityobjects.get_mut(id)
    }

    pub fn clear_cityobjects(&mut self) {
        self.cityobjects.clear();
    }

    // ========== Metadata, Extensions, Transform ==========

    pub fn metadata(&self) -> Option<&Metadata<SS, RR>> {
        self.metadata.as_ref()
    }

    pub fn metadata_mut(&mut self) -> &mut Metadata<SS, RR> {
        self.metadata.get_or_insert_with(Metadata::default)
    }

    pub fn extensions(&self) -> Option<&Extensions<SS>> {
        self.extensions.as_ref()
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions<SS> {
        self.extensions.get_or_insert_with(Extensions::default)
    }

    pub fn extra(&self) -> Option<&NestedAttributes<SS, RR>> {
        self.extra.as_ref()
    }

    pub fn extra_mut(&mut self) -> &mut NestedAttributes<SS, RR> {
        self.extra.get_or_insert_with(NestedAttributes::new)
    }

    pub fn transform(&self) -> Option<&Transform> {
        self.transform.as_ref()
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        self.transform.get_or_insert_with(Transform::default)
    }

    // ========== Model Metadata ==========

    pub fn type_citymodel(&self) -> &CityModelType {
        &self.type_cm
    }

    pub fn version(&self) -> Option<CityJSONVersion> {
        self.version
    }

    pub fn id(&self) -> Option<&SS::String> {
        self.id.as_ref()
    }

    pub fn set_id(&mut self, id: Option<SS::String>) {
        self.id = id;
    }

    pub fn appearance(&self) -> Option<&Appearance<SS>> {
        self.appearance.as_ref()
    }

    pub fn appearance_mut(&mut self) -> &mut Appearance<SS> {
        self.appearance.get_or_insert_with(Appearance::default)
    }

    pub fn geometry_templates(&self) -> Option<&GeometryTemplates<SS, RR>> {
        self.geometry_templates.as_ref()
    }

    pub fn geometry_templates_mut(&mut self) -> &mut GeometryTemplates<SS, RR> {
        self.geometry_templates
            .get_or_insert_with(GeometryTemplates::default)
    }
}

// ==================== CORE TYPES FOR VERSIONED API ====================

use crate::cityjson::core::attributes::Attributes;
use crate::cityjson::core::coordinate::{UVCoordinate, Vertices};
use crate::cityjson::core::vertex::{VertexIndex, VertexRef};
use crate::cityjson::traits::coordinate::Coordinate;
use crate::prelude::RealWorldCoordinate;
use crate::resources::pool::{DefaultResourcePool, ResourcePool, ResourceRef};

/// Core CityModel structure that is shared across all CityJSON versions.
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
    type_citymodel: CityModelType,
    version: Option<CityJSONVersion>,
    extensions: Option<Extensions>,
    extra: Option<Attributes<SS, RR>>,
    metadata: Option<Metadata>,
    cityobjects: CityObjects,
    transform: Option<Transform>,
    vertices: Vertices<VR, C>,
    geometries: DefaultResourcePool<Geometry, RR>,
    template_vertices: Vertices<VR, RealWorldCoordinate>,
    template_geometries: DefaultResourcePool<Geometry, RR>,
    semantics: DefaultResourcePool<Semantic, RR>,
    materials: DefaultResourcePool<Material, RR>,
    textures: DefaultResourcePool<Texture, RR>,
    vertices_texture: Vertices<VR, UVCoordinate>,
    default_theme_material: Option<RR>,
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

    // ==================== SEMANTICS ====================

    pub fn get_semantic(&self, id: RR) -> Option<&Semantic> {
        self.semantics.get(id)
    }

    pub fn get_semantic_mut(&mut self, id: RR) -> Option<&mut Semantic> {
        self.semantics.get_mut(id)
    }

    pub fn add_semantic(&mut self, semantic: Semantic) -> RR {
        self.semantics.add(semantic)
    }

    pub fn semantic_count(&self) -> usize {
        self.semantics.len()
    }

    pub fn has_semantics(&self) -> bool {
        !self.semantics.is_empty()
    }

    pub fn iter_semantics(&self) -> impl Iterator<Item = (RR, &Semantic)> + '_ {
        self.semantics.iter()
    }

    pub fn iter_semantics_mut(&mut self) -> impl Iterator<Item = (RR, &mut Semantic)> + '_ {
        self.semantics.iter_mut()
    }

    pub fn find_semantic(&self, semantic: &Semantic) -> Option<RR>
    where
        Semantic: PartialEq,
    {
        self.semantics.find(semantic)
    }

    pub fn remove_semantic(&mut self, id: RR) -> Option<Semantic> {
        self.semantics.remove(id)
    }

    pub fn clear_semantics(&mut self) {
        self.semantics.clear();
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

    // ==================== MATERIALS ====================

    pub fn get_material(&self, id: RR) -> Option<&Material> {
        self.materials.get(id)
    }

    pub fn get_material_mut(&mut self, id: RR) -> Option<&mut Material> {
        self.materials.get_mut(id)
    }

    pub fn add_material(&mut self, material: Material) -> RR {
        self.materials.add(material)
    }

    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    pub fn iter_materials(&self) -> impl Iterator<Item = (RR, &Material)> + '_ {
        self.materials.iter()
    }

    pub fn iter_materials_mut(&mut self) -> impl Iterator<Item = (RR, &mut Material)> + '_ {
        self.materials.iter_mut()
    }

    pub fn find_material(&self, material: &Material) -> Option<RR>
    where
        Material: PartialEq,
    {
        self.materials.find(material)
    }

    pub fn remove_material(&mut self, id: RR) -> Option<Material> {
        self.materials.remove(id)
    }

    pub fn clear_materials(&mut self) {
        self.materials.clear();
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

    // ==================== TEXTURES ====================

    pub fn get_texture(&self, id: RR) -> Option<&Texture> {
        self.textures.get(id)
    }

    pub fn get_texture_mut(&mut self, id: RR) -> Option<&mut Texture> {
        self.textures.get_mut(id)
    }

    pub fn add_texture(&mut self, texture: Texture) -> RR {
        self.textures.add(texture)
    }

    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    pub fn iter_textures(&self) -> impl Iterator<Item = (RR, &Texture)> + '_ {
        self.textures.iter()
    }

    pub fn iter_textures_mut(&mut self) -> impl Iterator<Item = (RR, &mut Texture)> + '_ {
        self.textures.iter_mut()
    }

    pub fn find_texture(&self, texture: &Texture) -> Option<RR>
    where
        Texture: PartialEq,
    {
        self.textures.find(texture)
    }

    pub fn remove_texture(&mut self, id: RR) -> Option<Texture> {
        self.textures.remove(id)
    }

    pub fn clear_textures(&mut self) {
        self.textures.clear();
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

    // ==================== GEOMETRIES ====================

    pub fn get_geometry(&self, id: RR) -> Option<&Geometry> {
        self.geometries.get(id)
    }

    pub fn get_geometry_mut(&mut self, id: RR) -> Option<&mut Geometry> {
        self.geometries.get_mut(id)
    }

    pub fn add_geometry(&mut self, geometry: Geometry) -> RR {
        self.geometries.add(geometry)
    }

    pub fn geometry_count(&self) -> usize {
        self.geometries.len()
    }

    pub fn iter_geometries(&self) -> impl Iterator<Item = (RR, &Geometry)> + '_ {
        self.geometries.iter()
    }

    pub fn iter_geometries_mut(&mut self) -> impl Iterator<Item = (RR, &mut Geometry)> + '_ {
        self.geometries.iter_mut()
    }

    pub fn remove_geometry(&mut self, id: RR) -> Option<Geometry> {
        self.geometries.remove(id)
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

    // ==================== TEMPLATE GEOMETRIES ====================

    pub fn get_template_geometry(&self, id: RR) -> Option<&Geometry> {
        self.template_geometries.get(id)
    }

    pub fn get_template_geometry_mut(&mut self, id: RR) -> Option<&mut Geometry> {
        self.template_geometries.get_mut(id)
    }

    pub fn add_template_geometry(&mut self, geometry: Geometry) -> RR {
        self.template_geometries.add(geometry)
    }

    pub fn template_geometry_count(&self) -> usize {
        self.template_geometries.len()
    }

    pub fn iter_template_geometries(&self) -> impl Iterator<Item = (RR, &Geometry)> + '_ {
        self.template_geometries.iter()
    }

    pub fn iter_template_geometries_mut(
        &mut self,
    ) -> impl Iterator<Item = (RR, &mut Geometry)> + '_ {
        self.template_geometries.iter_mut()
    }

    pub fn remove_template_geometry(&mut self, id: RR) -> Option<Geometry> {
        self.template_geometries.remove(id)
    }

    pub fn clear_template_geometries(&mut self) {
        self.template_geometries.clear();
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
