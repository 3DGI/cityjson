use crate::cityjson::core::appearance::ThemeName;
use crate::error::{Error, Result};
use crate::raw::{CityModelRawAccessor, DenseIndexRemap};
use crate::resources::storage::OwnedStringStorage;
use crate::symbols::SymbolStorageOptions;
use crate::v2_0::appearance::material::Material;
use crate::v2_0::appearance::texture::Texture;
use crate::v2_0::attributes::{AttributeValue, Attributes};
use crate::v2_0::boundary::Boundary;
use crate::v2_0::cityobject::{CityObject, CityObjectType};
use crate::v2_0::geometry::semantic::{Semantic, SemanticType};
use crate::v2_0::geometry::{
    AffineTransform3D, Geometry, GeometryType, LoD, StoredGeometryInstance, StoredGeometryParts,
};
use crate::v2_0::metadata::{BBox, CRS, CityModelIdentifier, Contact, ContactRole, ContactType};
use crate::v2_0::vertex::VertexIndex;
use crate::v2_0::{
    CityModelCapacities, Extension, MaterialMap, Metadata, OwnedCityModel, RealWorldCoordinate,
    SemanticMap, UVCoordinate, WrapMode,
};
use crate::{CityModelType, resources::mapping::textures::TextureMap as PublicTextureMap};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SymbolId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CityObjectId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GeometryId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GeometryTemplateId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SemanticId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MaterialId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TextureId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VertexId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UvVertexId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AttributeNodeId(pub u32);

macro_rules! impl_dense_id {
    ($name:ident) => {
        impl $name {
            #[allow(dead_code)]
            fn index(self) -> usize {
                usize::try_from(self.0).unwrap_or(usize::MAX)
            }
        }
    };
}

impl_dense_id!(SymbolId);
impl_dense_id!(CityObjectId);
impl_dense_id!(GeometryId);
impl_dense_id!(GeometryTemplateId);
impl_dense_id!(SemanticId);
impl_dense_id!(MaterialId);
impl_dense_id!(TextureId);
impl_dense_id!(VertexId);
impl_dense_id!(UvVertexId);
impl_dense_id!(AttributeNodeId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeNodeType {
    Null,
    Bool,
    Unsigned,
    Integer,
    Float,
    String,
    Array,
    Object,
    GeometryRef,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTableOwned {
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct VertexTableOwned {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
}

impl VertexTableOwned {
    #[must_use]
    pub fn len(&self) -> usize {
        self.x.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct UvVertexTableOwned {
    pub u: Vec<f64>,
    pub v: Vec<f64>,
}

impl UvVertexTableOwned {
    #[must_use]
    pub fn len(&self) -> usize {
        self.u.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.u.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CityObjectTableOwned {
    pub ids: Vec<CityObjectId>,
    pub external_id_symbols: Vec<SymbolId>,
    pub object_type_symbols: Vec<SymbolId>,
    pub parent_start: Vec<u32>,
    pub parent_len: Vec<u32>,
    pub parents: Vec<CityObjectId>,
    pub child_start: Vec<u32>,
    pub child_len: Vec<u32>,
    pub children: Vec<CityObjectId>,
    pub geometry_start: Vec<u32>,
    pub geometry_len: Vec<u32>,
    pub geometries: Vec<GeometryId>,
    pub attribute_root: Vec<Option<AttributeNodeId>>,
    pub bbox_min_x: Vec<Option<f64>>,
    pub bbox_min_y: Vec<Option<f64>>,
    pub bbox_min_z: Vec<Option<f64>>,
    pub bbox_max_x: Vec<Option<f64>>,
    pub bbox_max_y: Vec<Option<f64>>,
    pub bbox_max_z: Vec<Option<f64>>,
}

impl CityObjectTableOwned {
    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct SemanticTableOwned {
    pub ids: Vec<SemanticId>,
    pub semantic_type_symbols: Vec<SymbolId>,
    pub parent: Vec<Option<SemanticId>>,
    pub child_start: Vec<u32>,
    pub child_len: Vec<u32>,
    pub children: Vec<SemanticId>,
    pub attribute_root: Vec<Option<AttributeNodeId>>,
}

impl SemanticTableOwned {
    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MaterialTableOwned {
    pub ids: Vec<MaterialId>,
    pub name_symbols: Vec<SymbolId>,
    pub ambient_intensity: Vec<Option<f32>>,
    pub diffuse_color: Vec<Option<[f32; 3]>>,
    pub emissive_color: Vec<Option<[f32; 3]>>,
    pub specular_color: Vec<Option<[f32; 3]>>,
    pub shininess: Vec<Option<f32>>,
    pub transparency: Vec<Option<f32>>,
    pub is_smooth: Vec<Option<bool>>,
}

impl MaterialTableOwned {
    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct TextureTableOwned {
    pub ids: Vec<TextureId>,
    pub image_uri_symbols: Vec<SymbolId>,
    pub texture_type_symbols: Vec<Option<SymbolId>>,
    pub wrap_mode_symbols: Vec<Option<SymbolId>>,
    pub image_type_symbols: Vec<SymbolId>,
    pub border_color: Vec<Option<[f32; 4]>>,
}

impl TextureTableOwned {
    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct GeometryMaterialThemeOwned {
    pub geometry: GeometryId,
    pub theme_symbol: SymbolId,
    pub point_start: u32,
    pub point_len: u32,
    pub linestring_start: u32,
    pub linestring_len: u32,
    pub surface_start: u32,
    pub surface_len: u32,
}

#[derive(Debug, Clone, Default)]
pub struct GeometryTextureThemeOwned {
    pub geometry: GeometryId,
    pub theme_symbol: SymbolId,
    pub vertex_start: u32,
    pub vertex_len: u32,
    pub ring_start: u32,
    pub ring_len: u32,
    pub ring_texture_start: u32,
    pub ring_texture_len: u32,
}

#[derive(Debug, Clone, Default)]
pub struct GeometryTableOwned {
    pub ids: Vec<GeometryId>,
    pub geometry_type_symbols: Vec<SymbolId>,
    pub lod_symbols: Vec<Option<SymbolId>>,
    pub boundary_vertex_start: Vec<u32>,
    pub boundary_vertex_len: Vec<u32>,
    pub boundary_ring_start: Vec<u32>,
    pub boundary_ring_len: Vec<u32>,
    pub boundary_surface_start: Vec<u32>,
    pub boundary_surface_len: Vec<u32>,
    pub boundary_shell_start: Vec<u32>,
    pub boundary_shell_len: Vec<u32>,
    pub boundary_solid_start: Vec<u32>,
    pub boundary_solid_len: Vec<u32>,
    pub semantic_point_start: Vec<u32>,
    pub semantic_point_len: Vec<u32>,
    pub semantic_linestring_start: Vec<u32>,
    pub semantic_linestring_len: Vec<u32>,
    pub semantic_surface_start: Vec<u32>,
    pub semantic_surface_len: Vec<u32>,
    pub material_theme_start: Vec<u32>,
    pub material_theme_len: Vec<u32>,
    pub texture_theme_start: Vec<u32>,
    pub texture_theme_len: Vec<u32>,
    pub template_ref: Vec<Option<GeometryTemplateId>>,
    pub reference_point: Vec<Option<VertexId>>,
    pub transform_matrix: Vec<[f64; 16]>,
    pub boundary_vertices: Vec<VertexId>,
    pub boundary_rings: Vec<u32>,
    pub boundary_surfaces: Vec<u32>,
    pub boundary_shells: Vec<u32>,
    pub boundary_solids: Vec<u32>,
    pub semantic_points: Vec<Option<SemanticId>>,
    pub semantic_linestrings: Vec<Option<SemanticId>>,
    pub semantic_surfaces: Vec<Option<SemanticId>>,
    pub material_themes: Vec<GeometryMaterialThemeOwned>,
    pub material_points: Vec<Option<MaterialId>>,
    pub material_linestrings: Vec<Option<MaterialId>>,
    pub material_surfaces: Vec<Option<MaterialId>>,
    pub texture_themes: Vec<GeometryTextureThemeOwned>,
    pub texture_vertex_refs: Vec<Option<VertexId>>,
    pub texture_rings: Vec<u32>,
    pub texture_ring_textures: Vec<Option<TextureId>>,
}

impl GeometryTableOwned {
    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct AttributeArenaOwned {
    pub node_type: Vec<AttributeNodeType>,
    pub key_symbol: Vec<Option<SymbolId>>,
    pub string_value_symbol: Vec<Option<SymbolId>>,
    pub bool_value: Vec<Option<bool>>,
    pub unsigned_value: Vec<Option<u64>>,
    pub int_value: Vec<Option<i64>>,
    pub float_value: Vec<Option<f64>>,
    pub geometry_value: Vec<Option<GeometryId>>,
    pub first_child_offset: Vec<u32>,
    pub child_len: Vec<u32>,
    pub child_nodes: Vec<AttributeNodeId>,
}

impl AttributeArenaOwned {
    #[must_use]
    pub fn len(&self) -> usize {
        self.node_type.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.node_type.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExtensionTableOwned {
    pub name_symbols: Vec<SymbolId>,
    pub url_symbols: Vec<SymbolId>,
    pub version_symbols: Vec<SymbolId>,
}

impl ExtensionTableOwned {
    #[must_use]
    pub fn len(&self) -> usize {
        self.name_symbols.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.name_symbols.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ContactOwned {
    pub name_symbol: SymbolId,
    pub email_symbol: SymbolId,
    pub role_symbol: Option<SymbolId>,
    pub website_symbol: Option<SymbolId>,
    pub contact_type_symbol: Option<SymbolId>,
    pub address_root: Option<AttributeNodeId>,
    pub phone_symbol: Option<SymbolId>,
    pub organization_symbol: Option<SymbolId>,
}

#[derive(Debug, Clone, Default)]
pub struct MetadataOwned {
    pub geographical_extent: Option<[f64; 6]>,
    pub identifier_symbol: Option<SymbolId>,
    pub reference_date_symbol: Option<SymbolId>,
    pub reference_system_symbol: Option<SymbolId>,
    pub title_symbol: Option<SymbolId>,
    pub extra_root: Option<AttributeNodeId>,
    pub point_of_contact: Option<ContactOwned>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TransformOwned {
    pub scale: [f64; 3],
    pub translate: [f64; 3],
}

#[derive(Debug, Clone, Default)]
pub struct DefaultThemeOwned {
    pub material_theme_symbol: Option<SymbolId>,
    pub texture_theme_symbol: Option<SymbolId>,
}

pub struct SymbolTableView<'a> {
    pub values: &'a [String],
}

impl SymbolTableView<'_> {
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

pub struct VertexTableView<'a> {
    pub x: &'a [f64],
    pub y: &'a [f64],
    pub z: &'a [f64],
}

impl VertexTableView<'_> {
    #[must_use]
    pub fn len(&self) -> usize {
        self.x.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }
}

pub struct UvVertexTableView<'a> {
    pub u: &'a [f64],
    pub v: &'a [f64],
}

impl UvVertexTableView<'_> {
    #[must_use]
    pub fn len(&self) -> usize {
        self.u.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.u.is_empty()
    }
}

pub type CityObjectTableView<'a> = &'a CityObjectTableOwned;
pub type GeometryTableView<'a> = &'a GeometryTableOwned;
pub type SemanticTableView<'a> = &'a SemanticTableOwned;
pub type MaterialTableView<'a> = &'a MaterialTableOwned;
pub type TextureTableView<'a> = &'a TextureTableOwned;
pub type AttributeArenaView<'a> = &'a AttributeArenaOwned;
pub type ExtensionTableView<'a> = &'a ExtensionTableOwned;
pub type MetadataView<'a> = &'a MetadataOwned;
pub type TransformView<'a> = &'a TransformOwned;
pub type DefaultThemeView<'a> = &'a DefaultThemeOwned;

#[derive(Debug, Clone)]
pub struct ModelRelationalView<'a> {
    model: &'a OwnedCityModel,
    cityobject_remap: DenseIndexRemap,
    geometry_remap: DenseIndexRemap,
    geometry_template_remap: DenseIndexRemap,
    semantic_remap: DenseIndexRemap,
    material_remap: DenseIndexRemap,
    texture_remap: DenseIndexRemap,
}

impl<'a> ModelRelationalView<'a> {
    #[must_use]
    pub const fn model(&self) -> &'a OwnedCityModel {
        self.model
    }

    #[must_use]
    pub fn raw(&self) -> CityModelRawAccessor<'a, u32, OwnedStringStorage> {
        self.model.raw()
    }

    #[must_use]
    pub fn cityobjects(&self) -> &'a crate::v2_0::OwnedCityObjects {
        self.model.cityobjects()
    }

    #[must_use]
    pub fn cityobject_remap(&self) -> &DenseIndexRemap {
        &self.cityobject_remap
    }

    #[must_use]
    pub fn geometry_remap(&self) -> &DenseIndexRemap {
        &self.geometry_remap
    }

    #[must_use]
    pub fn geometry_template_remap(&self) -> &DenseIndexRemap {
        &self.geometry_template_remap
    }

    #[must_use]
    pub fn semantic_remap(&self) -> &DenseIndexRemap {
        &self.semantic_remap
    }

    #[must_use]
    pub fn material_remap(&self) -> &DenseIndexRemap {
        &self.material_remap
    }

    #[must_use]
    pub fn texture_remap(&self) -> &DenseIndexRemap {
        &self.texture_remap
    }

    #[must_use]
    pub fn feature_root(&self) -> Option<CityObjectId> {
        self.model
            .id()
            .and_then(|handle| {
                self.cityobject_remap
                    .get(usize::try_from(slot(handle)).unwrap_or(usize::MAX))
            })
            .and_then(|dense| u32::try_from(dense).ok())
            .map(CityObjectId)
    }

    #[must_use]
    pub fn snapshot(&self) -> OwnedRelationalSnapshot {
        build_relational_snapshot(self.model)
    }
}

#[derive(Debug, Clone, Default)]
pub struct OwnedRelationalSnapshot {
    symbols: SymbolTableOwned,
    vertices: VertexTableOwned,
    template_vertices: VertexTableOwned,
    uv_vertices: UvVertexTableOwned,
    cityobjects: CityObjectTableOwned,
    geometries: GeometryTableOwned,
    geometry_templates: GeometryTableOwned,
    semantics: SemanticTableOwned,
    materials: MaterialTableOwned,
    textures: TextureTableOwned,
    attributes: AttributeArenaOwned,
    metadata: Option<MetadataOwned>,
    transform: Option<TransformOwned>,
    defaults: DefaultThemeOwned,
    extensions: ExtensionTableOwned,
    feature_root: Option<CityObjectId>,
}

impl OwnedRelationalSnapshot {
    #[must_use]
    pub fn symbol_table(&self) -> &SymbolTableOwned {
        &self.symbols
    }

    #[must_use]
    pub fn symbols(&self) -> SymbolTableView<'_> {
        SymbolTableView {
            values: &self.symbols.values,
        }
    }

    #[must_use]
    pub fn vertex_table(&self) -> &VertexTableOwned {
        &self.vertices
    }

    #[must_use]
    pub fn vertices(&self) -> VertexTableView<'_> {
        VertexTableView {
            x: &self.vertices.x,
            y: &self.vertices.y,
            z: &self.vertices.z,
        }
    }

    #[must_use]
    pub fn template_vertices(&self) -> VertexTableView<'_> {
        VertexTableView {
            x: &self.template_vertices.x,
            y: &self.template_vertices.y,
            z: &self.template_vertices.z,
        }
    }

    #[must_use]
    pub fn template_vertex_table(&self) -> &VertexTableOwned {
        &self.template_vertices
    }

    #[must_use]
    pub fn uv_vertex_table(&self) -> &UvVertexTableOwned {
        &self.uv_vertices
    }

    #[must_use]
    pub fn uv_vertices(&self) -> UvVertexTableView<'_> {
        UvVertexTableView {
            u: &self.uv_vertices.u,
            v: &self.uv_vertices.v,
        }
    }

    #[must_use]
    pub fn cityobjects(&self) -> CityObjectTableView<'_> {
        &self.cityobjects
    }

    #[must_use]
    pub fn geometries(&self) -> GeometryTableView<'_> {
        &self.geometries
    }

    #[must_use]
    pub fn geometry_templates(&self) -> GeometryTableView<'_> {
        &self.geometry_templates
    }

    #[must_use]
    pub fn semantics(&self) -> SemanticTableView<'_> {
        &self.semantics
    }

    #[must_use]
    pub fn materials(&self) -> MaterialTableView<'_> {
        &self.materials
    }

    #[must_use]
    pub fn textures(&self) -> TextureTableView<'_> {
        &self.textures
    }

    #[must_use]
    pub fn attributes(&self) -> AttributeArenaView<'_> {
        &self.attributes
    }

    #[must_use]
    pub fn metadata(&self) -> Option<MetadataView<'_>> {
        self.metadata.as_ref()
    }

    #[must_use]
    pub fn transform(&self) -> Option<TransformView<'_>> {
        self.transform.as_ref()
    }

    #[must_use]
    pub fn defaults(&self) -> DefaultThemeView<'_> {
        &self.defaults
    }

    #[must_use]
    pub fn extensions(&self) -> ExtensionTableView<'_> {
        &self.extensions
    }

    #[must_use]
    pub fn metadata_owned(&self) -> Option<&MetadataOwned> {
        self.metadata.as_ref()
    }

    #[must_use]
    pub fn transform_owned(&self) -> Option<&TransformOwned> {
        self.transform.as_ref()
    }

    #[must_use]
    pub fn defaults_owned(&self) -> &DefaultThemeOwned {
        &self.defaults
    }

    #[must_use]
    pub fn feature_root(&self) -> Option<CityObjectId> {
        self.feature_root
    }

    fn symbol(&self, id: SymbolId) -> Result<&str> {
        self.symbols
            .values
            .get(id.index())
            .map(String::as_str)
            .ok_or_else(|| Error::Import(format!("missing symbol {}", id.0)))
    }
}

