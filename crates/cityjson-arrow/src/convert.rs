#![allow(clippy::wildcard_imports)]

use crate::error::{Error, Result};
use crate::schema::{
    CanonicalSchemaSet, CityArrowHeader, CityArrowPackageVersion, CityModelArrowParts,
    ProjectedFieldSpec, ProjectedStructSpec, ProjectedValueSpec, ProjectionLayout,
    canonical_schema_set,
};
use crate::transport::{
    CanonicalTable, CanonicalTableSink, canonical_table_order, canonical_table_position,
    collect_tables, schema_for_table, validate_schema,
};
use ::arrow::array::{
    Array, ArrayRef, BooleanArray, FixedSizeListArray, Float32Array, Float64Array, Int64Array,
    LargeStringArray, ListArray, NullArray, RecordBatch, StringArray, StructArray, UInt32Array,
    UInt64Array,
};
use ::arrow::datatypes::{DataType, FieldRef};
use arrow_buffer::{MutableBuffer, NullBuffer, OffsetBuffer, ScalarBuffer};
use cityjson_types::CityModelType;
use cityjson_types::relational::ModelRelationalView;
use cityjson_types::v2_0::geometry::{MaterialThemesView, TextureThemesView};
use cityjson_types::v2_0::{
    AttributeValue, BBox, Boundary, CRS, CityModelCapacities, CityModelIdentifier, CityObject,
    CityObjectIdentifier, CityObjectType, Contact, ContactRole, ContactType, Extension, Geometry,
    GeometryType, ImageType, LoD, MaterialMap, Metadata, OwnedAttributeValue, OwnedCityModel,
    OwnedMaterial, OwnedSemantic, OwnedTexture, RGB, RGBA, SemanticMap, SemanticType,
    StoredGeometryInstance, StoredGeometryParts, TextureMap, TextureType, ThemeName, UVCoordinate,
    WrapMode,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::Range;
use std::sync::Arc;

mod arrow;
mod export;
mod geometry;
mod import;
mod projection;

use self::arrow::*;
use self::export::{raw_id_from_handle, raw_index_from_handle, usize_to_i32, usize_to_u32};
use self::geometry::*;
use self::import::{decode_payload_f32, grouped_row_range};
use self::projection::*;

pub(crate) use self::export::{
    build_parts_from_tables, emit_part_tables, emit_tables, encode_parts,
};
pub(crate) use self::import::decode_parts;

const DEFAULT_CITYMODEL_ID: &str = "citymodel";
const FIELD_MATERIAL_NAME: &str = "payload.name";
const FIELD_MATERIAL_AMBIENT_INTENSITY: &str = "payload.ambient_intensity";
const FIELD_MATERIAL_DIFFUSE_COLOR: &str = "payload.diffuse_color";
const FIELD_MATERIAL_EMISSIVE_COLOR: &str = "payload.emissive_color";
const FIELD_MATERIAL_SPECULAR_COLOR: &str = "payload.specular_color";
const FIELD_MATERIAL_SHININESS: &str = "payload.shininess";
const FIELD_MATERIAL_TRANSPARENCY: &str = "payload.transparency";
const FIELD_MATERIAL_IS_SMOOTH: &str = "payload.is_smooth";
const FIELD_TEXTURE_IMAGE_TYPE: &str = "payload.image_type";
const FIELD_TEXTURE_WRAP_MODE: &str = "payload.wrap_mode";
const FIELD_TEXTURE_TEXTURE_TYPE: &str = "payload.texture_type";
const FIELD_TEXTURE_BORDER_COLOR: &str = "payload.border_color";
const PRIMITIVE_TYPE_POINT: &str = "point";
const PRIMITIVE_TYPE_LINESTRING: &str = "linestring";
const PRIMITIVE_TYPE_SURFACE: &str = "surface";

#[derive(Debug, Clone)]
struct MetadataContactRow {
    contact_name: String,
    email_address: String,
    role: Option<String>,
    website: Option<String>,
    contact_type: Option<String>,
    phone: Option<String>,
    organization: Option<String>,
    address: Option<cityjson_types::v2_0::OwnedAttributes>,
}

#[derive(Debug, Clone)]
struct MetadataRow {
    citymodel_id: String,
    cityjson_version: String,
    citymodel_kind: String,
    feature_root_id: Option<String>,
    identifier: Option<String>,
    title: Option<String>,
    reference_system: Option<String>,
    geographical_extent: Option<[f64; 6]>,
    reference_date: Option<String>,
    default_material_theme: Option<String>,
    default_texture_theme: Option<String>,
    point_of_contact: Option<MetadataContactRow>,
    root_extra: Option<cityjson_types::v2_0::OwnedAttributes>,
    metadata_extra: Option<cityjson_types::v2_0::OwnedAttributes>,
}

struct U32ListBatchBuffer {
    offsets: Vec<i32>,
    values: Vec<u32>,
    validity: Vec<bool>,
}

impl Default for U32ListBatchBuffer {
    fn default() -> Self {
        Self {
            offsets: vec![0],
            values: Vec::new(),
            validity: Vec::new(),
        }
    }
}

impl U32ListBatchBuffer {
    fn push_required(&mut self, values: &[u32]) -> Result<()> {
        self.values.extend_from_slice(values);
        self.offsets
            .push(usize_to_i32(self.values.len(), "list offset")?);
        self.validity.push(true);
        Ok(())
    }

    fn push_optional(&mut self, values: Option<&[u32]>) -> Result<()> {
        if let Some(values) = values {
            self.values.extend_from_slice(values);
            self.validity.push(true);
        } else {
            self.validity.push(false);
        }
        self.offsets
            .push(usize_to_i32(self.values.len(), "list offset")?);
        Ok(())
    }

    fn into_array(self, field: &FieldRef) -> Result<ListArray> {
        let nulls = if self.validity.iter().all(|item| *item) {
            None
        } else {
            Some(NullBuffer::from(self.validity))
        };
        ListArray::try_new(
            list_child_field(field)?,
            OffsetBuffer::new(ScalarBuffer::from(self.offsets)),
            Arc::new(UInt32Array::from(self.values)),
            nulls,
        )
        .map_err(Error::from)
    }
}

struct U64ListBatchBuffer {
    offsets: Vec<i32>,
    values: Vec<u64>,
    validity: Vec<bool>,
}

impl Default for U64ListBatchBuffer {
    fn default() -> Self {
        Self {
            offsets: vec![0],
            values: Vec::new(),
            validity: Vec::new(),
        }
    }
}

impl U64ListBatchBuffer {
    fn push_required(&mut self, values: &[u64]) -> Result<()> {
        self.values.extend_from_slice(values);
        self.offsets
            .push(usize_to_i32(self.values.len(), "list offset")?);
        self.validity.push(true);
        Ok(())
    }

    fn into_array(self, field: &FieldRef) -> Result<ListArray> {
        let nulls = if self.validity.iter().all(|item| *item) {
            None
        } else {
            Some(NullBuffer::from(self.validity))
        };
        ListArray::try_new(
            list_child_field(field)?,
            OffsetBuffer::new(ScalarBuffer::from(self.offsets)),
            Arc::new(UInt64Array::from(self.values)),
            nulls,
        )
        .map_err(Error::from)
    }
}

#[derive(Default)]
struct GeometryTableBuffer {
    geometry_id: Vec<u64>,
    cityobject_ix: Vec<u64>,
    geometry_ordinal: Vec<u32>,
    geometry_type: Vec<String>,
    lod: Vec<Option<String>>,
}

impl GeometryTableBuffer {
    fn push(
        &mut self,
        geometry_id: u64,
        cityobject_ix: u64,
        geometry_ordinal: u32,
        geometry_type: &str,
        lod: Option<String>,
    ) {
        self.geometry_id.push(geometry_id);
        self.cityobject_ix.push(cityobject_ix);
        self.geometry_ordinal.push(geometry_ordinal);
        self.geometry_type.push(geometry_type.to_string());
        self.lod.push(lod);
    }
}

#[derive(Default)]
struct GeometryBoundaryTableBuffer {
    geometry_id: Vec<u64>,
    vertex_indices: U32ListBatchBuffer,
    line_offsets: U32ListBatchBuffer,
    ring_offsets: U32ListBatchBuffer,
    surface_offsets: U32ListBatchBuffer,
    shell_offsets: U32ListBatchBuffer,
    solid_offsets: U32ListBatchBuffer,
}

impl GeometryBoundaryTableBuffer {
    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        geometry_id: u64,
        vertex_indices: &[u32],
        line_offsets: Option<&[u32]>,
        ring_offsets: Option<&[u32]>,
        surface_offsets: Option<&[u32]>,
        shell_offsets: Option<&[u32]>,
        solid_offsets: Option<&[u32]>,
    ) -> Result<()> {
        self.geometry_id.push(geometry_id);
        self.vertex_indices.push_required(vertex_indices)?;
        self.line_offsets.push_optional(line_offsets)?;
        self.ring_offsets.push_optional(ring_offsets)?;
        self.surface_offsets.push_optional(surface_offsets)?;
        self.shell_offsets.push_optional(shell_offsets)?;
        self.solid_offsets.push_optional(solid_offsets)?;
        Ok(())
    }
}

