use crate::backend::default::geometry::{
    GeometryBuilder as RawGeometryBuilder, GeometryModelOps,
};
use crate::cityjson::core::coordinate::{UVCoordinate, Vertices};
use crate::cityjson::core::vertex::{VertexIndex, VertexRef};
use crate::prelude::{QuantizedCoordinate, RealWorldCoordinate, Result};
use crate::raw::{RawAccess, RawPoolView, RawSliceView};
use crate::resources::handles::{
    CityObjectRef, GeometryRef, MaterialRef, SemanticRef, TemplateGeometryRef, TextureRef,
};
use crate::resources::pool::ResourceId32;
use crate::resources::storage::{OwnedStringStorage, StringStorage};
use crate::v2_0::appearance::material::Material;
use crate::v2_0::appearance::texture::Texture;
use crate::v2_0::geometry::semantic::Semantic;
use crate::v2_0::geometry::Geometry;
use crate::v2_0::metadata::Metadata;
use crate::v2_0::{CityObjects, Extensions, Transform};
use crate::{format_option, CityJSONVersion};
use std::collections::HashSet;
use std::fmt;

pub type GeometryBuilder<'a, VR, SS> = RawGeometryBuilder<
    'a,
    VR,
    ResourceId32,
    QuantizedCoordinate,
    Semantic<SS>,
    Material<SS>,
    Texture<SS>,
    Geometry<VR, SS>,
    CityModel<VR, SS>,
    SS,
>;

pub trait GeometryBuilderExt<VR: VertexRef, SS: StringStorage> {
    /// Use a typed template handle when building a `GeometryInstance`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::InvalidGeometryType`] if the builder is not configured
    /// for `GeometryType::GeometryInstance`.
    fn with_template_ref(self, template_ref: TemplateGeometryRef) -> Result<Self>
    where
        Self: Sized;

    /// Build and return a typed regular-geometry handle.
    ///
    /// # Errors
    ///
    /// Propagates errors from geometry validation and storage insertion.
    fn build_geometry(self) -> Result<GeometryRef>;

    /// Build and return a typed template-geometry handle.
    ///
    /// # Errors
    ///
    /// Propagates errors from geometry validation and storage insertion.
    fn build_template(self) -> Result<TemplateGeometryRef>;
}

impl<VR: VertexRef, SS: StringStorage> GeometryBuilderExt<VR, SS> for GeometryBuilder<'_, VR, SS> {
    fn with_template_ref(self, template_ref: TemplateGeometryRef) -> Result<Self> {
        self.with_template(template_ref.to_raw())
    }

    fn build_geometry(self) -> Result<GeometryRef> {
        self.build().map(GeometryRef::from_raw)
    }

    fn build_template(self) -> Result<TemplateGeometryRef> {
        self.build().map(TemplateGeometryRef::from_raw)
    }
}

#[derive(Debug, Clone)]
pub struct CityModel<VR: VertexRef = u32, SS: StringStorage = OwnedStringStorage> {
    #[allow(clippy::type_complexity)]
    inner: crate::cityjson::core::citymodel::CityModelCore<
        QuantizedCoordinate,
        VR,
        ResourceId32,
        SS,
        Semantic<SS>,
        Material<SS>,
        Texture<SS>,
        Geometry<VR, SS>,
        Metadata<SS>,
        Transform,
        Extensions<SS>,
        CityObjects<SS>,
    >,
}