pub trait RelationalAccess {
    fn relational(&self) -> ModelRelationalView<'_>;
    fn relational_snapshot(&self) -> OwnedRelationalSnapshot;
}

impl RelationalAccess for OwnedCityModel {
    fn relational(&self) -> ModelRelationalView<'_> {
        ModelRelationalView {
            model: self,
            cityobject_remap: dense_cityobject_remap(self),
            geometry_remap: self.raw().geometries().dense_index_remap(),
            geometry_template_remap: dense_geometry_template_remap(self),
            semantic_remap: self.raw().semantics().dense_index_remap(),
            material_remap: self.raw().materials().dense_index_remap(),
            texture_remap: self.raw().textures().dense_index_remap(),
        }
    }

    fn relational_snapshot(&self) -> OwnedRelationalSnapshot {
        build_relational_snapshot(self)
    }
}

#[derive(Debug, Clone)]
pub struct RelationalImportOptions {
    pub symbol_storage: SymbolStorageOptions,
    pub validate_default_themes: bool,
    pub validate_references: bool,
}

impl Default for RelationalImportOptions {
    fn default() -> Self {
        Self {
            symbol_storage: SymbolStorageOptions::default(),
            validate_default_themes: true,
            validate_references: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RelationalCapacities {
    pub symbols: usize,
    pub cityobjects: usize,
    pub geometries: usize,
    pub geometry_templates: usize,
    pub vertices: usize,
    pub template_vertices: usize,
    pub uv_vertices: usize,
    pub semantics: usize,
    pub materials: usize,
    pub textures: usize,
    pub attribute_nodes: usize,
    pub extensions: usize,
}

#[derive(Debug, Default)]
pub struct RelationalModelBuilder {
    model_type: CityModelType,
    options: RelationalImportOptions,
    symbols: Option<SymbolTableOwned>,
    cityobjects: Option<CityObjectTableOwned>,
    geometries: Option<GeometryTableOwned>,
    geometry_templates: Option<GeometryTableOwned>,
    vertices: Option<VertexTableOwned>,
    template_vertices: Option<VertexTableOwned>,
    uv_vertices: Option<UvVertexTableOwned>,
    semantics: Option<SemanticTableOwned>,
    materials: Option<MaterialTableOwned>,
    textures: Option<TextureTableOwned>,
    attributes: Option<AttributeArenaOwned>,
    metadata: Option<MetadataOwned>,
    transform: Option<TransformOwned>,
    defaults: Option<DefaultThemeOwned>,
    extensions: Option<ExtensionTableOwned>,
    feature_root: Option<CityObjectId>,
}

#[allow(clippy::missing_errors_doc)]
impl RelationalModelBuilder {
    #[must_use]
    pub fn new(model_type: CityModelType, options: RelationalImportOptions) -> Self {
        Self {
            model_type,
            options,
            ..Self::default()
        }
    }

    pub fn reserve(&mut self, _capacities: RelationalCapacities) -> Result<()> {
        Ok(())
    }

    pub fn push_symbols(&mut self, table: SymbolTableOwned) -> Result<()> {
        self.symbols = Some(table);
        Ok(())
    }

    pub fn push_vertices(&mut self, table: VertexTableOwned) -> Result<()> {
        self.vertices = Some(table);
        Ok(())
    }

    pub fn push_template_vertices(&mut self, table: VertexTableOwned) -> Result<()> {
        self.template_vertices = Some(table);
        Ok(())
    }

    pub fn push_uv_vertices(&mut self, table: UvVertexTableOwned) -> Result<()> {
        self.uv_vertices = Some(table);
        Ok(())
    }

    pub fn push_semantics(&mut self, table: SemanticTableOwned) -> Result<()> {
        self.semantics = Some(table);
        Ok(())
    }

    pub fn push_materials(&mut self, table: MaterialTableOwned) -> Result<()> {
        self.materials = Some(table);
        Ok(())
    }

    pub fn push_textures(&mut self, table: TextureTableOwned) -> Result<()> {
        self.textures = Some(table);
        Ok(())
    }

    pub fn push_attributes(&mut self, table: AttributeArenaOwned) -> Result<()> {
        self.attributes = Some(table);
        Ok(())
    }

    pub fn push_cityobjects(&mut self, table: CityObjectTableOwned) -> Result<()> {
        self.cityobjects = Some(table);
        Ok(())
    }

    pub fn push_geometries(&mut self, table: GeometryTableOwned) -> Result<()> {
        self.geometries = Some(table);
        Ok(())
    }

    pub fn push_geometry_templates(&mut self, table: GeometryTableOwned) -> Result<()> {
        self.geometry_templates = Some(table);
        Ok(())
    }

    pub fn push_metadata(&mut self, view: Option<MetadataOwned>) -> Result<()> {
        self.metadata = view;
        Ok(())
    }

    pub fn push_transform(&mut self, view: Option<TransformOwned>) -> Result<()> {
        self.transform = view;
        Ok(())
    }

    pub fn push_defaults(&mut self, view: DefaultThemeOwned) -> Result<()> {
        self.defaults = Some(view);
        Ok(())
    }

    pub fn push_extensions(&mut self, table: ExtensionTableOwned) -> Result<()> {
        self.extensions = Some(table);
        Ok(())
    }

    pub fn push_feature_root(&mut self, feature_root: Option<CityObjectId>) -> Result<()> {
        self.feature_root = feature_root;
        Ok(())
    }

    pub fn finish(self) -> Result<OwnedCityModel> {
        let view = OwnedRelationalSnapshot {
            symbols: self
                .symbols
                .ok_or_else(|| Error::Import("missing symbol table".to_string()))?,
            vertices: self.vertices.unwrap_or_default(),
            template_vertices: self.template_vertices.unwrap_or_default(),
            uv_vertices: self.uv_vertices.unwrap_or_default(),
            cityobjects: self.cityobjects.unwrap_or_default(),
            geometries: self.geometries.unwrap_or_default(),
            geometry_templates: self.geometry_templates.unwrap_or_default(),
            semantics: self.semantics.unwrap_or_default(),
            materials: self.materials.unwrap_or_default(),
            textures: self.textures.unwrap_or_default(),
            attributes: self.attributes.unwrap_or_default(),
            metadata: self.metadata,
            transform: self.transform,
            defaults: self.defaults.unwrap_or_default(),
            extensions: self.extensions.unwrap_or_default(),
            feature_root: self.feature_root,
        };

        build_model_from_relational(self.model_type, &self.options, &view)
    }
}

#[derive(Default)]
struct SymbolCollector {
    ids_by_value: HashMap<String, SymbolId>,
    values: Vec<String>,
}

impl SymbolCollector {
    fn intern(&mut self, value: &str) -> SymbolId {
        if let Some(id) = self.ids_by_value.get(value) {
            return *id;
        }

        let id = SymbolId(u32::try_from(self.values.len()).unwrap_or(u32::MAX));
        let owned = value.to_string();
        self.values.push(owned.clone());
        self.ids_by_value.insert(owned, id);
        id
    }
}

fn build_relational_snapshot(model: &OwnedCityModel) -> OwnedRelationalSnapshot {
    let mut symbols = SymbolCollector::default();
    let mut attributes = AttributeArenaOwned::default();

    let cityobject_ids = dense_cityobject_ids(model);
    let geometry_ids = dense_geometry_ids(model);
    let geometry_template_ids = dense_geometry_template_ids(model);
    let semantic_ids = dense_semantic_ids(model);
    let material_ids = dense_material_ids(model);
    let texture_ids = dense_texture_ids(model);

    let vertices = encode_vertex_table(model.vertices().as_slice());
    let template_vertices = encode_vertex_table(model.template_vertices().as_slice());
    let uv_vertices = encode_uv_table(model.vertices_texture().as_slice());

    let cityobjects = encode_cityobjects(
        model,
        &cityobject_ids,
        &geometry_ids,
        &mut symbols,
        &mut attributes,
    );
    let semantics = encode_semantics(model, &semantic_ids, &mut symbols, &mut attributes);
    let materials = encode_materials(model, &material_ids, &mut symbols);
    let textures = encode_textures(model, &texture_ids, &mut symbols);
    let geometries = encode_geometries(
        model
            .iter_geometries()
            .map(|(id, geometry)| (GeometryId(geometry_ids[&slot(id)]), geometry)),
        &semantic_ids,
        &material_ids,
        &texture_ids,
        &geometry_template_ids,
        &mut symbols,
    );
    let geometry_templates = encode_geometries(
        model
            .iter_geometry_templates()
            .map(|(id, geometry)| (GeometryId(geometry_template_ids[&slot(id)]), geometry)),
        &semantic_ids,
        &material_ids,
        &texture_ids,
        &geometry_template_ids,
        &mut symbols,
    );
    let metadata = encode_metadata(
        model.metadata(),
        &geometry_ids,
        &mut symbols,
        &mut attributes,
    );
    let transform = model.transform().map(|transform| TransformOwned {
        scale: transform.scale(),
        translate: transform.translate(),
    });
    let defaults = DefaultThemeOwned {
        material_theme_symbol: model
            .default_material_theme()
            .map(|theme| symbols.intern(theme.as_ref())),
        texture_theme_symbol: model
            .default_texture_theme()
            .map(|theme| symbols.intern(theme.as_ref())),
    };
    let extensions = encode_extensions(model.extensions(), &mut symbols);
    let feature_root = model.id().map(|id| CityObjectId(cityobject_ids[&slot(id)]));

    OwnedRelationalSnapshot {
        symbols: SymbolTableOwned {
            values: symbols.values,
        },
        vertices,
        template_vertices,
        uv_vertices,
        cityobjects,
        geometries,
        geometry_templates,
        semantics,
        materials,
        textures,
        attributes,
        metadata,
        transform,
        defaults,
        extensions,
        feature_root,
    }
}

#[allow(clippy::cast_possible_truncation, clippy::too_many_lines)]
fn build_model_from_relational(
    model_type: CityModelType,
    options: &RelationalImportOptions,
    relational: &OwnedRelationalSnapshot,
) -> Result<OwnedCityModel> {
    let capacities = CityModelCapacities {
        cityobjects: relational.cityobjects.len(),
        vertices: relational.vertices.len(),
        semantics: relational.semantics.len(),
        materials: relational.materials.len(),
        textures: relational.textures.len(),
        geometries: relational.geometries.len(),
        template_vertices: relational.template_vertices.len(),
        template_geometries: relational.geometry_templates.len(),
        uv_coordinates: relational.uv_vertices.len(),
    };

    let mut model = OwnedCityModel::with_capacities(model_type, capacities);

    for i in 0..relational.vertices.len() {
        model.add_vertex(RealWorldCoordinate::new(
            relational.vertices.x[i],
            relational.vertices.y[i],
            relational.vertices.z[i],
        ))?;
    }

    for i in 0..relational.template_vertices.len() {
        model.add_template_vertex(RealWorldCoordinate::new(
            relational.template_vertices.x[i],
            relational.template_vertices.y[i],
            relational.template_vertices.z[i],
        ))?;
    }

    for i in 0..relational.uv_vertices.len() {
        model.add_uv_coordinate(UVCoordinate::new(
            relational.uv_vertices.u[i] as f32,
            relational.uv_vertices.v[i] as f32,
        ))?;
    }

    let mut semantic_handles = Vec::with_capacity(relational.semantics.len());
    for i in 0..relational.semantics.len() {
        let semantic = Semantic::new(parse_semantic_type(
            relational.symbol(relational.semantics.semantic_type_symbols[i])?,
        ));
        semantic_handles.push(model.add_semantic(semantic)?);
    }
    for i in 0..relational.semantics.len() {
        if let Some(root) = relational.semantics.attribute_root[i] {
            let attrs = decode_attributes(
                &relational.attributes,
                &relational.symbols.values,
                root,
                None,
            )?
            .ok_or_else(|| {
                Error::Import("semantic attribute root was not an object".to_string())
            })?;
            model
                .get_semantic_mut(semantic_handles[i])
                .ok_or_else(|| Error::Import("missing semantic during import".to_string()))?
                .attributes_mut()
                .clone_from(&attrs);
        }
        if let Some(parent) = relational.semantics.parent[i] {
            model
                .get_semantic_mut(semantic_handles[i])
                .ok_or_else(|| Error::Import("missing semantic parent target".to_string()))?
                .set_parent(semantic_handles[parent.index()]);
        }
        let start = usize::try_from(relational.semantics.child_start[i]).unwrap_or(usize::MAX);
        let len = usize::try_from(relational.semantics.child_len[i]).unwrap_or(usize::MAX);
        for child in &relational.semantics.children[start..start + len] {
            model
                .get_semantic_mut(semantic_handles[i])
                .ok_or_else(|| Error::Import("missing semantic child target".to_string()))?
                .children_mut()
                .push(semantic_handles[child.index()]);
        }
    }

    let mut material_handles = Vec::with_capacity(relational.materials.len());
    for i in 0..relational.materials.len() {
        let mut material = Material::new(
            relational
                .symbol(relational.materials.name_symbols[i])?
                .to_string(),
        );
        material.set_ambient_intensity(relational.materials.ambient_intensity[i]);
        material.set_diffuse_color(relational.materials.diffuse_color[i].map(Into::into));
        material.set_emissive_color(relational.materials.emissive_color[i].map(Into::into));
        material.set_specular_color(relational.materials.specular_color[i].map(Into::into));
        material.set_shininess(relational.materials.shininess[i]);
        material.set_transparency(relational.materials.transparency[i]);
        material.set_is_smooth(relational.materials.is_smooth[i]);
        material_handles.push(model.add_material(material)?);
    }

    let mut texture_handles = Vec::with_capacity(relational.textures.len());
    for i in 0..relational.textures.len() {
        let mut texture = Texture::new(
            relational
                .symbol(relational.textures.image_uri_symbols[i])?
                .to_string(),
            parse_image_type(relational.symbol(relational.textures.image_type_symbols[i])?),
        );
        texture.set_texture_type(
            relational.textures.texture_type_symbols[i]
                .map(|id| parse_texture_type(relational.symbol(id).unwrap_or("unknown"))),
        );
        texture.set_wrap_mode(
            relational.textures.wrap_mode_symbols[i]
                .map(|id| parse_wrap_mode(relational.symbol(id).unwrap_or("none"))),
        );
        texture.set_border_color(relational.textures.border_color[i].map(Into::into));
        texture_handles.push(model.add_texture(texture)?);
    }

    let mut template_handles = Vec::with_capacity(relational.geometry_templates.len());
    for i in 0..relational.geometry_templates.len() {
        let geometry = decode_geometry(
            &relational.geometry_templates,
            i,
            &template_handles,
            &semantic_handles,
            &material_handles,
            &texture_handles,
            true,
            relational,
        )?;
        template_handles.push(model.add_geometry_template_unchecked(geometry)?);
    }

    let mut geometry_handles = Vec::with_capacity(relational.geometries.len());
    for i in 0..relational.geometries.len() {
        let geometry = decode_geometry(
            &relational.geometries,
            i,
            &template_handles,
            &semantic_handles,
            &material_handles,
            &texture_handles,
            false,
            relational,
        )?;
        geometry_handles.push(model.add_geometry_unchecked(geometry)?);
    }

    let mut cityobject_handles = Vec::with_capacity(relational.cityobjects.len());
    for i in 0..relational.cityobjects.len() {
        let mut object = CityObject::new(
            crate::v2_0::CityObjectIdentifier::new(
                relational
                    .symbol(relational.cityobjects.external_id_symbols[i])?
                    .to_string(),
            ),
            parse_cityobject_type(
                relational.symbol(relational.cityobjects.object_type_symbols[i])?,
            )?,
        );
        if let Some(root) = relational.cityobjects.attribute_root[i] {
            let attrs = decode_attributes(
                &relational.attributes,
                &relational.symbols.values,
                root,
                Some(&geometry_handles),
            )?
            .ok_or_else(|| {
                Error::Import("cityobject attribute root was not an object".to_string())
            })?;
            *object.attributes_mut() = attrs;
        }
        if let Some(min_x) = relational.cityobjects.bbox_min_x[i] {
            object.set_geographical_extent(Some(BBox::new(
                min_x,
                relational.cityobjects.bbox_min_y[i].unwrap_or_default(),
                relational.cityobjects.bbox_min_z[i].unwrap_or_default(),
                relational.cityobjects.bbox_max_x[i].unwrap_or_default(),
                relational.cityobjects.bbox_max_y[i].unwrap_or_default(),
                relational.cityobjects.bbox_max_z[i].unwrap_or_default(),
            )));
        }
        let start = usize::try_from(relational.cityobjects.geometry_start[i]).unwrap_or(usize::MAX);
        let len = usize::try_from(relational.cityobjects.geometry_len[i]).unwrap_or(usize::MAX);
        for geometry in &relational.cityobjects.geometries[start..start + len] {
            object.add_geometry(geometry_handles[geometry.index()]);
        }
        cityobject_handles.push(model.cityobjects_mut().add(object)?);
    }

    for i in 0..relational.cityobjects.len() {
        let parents_start =
            usize::try_from(relational.cityobjects.parent_start[i]).unwrap_or(usize::MAX);
        let parents_len =
            usize::try_from(relational.cityobjects.parent_len[i]).unwrap_or(usize::MAX);
        for parent in &relational.cityobjects.parents[parents_start..parents_start + parents_len] {
            model
                .cityobjects_mut()
                .get_mut(cityobject_handles[i])
                .ok_or_else(|| {
                    Error::Import("missing cityobject during parent import".to_string())
                })?
                .add_parent(cityobject_handles[parent.index()]);
        }
        let children_start =
            usize::try_from(relational.cityobjects.child_start[i]).unwrap_or(usize::MAX);
        let children_len =
            usize::try_from(relational.cityobjects.child_len[i]).unwrap_or(usize::MAX);
        for child in &relational.cityobjects.children[children_start..children_start + children_len]
        {
            model
                .cityobjects_mut()
                .get_mut(cityobject_handles[i])
                .ok_or_else(|| Error::Import("missing cityobject during child import".to_string()))?
                .add_child(cityobject_handles[child.index()]);
        }
    }

    if let Some(feature_root) = relational.feature_root {
        model.set_id(Some(cityobject_handles[feature_root.index()]));
    }

    if let Some(metadata) = relational.metadata() {
        let target = model.metadata_mut();
        if let Some(extent) = metadata.geographical_extent {
            target.set_geographical_extent(BBox::new(
                extent[0], extent[1], extent[2], extent[3], extent[4], extent[5],
            ));
        }
        if let Some(id) = metadata.identifier_symbol {
            target.set_identifier(CityModelIdentifier::new(relational.symbol(id)?.to_string()));
        }
        if let Some(date) = metadata.reference_date_symbol {
            target.set_reference_date(crate::v2_0::Date::new(relational.symbol(date)?.to_string()));
        }
        if let Some(crs) = metadata.reference_system_symbol {
            target.set_reference_system(CRS::new(relational.symbol(crs)?.to_string()));
        }
        if let Some(title) = metadata.title_symbol {
            target.set_title(relational.symbol(title)?.to_string());
        }
        if let Some(extra_root) = metadata.extra_root {
            let attrs = decode_attributes(
                &relational.attributes,
                &relational.symbols.values,
                extra_root,
                Some(&geometry_handles),
            )?
            .ok_or_else(|| Error::Import("metadata extra root was not an object".to_string()))?;
            target.set_extra(Some(attrs));
        }
        if let Some(contact) = &metadata.point_of_contact {
            let mut target_contact = Contact::new();
            target_contact.set_contact_name(relational.symbol(contact.name_symbol)?.to_string());
            target_contact.set_email_address(relational.symbol(contact.email_symbol)?.to_string());
            target_contact.set_role(
                contact
                    .role_symbol
                    .map(|id| parse_contact_role(relational.symbol(id).unwrap_or("Author"))),
            );
            target_contact.set_website(
                contact
                    .website_symbol
                    .map(|id| relational.symbol(id).unwrap_or_default().to_string()),
            );
            target_contact.set_contact_type(
                contact
                    .contact_type_symbol
                    .map(|id| parse_contact_type(relational.symbol(id).unwrap_or("Individual"))),
            );
            target_contact.set_phone(
                contact
                    .phone_symbol
                    .map(|id| relational.symbol(id).unwrap_or_default().to_string()),
            );
            target_contact.set_organization(
                contact
                    .organization_symbol
                    .map(|id| relational.symbol(id).unwrap_or_default().to_string()),
            );
            if let Some(address_root) = contact.address_root {
                let attrs = decode_attributes(
                    &relational.attributes,
                    &relational.symbols.values,
                    address_root,
                    Some(&geometry_handles),
                )?
                .ok_or_else(|| {
                    Error::Import("contact address root was not an object".to_string())
                })?;
                target_contact.set_address(Some(attrs));
            }
            target.set_point_of_contact(Some(target_contact));
        }
    }

    if let Some(transform) = relational.transform() {
        model.transform_mut().set_scale(transform.scale);
        model.transform_mut().set_translate(transform.translate);
    }

    if let Some(theme) = relational.defaults.material_theme_symbol {
        model.set_default_material_theme(Some(ThemeName::new(
            relational.symbol(theme)?.to_string(),
        )));
    }
    if let Some(theme) = relational.defaults.texture_theme_symbol {
        model
            .set_default_texture_theme(Some(ThemeName::new(relational.symbol(theme)?.to_string())));
    }

    for i in 0..relational.extensions.len() {
        model.extensions_mut().add(Extension::new(
            relational
                .symbol(relational.extensions.name_symbols[i])?
                .to_string(),
            relational
                .symbol(relational.extensions.url_symbols[i])?
                .to_string(),
            relational
                .symbol(relational.extensions.version_symbols[i])?
                .to_string(),
        ));
    }

    if options.validate_default_themes {
        model.validate_default_themes()?;
    }

    Ok(model)
}

fn encode_vertex_table(vertices: &[RealWorldCoordinate]) -> VertexTableOwned {
    let mut table = VertexTableOwned::default();
    table.x.reserve(vertices.len());
    table.y.reserve(vertices.len());
    table.z.reserve(vertices.len());
    for vertex in vertices {
        table.x.push(vertex.x());
        table.y.push(vertex.y());
        table.z.push(vertex.z());
    }
    table
}

fn encode_uv_table(vertices: &[UVCoordinate]) -> UvVertexTableOwned {
    let mut table = UvVertexTableOwned::default();
    table.u.reserve(vertices.len());
    table.v.reserve(vertices.len());
    for vertex in vertices {
        table.u.push(f64::from(vertex.u()));
        table.v.push(f64::from(vertex.v()));
    }
    table
}

fn encode_cityobjects(
    model: &OwnedCityModel,
    cityobject_ids: &HashMap<u32, u32>,
    geometry_ids: &HashMap<u32, u32>,
    symbols: &mut SymbolCollector,
    attributes: &mut AttributeArenaOwned,
) -> CityObjectTableOwned {
    let mut table = CityObjectTableOwned::default();

    for (dense_index, (handle, object)) in model.cityobjects().iter().enumerate() {
        table
            .ids
            .push(CityObjectId(u32::try_from(dense_index).unwrap_or(u32::MAX)));
        table.external_id_symbols.push(symbols.intern(object.id()));
        table
            .object_type_symbols
            .push(symbols.intern(&cityobject_type_name(object.type_cityobject())));

        let parent_start = table.parents.len();
        for parent in object.parents().into_iter().flatten() {
            table.parents.push(CityObjectId(
                *cityobject_ids.get(&slot(*parent)).unwrap_or(&u32::MAX),
            ));
        }
        table
            .parent_start
            .push(u32::try_from(parent_start).unwrap_or(u32::MAX));
        table
            .parent_len
            .push(u32::try_from(table.parents.len() - parent_start).unwrap_or(u32::MAX));

        let child_start = table.children.len();
        for child in object.children().into_iter().flatten() {
            table.children.push(CityObjectId(
                *cityobject_ids.get(&slot(*child)).unwrap_or(&u32::MAX),
            ));
        }
        table
            .child_start
            .push(u32::try_from(child_start).unwrap_or(u32::MAX));
        table
            .child_len
            .push(u32::try_from(table.children.len() - child_start).unwrap_or(u32::MAX));

        let geometry_start = table.geometries.len();
        for geometry in object.geometry().into_iter().flatten() {
            table.geometries.push(GeometryId(
                *geometry_ids.get(&slot(*geometry)).unwrap_or(&u32::MAX),
            ));
        }
        table
            .geometry_start
            .push(u32::try_from(geometry_start).unwrap_or(u32::MAX));
        table
            .geometry_len
            .push(u32::try_from(table.geometries.len() - geometry_start).unwrap_or(u32::MAX));

        table.attribute_root.push(
            object
                .attributes()
                .map(|values| encode_attributes(values, geometry_ids, symbols, attributes)),
        );

        if let Some(bbox) = object.geographical_extent() {
            let values: [f64; 6] = (*bbox).into();
            table.bbox_min_x.push(Some(values[0]));
            table.bbox_min_y.push(Some(values[1]));
            table.bbox_min_z.push(Some(values[2]));
            table.bbox_max_x.push(Some(values[3]));
            table.bbox_max_y.push(Some(values[4]));
            table.bbox_max_z.push(Some(values[5]));
        } else {
            table.bbox_min_x.push(None);
            table.bbox_min_y.push(None);
            table.bbox_min_z.push(None);
            table.bbox_max_x.push(None);
            table.bbox_max_y.push(None);
            table.bbox_max_z.push(None);
        }

        let _ = handle;
    }

    table
}

fn encode_semantics(
    model: &OwnedCityModel,
    semantic_ids: &HashMap<u32, u32>,
    symbols: &mut SymbolCollector,
    attributes: &mut AttributeArenaOwned,
) -> SemanticTableOwned {
    let mut table = SemanticTableOwned::default();

    for (dense_index, (_handle, semantic)) in model.iter_semantics().enumerate() {
        table
            .ids
            .push(SemanticId(u32::try_from(dense_index).unwrap_or(u32::MAX)));
        table
            .semantic_type_symbols
            .push(symbols.intern(&semantic_type_name(semantic.type_semantic())));
        table.parent.push(
            semantic
                .parent()
                .map(|parent| SemanticId(*semantic_ids.get(&slot(parent)).unwrap_or(&u32::MAX))),
        );
        let child_start = table.children.len();
        for child in semantic.children().into_iter().flatten() {
            table.children.push(SemanticId(
                *semantic_ids.get(&slot(*child)).unwrap_or(&u32::MAX),
            ));
        }
        table
            .child_start
            .push(u32::try_from(child_start).unwrap_or(u32::MAX));
        table
            .child_len
            .push(u32::try_from(table.children.len() - child_start).unwrap_or(u32::MAX));
        table.attribute_root.push(
            semantic
                .attributes()
                .map(|values| encode_attributes(values, &HashMap::new(), symbols, attributes)),
        );
    }

    table
}

fn encode_materials(
    model: &OwnedCityModel,
    _material_ids: &HashMap<u32, u32>,
    symbols: &mut SymbolCollector,
) -> MaterialTableOwned {
    let mut table = MaterialTableOwned::default();
    for (dense_index, (_handle, material)) in model.iter_materials().enumerate() {
        table
            .ids
            .push(MaterialId(u32::try_from(dense_index).unwrap_or(u32::MAX)));
        table.name_symbols.push(symbols.intern(material.name()));
        table.ambient_intensity.push(material.ambient_intensity());
        table
            .diffuse_color
            .push(material.diffuse_color().map(Into::into));
        table
            .emissive_color
            .push(material.emissive_color().map(Into::into));
        table
            .specular_color
            .push(material.specular_color().map(Into::into));
        table.shininess.push(material.shininess());
        table.transparency.push(material.transparency());
        table.is_smooth.push(material.is_smooth());
    }
    table
}

fn encode_textures(
    model: &OwnedCityModel,
    _texture_ids: &HashMap<u32, u32>,
    symbols: &mut SymbolCollector,
) -> TextureTableOwned {
    let mut table = TextureTableOwned::default();
    for (dense_index, (_handle, texture)) in model.iter_textures().enumerate() {
        table
            .ids
            .push(TextureId(u32::try_from(dense_index).unwrap_or(u32::MAX)));
        table
            .image_uri_symbols
            .push(symbols.intern(texture.image()));
        table.texture_type_symbols.push(
            texture
                .texture_type()
                .map(|value| symbols.intern(&value.to_string())),
        );
        table.wrap_mode_symbols.push(
            texture
                .wrap_mode()
                .map(|value| symbols.intern(&value.to_string())),
        );
        table
            .image_type_symbols
            .push(symbols.intern(&texture.image_type().to_string()));
        table
            .border_color
            .push(texture.border_color().map(Into::into));
    }
    table
}

#[allow(clippy::too_many_lines)]
fn encode_geometries<'a>(
    items: impl Iterator<
        Item = (
            GeometryId,
            &'a Geometry<u32, crate::resources::storage::OwnedStringStorage>,
        ),
    >,
    semantic_ids: &HashMap<u32, u32>,
    material_ids: &HashMap<u32, u32>,
    texture_ids: &HashMap<u32, u32>,
    geometry_template_ids: &HashMap<u32, u32>,
    symbols: &mut SymbolCollector,
) -> GeometryTableOwned {
    let mut table = GeometryTableOwned::default();

    for (dense_id, geometry) in items {
        table.ids.push(dense_id);
        table
            .geometry_type_symbols
            .push(symbols.intern(&geometry.type_geometry().to_string()));
        table
            .lod_symbols
            .push(geometry.lod().map(|lod| symbols.intern(&lod.to_string())));

        if let Some(boundary) = geometry.boundaries() {
            push_boundary(boundary, &mut table);
        } else {
            push_empty_boundary(&mut table);
        }

        if let Some(semantics) = geometry.semantics() {
            push_semantic_assignments(semantics, semantic_ids, &mut table);
        } else {
            push_empty_semantic_assignments(&mut table);
        }

        let material_start = table.material_themes.len();
        if let Some(materials) = geometry.raw().materials() {
            for (theme, mapping) in materials {
                let theme_row_start_points = table.material_points.len();
                table
                    .material_points
                    .extend(mapping.points().iter().map(|value| {
                        value.map(|id| {
                            MaterialId(*material_ids.get(&id.index()).unwrap_or(&u32::MAX))
                        })
                    }));
                let theme_row_start_linestrings = table.material_linestrings.len();
                table
                    .material_linestrings
                    .extend(mapping.linestrings().iter().map(|value| {
                        value.map(|id| {
                            MaterialId(*material_ids.get(&id.index()).unwrap_or(&u32::MAX))
                        })
                    }));
                let theme_row_start_surfaces = table.material_surfaces.len();
                table
                    .material_surfaces
                    .extend(mapping.surfaces().iter().map(|value| {
                        value.map(|id| {
                            MaterialId(*material_ids.get(&id.index()).unwrap_or(&u32::MAX))
                        })
                    }));
                table.material_themes.push(GeometryMaterialThemeOwned {
                    geometry: dense_id,
                    theme_symbol: symbols.intern(theme.as_ref()),
                    point_start: u32::try_from(theme_row_start_points).unwrap_or(u32::MAX),
                    point_len: u32::try_from(table.material_points.len() - theme_row_start_points)
                        .unwrap_or(u32::MAX),
                    linestring_start: u32::try_from(theme_row_start_linestrings)
                        .unwrap_or(u32::MAX),
                    linestring_len: u32::try_from(
                        table.material_linestrings.len() - theme_row_start_linestrings,
                    )
                    .unwrap_or(u32::MAX),
                    surface_start: u32::try_from(theme_row_start_surfaces).unwrap_or(u32::MAX),
                    surface_len: u32::try_from(
                        table.material_surfaces.len() - theme_row_start_surfaces,
                    )
                    .unwrap_or(u32::MAX),
                });
            }
        }
        table
            .material_theme_start
            .push(u32::try_from(material_start).unwrap_or(u32::MAX));
        table
            .material_theme_len
            .push(u32::try_from(table.material_themes.len() - material_start).unwrap_or(u32::MAX));

        let texture_start = table.texture_themes.len();
        if let Some(textures) = geometry.raw().textures() {
            for (theme, mapping) in textures {
                let vertex_start = table.texture_vertex_refs.len();
                table.texture_vertex_refs.extend(
                    mapping
                        .vertices()
                        .iter()
                        .map(|value| value.map(|vertex| VertexId(vertex.value()))),
                );
                let ring_start = table.texture_rings.len();
                table
                    .texture_rings
                    .extend(mapping.rings().iter().map(crate::v2_0::VertexIndex::value));
                let ring_texture_start = table.texture_ring_textures.len();
                table
                    .texture_ring_textures
                    .extend(mapping.ring_textures().iter().copied().map(|value| {
                        value
                            .map(|id| TextureId(*texture_ids.get(&id.index()).unwrap_or(&u32::MAX)))
                    }));
                table.texture_themes.push(GeometryTextureThemeOwned {
                    geometry: dense_id,
                    theme_symbol: symbols.intern(theme.as_ref()),
                    vertex_start: u32::try_from(vertex_start).unwrap_or(u32::MAX),
                    vertex_len: u32::try_from(table.texture_vertex_refs.len() - vertex_start)
                        .unwrap_or(u32::MAX),
                    ring_start: u32::try_from(ring_start).unwrap_or(u32::MAX),
                    ring_len: u32::try_from(table.texture_rings.len() - ring_start)
                        .unwrap_or(u32::MAX),
                    ring_texture_start: u32::try_from(ring_texture_start).unwrap_or(u32::MAX),
                    ring_texture_len: u32::try_from(
                        table.texture_ring_textures.len() - ring_texture_start,
                    )
                    .unwrap_or(u32::MAX),
                });
            }
        }
        table
            .texture_theme_start
            .push(u32::try_from(texture_start).unwrap_or(u32::MAX));
        table
            .texture_theme_len
            .push(u32::try_from(table.texture_themes.len() - texture_start).unwrap_or(u32::MAX));

        if let Some(instance) = geometry.instance() {
            table.template_ref.push(Some(GeometryTemplateId(
                *geometry_template_ids
                    .get(&slot(instance.template()))
                    .unwrap_or(&u32::MAX),
            )));
            table
                .reference_point
                .push(Some(VertexId(instance.reference_point().value())));
            table
                .transform_matrix
                .push(instance.transformation().into_array());
        } else {
            table.template_ref.push(None);
            table.reference_point.push(None);
            table
                .transform_matrix
                .push(AffineTransform3D::identity().into_array());
        }
    }

    table
}

fn encode_extensions(
    extensions: Option<&crate::v2_0::Extensions<crate::resources::storage::OwnedStringStorage>>,
    symbols: &mut SymbolCollector,
) -> ExtensionTableOwned {
    let mut table = ExtensionTableOwned::default();
    if let Some(extensions) = extensions {
        for extension in extensions {
            table.name_symbols.push(symbols.intern(extension.name()));
            table.url_symbols.push(symbols.intern(extension.url()));
            table
                .version_symbols
                .push(symbols.intern(extension.version()));
        }
    }
    table
}

fn encode_metadata(
    metadata: Option<&Metadata<crate::resources::storage::OwnedStringStorage>>,
    geometry_ids: &HashMap<u32, u32>,
    symbols: &mut SymbolCollector,
    attributes: &mut AttributeArenaOwned,
) -> Option<MetadataOwned> {
    metadata.map(|metadata| MetadataOwned {
        geographical_extent: metadata.geographical_extent().map(|bbox| (*bbox).into()),
        identifier_symbol: metadata
            .identifier()
            .map(|value| symbols.intern(&value.to_string())),
        reference_date_symbol: metadata
            .reference_date()
            .map(|value| symbols.intern(&value.to_string())),
        reference_system_symbol: metadata
            .reference_system()
            .map(|value| symbols.intern(&value.to_string())),
        title_symbol: metadata.title().map(|value| symbols.intern(value)),
        extra_root: metadata
            .extra()
            .map(|value| encode_attributes(value, geometry_ids, symbols, attributes)),
        point_of_contact: metadata.point_of_contact().map(|contact| ContactOwned {
            name_symbol: symbols.intern(contact.contact_name()),
            email_symbol: symbols.intern(contact.email_address()),
            role_symbol: contact.role().map(|role| symbols.intern(&role.to_string())),
            website_symbol: contact
                .website()
                .as_ref()
                .map(|value| symbols.intern(value)),
            contact_type_symbol: contact
                .contact_type()
                .map(|value| symbols.intern(&value.to_string())),
            address_root: contact
                .address()
                .map(|value| encode_attributes(value, geometry_ids, symbols, attributes)),
            phone_symbol: contact.phone().as_ref().map(|value| symbols.intern(value)),
            organization_symbol: contact
                .organization()
                .as_ref()
                .map(|value| symbols.intern(value)),
        }),
    })
}

fn encode_attributes(
    attributes_map: &Attributes<crate::resources::storage::OwnedStringStorage>,
    geometry_ids: &HashMap<u32, u32>,
    symbols: &mut SymbolCollector,
    arena: &mut AttributeArenaOwned,
) -> AttributeNodeId {
    encode_attribute_value(
        &AttributeValue::Map(
            attributes_map
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        ),
        None,
        geometry_ids,
        symbols,
        arena,
    )
}

fn encode_attribute_value(
    value: &AttributeValue<crate::resources::storage::OwnedStringStorage>,
    key: Option<SymbolId>,
    geometry_ids: &HashMap<u32, u32>,
    symbols: &mut SymbolCollector,
    arena: &mut AttributeArenaOwned,
) -> AttributeNodeId {
    let id = AttributeNodeId(u32::try_from(arena.node_type.len()).unwrap_or(u32::MAX));
    arena.key_symbol.push(key);
    arena.string_value_symbol.push(None);
    arena.bool_value.push(None);
    arena.unsigned_value.push(None);
    arena.int_value.push(None);
    arena.float_value.push(None);
    arena.geometry_value.push(None);
    arena
        .first_child_offset
        .push(u32::try_from(arena.child_nodes.len()).unwrap_or(u32::MAX));
    arena.child_len.push(0);

    match value {
        AttributeValue::Null => arena.node_type.push(AttributeNodeType::Null),
        AttributeValue::Bool(value) => {
            arena.node_type.push(AttributeNodeType::Bool);
            arena.bool_value[id.index()] = Some(*value);
        }
        AttributeValue::Unsigned(value) => {
            arena.node_type.push(AttributeNodeType::Unsigned);
            arena.unsigned_value[id.index()] = Some(*value);
        }
        AttributeValue::Integer(value) => {
            arena.node_type.push(AttributeNodeType::Integer);
            arena.int_value[id.index()] = Some(*value);
        }
        AttributeValue::Float(value) => {
            arena.node_type.push(AttributeNodeType::Float);
            arena.float_value[id.index()] = Some(*value);
        }
        AttributeValue::String(value) => {
            arena.node_type.push(AttributeNodeType::String);
            arena.string_value_symbol[id.index()] = Some(symbols.intern(value));
        }
        AttributeValue::Geometry(handle) => {
            arena.node_type.push(AttributeNodeType::GeometryRef);
            arena.geometry_value[id.index()] = Some(GeometryId(
                *geometry_ids.get(&slot(*handle)).unwrap_or(&u32::MAX),
            ));
        }
        AttributeValue::Vec(values) => {
            arena.node_type.push(AttributeNodeType::Array);
            let start = arena.child_nodes.len();
            for child in values {
                let child_id = encode_attribute_value(child, None, geometry_ids, symbols, arena);
                arena.child_nodes.push(child_id);
            }
            arena.first_child_offset[id.index()] = u32::try_from(start).unwrap_or(u32::MAX);
            arena.child_len[id.index()] =
                u32::try_from(arena.child_nodes.len() - start).unwrap_or(u32::MAX);
        }
        AttributeValue::Map(values) => {
            arena.node_type.push(AttributeNodeType::Object);
            let mut ordered = values.iter().collect::<Vec<_>>();
            ordered.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));
            let start = arena.child_nodes.len();
            for (child_key, child_value) in ordered {
                let child_id = encode_attribute_value(
                    child_value,
                    Some(symbols.intern(child_key)),
                    geometry_ids,
                    symbols,
                    arena,
                );
                arena.child_nodes.push(child_id);
            }
            arena.first_child_offset[id.index()] = u32::try_from(start).unwrap_or(u32::MAX);
            arena.child_len[id.index()] =
                u32::try_from(arena.child_nodes.len() - start).unwrap_or(u32::MAX);
        }
    }