#[derive(Default)]
struct GeometryInstanceTableBuffer {
    geometry_id: Vec<u64>,
    cityobject_ix: Vec<u64>,
    geometry_ordinal: Vec<u32>,
    lod: Vec<Option<String>>,
    template_geometry_id: Vec<u64>,
    reference_point_vertex_id: Vec<u64>,
    transform_matrix: Vec<Option<[f64; 16]>>,
}

impl GeometryInstanceTableBuffer {
    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        geometry_id: u64,
        cityobject_ix: u64,
        geometry_ordinal: u32,
        lod: Option<String>,
        template_geometry_id: u64,
        reference_point_vertex_id: u64,
        transform_matrix: Option<[f64; 16]>,
    ) {
        self.geometry_id.push(geometry_id);
        self.cityobject_ix.push(cityobject_ix);
        self.geometry_ordinal.push(geometry_ordinal);
        self.lod.push(lod);
        self.template_geometry_id.push(template_geometry_id);
        self.reference_point_vertex_id
            .push(reference_point_vertex_id);
        self.transform_matrix.push(transform_matrix);
    }

    fn is_empty(&self) -> bool {
        self.geometry_id.is_empty()
    }
}

#[derive(Default)]
struct GeometrySurfaceSemanticTableBuffer {
    geometry_id: Vec<u64>,
    surface_ordinal: Vec<u32>,
    semantic_id: Vec<Option<u64>>,
}