impl<VR: VertexRef, SS: StringStorage> CityModel<VR, SS> {
    #[must_use]
    pub fn new(type_citymodel: crate::CityModelType) -> Self {
        Self {
            inner: crate::cityjson::core::citymodel::CityModelCore::new(
                type_citymodel,
                Some(CityJSONVersion::V2_0),
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_capacity(
        type_citymodel: crate::CityModelType,
        cityobjects_capacity: usize,
        vertex_capacity: usize,
        semantic_capacity: usize,
        material_capacity: usize,
        texture_capacity: usize,
        geometry_capacity: usize,
    ) -> Self {
        Self {
            inner: crate::cityjson::core::citymodel::CityModelCore::with_capacity(
                type_citymodel,
                Some(CityJSONVersion::V2_0),
                cityobjects_capacity,
                vertex_capacity,
                semantic_capacity,
                material_capacity,
                texture_capacity,
                geometry_capacity,
                CityObjects::with_capacity,
            ),
        }
    }

    pub fn get_semantic(&self, id: SemanticRef) -> Option<&Semantic<SS>> {
        self.inner.get_semantic(id.to_raw())
    }

    pub fn get_semantic_mut(&mut self, id: SemanticRef) -> Option<&mut Semantic<SS>> {
        self.inner.get_semantic_mut(id.to_raw())
    }

    /// Add a semantic and return its handle.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when the semantic pool cannot store
    /// additional entries for `ResourceId32`.
    pub fn add_semantic(&mut self, semantic: Semantic<SS>) -> Result<SemanticRef> {
        self.inner.add_semantic(semantic).map(SemanticRef::from_raw)
    }

    pub fn semantic_count(&self) -> usize {
        self.inner.semantic_count()
    }

    pub fn has_semantics(&self) -> bool {
        self.inner.has_semantics()
    }

    pub fn iter_semantics(&self) -> impl Iterator<Item = (SemanticRef, &Semantic<SS>)> + '_ {
        self.inner
            .iter_semantics()
            .map(|(id, v)| (SemanticRef::from_raw(id), v))
    }

    pub fn iter_semantics_mut(
        &mut self,
    ) -> impl Iterator<Item = (SemanticRef, &mut Semantic<SS>)> + '_ {
        self.inner
            .iter_semantics_mut()
            .map(|(id, v)| (SemanticRef::from_raw(id), v))
    }

    pub fn find_semantic(&self, semantic: &Semantic<SS>) -> Option<SemanticRef>
    where
        Semantic<SS>: PartialEq,
    {
        self.inner
            .find_semantic(semantic)
            .map(SemanticRef::from_raw)
    }

    pub fn remove_semantic(&mut self, id: SemanticRef) -> Option<Semantic<SS>> {
        self.inner.remove_semantic(id.to_raw())
    }

    pub fn clear_semantics(&mut self) {
        self.inner.clear_semantics();
    }

    /// Return an existing semantic handle or insert a new semantic.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when inserting a new semantic exceeds
    /// the semantic pool capacity.
    pub fn get_or_insert_semantic(&mut self, semantic: Semantic<SS>) -> Result<SemanticRef>
    where
        Semantic<SS>: PartialEq,
    {
        self.inner
            .get_or_insert_semantic(semantic)
            .map(SemanticRef::from_raw)
    }

    pub fn get_material(&self, id: MaterialRef) -> Option<&Material<SS>> {
        self.inner.get_material(id.to_raw())
    }

    pub fn get_material_mut(&mut self, id: MaterialRef) -> Option<&mut Material<SS>> {
        self.inner.get_material_mut(id.to_raw())
    }

    /// Add a material and return its handle.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when the material pool cannot store
    /// additional entries for `ResourceId32`.
    pub fn add_material(&mut self, material: Material<SS>) -> Result<MaterialRef> {
        self.inner.add_material(material).map(MaterialRef::from_raw)
    }

    pub fn material_count(&self) -> usize {
        self.inner.material_count()
    }

    pub fn iter_materials(&self) -> impl Iterator<Item = (MaterialRef, &Material<SS>)> + '_ {
        self.inner
            .iter_materials()
            .map(|(id, v)| (MaterialRef::from_raw(id), v))
    }