    id
}

#[allow(clippy::too_many_arguments)]
fn decode_geometry(
    table: &GeometryTableOwned,
    index: usize,
    template_handles: &[crate::resources::handles::GeometryTemplateHandle],
    semantic_handles: &[crate::resources::handles::SemanticHandle],
    material_handles: &[crate::resources::handles::MaterialHandle],
    texture_handles: &[crate::resources::handles::TextureHandle],
    template_vertices: bool,
    relational: &OwnedRelationalSnapshot,
) -> Result<Geometry<u32, crate::resources::storage::OwnedStringStorage>> {
    let boundary = decode_boundary(table, index)?;
    let semantics = decode_semantic_map(table, index, semantic_handles);
    let materials = decode_material_maps(table, index, material_handles, relational)?;
    let textures = decode_texture_maps(table, index, texture_handles, relational)?;
    let instance = match (table.template_ref[index], table.reference_point[index]) {
        (Some(template), Some(reference_point)) => Some(StoredGeometryInstance {
            template: template_handles[template.index()],
            reference_point: VertexIndex::new(reference_point.0),
            transformation: AffineTransform3D::new(table.transform_matrix[index]),
        }),
        _ => None,
    };

    let _ = template_vertices;

    Ok(Geometry::from_stored_parts(StoredGeometryParts {
        type_geometry: parse_geometry_type(relational.symbol(table.geometry_type_symbols[index])?),
        lod: table.lod_symbols[index]
            .map(|symbol| parse_lod(relational.symbol(symbol).unwrap_or("0"))),
        boundaries: boundary,
        semantics,
        materials,
        textures,
        instance,
    }))
}