impl GeometrySurfaceSemanticTableBuffer {
    fn push(&mut self, geometry_id: u64, surface_ordinal: u32, semantic_id: Option<u64>) {
        self.geometry_id.push(geometry_id);
        self.surface_ordinal.push(surface_ordinal);
        self.semantic_id.push(semantic_id);
    }

    fn is_empty(&self) -> bool {
        self.geometry_id.is_empty()
    }
}

#[derive(Default)]
struct GeometryPointSemanticTableBuffer {
    geometry_id: Vec<u64>,
    point_ordinal: Vec<u32>,
    semantic_id: Vec<Option<u64>>,
}

impl GeometryPointSemanticTableBuffer {
    fn push(&mut self, geometry_id: u64, point_ordinal: u32, semantic_id: Option<u64>) {
        self.geometry_id.push(geometry_id);
        self.point_ordinal.push(point_ordinal);
        self.semantic_id.push(semantic_id);
    }

    fn is_empty(&self) -> bool {
        self.geometry_id.is_empty()
    }
}

#[derive(Default)]
struct GeometryLinestringSemanticTableBuffer {
    geometry_id: Vec<u64>,
    linestring_ordinal: Vec<u32>,
    semantic_id: Vec<Option<u64>>,
}

impl GeometryLinestringSemanticTableBuffer {
    fn push(&mut self, geometry_id: u64, linestring_ordinal: u32, semantic_id: Option<u64>) {
        self.geometry_id.push(geometry_id);
        self.linestring_ordinal.push(linestring_ordinal);
        self.semantic_id.push(semantic_id);
    }