    pub fn iter_materials_mut(
        &mut self,
    ) -> impl Iterator<Item = (MaterialRef, &mut Material<SS>)> + '_ {
        self.inner
            .iter_materials_mut()
            .map(|(id, v)| (MaterialRef::from_raw(id), v))
    }

    pub fn find_material(&self, material: &Material<SS>) -> Option<MaterialRef>
    where
        Material<SS>: PartialEq,
    {
        self.inner
            .find_material(material)
            .map(MaterialRef::from_raw)
    }

    pub fn remove_material(&mut self, id: MaterialRef) -> Option<Material<SS>> {
        self.inner.remove_material(id.to_raw())
    }

    pub fn clear_materials(&mut self) {
        self.inner.clear_materials();
    }

    /// Return an existing material handle or insert a new material.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when inserting a new material exceeds
    /// the material pool capacity.
    pub fn get_or_insert_material(&mut self, material: Material<SS>) -> Result<MaterialRef>
    where
        Material<SS>: PartialEq,
    {
        self.inner
            .get_or_insert_material(material)
            .map(MaterialRef::from_raw)
    }

    pub fn get_texture(&self, id: TextureRef) -> Option<&Texture<SS>> {
        self.inner.get_texture(id.to_raw())
    }

    pub fn get_texture_mut(&mut self, id: TextureRef) -> Option<&mut Texture<SS>> {
        self.inner.get_texture_mut(id.to_raw())
    }

    /// Add a texture and return its handle.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when the texture pool cannot store
    /// additional entries for `ResourceId32`.
    pub fn add_texture(&mut self, texture: Texture<SS>) -> Result<TextureRef> {
        self.inner.add_texture(texture).map(TextureRef::from_raw)
    }

    pub fn texture_count(&self) -> usize {
        self.inner.texture_count()
    }

    pub fn iter_textures(&self) -> impl Iterator<Item = (TextureRef, &Texture<SS>)> + '_ {
        self.inner
            .iter_textures()
            .map(|(id, v)| (TextureRef::from_raw(id), v))
    }

    pub fn iter_textures_mut(
        &mut self,
    ) -> impl Iterator<Item = (TextureRef, &mut Texture<SS>)> + '_ {
        self.inner
            .iter_textures_mut()
            .map(|(id, v)| (TextureRef::from_raw(id), v))
    }

    pub fn find_texture(&self, texture: &Texture<SS>) -> Option<TextureRef>
    where
        Texture<SS>: PartialEq,
    {
        self.inner.find_texture(texture).map(TextureRef::from_raw)
    }

    pub fn remove_texture(&mut self, id: TextureRef) -> Option<Texture<SS>> {
        self.inner.remove_texture(id.to_raw())
    }

    pub fn clear_textures(&mut self) {
        self.inner.clear_textures();
    }

    /// Return an existing texture handle or insert a new texture.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when inserting a new texture exceeds
    /// the texture pool capacity.
    pub fn get_or_insert_texture(&mut self, texture: Texture<SS>) -> Result<TextureRef>
    where
        Texture<SS>: PartialEq,
    {
        self.inner
            .get_or_insert_texture(texture)
            .map(TextureRef::from_raw)
    }

    pub fn get_geometry(&self, id: GeometryRef) -> Option<&Geometry<VR, SS>> {
        self.inner.get_geometry(id.to_raw())
    }

    pub fn get_geometry_mut(&mut self, id: GeometryRef) -> Option<&mut Geometry<VR, SS>> {
        self.inner.get_geometry_mut(id.to_raw())
    }

    /// Add a geometry and return its handle.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when the geometry pool cannot store
    /// additional entries for `ResourceId32`.
    pub fn add_geometry(&mut self, geometry: Geometry<VR, SS>) -> Result<GeometryRef> {
        self.inner.add_geometry(geometry).map(GeometryRef::from_raw)
    }

    pub fn geometry_count(&self) -> usize {
        self.inner.geometry_count()
    }

    pub fn iter_geometries(&self) -> impl Iterator<Item = (GeometryRef, &Geometry<VR, SS>)> + '_ {
        self.inner
            .iter_geometries()
            .map(|(id, v)| (GeometryRef::from_raw(id), v))
    }

    pub fn iter_geometries_mut(
        &mut self,
    ) -> impl Iterator<Item = (GeometryRef, &mut Geometry<VR, SS>)> + '_ {
        self.inner
            .iter_geometries_mut()
            .map(|(id, v)| (GeometryRef::from_raw(id), v))
    }

    pub fn remove_geometry(&mut self, id: GeometryRef) -> Option<Geometry<VR, SS>> {
        self.inner.remove_geometry(id.to_raw())
    }

    pub fn clear_geometries(&mut self) {
        self.inner.clear_geometries();
    }

    pub fn vertices(&self) -> &Vertices<VR, QuantizedCoordinate> {
        self.inner.vertices()
    }

    pub fn vertices_mut(&mut self) -> &mut Vertices<VR, QuantizedCoordinate> {
        self.inner.vertices_mut()
    }

    pub fn clear_vertices(&mut self) {
        self.inner.clear_vertices();
    }

    /// Add a quantized vertex and return its index.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::VerticesContainerFull`] when the quantized vertex
    /// container cannot represent more vertices for `VR`.
    pub fn add_vertex(
        &mut self,
        coordinate: QuantizedCoordinate,
    ) -> crate::error::Result<VertexIndex<VR>> {
        self.inner.add_vertex(coordinate)
    }

    pub fn get_vertex(&self, index: VertexIndex<VR>) -> Option<&QuantizedCoordinate> {
        self.inner.get_vertex(index)
    }

    pub fn metadata(&self) -> Option<&Metadata<SS>> {
        self.inner.metadata()
    }

    pub fn metadata_mut(&mut self) -> &mut Metadata<SS> {
        self.inner.metadata_mut()
    }

    pub fn extra(&self) -> Option<&crate::cityjson::core::attributes::Attributes<SS>> {
        self.inner.extra()
    }

    pub fn extra_mut(&mut self) -> &mut crate::cityjson::core::attributes::Attributes<SS> {
        self.inner.extra_mut()
    }

    pub fn transform(&self) -> Option<&Transform> {
        self.inner.transform()
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        self.inner.transform_mut()
    }

    pub fn extensions(&self) -> Option<&Extensions<SS>> {
        self.inner.extensions()
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions<SS> {
        self.inner.extensions_mut()
    }

    pub fn cityobjects(&self) -> &CityObjects<SS> {
        self.inner.cityobjects()
    }

    /// Returns a raw accessor for zero-copy reads of internal model pools.
    #[inline]
    pub fn raw(&self) -> CityModelRawAccessor<'_, VR, SS> {
        CityModelRawAccessor { model: self }
    }

    pub fn cityobjects_mut(&mut self) -> &mut CityObjects<SS> {
        self.inner.cityobjects_mut()
    }

    pub fn clear_cityobjects(&mut self) {
        self.inner.cityobjects_mut().clear();
    }

    /// Add a UV coordinate and return its vertex index.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::VerticesContainerFull`] when the UV-coordinate container
    /// cannot represent more vertices for `VR`.
    pub fn add_uv_coordinate(
        &mut self,
        uvcoordinate: UVCoordinate,
    ) -> crate::error::Result<VertexIndex<VR>> {
        self.inner.add_uv_coordinate(uvcoordinate)
    }

    pub fn get_uv_coordinate(&self, index: VertexIndex<VR>) -> Option<&UVCoordinate> {
        self.inner.get_uv_coordinate(index)
    }

    pub fn vertices_texture(&self) -> &Vertices<VR, UVCoordinate> {
        self.inner.vertices_texture()
    }

    pub fn vertices_texture_mut(&mut self) -> &mut Vertices<VR, UVCoordinate> {
        self.inner.vertices_texture_mut()
    }

    /// Add a template vertex and return its index.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::VerticesContainerFull`] when the template-vertex
    /// container cannot represent more vertices for `VR`.
    pub fn add_template_vertex(
        &mut self,
        coordinate: RealWorldCoordinate,
    ) -> crate::error::Result<VertexIndex<VR>> {
        self.inner.add_template_vertex(coordinate)
    }

    pub fn get_template_vertex(&self, index: VertexIndex<VR>) -> Option<&RealWorldCoordinate> {
        self.inner.get_template_vertex(index)
    }

    pub fn template_vertices(&self) -> &Vertices<VR, RealWorldCoordinate> {
        self.inner.template_vertices()
    }

    pub fn template_vertices_mut(&mut self) -> &mut Vertices<VR, RealWorldCoordinate> {
        self.inner.template_vertices_mut()
    }

    pub fn clear_template_vertices(&mut self) {
        self.inner.clear_template_vertices();
    }

    pub fn get_template_geometry(&self, id: TemplateGeometryRef) -> Option<&Geometry<VR, SS>> {
        self.inner.get_template_geometry(id.to_raw())
    }

    pub fn get_template_geometry_mut(
        &mut self,
        id: TemplateGeometryRef,
    ) -> Option<&mut Geometry<VR, SS>> {
        self.inner.get_template_geometry_mut(id.to_raw())
    }

    /// Add a template geometry and return its handle.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when the template-geometry pool cannot
    /// store additional entries for `ResourceId32`.
    pub fn add_template_geometry(
        &mut self,
        geometry: Geometry<VR, SS>,
    ) -> Result<TemplateGeometryRef> {
        self.inner
            .add_template_geometry(geometry)
            .map(TemplateGeometryRef::from_raw)
    }

    pub fn template_geometry_count(&self) -> usize {
        self.inner.template_geometry_count()
    }

    pub fn iter_template_geometries(
        &self,
    ) -> impl Iterator<Item = (TemplateGeometryRef, &Geometry<VR, SS>)> + '_ {
        self.inner
            .iter_template_geometries()
            .map(|(id, v)| (TemplateGeometryRef::from_raw(id), v))
    }

    pub fn iter_template_geometries_mut(
        &mut self,
    ) -> impl Iterator<Item = (TemplateGeometryRef, &mut Geometry<VR, SS>)> + '_ {
        self.inner
            .iter_template_geometries_mut()
            .map(|(id, v)| (TemplateGeometryRef::from_raw(id), v))
    }

    pub fn remove_template_geometry(
        &mut self,
        id: TemplateGeometryRef,
    ) -> Option<Geometry<VR, SS>> {
        self.inner.remove_template_geometry(id.to_raw())
    }

    pub fn clear_template_geometries(&mut self) {
        self.inner.clear_template_geometries();
    }

    pub fn type_citymodel(&self) -> crate::CityModelType {
        self.inner.type_citymodel()
    }

    pub fn version(&self) -> Option<crate::CityJSONVersion> {
        self.inner.version()
    }

    pub fn default_theme_material(&self) -> Option<MaterialRef> {
        self.inner
            .default_theme_material()
            .map(MaterialRef::from_raw)
    }

    pub fn set_default_theme_material(&mut self, material_ref: Option<MaterialRef>) {
        self.inner.set_default_theme_material(
            material_ref.map(super::super::resources::handles::MaterialRef::to_raw),
        );
    }

    pub fn default_theme_texture(&self) -> Option<TextureRef> {
        self.inner.default_theme_texture().map(TextureRef::from_raw)
    }

    pub fn set_default_theme_texture(&mut self, texture_ref: Option<TextureRef>) {
        self.inner.set_default_theme_texture(
            texture_ref.map(super::super::resources::handles::TextureRef::to_raw),
        );
    }

    /// Extracts a float attribute column from all `CityObjects`.
    ///
    /// Returns `(object_refs, values)` where each index in both vectors corresponds.
    pub fn extract_float_column(&self, key: &str) -> (Vec<CityObjectRef>, Vec<f64>) {
        let mut object_refs = Vec::new();
        let mut values = Vec::new();

        for (id, cityobject) in self.cityobjects().iter() {
            if let Some(attributes) = cityobject.attributes()
                && let Some(crate::cityjson::core::attributes::AttributeValue::Float(value)) =
                    attributes.get(key)
            {
                object_refs.push(id);
                values.push(*value);
            }
        }

        (object_refs, values)
    }

    /// Extracts an integer attribute column from all `CityObjects`.
    ///
    /// Returns `(object_refs, values)` where each index in both vectors corresponds.
    pub fn extract_integer_column(&self, key: &str) -> (Vec<CityObjectRef>, Vec<i64>) {
        let mut object_refs = Vec::new();
        let mut values = Vec::new();

        for (id, cityobject) in self.cityobjects().iter() {
            if let Some(attributes) = cityobject.attributes()
                && let Some(crate::cityjson::core::attributes::AttributeValue::Integer(value)) =
                    attributes.get(key)
            {
                object_refs.push(id);
                values.push(*value);
            }
        }

        (object_refs, values)
    }

    /// Extracts a string attribute column from all `CityObjects`.
    ///
    /// Returns `(object_refs, values)` where each index in both vectors corresponds.
    pub fn extract_string_column<'a>(
        &'a self,
        key: &str,
    ) -> (Vec<CityObjectRef>, Vec<&'a SS::String>) {
        let mut object_refs = Vec::new();
        let mut values = Vec::new();

        for (id, cityobject) in self.cityobjects().iter() {
            if let Some(attributes) = cityobject.attributes()
                && let Some(crate::cityjson::core::attributes::AttributeValue::String(value)) =
                    attributes.get(key)
            {
                object_refs.push(id);
                values.push(value);
            }
        }

        (object_refs, values)
    }

    /// Returns all unique attribute keys from all `CityObjects`.
    pub fn attribute_keys(&self) -> HashSet<&str> {
        let mut keys = HashSet::new();

        for (_, cityobject) in self.cityobjects().iter() {
            if let Some(attributes) = cityobject.attributes() {
                for key in attributes.keys() {
                    keys.insert(key.as_ref());
                }
            }
        }

        keys
    }
}