fn decode_boundary(table: &GeometryTableOwned, index: usize) -> Result<Option<Boundary<u32>>> {
    let vertices: Vec<VertexIndex<u32>> = slice_u32(
        &table.boundary_vertices,
        table.boundary_vertex_start[index],
        table.boundary_vertex_len[index],
    )?
    .iter()
    .map(|value| VertexIndex::new(value.0))
    .collect();
    let rings: Vec<VertexIndex<u32>> = slice_copy(
        &table.boundary_rings,
        table.boundary_ring_start[index],
        table.boundary_ring_len[index],
    )?
    .into_iter()
    .map(VertexIndex::new)
    .collect();
    let surfaces: Vec<VertexIndex<u32>> = slice_copy(
        &table.boundary_surfaces,
        table.boundary_surface_start[index],
        table.boundary_surface_len[index],
    )?
    .into_iter()
    .map(VertexIndex::new)
    .collect();
    let shells: Vec<VertexIndex<u32>> = slice_copy(
        &table.boundary_shells,
        table.boundary_shell_start[index],
        table.boundary_shell_len[index],
    )?
    .into_iter()
    .map(VertexIndex::new)
    .collect();
    let solids: Vec<VertexIndex<u32>> = slice_copy(
        &table.boundary_solids,
        table.boundary_solid_start[index],
        table.boundary_solid_len[index],
    )?
    .into_iter()
    .map(VertexIndex::new)
    .collect();

    if vertices.is_empty()
        && rings.is_empty()
        && surfaces.is_empty()
        && shells.is_empty()
        && solids.is_empty()
    {
        return Ok(None);
    }

    Ok(Some(Boundary::from_parts(
        vertices, rings, surfaces, shells, solids,
    )?))
}