    fn is_empty(&self) -> bool {
        self.geometry_id.is_empty()
    }
}

#[derive(Default)]
struct GeometrySurfaceMaterialTableBuffer {
    geometry_id: Vec<u64>,
    surface_ordinal: Vec<u32>,
    theme: Vec<String>,
    material_id: Vec<u64>,
}

impl GeometrySurfaceMaterialTableBuffer {
    fn push(&mut self, geometry_id: u64, surface_ordinal: u32, theme: &str, material_id: u64) {
        self.geometry_id.push(geometry_id);
        self.surface_ordinal.push(surface_ordinal);
        self.theme.push(theme.to_string());
        self.material_id.push(material_id);
    }

    fn is_empty(&self) -> bool {
        self.geometry_id.is_empty()
    }
}

#[derive(Default)]
struct GeometryRingTextureTableBuffer {
    geometry_id: Vec<u64>,
    surface_ordinal: Vec<u32>,
    ring_ordinal: Vec<u32>,
    theme: Vec<String>,
    texture_id: Vec<u64>,
    uv_indices: U64ListBatchBuffer,
}

impl GeometryRingTextureTableBuffer {
    fn push(
        &mut self,
        geometry_id: u64,
        surface_ordinal: u32,
        ring_ordinal: u32,
        theme: &str,
        texture_id: u64,
        uv_indices: &[u64],
    ) -> Result<()> {
        self.geometry_id.push(geometry_id);
        self.surface_ordinal.push(surface_ordinal);
        self.ring_ordinal.push(ring_ordinal);
        self.theme.push(theme.to_string());
        self.texture_id.push(texture_id);
        self.uv_indices.push_required(uv_indices)?;
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.geometry_id.is_empty()
    }
}

#[derive(Default)]
struct TemplateGeometryTableBuffer {
    template_geometry_id: Vec<u64>,
    geometry_type: Vec<String>,
    lod: Vec<Option<String>>,
}

impl TemplateGeometryTableBuffer {
    fn push(&mut self, template_geometry_id: u64, geometry_type: &str, lod: Option<String>) {
        self.template_geometry_id.push(template_geometry_id);
        self.geometry_type.push(geometry_type.to_string());
        self.lod.push(lod);
    }

    fn is_empty(&self) -> bool {
        self.template_geometry_id.is_empty()
    }
}

#[derive(Default)]
struct TemplateGeometryBoundaryTableBuffer {
    template_geometry_id: Vec<u64>,
    vertex_indices: U32ListBatchBuffer,
    line_offsets: U32ListBatchBuffer,
    ring_offsets: U32ListBatchBuffer,
    surface_offsets: U32ListBatchBuffer,
    shell_offsets: U32ListBatchBuffer,
    solid_offsets: U32ListBatchBuffer,
}

impl TemplateGeometryBoundaryTableBuffer {
    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        template_geometry_id: u64,
        vertex_indices: &[u32],
        line_offsets: Option<&[u32]>,
        ring_offsets: Option<&[u32]>,
        surface_offsets: Option<&[u32]>,
        shell_offsets: Option<&[u32]>,
        solid_offsets: Option<&[u32]>,
    ) -> Result<()> {
        self.template_geometry_id.push(template_geometry_id);
        self.vertex_indices.push_required(vertex_indices)?;
        self.line_offsets.push_optional(line_offsets)?;
        self.ring_offsets.push_optional(ring_offsets)?;
        self.surface_offsets.push_optional(surface_offsets)?;
        self.shell_offsets.push_optional(shell_offsets)?;
        self.solid_offsets.push_optional(solid_offsets)?;
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.template_geometry_id.is_empty()
    }
}

