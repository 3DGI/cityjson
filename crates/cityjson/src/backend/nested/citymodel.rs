//! CityModel type for the nested backend.
//!

use crate::Error;
use crate::backend::nested::appearance::{Appearance, Material, Texture};
use crate::backend::nested::attributes::Attributes;
use crate::backend::nested::cityobject::{CityObject, CityObjects};
use crate::backend::nested::coordinate::Vertices;
use crate::backend::nested::geometry::{Geometry, GeometryTemplates};
use crate::prelude::{
    CityJSONVersion, CityModelType, QuantizedCoordinate, RealWorldCoordinate, StringStorage,
    UVCoordinate, VertexIndex,
};
use crate::v2_0::extension::Extensions;
use crate::v2_0::metadata::Metadata;
use crate::v2_0::transform::Transform;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CityModel<SS: StringStorage> {
    id: Option<SS::String>,
    type_cm: CityModelType,
    version: Option<CityJSONVersion>,
    transform: Option<Transform>,
    cityobjects: CityObjects<SS>,
    metadata: Option<Metadata<SS>>,
    appearance: Option<Appearance<SS>>,
    geometry_templates: Option<GeometryTemplates<SS>>,
    extra: Option<Attributes<SS>>,
    extensions: Option<Extensions<SS>>,
    vertices: Vertices<u32, QuantizedCoordinate>,
    vertices_texture: Vertices<u32, UVCoordinate>,
    vertices_template: Vertices<u32, RealWorldCoordinate>,
}