/// Raw accessor for zero-copy access to internal `CityModel` storage.
pub struct CityModelRawAccessor<'a, VR: VertexRef, SS: StringStorage> {
    model: &'a CityModel<VR, SS>,
}

impl<'a, VR: VertexRef, SS: StringStorage> CityModelRawAccessor<'a, VR, SS> {
    #[inline]
    #[must_use]
    pub fn vertices(&self) -> &'a [QuantizedCoordinate] {
        self.model.inner.vertices().as_slice()
    }

    #[inline]
    #[must_use]
    pub fn geometries(&self) -> RawPoolView<'a, Geometry<VR, SS>> {
        self.model.inner.geometries_raw()
    }

    #[inline]
    #[must_use]
    pub fn semantics(&self) -> RawPoolView<'a, Semantic<SS>> {
        self.model.inner.semantics_raw()
    }

    #[inline]
    #[must_use]
    pub fn materials(&self) -> RawPoolView<'a, Material<SS>> {
        self.model.inner.materials_raw()
    }

    #[inline]
    #[must_use]
    pub fn textures(&self) -> RawPoolView<'a, Texture<SS>> {
        self.model.inner.textures_raw()
    }

    #[inline]
    #[must_use]
    pub fn cityobjects(&self) -> &'a CityObjects<SS> {
        self.model.cityobjects()
    }

    #[inline]
    #[must_use]
    pub fn template_vertices(&self) -> &'a [RealWorldCoordinate] {
        self.model.inner.template_vertices().as_slice()
    }

    #[inline]
    #[must_use]
    pub fn uv_coordinates(&self) -> &'a [UVCoordinate] {
        self.model.inner.vertices_texture().as_slice()
    }
}