#[derive(Default)]
struct TemplateGeometrySemanticTableBuffer {
    template_geometry_id: Vec<u64>,
    primitive_type: Vec<String>,
    primitive_ordinal: Vec<u32>,
    semantic_id: Vec<Option<u64>>,
}

impl TemplateGeometrySemanticTableBuffer {
    fn push(
        &mut self,
        template_geometry_id: u64,
        primitive_type: &str,
        primitive_ordinal: u32,
        semantic_id: Option<u64>,
    ) {
        self.template_geometry_id.push(template_geometry_id);
        self.primitive_type.push(primitive_type.to_string());
        self.primitive_ordinal.push(primitive_ordinal);
        self.semantic_id.push(semantic_id);
    }

    fn is_empty(&self) -> bool {
        self.template_geometry_id.is_empty()
    }
}

#[derive(Default)]
struct TemplateGeometryMaterialTableBuffer {
    template_geometry_id: Vec<u64>,
    primitive_type: Vec<String>,
    primitive_ordinal: Vec<u32>,
    theme: Vec<String>,
    material_id: Vec<u64>,
}

impl TemplateGeometryMaterialTableBuffer {
    fn push(
        &mut self,
        template_geometry_id: u64,
        primitive_type: &str,
        primitive_ordinal: u32,
        theme: &str,
        material_id: u64,
    ) {
        self.template_geometry_id.push(template_geometry_id);
        self.primitive_type.push(primitive_type.to_string());
        self.primitive_ordinal.push(primitive_ordinal);
        self.theme.push(theme.to_string());
        self.material_id.push(material_id);
    }

    fn is_empty(&self) -> bool {
        self.template_geometry_id.is_empty()
    }
}

#[derive(Default)]
struct TemplateGeometryRingTextureTableBuffer {
    template_geometry_id: Vec<u64>,
    surface_ordinal: Vec<u32>,
    ring_ordinal: Vec<u32>,
    theme: Vec<String>,
    texture_id: Vec<u64>,
    uv_indices: U64ListBatchBuffer,
}

impl TemplateGeometryRingTextureTableBuffer {
    fn push(
        &mut self,
        template_geometry_id: u64,
        surface_ordinal: u32,
        ring_ordinal: u32,
        theme: &str,
        texture_id: u64,
        uv_indices: &[u64],
    ) -> Result<()> {
        self.template_geometry_id.push(template_geometry_id);
        self.surface_ordinal.push(surface_ordinal);
        self.ring_ordinal.push(ring_ordinal);
        self.theme.push(theme.to_string());
        self.texture_id.push(texture_id);
        self.uv_indices.push_required(uv_indices)?;
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.template_geometry_id.is_empty()
    }
}

#[derive(Default)]
struct ExportedGeometryTables {
    geometries: GeometryTableBuffer,
    boundaries: GeometryBoundaryTableBuffer,
    instances: GeometryInstanceTableBuffer,
    surface_semantics: GeometrySurfaceSemanticTableBuffer,
    point_semantics: GeometryPointSemanticTableBuffer,
    linestring_semantics: GeometryLinestringSemanticTableBuffer,
    surface_materials: GeometrySurfaceMaterialTableBuffer,
    ring_textures: GeometryRingTextureTableBuffer,
}

#[derive(Default)]
struct ExportedTemplateGeometryTables {
    geometries: TemplateGeometryTableBuffer,
    boundaries: TemplateGeometryBoundaryTableBuffer,
    semantics: TemplateGeometrySemanticTableBuffer,
    materials: TemplateGeometryMaterialTableBuffer,
    ring_textures: TemplateGeometryRingTextureTableBuffer,
}

type MaterialThemeMaps = Vec<(
    ThemeName<cityjson_types::prelude::OwnedStringStorage>,
    MaterialMap<u32>,
)>;
type TextureThemeMaps = Vec<(
    ThemeName<cityjson_types::prelude::OwnedStringStorage>,
    TextureMap<u32>,
)>;