impl<SS: StringStorage> CityModel<SS> {
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
            vertices: Vertices::new(),
            vertices_texture: Vertices::new(),
            vertices_template: Vertices::new(),
        }
    }

    pub fn with_capacity(
        type_citymodel: CityModelType,
        cityobjects_capacity: usize,
        vertex_capacity: usize,
        material_capacity: usize,
        texture_capacity: usize,
    ) -> Self {
        let mut appearance = Appearance::default();
        appearance.materials = Some(Vec::with_capacity(material_capacity));
        appearance.textures = Some(Vec::with_capacity(texture_capacity));

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
            vertices: Vertices::with_capacity(vertex_capacity),
            vertices_texture: Vertices::new(),
            vertices_template: Vertices::new(),
        }
    }

    // ========== Vertex Management (Regular) ==========

    pub fn add_vertex(
        &mut self,
        coordinate: QuantizedCoordinate,
    ) -> Result<VertexIndex<u32>, Error> {
        self.vertices.push(coordinate)
    }

    pub fn get_vertex(&self, index: VertexIndex<u32>) -> Option<&QuantizedCoordinate> {
        self.vertices.get(index)
    }

    pub fn vertices(&self) -> &Vertices<u32, QuantizedCoordinate> {
        &self.vertices
    }

    pub fn vertices_mut(&mut self) -> &mut Vertices<u32, QuantizedCoordinate> {
        &mut self.vertices
    }

    pub fn clear_vertices(&mut self) {
        self.vertices.clear();
    }

    // ========== UV Coordinate Management ==========

    pub fn add_uv_coordinate(
        &mut self,
        uvcoordinate: UVCoordinate,
    ) -> Result<VertexIndex<u32>, Error> {
        self.vertices_texture.push(uvcoordinate)
    }

    pub fn get_uv_coordinate(&self, index: VertexIndex<u32>) -> Option<&UVCoordinate> {
        self.vertices_texture.get(index)
    }

    pub fn vertices_texture(&self) -> &Vertices<u32, UVCoordinate> {
        &self.vertices_texture
    }

    pub fn vertices_texture_mut(&mut self) -> &mut Vertices<u32, UVCoordinate> {
        &mut self.vertices_texture
    }

    // ========== Template Vertex Management ==========

    pub fn add_template_vertex(
        &mut self,
        coordinate: RealWorldCoordinate,
    ) -> Result<VertexIndex<u32>, Error> {
        self.vertices_template.push(coordinate)
    }

    pub fn get_template_vertex(&self, index: VertexIndex<u32>) -> Option<&RealWorldCoordinate> {
        self.vertices_template.get(index)
    }

    pub fn template_vertices(&self) -> &Vertices<u32, RealWorldCoordinate> {
        &self.vertices_template
    }

    pub fn template_vertices_mut(&mut self) -> &mut Vertices<u32, RealWorldCoordinate> {
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

    pub fn default_theme_material(&self) -> Option<&SS> {
        self.appearance.as_ref()?.default_theme_material.as_ref()
    }

    pub fn set_default_theme_material(&mut self, theme: Option<SS>) {
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

    pub fn default_theme_texture(&self) -> Option<&SS> {
        self.appearance.as_ref()?.default_theme_texture.as_ref()
    }

    pub fn set_default_theme_texture(&mut self, theme: Option<SS>) {
        if self.appearance.is_none() {
            self.appearance = Some(Appearance::default());
        }
        self.appearance.as_mut().unwrap().default_theme_texture = theme;
    }

    // ========== Geometries Management ==========

    pub fn add_geometry_to_cityobject(
        &mut self,
        cityobject_id: &str,
        geometry: Geometry<SS>,
    ) -> Result<usize, Error> {
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
    ) -> Option<&Geometry<SS>> {
        self.cityobjects
            .get(cityobject_id)?
            .geometry()?
            .get(geometry_idx)
    }

    pub fn add_template_geometry(&mut self, geometry: Geometry<SS>) -> usize {
        if self.geometry_templates.is_none() {
            self.geometry_templates = Some(GeometryTemplates::default());
        }

        let templates = self.geometry_templates.as_mut().unwrap();
        let idx = templates.templates.len();
        templates.templates.push(geometry);
        idx
    }

    pub fn get_template_geometry(&self, idx: usize) -> Option<&Geometry<SS>> {
        self.geometry_templates.as_ref()?.templates.get(idx)
    }

    pub fn get_template_geometry_mut(&mut self, idx: usize) -> Option<&mut Geometry<SS>> {
        self.geometry_templates.as_mut()?.templates.get_mut(idx)
    }

    pub fn template_geometry_count(&self) -> usize {
        self.geometry_templates
            .as_ref()
            .map(|gt| gt.templates.len())
            .unwrap_or(0)
    }

    // ========== CityObjects Management ==========

    pub fn cityobjects(&self) -> &HashMap<String, CityObject<SS>> {
        &self.cityobjects
    }

    pub fn cityobjects_mut(&mut self) -> &mut HashMap<String, CityObject<SS>> {
        &mut self.cityobjects
    }

    pub fn add_cityobject(&mut self, id: String, cityobject: CityObject<SS>) {
        self.cityobjects.insert(id, cityobject);
    }

    pub fn get_cityobject(&self, id: &str) -> Option<&CityObject<SS>> {
        self.cityobjects.get(id)
    }

    pub fn get_cityobject_mut(&mut self, id: &str) -> Option<&mut CityObject<SS>> {
        self.cityobjects.get_mut(id)
    }

    pub fn clear_cityobjects(&mut self) {
        self.cityobjects.clear();
    }

    // ========== Metadata, Extensions, Transform ==========

    pub fn metadata(&self) -> Option<&Metadata<SS>> {
        self.metadata.as_ref()
    }

    pub fn metadata_mut(&mut self) -> &mut Metadata<SS> {
        self.metadata.get_or_insert_with(Metadata::default)
    }

    pub fn extensions(&self) -> Option<&Extensions<SS>> {
        self.extensions.as_ref()
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions<SS> {
        self.extensions.get_or_insert_with(Extensions::default)
    }

    pub fn extra(&self) -> Option<&Attributes<SS>> {
        self.extra.as_ref()
    }

    pub fn extra_mut(&mut self) -> &mut Attributes<SS> {
        self.extra.get_or_insert_with(Attributes::new)
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

    pub fn geometry_templates(&self) -> Option<&GeometryTemplates<SS>> {
        self.geometry_templates.as_ref()
    }

    pub fn geometry_templates_mut(&mut self) -> &mut GeometryTemplates<SS> {
        self.geometry_templates
            .get_or_insert_with(GeometryTemplates::default)
    }
}