fn decode_semantic_map(
    table: &GeometryTableOwned,
    index: usize,
    semantic_handles: &[crate::resources::handles::SemanticHandle],
) -> Option<SemanticMap<u32>> {
    let points = slice_copy(
        &table.semantic_points,
        table.semantic_point_start[index],
        table.semantic_point_len[index],
    )
    .ok()?;
    let linestrings = slice_copy(
        &table.semantic_linestrings,
        table.semantic_linestring_start[index],
        table.semantic_linestring_len[index],
    )
    .ok()?;
    let surfaces = slice_copy(
        &table.semantic_surfaces,
        table.semantic_surface_start[index],
        table.semantic_surface_len[index],
    )
    .ok()?;

    if points.is_empty() && linestrings.is_empty() && surfaces.is_empty() {
        return None;
    }

    let mut map = SemanticMap::new();
    for value in points {
        map.add_point(value.map(|id| semantic_handles[id.index()]));
    }
    for value in linestrings {
        map.add_linestring(value.map(|id| semantic_handles[id.index()]));
    }
    for value in surfaces {
        map.add_surface(value.map(|id| semantic_handles[id.index()]));
    }
    Some(map)
}

#[allow(clippy::type_complexity)]
fn decode_material_maps(
    table: &GeometryTableOwned,
    index: usize,
    material_handles: &[crate::resources::handles::MaterialHandle],
    relational: &OwnedRelationalSnapshot,
) -> Result<
    Option<
        Vec<(
            ThemeName<crate::resources::storage::OwnedStringStorage>,
            MaterialMap<u32>,
        )>,
    >,