struct ExportContext<'a> {
    relational: &'a ModelRelationalView<'a>,
    header: CityArrowHeader,
    projection: ProjectionLayout,
    schemas: CanonicalSchemaSet,
}

struct ExportCoreBatches {
    metadata: RecordBatch,
    extensions: Option<RecordBatch>,
    vertices: RecordBatch,
    cityobjects: RecordBatch,
    cityobject_children: Option<RecordBatch>,
}

struct ExportGeometryBatches {
    geometries: RecordBatch,
    geometry_boundaries: RecordBatch,
    geometry_instances: Option<RecordBatch>,
    template_vertices: Option<RecordBatch>,
    template_geometries: Option<RecordBatch>,
    template_geometry_boundaries: Option<RecordBatch>,
}

struct ExportSemanticBatches {
    semantics: Option<RecordBatch>,
    semantic_children: Option<RecordBatch>,
    geometry_surface_semantics: Option<RecordBatch>,
    geometry_point_semantics: Option<RecordBatch>,
    geometry_linestring_semantics: Option<RecordBatch>,
    template_geometry_semantics: Option<RecordBatch>,
}

struct ExportAppearanceBatches {
    materials: Option<RecordBatch>,
    geometry_surface_materials: Option<RecordBatch>,
    template_geometry_materials: Option<RecordBatch>,
    textures: Option<RecordBatch>,
    texture_vertices: Option<RecordBatch>,
    geometry_ring_textures: Option<RecordBatch>,
    template_geometry_ring_textures: Option<RecordBatch>,
}

#[derive(Default)]
struct PartsSink {
    header: Option<CityArrowHeader>,
    projection: Option<ProjectionLayout>,
    metadata: Option<RecordBatch>,
    extensions: Option<RecordBatch>,
    vertices: Option<RecordBatch>,
    template_vertices: Option<RecordBatch>,
    texture_vertices: Option<RecordBatch>,
    semantics: Option<RecordBatch>,
    semantic_children: Option<RecordBatch>,
    materials: Option<RecordBatch>,
    textures: Option<RecordBatch>,
    template_geometry_boundaries: Option<RecordBatch>,
    template_geometry_semantics: Option<RecordBatch>,
    template_geometry_materials: Option<RecordBatch>,
    template_geometry_ring_textures: Option<RecordBatch>,
    template_geometries: Option<RecordBatch>,
    geometry_boundaries: Option<RecordBatch>,
    geometry_surface_semantics: Option<RecordBatch>,
    geometry_point_semantics: Option<RecordBatch>,
    geometry_linestring_semantics: Option<RecordBatch>,
    geometry_surface_materials: Option<RecordBatch>,
    geometry_ring_textures: Option<RecordBatch>,
    geometry_instances: Option<RecordBatch>,
    geometries: Option<RecordBatch>,
    cityobjects: Option<RecordBatch>,
    cityobject_children: Option<RecordBatch>,
}

struct GeometryExportContext<'a> {
    relational: &'a ModelRelationalView<'a>,
}

struct UniqueBatchView<V> {
    view: V,
    row_by_id: HashMap<u64, usize>,
}

struct GroupedBatchView<V> {
    view: V,
    rows_by_id: HashMap<u64, Range<usize>>,
}

struct ImportState {
    model: OwnedCityModel,
    pending_feature_root_id: Option<String>,
    semantic_handle_by_id: HashMap<u64, cityjson_types::prelude::SemanticHandle>,
    material_handle_by_id: HashMap<u64, cityjson_types::prelude::MaterialHandle>,
    texture_handle_by_id: HashMap<u64, cityjson_types::prelude::TextureHandle>,
    template_handle_by_id: HashMap<u64, cityjson_types::prelude::GeometryTemplateHandle>,
    geometry_handle_by_id: HashMap<u64, cityjson_types::prelude::GeometryHandle>,
    cityobject_handle_by_ix: Vec<Option<cityjson_types::prelude::CityObjectHandle>>,
    pending_geometry_attachments: Vec<Vec<(u32, u64)>>,
    fully_reserved: bool,
}