impl<VR: VertexRef, SS: StringStorage> RawAccess for CityModel<VR, SS> {
    type Vertex = QuantizedCoordinate;
    type Geometry = Geometry<VR, SS>;
    type Semantic = Semantic<SS>;
    type Material = Material<SS>;
    type Texture = Texture<SS>;

    fn vertices_raw(&self) -> RawSliceView<'_, Self::Vertex> {
        RawSliceView::new(self.vertices().as_slice())
    }

    fn geometries_raw(&self) -> RawPoolView<'_, Self::Geometry> {
        self.inner.geometries_raw()
    }

    fn semantics_raw(&self) -> RawPoolView<'_, Self::Semantic> {
        self.inner.semantics_raw()
    }

    fn materials_raw(&self) -> RawPoolView<'_, Self::Material> {
        self.inner.materials_raw()
    }

    fn textures_raw(&self) -> RawPoolView<'_, Self::Texture> {
        self.inner.textures_raw()
    }
}

impl<VR: VertexRef, SS: StringStorage> fmt::Display for CityModel<VR, SS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "CityModel {{")?;
        writeln!(f, "\ttype: {}", self.type_citymodel())?;
        writeln!(f, "\tversion: {}", format_option(self.version().as_ref()))?;
        writeln!(
            f,
            "\textensions: {{ {} }}",
            format_option(self.extensions())
        )?;
        writeln!(f, "\ttransform: {{ {} }}", format_option(self.transform()))?;
        writeln!(f, "\tmetadata: {}", format_option(self.metadata()))?;
        writeln!(
            f,
            "\tCityObjects: {{ nr. cityobjects: {}, nr. geometries: {} }}",
            self.cityobjects().len(),
            self.geometry_count()
        )?;
        writeln!(
            f,
            "\tappearance: {{ nr. materials: {}, nr. textures: {}, nr. vertices-texture: {}, default-theme-texture: {}, default-theme-material: {} }}",
            self.material_count(),
            self.texture_count(),
            self.vertices_texture().len(),
            format_option(self.default_theme_texture().as_ref()),
            format_option(self.default_theme_material().as_ref())
        )?;
        writeln!(f, "\tgeometry-templates: not implemented")?;
        writeln!(
            f,
            "\tvertices: {{ nr. vertices: {}, quantized coordinates: not implemented }}",
            self.vertices().len()
        )?;
        writeln!(f, "\textra: {}", format_option(self.extra()))?;
        writeln!(f, "}}")
    }
}