> {
    let start = usize::try_from(table.material_theme_start[index]).unwrap_or(usize::MAX);
    let len = usize::try_from(table.material_theme_len[index]).unwrap_or(usize::MAX);
    if len == 0 {
        return Ok(None);
    }

    let mut items = Vec::with_capacity(len);
    for row in &table.material_themes[start..start + len] {
        let mut map = MaterialMap::new();
        for value in slice_copy(&table.material_points, row.point_start, row.point_len)? {
            map.add_point(value.map(|id| material_handles[id.index()]));
        }
        for value in slice_copy(
            &table.material_linestrings,
            row.linestring_start,
            row.linestring_len,
        )? {
            map.add_linestring(value.map(|id| material_handles[id.index()]));
        }
        for value in slice_copy(&table.material_surfaces, row.surface_start, row.surface_len)? {
            map.add_surface(value.map(|id| material_handles[id.index()]));
        }
        items.push((
            ThemeName::new(relational.symbol(row.theme_symbol)?.to_string()),
            map,
        ));
    }

    Ok(Some(items))
}

#[allow(clippy::type_complexity)]
fn decode_texture_maps(
    table: &GeometryTableOwned,
    index: usize,
    texture_handles: &[crate::resources::handles::TextureHandle],
    relational: &OwnedRelationalSnapshot,
) -> Result<
    Option<
        Vec<(
            ThemeName<crate::resources::storage::OwnedStringStorage>,
            PublicTextureMap<u32>,
        )>,
    >,
> {
    let start = usize::try_from(table.texture_theme_start[index]).unwrap_or(usize::MAX);
    let len = usize::try_from(table.texture_theme_len[index]).unwrap_or(usize::MAX);
    if len == 0 {
        return Ok(None);
    }

    let mut items = Vec::with_capacity(len);
    for row in &table.texture_themes[start..start + len] {
        let mut map = PublicTextureMap::new();
        for value in slice_copy(&table.texture_vertex_refs, row.vertex_start, row.vertex_len)? {
            map.add_vertex(value.map(|id| VertexIndex::new(id.0)));
        }
        for value in slice_copy(&table.texture_rings, row.ring_start, row.ring_len)? {
            map.add_ring(VertexIndex::new(value));
        }
        for value in slice_copy(
            &table.texture_ring_textures,
            row.ring_texture_start,
            row.ring_texture_len,
        )? {
            map.add_ring_texture(value.map(|id| texture_handles[id.index()]));
        }
        items.push((
            ThemeName::new(relational.symbol(row.theme_symbol)?.to_string()),
            map,
        ));
    }

    Ok(Some(items))
}