#[derive(Default)]
struct PartBatchViews {
    boundaries: Option<UniqueBatchView<BoundaryBatchView>>,
    template_boundaries: Option<UniqueBatchView<BoundaryBatchView>>,
    surface_semantics: Option<GroupedBatchView<IndexedSemanticBatchView>>,
    point_semantics: Option<GroupedBatchView<IndexedSemanticBatchView>>,
    linestring_semantics: Option<GroupedBatchView<IndexedSemanticBatchView>>,
    template_semantics: Option<GroupedBatchView<TemplateSemanticBatchView>>,
    surface_materials: Option<GroupedBatchView<GeometrySurfaceMaterialBatchView>>,
    template_materials: Option<GroupedBatchView<TemplateMaterialBatchView>>,
    ring_textures: Option<GroupedBatchView<RingTextureBatchView>>,
    template_ring_textures: Option<GroupedBatchView<RingTextureBatchView>>,
}

pub(crate) struct IncrementalDecoder {
    header: CityArrowHeader,
    projection: ProjectionLayout,
    schemas: CanonicalSchemaSet,
    state: Option<ImportState>,
    grouped_rows: PartBatchViews,
    last_table_position: Option<usize>,
    seen_tables: BTreeSet<CanonicalTable>,
}

struct VertexColumns<'a> {
    vertex_id: &'a UInt64Array,
    x: &'a Float64Array,
    y: &'a Float64Array,
    z: &'a Float64Array,
}

struct UvColumns<'a> {
    uv_id: &'a UInt64Array,
    u: &'a Float32Array,
    v: &'a Float32Array,
}

struct GeometryColumns<'a> {
    geometry_id: &'a UInt64Array,
    cityobject_ix: &'a UInt64Array,
    geometry_ordinal: &'a UInt32Array,
    geometry_type: &'a StringArray,
    lod: &'a StringArray,
}

struct TemplateGeometryColumns<'a> {
    template_geometry_id: &'a UInt64Array,
    geometry_type: &'a StringArray,
    lod: &'a StringArray,
}

struct GeometryInstanceColumns<'a> {
    geometry_id: &'a UInt64Array,
    cityobject_ix: &'a UInt64Array,
    geometry_ordinal: &'a UInt32Array,
    lod: &'a StringArray,
    template_geometry_id: &'a UInt64Array,
    reference_point_vertex_id: &'a UInt64Array,
    transform_matrix: &'a FixedSizeListArray,
}

struct CityObjectColumns<'a> {
    cityobject_id: &'a LargeStringArray,
    cityobject_ix: &'a UInt64Array,
    object_type: &'a StringArray,
    geographical_extent: &'a FixedSizeListArray,
    attributes: Option<&'a StructArray>,
    extra: Option<&'a StructArray>,
}

struct SemanticColumns<'a> {
    semantic_id: &'a UInt64Array,
    semantic_type: &'a StringArray,
    parent_semantic_id: &'a UInt64Array,
    attributes: Option<&'a StructArray>,
}

struct MaterialColumns<'a> {
    material_id: &'a UInt64Array,
    name: &'a LargeStringArray,
    ambient_intensity: Option<&'a Float64Array>,
    diffuse_color: Option<&'a ListArray>,
    emissive_color: Option<&'a ListArray>,
    specular_color: Option<&'a ListArray>,
    shininess: Option<&'a Float64Array>,
    transparency: Option<&'a Float64Array>,
    is_smooth: Option<&'a ::arrow::array::BooleanArray>,
}

struct TextureColumns<'a> {
    texture_id: &'a UInt64Array,
    image_uri: &'a LargeStringArray,
    image_type: &'a LargeStringArray,
    wrap_mode: Option<&'a LargeStringArray>,
    texture_type: Option<&'a LargeStringArray>,
    border_color: Option<&'a ListArray>,
}

#[derive(Clone)]
struct U32ListColumnView {
    list: ListArray,
    values: UInt32Array,
}