impl<VR: VertexRef, SS: StringStorage>
    GeometryModelOps<
        VR,
        ResourceId32,
        QuantizedCoordinate,
        Semantic<SS>,
        Material<SS>,
        Texture<SS>,
        Geometry<VR, SS>,
        SS,
    > for CityModel<VR, SS>
{
    fn add_semantic(&mut self, semantic: Semantic<SS>) -> Result<ResourceId32> {
        self.add_semantic(semantic)
            .map(super::super::resources::handles::SemanticRef::to_raw)
    }

    fn get_or_insert_semantic(&mut self, semantic: Semantic<SS>) -> Result<ResourceId32> {
        self.get_or_insert_semantic(semantic)
            .map(super::super::resources::handles::SemanticRef::to_raw)
    }

    fn add_material(&mut self, material: Material<SS>) -> Result<ResourceId32> {
        self.add_material(material)
            .map(super::super::resources::handles::MaterialRef::to_raw)
    }

    fn get_or_insert_material(&mut self, material: Material<SS>) -> Result<ResourceId32> {
        self.get_or_insert_material(material)
            .map(super::super::resources::handles::MaterialRef::to_raw)
    }

    fn add_texture(&mut self, texture: Texture<SS>) -> Result<ResourceId32> {
        self.add_texture(texture)
            .map(super::super::resources::handles::TextureRef::to_raw)
    }

    fn get_or_insert_texture(&mut self, texture: Texture<SS>) -> Result<ResourceId32> {
        self.get_or_insert_texture(texture)
            .map(super::super::resources::handles::TextureRef::to_raw)
    }

    fn add_uv_coordinate(&mut self, uvcoordinate: UVCoordinate) -> Result<VertexIndex<VR>> {
        self.add_uv_coordinate(uvcoordinate)
    }

    fn add_geometry(&mut self, geometry: Geometry<VR, SS>) -> Result<ResourceId32> {
        self.add_geometry(geometry)
            .map(super::super::resources::handles::GeometryRef::to_raw)
    }

    fn add_template_geometry(&mut self, geometry: Geometry<VR, SS>) -> Result<ResourceId32> {
        self.add_template_geometry(geometry)
            .map(super::super::resources::handles::TemplateGeometryRef::to_raw)
    }

    fn add_vertex(&mut self, coordinate: QuantizedCoordinate) -> Result<VertexIndex<VR>> {
        self.add_vertex(coordinate)
    }

    fn vertices_mut(&mut self) -> &mut Vertices<VR, QuantizedCoordinate> {
        self.vertices_mut()
    }

    fn add_template_vertex(&mut self, coordinate: RealWorldCoordinate) -> Result<VertexIndex<VR>> {
        self.add_template_vertex(coordinate)
    }

    fn template_vertices_mut(&mut self) -> &mut Vertices<VR, RealWorldCoordinate> {
        self.template_vertices_mut()
    }
}