fn decode_attributes(
    arena: &AttributeArenaOwned,
    symbols: &[String],
    root: AttributeNodeId,
    geometry_handles: Option<&[crate::resources::handles::GeometryHandle]>,
) -> Result<Option<Attributes<crate::resources::storage::OwnedStringStorage>>> {
    match decode_attribute_value(arena, symbols, root, geometry_handles)? {
        AttributeValue::Map(values) => Ok(Some(Attributes::from(values))),
        _ => Ok(None),
    }
}

fn decode_attribute_value(
    arena: &AttributeArenaOwned,
    symbols: &[String],
    id: AttributeNodeId,
    geometry_handles: Option<&[crate::resources::handles::GeometryHandle]>,
) -> Result<AttributeValue<crate::resources::storage::OwnedStringStorage>> {
    let index = id.index();
    let kind = arena
        .node_type
        .get(index)
        .copied()
        .ok_or_else(|| Error::Import(format!("missing attribute node {}", id.0)))?;

    Ok(match kind {
        AttributeNodeType::Null => AttributeValue::Null,
        AttributeNodeType::Bool => AttributeValue::Bool(arena.bool_value[index].unwrap_or(false)),
        AttributeNodeType::Unsigned => {
            AttributeValue::Unsigned(arena.unsigned_value[index].unwrap_or_default())
        }
        AttributeNodeType::Integer => {
            AttributeValue::Integer(arena.int_value[index].unwrap_or_default())
        }
        AttributeNodeType::Float => {
            AttributeValue::Float(arena.float_value[index].unwrap_or_default())
        }
        AttributeNodeType::String => AttributeValue::String(
            arena
                .string_value_symbol
                .get(index)
                .copied()
                .flatten()
                .and_then(|symbol| symbols.get(symbol.index()).cloned())
                .unwrap_or_default(),
        ),
        AttributeNodeType::GeometryRef => AttributeValue::Geometry(
            arena.geometry_value[index]
                .and_then(|geometry| {
                    geometry_handles.and_then(|handles| handles.get(geometry.index()).copied())
                })
                .unwrap_or_default(),
        ),
        AttributeNodeType::Array => {
            let mut values = Vec::new();
            let start = usize::try_from(arena.first_child_offset[index]).unwrap_or(usize::MAX);
            let len = usize::try_from(arena.child_len[index]).unwrap_or(usize::MAX);
            for child in &arena.child_nodes[start..start + len] {
                values.push(decode_attribute_value(
                    arena,
                    symbols,
                    *child,
                    geometry_handles,
                )?);
            }
            AttributeValue::Vec(values)
        }
        AttributeNodeType::Object => {
            let mut values = HashMap::new();
            let start = usize::try_from(arena.first_child_offset[index]).unwrap_or(usize::MAX);
            let len = usize::try_from(arena.child_len[index]).unwrap_or(usize::MAX);
            for child in &arena.child_nodes[start..start + len] {
                let child_index = child.index();
                let key = arena.key_symbol[child_index]
                    .and_then(|symbol| symbols.get(symbol.index()).cloned())
                    .unwrap_or_default();
                values.insert(
                    key,
                    decode_attribute_value(arena, symbols, *child, geometry_handles)?,
                );
            }
            AttributeValue::Map(values)
        }
    })
}

fn dense_cityobject_ids(model: &OwnedCityModel) -> HashMap<u32, u32> {
    model
        .cityobjects()
        .iter()
        .enumerate()
        .map(|(dense, (handle, _))| (slot(handle), u32::try_from(dense).unwrap_or(u32::MAX)))
        .collect()
}

fn dense_cityobject_remap(model: &OwnedCityModel) -> DenseIndexRemap {
    dense_remap_from_slots(model.cityobjects().iter().map(|(handle, _)| slot(handle)))
}

fn dense_geometry_ids(model: &OwnedCityModel) -> HashMap<u32, u32> {
    model
        .iter_geometries()
        .enumerate()
        .map(|(dense, (handle, _))| (slot(handle), u32::try_from(dense).unwrap_or(u32::MAX)))
        .collect()
}

fn dense_geometry_template_ids(model: &OwnedCityModel) -> HashMap<u32, u32> {
    model
        .iter_geometry_templates()
        .enumerate()
        .map(|(dense, (handle, _))| (slot(handle), u32::try_from(dense).unwrap_or(u32::MAX)))
        .collect()
}

fn dense_geometry_template_remap(model: &OwnedCityModel) -> DenseIndexRemap {
    dense_remap_from_slots(
        model
            .iter_geometry_templates()
            .map(|(handle, _)| slot(handle)),
    )
}

fn dense_semantic_ids(model: &OwnedCityModel) -> HashMap<u32, u32> {
    model
        .iter_semantics()
        .enumerate()
        .map(|(dense, (handle, _))| (slot(handle), u32::try_from(dense).unwrap_or(u32::MAX)))
        .collect()
}

fn dense_material_ids(model: &OwnedCityModel) -> HashMap<u32, u32> {
    model
        .iter_materials()
        .enumerate()
        .map(|(dense, (handle, _))| (slot(handle), u32::try_from(dense).unwrap_or(u32::MAX)))
        .collect()
}

fn dense_texture_ids(model: &OwnedCityModel) -> HashMap<u32, u32> {
    model
        .iter_textures()
        .enumerate()
        .map(|(dense, (handle, _))| (slot(handle), u32::try_from(dense).unwrap_or(u32::MAX)))
        .collect()
}

fn dense_remap_from_slots(slots: impl Iterator<Item = u32>) -> DenseIndexRemap {
    let occupied = slots
        .map(|slot| usize::try_from(slot).unwrap_or(usize::MAX))
        .collect::<Vec<_>>();
    let capacity = occupied.iter().copied().max().map_or(0, |max| max + 1);
    DenseIndexRemap::from_occupied_indices(capacity, occupied)
}

fn slot<H>(handle: H) -> u32
where
    H: Copy,
    H: crate::resources::handles::HandleType,
{
    handle.to_raw().index()
}

fn push_boundary(boundary: &Boundary<u32>, table: &mut GeometryTableOwned) {
    let vertex_start = table.boundary_vertices.len();
    table.boundary_vertices.extend(
        boundary
            .vertices()
            .iter()
            .map(|value| VertexId(value.value())),
    );
    table
        .boundary_vertex_start
        .push(u32::try_from(vertex_start).unwrap_or(u32::MAX));
    table
        .boundary_vertex_len
        .push(u32::try_from(table.boundary_vertices.len() - vertex_start).unwrap_or(u32::MAX));

    let ring_start = table.boundary_rings.len();
    table
        .boundary_rings
        .extend(boundary.rings_raw().iter().copied());
    table
        .boundary_ring_start
        .push(u32::try_from(ring_start).unwrap_or(u32::MAX));
    table
        .boundary_ring_len
        .push(u32::try_from(table.boundary_rings.len() - ring_start).unwrap_or(u32::MAX));

    let surface_start = table.boundary_surfaces.len();
    table
        .boundary_surfaces
        .extend(boundary.surfaces_raw().iter().copied());
    table
        .boundary_surface_start
        .push(u32::try_from(surface_start).unwrap_or(u32::MAX));
    table
        .boundary_surface_len
        .push(u32::try_from(table.boundary_surfaces.len() - surface_start).unwrap_or(u32::MAX));

    let shell_start = table.boundary_shells.len();
    table
        .boundary_shells
        .extend(boundary.shells_raw().iter().copied());
    table
        .boundary_shell_start
        .push(u32::try_from(shell_start).unwrap_or(u32::MAX));
    table
        .boundary_shell_len
        .push(u32::try_from(table.boundary_shells.len() - shell_start).unwrap_or(u32::MAX));

    let solid_start = table.boundary_solids.len();
    table
        .boundary_solids
        .extend(boundary.solids_raw().iter().copied());
    table
        .boundary_solid_start
        .push(u32::try_from(solid_start).unwrap_or(u32::MAX));
    table
        .boundary_solid_len
        .push(u32::try_from(table.boundary_solids.len() - solid_start).unwrap_or(u32::MAX));
}

fn push_empty_boundary(table: &mut GeometryTableOwned) {
    table
        .boundary_vertex_start
        .push(u32::try_from(table.boundary_vertices.len()).unwrap_or(u32::MAX));
    table.boundary_vertex_len.push(0);
    table
        .boundary_ring_start
        .push(u32::try_from(table.boundary_rings.len()).unwrap_or(u32::MAX));
    table.boundary_ring_len.push(0);
    table
        .boundary_surface_start
        .push(u32::try_from(table.boundary_surfaces.len()).unwrap_or(u32::MAX));
    table.boundary_surface_len.push(0);
    table
        .boundary_shell_start
        .push(u32::try_from(table.boundary_shells.len()).unwrap_or(u32::MAX));
    table.boundary_shell_len.push(0);
    table
        .boundary_solid_start
        .push(u32::try_from(table.boundary_solids.len()).unwrap_or(u32::MAX));
    table.boundary_solid_len.push(0);
}

fn push_semantic_assignments(
    semantics: crate::v2_0::geometry::SemanticMapView<'_, u32>,
    semantic_ids: &HashMap<u32, u32>,
    table: &mut GeometryTableOwned,
) {
    let point_start = table.semantic_points.len();
    table
        .semantic_points
        .extend(semantics.points().iter().map(|value| {
            value.map(|id| SemanticId(*semantic_ids.get(&slot(id)).unwrap_or(&u32::MAX)))
        }));
    table
        .semantic_point_start
        .push(u32::try_from(point_start).unwrap_or(u32::MAX));
    table
        .semantic_point_len
        .push(u32::try_from(table.semantic_points.len() - point_start).unwrap_or(u32::MAX));

    let linestring_start = table.semantic_linestrings.len();
    table
        .semantic_linestrings
        .extend(semantics.linestrings().iter().map(|value| {
            value.map(|id| SemanticId(*semantic_ids.get(&slot(id)).unwrap_or(&u32::MAX)))
        }));
    table
        .semantic_linestring_start
        .push(u32::try_from(linestring_start).unwrap_or(u32::MAX));
    table.semantic_linestring_len.push(
        u32::try_from(table.semantic_linestrings.len() - linestring_start).unwrap_or(u32::MAX),
    );

    let surface_start = table.semantic_surfaces.len();
    table
        .semantic_surfaces
        .extend(semantics.surfaces().iter().map(|value| {
            value.map(|id| SemanticId(*semantic_ids.get(&slot(id)).unwrap_or(&u32::MAX)))
        }));
    table
        .semantic_surface_start
        .push(u32::try_from(surface_start).unwrap_or(u32::MAX));
    table
        .semantic_surface_len
        .push(u32::try_from(table.semantic_surfaces.len() - surface_start).unwrap_or(u32::MAX));
}

fn push_empty_semantic_assignments(table: &mut GeometryTableOwned) {
    table
        .semantic_point_start
        .push(u32::try_from(table.semantic_points.len()).unwrap_or(u32::MAX));
    table.semantic_point_len.push(0);
    table
        .semantic_linestring_start
        .push(u32::try_from(table.semantic_linestrings.len()).unwrap_or(u32::MAX));
    table.semantic_linestring_len.push(0);
    table
        .semantic_surface_start
        .push(u32::try_from(table.semantic_surfaces.len()).unwrap_or(u32::MAX));
    table.semantic_surface_len.push(0);
}