#[derive(Clone)]
struct U64ListColumnView {
    list: ListArray,
    values: UInt64Array,
}

#[derive(Clone)]
struct BoundaryBatchView {
    id: UInt64Array,
    vertex_indices: U32ListColumnView,
    line_offsets: U32ListColumnView,
    ring_offsets: U32ListColumnView,
    surface_offsets: U32ListColumnView,
    shell_offsets: U32ListColumnView,
    solid_offsets: U32ListColumnView,
}

#[derive(Clone)]
struct IndexedSemanticBatchView {
    semantic_id: UInt64Array,
    ordinal: UInt32Array,
}

#[derive(Clone)]
struct TemplateSemanticBatchView {
    primitive_type: StringArray,
    primitive_ordinal: UInt32Array,
    semantic_id: UInt64Array,
}

#[derive(Clone)]
struct GeometrySurfaceMaterialBatchView {
    theme: StringArray,
    surface_ordinal: UInt32Array,
    material_id: UInt64Array,
}

#[derive(Clone)]
struct TemplateMaterialBatchView {
    primitive_type: StringArray,
    primitive_ordinal: UInt32Array,
    theme: StringArray,
    material_id: UInt64Array,
}

#[derive(Clone)]
struct RingTextureBatchView {
    surface_ordinal: UInt32Array,
    ring_ordinal: UInt32Array,
    theme: StringArray,
    texture_id: UInt64Array,
    uv_indices: U64ListColumnView,
}

struct BorrowedBoundary<'a> {
    vertex_indices: &'a [u32],
    line_offsets: Option<&'a [u32]>,
    ring_offsets: Option<&'a [u32]>,
    surface_offsets: Option<&'a [u32]>,
    shell_offsets: Option<&'a [u32]>,
    solid_offsets: Option<&'a [u32]>,
}

impl U32ListColumnView {
    fn value(&self, row: usize) -> Result<&[u32]> {
        let offsets = self.list.value_offsets();
        let start = usize::try_from(offsets[row]).expect("offset fits into usize");
        let end = usize::try_from(offsets[row + 1]).expect("offset fits into usize");
        let values = self.values.values();
        values.get(start..end).ok_or_else(|| {
            Error::Conversion("u32 list offsets are outside the child buffer".to_string())
        })
    }

    fn optional_value(&self, row: usize) -> Result<Option<&[u32]>> {
        if self.list.is_null(row) {
            return Ok(None);
        }
        self.value(row).map(Some)
    }
}

impl U64ListColumnView {
    fn value(&self, row: usize) -> Result<&[u64]> {
        let offsets = self.list.value_offsets();
        let start = usize::try_from(offsets[row]).expect("offset fits into usize");
        let end = usize::try_from(offsets[row + 1]).expect("offset fits into usize");
        let values = self.values.values();
        values.get(start..end).ok_or_else(|| {
            Error::Conversion("u64 list offsets are outside the child buffer".to_string())
        })
    }
}

impl BoundaryBatchView {
    fn payload(&self, row: usize) -> Result<BorrowedBoundary<'_>> {
        Ok(BorrowedBoundary {
            vertex_indices: self.vertex_indices.value(row)?,
            line_offsets: self.line_offsets.optional_value(row)?,
            ring_offsets: self.ring_offsets.optional_value(row)?,
            surface_offsets: self.surface_offsets.optional_value(row)?,
            shell_offsets: self.shell_offsets.optional_value(row)?,
            solid_offsets: self.solid_offsets.optional_value(row)?,
        })
    }
}

fn empty_texture_map(ring_layouts: &[RingLayout]) -> Result<TextureMap<u32>> {
    let mut map = TextureMap::new();
    for layout in ring_layouts {
        map.add_ring(cityjson_types::v2_0::VertexIndex::new(usize_to_u32(
            layout.start,
            "ring vertex start",
        )?));
        map.add_ring_texture(None);
        for _ in 0..layout.len {
            map.add_vertex(None);
        }
    }
    Ok(map)
}