fn slice_copy<T: Copy>(items: &[T], start: u32, len: u32) -> Result<Vec<T>> {
    let start = usize::try_from(start).unwrap_or(usize::MAX);
    let len = usize::try_from(len).unwrap_or(usize::MAX);
    items
        .get(start..start + len)
        .map(<[T]>::to_vec)
        .ok_or_else(|| Error::Import("invalid relational slice".to_string()))
}

fn slice_u32(items: &[VertexId], start: u32, len: u32) -> Result<&[VertexId]> {
    let start = usize::try_from(start).unwrap_or(usize::MAX);
    let len = usize::try_from(len).unwrap_or(usize::MAX);
    items
        .get(start..start + len)
        .ok_or_else(|| Error::Import("invalid relational vertex slice".to_string()))
}

fn cityobject_type_name(
    value: &CityObjectType<crate::resources::storage::OwnedStringStorage>,
) -> String {
    value.to_string()
}

fn semantic_type_name(
    value: &SemanticType<crate::resources::storage::OwnedStringStorage>,
) -> String {
    match value {
        SemanticType::Extension(value) => value.clone(),
        _ => format!("{value:?}"),
    }
}

fn parse_cityobject_type(
    value: &str,
) -> Result<CityObjectType<crate::resources::storage::OwnedStringStorage>> {
    CityObjectType::from_str(value)
}

fn parse_geometry_type(value: &str) -> GeometryType {
    GeometryType::from_str(value).unwrap_or(GeometryType::MultiPoint)
}

fn parse_semantic_type(value: &str) -> SemanticType<crate::resources::storage::OwnedStringStorage> {
    match value {
        "Default" => SemanticType::Default,
        "RoofSurface" => SemanticType::RoofSurface,
        "GroundSurface" => SemanticType::GroundSurface,
        "WallSurface" => SemanticType::WallSurface,
        "ClosureSurface" => SemanticType::ClosureSurface,
        "OuterCeilingSurface" => SemanticType::OuterCeilingSurface,
        "OuterFloorSurface" => SemanticType::OuterFloorSurface,
        "Window" => SemanticType::Window,
        "Door" => SemanticType::Door,
        "InteriorWallSurface" => SemanticType::InteriorWallSurface,
        "CeilingSurface" => SemanticType::CeilingSurface,
        "FloorSurface" => SemanticType::FloorSurface,
        "WaterSurface" => SemanticType::WaterSurface,
        "WaterGroundSurface" => SemanticType::WaterGroundSurface,
        "WaterClosureSurface" => SemanticType::WaterClosureSurface,
        "TrafficArea" => SemanticType::TrafficArea,
        "AuxiliaryTrafficArea" => SemanticType::AuxiliaryTrafficArea,
        "TransportationMarking" => SemanticType::TransportationMarking,
        "TransportationHole" => SemanticType::TransportationHole,
        other => SemanticType::Extension(other.to_string()),
    }
}

fn parse_lod(value: &str) -> LoD {
    match value {
        "0.0" => LoD::LoD0_0,
        "0.1" => LoD::LoD0_1,
        "0.2" => LoD::LoD0_2,
        "0.3" => LoD::LoD0_3,
        "1" => LoD::LoD1,
        "1.0" => LoD::LoD1_0,
        "1.1" => LoD::LoD1_1,
        "1.2" => LoD::LoD1_2,
        "1.3" => LoD::LoD1_3,
        "2" => LoD::LoD2,
        "2.0" => LoD::LoD2_0,
        "2.1" => LoD::LoD2_1,
        "2.2" => LoD::LoD2_2,
        "2.3" => LoD::LoD2_3,
        "3" => LoD::LoD3,
        "3.0" => LoD::LoD3_0,
        "3.1" => LoD::LoD3_1,
        "3.2" => LoD::LoD3_2,
        "3.3" => LoD::LoD3_3,
        _ => LoD::LoD0,
    }
}

fn parse_image_type(value: &str) -> crate::v2_0::ImageType {
    match value {
        "JPG" => crate::v2_0::ImageType::Jpg,
        _ => crate::v2_0::ImageType::Png,
    }
}

fn parse_wrap_mode(value: &str) -> WrapMode {
    match value {
        "wrap" => WrapMode::Wrap,
        "mirror" => WrapMode::Mirror,
        "clamp" => WrapMode::Clamp,
        "border" => WrapMode::Border,
        _ => WrapMode::None,
    }
}

fn parse_texture_type(value: &str) -> crate::v2_0::TextureType {
    match value {
        "specific" => crate::v2_0::TextureType::Specific,
        "typical" => crate::v2_0::TextureType::Typical,
        _ => crate::v2_0::TextureType::Unknown,
    }
}

fn parse_contact_role(value: &str) -> ContactRole {
    match value {
        "CoAuthor" => ContactRole::CoAuthor,
        "Processor" => ContactRole::Processor,
        "PointOfContact" => ContactRole::PointOfContact,
        "Owner" => ContactRole::Owner,
        "User" => ContactRole::User,
        "Distributor" => ContactRole::Distributor,
        "Originator" => ContactRole::Originator,
        "Custodian" => ContactRole::Custodian,
        "ResourceProvider" => ContactRole::ResourceProvider,
        "RightsHolder" => ContactRole::RightsHolder,
        "Sponsor" => ContactRole::Sponsor,
        "PrincipalInvestigator" => ContactRole::PrincipalInvestigator,
        "Stakeholder" => ContactRole::Stakeholder,
        "Publisher" => ContactRole::Publisher,
        _ => ContactRole::Author,
    }
}

fn parse_contact_type(value: &str) -> ContactType {
    match value {
        "Organization" => ContactType::Organization,
        _ => ContactType::Individual,
    }
}

#[cfg(test)]
mod tests {
    use super::{RelationalAccess, RelationalImportOptions, RelationalModelBuilder};
    use crate::CityModelType;
    use crate::query::summary;
    use crate::v2_0::geometry::semantic::SemanticType;
    use crate::v2_0::{
        CityObject, CityObjectType, Extension, GeometryDraft, ImageType, Material,
        OwnedAttributeValue, OwnedCityModel, RingDraft, Semantic, SurfaceDraft, Texture, ThemeName,
    };

    #[allow(clippy::too_many_lines)]
    #[test]
    fn owned_model_roundtrips_through_relational_builder() {
        let mut model = OwnedCityModel::new(CityModelType::CityJSONFeature);

        let roof_semantic = model
            .add_semantic(Semantic::new(SemanticType::RoofSurface))
            .unwrap();
        let roof_material = model
            .add_material(Material::new("roof".to_string()))
            .unwrap();
        let roof_texture = model
            .add_texture(Texture::new("roof.png".to_string(), ImageType::Png))
            .unwrap();

        let geometry = GeometryDraft::multi_surface(
            None,
            [SurfaceDraft::new(
                RingDraft::new([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]).with_texture(
                    "tex".to_string(),
                    roof_texture,
                    [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
                ),
                [],
            )
            .with_semantic(roof_semantic)
            .with_material("mat".to_string(), roof_material)],
        )
        .insert_into(&mut model)
        .unwrap();

        let mut building = CityObject::new(
            crate::v2_0::CityObjectIdentifier::new("building-1".to_string()),
            CityObjectType::Building,
        );
        building.add_geometry(geometry);
        building.attributes_mut().insert(
            "name".to_string(),
            OwnedAttributeValue::String("demo".to_string()),
        );
        let building_handle = model.cityobjects_mut().add(building).unwrap();
        model.set_id(Some(building_handle));
        model.metadata_mut().set_title("demo dataset".to_string());
        model.metadata_mut().extra_mut().insert(
            "source".to_string(),
            OwnedAttributeValue::String("unit-test".to_string()),
        );
        model.set_default_material_theme(Some(ThemeName::new("mat".to_string())));
        model.set_default_texture_theme(Some(ThemeName::new("tex".to_string())));
        model.extensions_mut().add(Extension::new(
            "noise".to_string(),
            "https://example.com/noise.json".to_string(),
            "1.0".to_string(),
        ));

        let relational = model.relational_snapshot();
        let baseline = summary(&model);

        assert_eq!(relational.cityobjects().len(), 1);
        assert_eq!(relational.geometries().len(), 1);
        assert!(relational.symbols().len() >= 6);

        let mut builder =
            RelationalModelBuilder::new(model.type_citymodel(), RelationalImportOptions::default());
        builder
            .push_symbols(relational.symbol_table().clone())
            .unwrap();
        builder
            .push_vertices(relational.vertex_table().clone())
            .unwrap();
        builder
            .push_template_vertices(relational.template_vertex_table().clone())
            .unwrap();
        builder
            .push_uv_vertices(relational.uv_vertex_table().clone())
            .unwrap();
        builder
            .push_semantics(relational.semantics().clone())
            .unwrap();
        builder
            .push_materials(relational.materials().clone())
            .unwrap();
        builder
            .push_textures(relational.textures().clone())
            .unwrap();
        builder
            .push_attributes(relational.attributes().clone())
            .unwrap();
        builder
            .push_cityobjects(relational.cityobjects().clone())
            .unwrap();
        builder
            .push_geometries(relational.geometries().clone())
            .unwrap();
        builder
            .push_geometry_templates(relational.geometry_templates().clone())
            .unwrap();
        builder
            .push_metadata(relational.metadata_owned().cloned())
            .unwrap();
        builder
            .push_transform(relational.transform_owned().copied())
            .unwrap();
        builder
            .push_defaults(relational.defaults_owned().clone())
            .unwrap();
        builder
            .push_extensions(relational.extensions().clone())
            .unwrap();
        builder
            .push_feature_root(relational.feature_root())
            .unwrap();

        let rebuilt = builder.finish().unwrap();
        let rebuilt_summary = summary(&rebuilt);

        assert_eq!(rebuilt_summary.cityobject_count, baseline.cityobject_count);
        assert_eq!(rebuilt_summary.geometry_count, baseline.geometry_count);
        assert_eq!(rebuilt_summary.semantic_count, baseline.semantic_count);
        assert_eq!(rebuilt_summary.material_count, baseline.material_count);
        assert_eq!(rebuilt_summary.texture_count, baseline.texture_count);
        assert_eq!(
            rebuilt.id().unwrap(),
            rebuilt.cityobjects().first().unwrap().0
        );
        assert_eq!(rebuilt.metadata().unwrap().title(), Some("demo dataset"));
        assert!(rebuilt.has_material_theme("mat"));
        assert!(rebuilt.has_texture_theme("tex"));
        assert_eq!(rebuilt.extensions().unwrap().len(), 1);
        assert_eq!(
            rebuilt
                .cityobjects()
                .first()
                .unwrap()
                .1
                .attributes()
                .unwrap()
                .get("name"),
            Some(&OwnedAttributeValue::String("demo".to_string()))
        );
    }
}
