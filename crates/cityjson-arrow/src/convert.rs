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
use cityjson::CityModelType;
use cityjson::v2_0::geometry::{MaterialThemesView, TextureThemesView};
use cityjson::v2_0::{
    AttributeValue, BBox, Boundary, CRS, CityModelCapacities, CityModelIdentifier, CityObject,
    CityObjectIdentifier, CityObjectType, Contact, ContactRole, ContactType, Extension, Geometry,
    GeometryType, ImageType, LoD, MaterialMap, Metadata, OwnedAttributeValue, OwnedCityModel,
    OwnedMaterial, OwnedSemantic, OwnedTexture, RGB, RGBA, SemanticMap, SemanticType,
    StoredGeometryInstance, StoredGeometryParts, TextureMap, TextureType, ThemeName, UVCoordinate,
    WrapMode,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{Read, Write as IoWrite};
use std::ops::Range;
use std::sync::Arc;

mod arrow;
mod geometry;
mod projection;

use self::{arrow::*, geometry::*, projection::*};

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

#[derive(Debug, Default, Clone, Copy)]
pub struct ModelEncoder;

impl ModelEncoder {
    /// Encodes an in-memory model as a live Arrow IPC stream.
    ///
    /// # Errors
    ///
    /// Returns an error when model conversion or stream serialization fails.
    pub fn encode<W: IoWrite>(&self, model: &OwnedCityModel, writer: W) -> Result<()> {
        crate::stream::write_model_stream(model, writer)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ModelDecoder;

impl ModelDecoder {
    /// Decodes an in-memory model from a live Arrow IPC stream.
    ///
    /// # Errors
    ///
    /// Returns an error when stream decoding or model reconstruction fails.
    pub fn decode<R: Read>(&self, reader: R) -> Result<OwnedCityModel> {
        crate::stream::read_model_stream(reader)
    }
}

#[derive(Debug, Clone)]
struct MetadataContactRow {
    contact_name: String,
    email_address: String,
    role: Option<String>,
    website: Option<String>,
    contact_type: Option<String>,
    phone: Option<String>,
    organization: Option<String>,
    address: Option<cityjson::v2_0::OwnedAttributes>,
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
    root_extra: Option<cityjson::v2_0::OwnedAttributes>,
    metadata_extra: Option<cityjson::v2_0::OwnedAttributes>,
}

#[derive(Debug, Clone, Copy)]
struct TransformRow {
    scale: [f64; 3],
    translate: [f64; 3],
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
    ThemeName<cityjson::prelude::OwnedStringStorage>,
    MaterialMap<u32>,
)>;
type TextureThemeMaps = Vec<(
    ThemeName<cityjson::prelude::OwnedStringStorage>,
    TextureMap<u32>,
)>;

struct ExportContext<'a> {
    model: &'a OwnedCityModel,
    header: CityArrowHeader,
    projection: ProjectionLayout,
    schemas: CanonicalSchemaSet,
}

struct ExportCoreBatches {
    metadata: RecordBatch,
    transform: Option<RecordBatch>,
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
    transform: Option<RecordBatch>,
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
    model: &'a OwnedCityModel,
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
    semantic_handle_by_id: HashMap<u64, cityjson::prelude::SemanticHandle>,
    material_handle_by_id: HashMap<u64, cityjson::prelude::MaterialHandle>,
    texture_handle_by_id: HashMap<u64, cityjson::prelude::TextureHandle>,
    template_handle_by_id: HashMap<u64, cityjson::prelude::GeometryTemplateHandle>,
    geometry_handle_by_id: HashMap<u64, cityjson::prelude::GeometryHandle>,
    cityobject_handle_by_ix: Vec<Option<cityjson::prelude::CityObjectHandle>>,
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
        map.add_ring(cityjson::v2_0::VertexIndex::new(usize_to_u32(
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

/// Converts an in-memory `CityJSON` model into the canonical Arrow table set.
///
/// # Errors
///
/// Returns an error when the model contains unsupported `CityJSON` features or when
/// Arrow-compatible table rows cannot be derived from the model data.
pub(crate) fn encode_parts(model: &OwnedCityModel) -> Result<CityModelArrowParts> {
    let mut sink = PartsSink::default();
    emit_tables(model, &mut sink)?;
    sink.finish()
}

#[allow(clippy::too_many_lines)]
pub(crate) fn emit_tables<S: CanonicalTableSink>(
    model: &OwnedCityModel,
    sink: &mut S,
) -> Result<()> {
    reject_unsupported_modules(model)?;
    let context = build_export_context(model)?;
    sink.start(&context.header, &context.projection)?;

    let core = export_core_batches(&context)?;
    sink.push_batch(CanonicalTable::Metadata, core.metadata)?;
    push_optional_batch(sink, CanonicalTable::Transform, core.transform)?;
    push_optional_batch(sink, CanonicalTable::Extensions, core.extensions)?;
    sink.push_batch(CanonicalTable::Vertices, core.vertices)?;

    let ExportedGeometryTables {
        geometries,
        boundaries,
        instances,
        surface_semantics,
        point_semantics,
        linestring_semantics,
        surface_materials,
        ring_textures,
    } = geometry_tables(context.model)?;
    let ExportedTemplateGeometryTables {
        geometries: template_geometries,
        boundaries: template_geometry_boundaries,
        semantics: template_geometry_semantics,
        materials: template_geometry_materials,
        ring_textures: template_geometry_ring_textures,
    } = template_geometry_tables(context.model)?;
    let geometry = export_geometry_batches(
        &context,
        geometries,
        boundaries,
        instances,
        template_geometries,
        template_geometry_boundaries,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::TemplateVertices,
        geometry.template_vertices,
    )?;

    let semantics = export_semantic_batches(
        &context,
        surface_semantics,
        point_semantics,
        linestring_semantics,
        template_geometry_semantics,
    )?;
    let appearance = export_appearance_batches(
        &context,
        surface_materials,
        ring_textures,
        template_geometry_materials,
        template_geometry_ring_textures,
    )?;

    push_optional_batch(
        sink,
        CanonicalTable::TextureVertices,
        appearance.texture_vertices,
    )?;
    push_optional_batch(sink, CanonicalTable::Semantics, semantics.semantics)?;
    push_optional_batch(
        sink,
        CanonicalTable::SemanticChildren,
        semantics.semantic_children,
    )?;
    push_optional_batch(sink, CanonicalTable::Materials, appearance.materials)?;
    push_optional_batch(sink, CanonicalTable::Textures, appearance.textures)?;
    push_optional_batch(
        sink,
        CanonicalTable::TemplateGeometryBoundaries,
        geometry.template_geometry_boundaries,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::TemplateGeometrySemantics,
        semantics.template_geometry_semantics,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::TemplateGeometryMaterials,
        appearance.template_geometry_materials,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::TemplateGeometryRingTextures,
        appearance.template_geometry_ring_textures,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::TemplateGeometries,
        geometry.template_geometries,
    )?;
    sink.push_batch(
        CanonicalTable::GeometryBoundaries,
        geometry.geometry_boundaries,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::GeometrySurfaceSemantics,
        semantics.geometry_surface_semantics,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::GeometryPointSemantics,
        semantics.geometry_point_semantics,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::GeometryLinestringSemantics,
        semantics.geometry_linestring_semantics,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::GeometrySurfaceMaterials,
        appearance.geometry_surface_materials,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::GeometryRingTextures,
        appearance.geometry_ring_textures,
    )?;
    push_optional_batch(
        sink,
        CanonicalTable::GeometryInstances,
        geometry.geometry_instances,
    )?;
    sink.push_batch(CanonicalTable::Geometries, geometry.geometries)?;
    sink.push_batch(CanonicalTable::CityObjects, core.cityobjects)?;
    push_optional_batch(
        sink,
        CanonicalTable::CityObjectChildren,
        core.cityobject_children,
    )?;

    Ok(())
}

pub(crate) fn emit_part_tables<S: CanonicalTableSink>(
    parts: &CityModelArrowParts,
    sink: &mut S,
) -> Result<()> {
    sink.start(&parts.header, &parts.projection)?;
    for (table, batch) in collect_tables(parts) {
        sink.push_batch(table, batch)?;
    }
    Ok(())
}

pub(crate) fn build_parts_from_tables(
    header: &CityArrowHeader,
    projection: &ProjectionLayout,
    tables: Vec<(CanonicalTable, RecordBatch)>,
) -> Result<CityModelArrowParts> {
    let mut sink = PartsSink::default();
    sink.start(header, projection)?;
    for (table, batch) in tables {
        sink.push_batch(table, batch)?;
    }
    sink.finish()
}

fn build_export_context(model: &OwnedCityModel) -> Result<ExportContext<'_>> {
    let citymodel_id = infer_citymodel_id(model);
    let projection = discover_projection_layout(model)?;
    Ok(ExportContext {
        model,
        header: CityArrowHeader::new(
            CityArrowPackageVersion::V3Alpha2,
            citymodel_id,
            model
                .version()
                .unwrap_or(cityjson::CityJSONVersion::V2_0)
                .to_string(),
        ),
        projection: projection.clone(),
        schemas: canonical_schema_set(&projection),
    })
}

fn push_optional_batch<S: CanonicalTableSink>(
    sink: &mut S,
    table: CanonicalTable,
    batch: Option<RecordBatch>,
) -> Result<()> {
    if let Some(batch) = batch {
        sink.push_batch(table, batch)?;
    }
    Ok(())
}

impl CanonicalTableSink for PartsSink {
    fn start(&mut self, header: &CityArrowHeader, projection: &ProjectionLayout) -> Result<()> {
        self.header = Some(header.clone());
        self.projection = Some(projection.clone());
        Ok(())
    }

    fn push_batch(&mut self, table: CanonicalTable, batch: RecordBatch) -> Result<()> {
        let slot = match table {
            CanonicalTable::Metadata => &mut self.metadata,
            CanonicalTable::Transform => &mut self.transform,
            CanonicalTable::Extensions => &mut self.extensions,
            CanonicalTable::Vertices => &mut self.vertices,
            CanonicalTable::TemplateVertices => &mut self.template_vertices,
            CanonicalTable::TextureVertices => &mut self.texture_vertices,
            CanonicalTable::Semantics => &mut self.semantics,
            CanonicalTable::SemanticChildren => &mut self.semantic_children,
            CanonicalTable::Materials => &mut self.materials,
            CanonicalTable::Textures => &mut self.textures,
            CanonicalTable::TemplateGeometryBoundaries => &mut self.template_geometry_boundaries,
            CanonicalTable::TemplateGeometrySemantics => &mut self.template_geometry_semantics,
            CanonicalTable::TemplateGeometryMaterials => &mut self.template_geometry_materials,
            CanonicalTable::TemplateGeometryRingTextures => {
                &mut self.template_geometry_ring_textures
            }
            CanonicalTable::TemplateGeometries => &mut self.template_geometries,
            CanonicalTable::GeometryBoundaries => &mut self.geometry_boundaries,
            CanonicalTable::GeometrySurfaceSemantics => &mut self.geometry_surface_semantics,
            CanonicalTable::GeometryPointSemantics => &mut self.geometry_point_semantics,
            CanonicalTable::GeometryLinestringSemantics => &mut self.geometry_linestring_semantics,
            CanonicalTable::GeometrySurfaceMaterials => &mut self.geometry_surface_materials,
            CanonicalTable::GeometryRingTextures => &mut self.geometry_ring_textures,
            CanonicalTable::GeometryInstances => &mut self.geometry_instances,
            CanonicalTable::Geometries => &mut self.geometries,
            CanonicalTable::CityObjects => &mut self.cityobjects,
            CanonicalTable::CityObjectChildren => &mut self.cityobject_children,
        };
        assign_table_slot(slot, table, batch)
    }
}

impl PartsSink {
    fn finish(self) -> Result<CityModelArrowParts> {
        Ok(CityModelArrowParts {
            header: self
                .header
                .ok_or_else(|| Error::Conversion("missing canonical table header".to_string()))?,
            projection: self.projection.ok_or_else(|| {
                Error::Conversion("missing canonical table projection".to_string())
            })?,
            metadata: required_batch(self.metadata, CanonicalTable::Metadata)?,
            transform: self.transform,
            extensions: self.extensions,
            vertices: required_batch(self.vertices, CanonicalTable::Vertices)?,
            cityobjects: required_batch(self.cityobjects, CanonicalTable::CityObjects)?,
            cityobject_children: self.cityobject_children,
            geometries: required_batch(self.geometries, CanonicalTable::Geometries)?,
            geometry_boundaries: required_batch(
                self.geometry_boundaries,
                CanonicalTable::GeometryBoundaries,
            )?,
            geometry_instances: self.geometry_instances,
            template_vertices: self.template_vertices,
            template_geometries: self.template_geometries,
            template_geometry_boundaries: self.template_geometry_boundaries,
            semantics: self.semantics,
            semantic_children: self.semantic_children,
            geometry_surface_semantics: self.geometry_surface_semantics,
            geometry_point_semantics: self.geometry_point_semantics,
            geometry_linestring_semantics: self.geometry_linestring_semantics,
            template_geometry_semantics: self.template_geometry_semantics,
            materials: self.materials,
            geometry_surface_materials: self.geometry_surface_materials,
            template_geometry_materials: self.template_geometry_materials,
            textures: self.textures,
            texture_vertices: self.texture_vertices,
            geometry_ring_textures: self.geometry_ring_textures,
            template_geometry_ring_textures: self.template_geometry_ring_textures,
        })
    }
}

fn assign_table_slot(
    slot: &mut Option<RecordBatch>,
    table: CanonicalTable,
    batch: RecordBatch,
) -> Result<()> {
    if slot.replace(batch).is_some() {
        return Err(Error::Unsupported(format!(
            "duplicate '{}' canonical table batch",
            table.as_str()
        )));
    }
    Ok(())
}

fn required_batch(batch: Option<RecordBatch>, table: CanonicalTable) -> Result<RecordBatch> {
    batch.ok_or_else(|| {
        Error::Unsupported(format!(
            "package or stream is missing required '{}' table",
            table.as_str()
        ))
    })
}

fn export_core_batches(context: &ExportContext<'_>) -> Result<ExportCoreBatches> {
    let metadata = metadata_batch(
        &context.schemas.metadata,
        metadata_row(context.model, &context.header),
        &context.projection,
    )?;
    let transform_row = context.model.transform().map(|transform| TransformRow {
        scale: transform.scale(),
        translate: transform.translate(),
    });

    Ok(ExportCoreBatches {
        metadata,
        transform: transform_row
            .map(|row| transform_batch(&context.schemas.transform, row))
            .transpose()?,
        extensions: extensions_batch_from_model(&context.schemas.extensions, context.model)?,
        vertices: vertices_batch_from_model(&context.schemas.vertices, context.model)?,
        cityobjects: cityobjects_batch_from_model(
            &context.schemas.cityobjects,
            context.model,
            &context.projection,
        )?,
        cityobject_children: cityobject_children_batch_from_model(
            &context.schemas.cityobject_children,
            context.model,
        )?,
    })
}

fn export_geometry_batches(
    context: &ExportContext<'_>,
    geometries: GeometryTableBuffer,
    geometry_boundaries: GeometryBoundaryTableBuffer,
    geometry_instances: GeometryInstanceTableBuffer,
    template_geometries: TemplateGeometryTableBuffer,
    template_geometry_boundaries: TemplateGeometryBoundaryTableBuffer,
) -> Result<ExportGeometryBatches> {
    Ok(ExportGeometryBatches {
        geometries: geometries_batch(&context.schemas.geometries, geometries)?,
        geometry_boundaries: geometry_boundaries_batch(
            &context.schemas.geometry_boundaries,
            geometry_boundaries,
        )?,
        geometry_instances: optional_batch_from(geometry_instances.is_empty(), || {
            geometry_instances_batch(&context.schemas.geometry_instances, geometry_instances)
        })?,
        template_vertices: template_vertices_batch_from_model(
            &context.schemas.template_vertices,
            context.model,
        )?,
        template_geometries: optional_batch_from(template_geometries.is_empty(), || {
            template_geometries_batch(&context.schemas.template_geometries, template_geometries)
        })?,
        template_geometry_boundaries: optional_batch_from(
            template_geometry_boundaries.is_empty(),
            || {
                template_geometry_boundaries_batch(
                    &context.schemas.template_geometry_boundaries,
                    template_geometry_boundaries,
                )
            },
        )?,
    })
}

fn export_semantic_batches(
    context: &ExportContext<'_>,
    geometry_surface_semantics: GeometrySurfaceSemanticTableBuffer,
    geometry_point_semantics: GeometryPointSemanticTableBuffer,
    geometry_linestring_semantics: GeometryLinestringSemanticTableBuffer,
    template_geometry_semantics: TemplateGeometrySemanticTableBuffer,
) -> Result<ExportSemanticBatches> {
    Ok(ExportSemanticBatches {
        semantics: semantics_batch_from_model(
            &context.schemas.semantics,
            context.model,
            &context.projection,
        )?,
        semantic_children: semantic_children_batch_from_model(
            &context.schemas.semantic_children,
            context.model,
        )?,
        geometry_surface_semantics: optional_batch_from(
            geometry_surface_semantics.is_empty(),
            || {
                geometry_surface_semantics_batch(
                    &context.schemas.geometry_surface_semantics,
                    geometry_surface_semantics,
                )
            },
        )?,
        geometry_point_semantics: optional_batch_from(geometry_point_semantics.is_empty(), || {
            geometry_point_semantics_batch(
                &context.schemas.geometry_point_semantics,
                geometry_point_semantics,
            )
        })?,
        geometry_linestring_semantics: optional_batch_from(
            geometry_linestring_semantics.is_empty(),
            || {
                geometry_linestring_semantics_batch(
                    &context.schemas.geometry_linestring_semantics,
                    geometry_linestring_semantics,
                )
            },
        )?,
        template_geometry_semantics: optional_batch_from(
            template_geometry_semantics.is_empty(),
            || {
                template_geometry_semantics_batch(
                    &context.schemas.template_geometry_semantics,
                    template_geometry_semantics,
                )
            },
        )?,
    })
}

fn export_appearance_batches(
    context: &ExportContext<'_>,
    geometry_surface_materials: GeometrySurfaceMaterialTableBuffer,
    geometry_ring_textures: GeometryRingTextureTableBuffer,
    template_geometry_materials: TemplateGeometryMaterialTableBuffer,
    template_geometry_ring_textures: TemplateGeometryRingTextureTableBuffer,
) -> Result<ExportAppearanceBatches> {
    Ok(ExportAppearanceBatches {
        materials: materials_batch_from_model(
            &context.schemas.materials,
            context.model,
            &context.projection,
        )?,
        geometry_surface_materials: optional_batch_from(
            geometry_surface_materials.is_empty(),
            || {
                geometry_surface_materials_batch(
                    &context.schemas.geometry_surface_materials,
                    geometry_surface_materials,
                )
            },
        )?,
        template_geometry_materials: optional_batch_from(
            template_geometry_materials.is_empty(),
            || {
                template_geometry_materials_batch(
                    &context.schemas.template_geometry_materials,
                    template_geometry_materials,
                )
            },
        )?,
        textures: textures_batch_from_model(
            &context.schemas.textures,
            context.model,
            &context.projection,
        )?,
        texture_vertices: texture_vertices_batch_from_model(
            &context.schemas.texture_vertices,
            context.model,
        )?,
        geometry_ring_textures: optional_batch_from(geometry_ring_textures.is_empty(), || {
            geometry_ring_textures_batch(
                &context.schemas.geometry_ring_textures,
                geometry_ring_textures,
            )
        })?,
        template_geometry_ring_textures: optional_batch_from(
            template_geometry_ring_textures.is_empty(),
            || {
                template_geometry_ring_textures_batch(
                    &context.schemas.template_geometry_ring_textures,
                    template_geometry_ring_textures,
                )
            },
        )?,
    })
}

/// Reconstructs an in-memory `CityJSON` model from the canonical Arrow table set.
///
/// # Errors
///
/// Returns an error when the provided tables are inconsistent, use unsupported
/// combinations, or contain values that cannot be converted back into `CityJSON`.
pub(crate) fn decode_parts(parts: &CityModelArrowParts) -> Result<OwnedCityModel> {
    let mut decoder = IncrementalDecoder::new(parts.header.clone(), parts.projection.clone())?;
    let mut state =
        initialize_model_from_metadata(&parts.header, &parts.projection, &parts.metadata)?;
    reserve_parts_import_state(&mut state, parts)?;
    decoder.state = Some(state);
    decoder.seen_tables.insert(CanonicalTable::Metadata);
    decoder.last_table_position = Some(canonical_table_position(CanonicalTable::Metadata));
    for (table, batch) in collect_tables(parts).into_iter().skip(1) {
        decoder.push_batch(table, &batch)?;
    }
    decoder.finish()
}

impl IncrementalDecoder {
    pub(crate) fn new(header: CityArrowHeader, projection: ProjectionLayout) -> Result<Self> {
        validate_appearance_projection_layout(&projection)?;
        Ok(Self {
            header,
            schemas: canonical_schema_set(&projection),
            projection,
            state: None,
            grouped_rows: PartBatchViews::default(),
            last_table_position: None,
            seen_tables: BTreeSet::new(),
        })
    }

    pub(crate) fn push_batch(&mut self, table: CanonicalTable, batch: &RecordBatch) -> Result<()> {
        validate_schema(
            schema_for_table(&self.schemas, table),
            batch.schema(),
            table,
        )?;
        self.validate_table_order(table)?;
        self.dispatch_table(table, batch)?;
        self.seen_tables.insert(table);
        self.last_table_position = Some(canonical_table_position(table));
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<OwnedCityModel> {
        ensure_required_tables_seen(&self.seen_tables)?;
        let mut state = self.state.ok_or_else(|| {
            Error::Unsupported("stream or package is missing metadata".to_string())
        })?;
        attach_cityobject_geometries(&mut state)?;
        apply_feature_root_id(&mut state.model, state.pending_feature_root_id.as_deref())?;
        Ok(state.model)
    }

    fn validate_table_order(&self, table: CanonicalTable) -> Result<()> {
        if self.seen_tables.contains(&table) {
            return Err(Error::Unsupported(format!(
                "duplicate '{}' canonical table batch",
                table.as_str()
            )));
        }
        let position = canonical_table_position(table);
        if let Some(previous) = self.last_table_position
            && position <= previous
        {
            return Err(Error::Unsupported(format!(
                "canonical table '{}' arrived out of order",
                table.as_str()
            )));
        }

        for required in canonical_table_order()
            .iter()
            .take(position)
            .copied()
            .filter(|candidate| candidate.is_required())
        {
            if !self.seen_tables.contains(&required) {
                return Err(Error::Unsupported(format!(
                    "missing required '{}' table before '{}'",
                    required.as_str(),
                    table.as_str()
                )));
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn dispatch_table(&mut self, table: CanonicalTable, batch: &RecordBatch) -> Result<()> {
        match table {
            CanonicalTable::Metadata => {
                self.state = Some(initialize_model_from_metadata(
                    &self.header,
                    &self.projection,
                    batch,
                )?);
            }
            CanonicalTable::Transform => import_transform_batch(batch, self.state_mut()?)?,
            CanonicalTable::Extensions => import_extensions_batch(batch, self.state_mut()?)?,
            CanonicalTable::Vertices => import_vertex_batch(batch, self.state_mut()?)?,
            CanonicalTable::TemplateVertices => {
                import_template_vertex_batch(batch, self.state_mut()?)?;
            }
            CanonicalTable::TextureVertices => {
                import_texture_vertex_batch(batch, self.state_mut()?)?;
            }
            CanonicalTable::Semantics => {
                let projection = &self.projection;
                let state = self.state.as_mut().ok_or_else(|| {
                    Error::Unsupported(
                        "metadata table must arrive before other canonical tables".to_string(),
                    )
                })?;
                reserve_model_import(
                    state,
                    CityModelCapacities {
                        semantics: batch.num_rows(),
                        ..CityModelCapacities::default()
                    },
                )?;
                let handles = import_semantics_batch(batch, projection, &mut state.model)?;
                state.semantic_handle_by_id = handles;
            }
            CanonicalTable::SemanticChildren => {
                import_semantic_child_batch(batch, self.state_mut()?)?;
            }
            CanonicalTable::Materials => {
                let projection = &self.projection;
                let state = self.state.as_mut().ok_or_else(|| {
                    Error::Unsupported(
                        "metadata table must arrive before other canonical tables".to_string(),
                    )
                })?;
                reserve_model_import(
                    state,
                    CityModelCapacities {
                        materials: batch.num_rows(),
                        ..CityModelCapacities::default()
                    },
                )?;
                let handles = import_materials_batch(batch, projection, &mut state.model)?;
                state.material_handle_by_id = handles;
            }
            CanonicalTable::Textures => {
                let projection = &self.projection;
                let state = self.state.as_mut().ok_or_else(|| {
                    Error::Unsupported(
                        "metadata table must arrive before other canonical tables".to_string(),
                    )
                })?;
                reserve_model_import(
                    state,
                    CityModelCapacities {
                        textures: batch.num_rows(),
                        ..CityModelCapacities::default()
                    },
                )?;
                let handles = import_textures_batch(batch, projection, &mut state.model)?;
                state.texture_handle_by_id = handles;
            }
            CanonicalTable::TemplateGeometryBoundaries => {
                let view = bind_boundary_batch_view(batch, "template_geometry_id")?;
                let row_by_id = index_unique_ids(
                    &view.id,
                    "template_geometry_id",
                    "template geometry boundary",
                )?;
                self.grouped_rows.template_boundaries = Some(UniqueBatchView { view, row_by_id });
            }
            CanonicalTable::TemplateGeometrySemantics => {
                let ids = downcast_required::<UInt64Array>(batch, "template_geometry_id")?;
                self.grouped_rows.template_semantics = Some(GroupedBatchView {
                    view: bind_template_semantic_batch_view(batch)?,
                    rows_by_id: index_grouped_ids(ids, "template_geometry_id")?,
                });
            }
            CanonicalTable::TemplateGeometryMaterials => {
                let ids = downcast_required::<UInt64Array>(batch, "template_geometry_id")?;
                self.grouped_rows.template_materials = Some(GroupedBatchView {
                    view: bind_template_material_batch_view(batch)?,
                    rows_by_id: index_grouped_ids(ids, "template_geometry_id")?,
                });
            }
            CanonicalTable::TemplateGeometryRingTextures => {
                let ids = downcast_required::<UInt64Array>(batch, "template_geometry_id")?;
                self.grouped_rows.template_ring_textures = Some(GroupedBatchView {
                    view: bind_ring_texture_batch_view(batch)?,
                    rows_by_id: index_grouped_ids(ids, "template_geometry_id")?,
                });
            }
            CanonicalTable::TemplateGeometries => {
                let grouped_rows = &self.grouped_rows;
                let state = self.state.as_mut().ok_or_else(|| {
                    Error::Unsupported(
                        "metadata table must arrive before other canonical tables".to_string(),
                    )
                })?;
                import_template_geometries_batch(batch, state, grouped_rows)?;
            }
            CanonicalTable::GeometryBoundaries => {
                let view = bind_boundary_batch_view(batch, "geometry_id")?;
                let row_by_id = index_unique_ids(&view.id, "geometry_id", "geometry boundary")?;
                self.grouped_rows.boundaries = Some(UniqueBatchView { view, row_by_id });
            }
            CanonicalTable::GeometrySurfaceSemantics => {
                let ids = downcast_required::<UInt64Array>(batch, "geometry_id")?;
                self.grouped_rows.surface_semantics = Some(GroupedBatchView {
                    view: bind_indexed_semantic_batch_view(batch, "surface_ordinal")?,
                    rows_by_id: index_grouped_ids(ids, "geometry_id")?,
                });
            }
            CanonicalTable::GeometryPointSemantics => {
                let ids = downcast_required::<UInt64Array>(batch, "geometry_id")?;
                self.grouped_rows.point_semantics = Some(GroupedBatchView {
                    view: bind_indexed_semantic_batch_view(batch, "point_ordinal")?,
                    rows_by_id: index_grouped_ids(ids, "geometry_id")?,
                });
            }
            CanonicalTable::GeometryLinestringSemantics => {
                let ids = downcast_required::<UInt64Array>(batch, "geometry_id")?;
                self.grouped_rows.linestring_semantics = Some(GroupedBatchView {
                    view: bind_indexed_semantic_batch_view(batch, "linestring_ordinal")?,
                    rows_by_id: index_grouped_ids(ids, "geometry_id")?,
                });
            }
            CanonicalTable::GeometrySurfaceMaterials => {
                let ids = downcast_required::<UInt64Array>(batch, "geometry_id")?;
                self.grouped_rows.surface_materials = Some(GroupedBatchView {
                    view: bind_geometry_surface_material_batch_view(batch)?,
                    rows_by_id: index_grouped_ids(ids, "geometry_id")?,
                });
            }
            CanonicalTable::GeometryRingTextures => {
                let ids = downcast_required::<UInt64Array>(batch, "geometry_id")?;
                self.grouped_rows.ring_textures = Some(GroupedBatchView {
                    view: bind_ring_texture_batch_view(batch)?,
                    rows_by_id: index_grouped_ids(ids, "geometry_id")?,
                });
            }
            CanonicalTable::GeometryInstances => {
                import_instance_geometries_batch(batch, self.state_mut()?)?;
            }
            CanonicalTable::Geometries => {
                let grouped_rows = &self.grouped_rows;
                let state = self.state.as_mut().ok_or_else(|| {
                    Error::Unsupported(
                        "metadata table must arrive before other canonical tables".to_string(),
                    )
                })?;
                import_boundary_geometries_batch(batch, state, grouped_rows)?;
            }
            CanonicalTable::CityObjects => {
                let projection = &self.projection;
                let state = self.state.as_mut().ok_or_else(|| {
                    Error::Unsupported(
                        "metadata table must arrive before other canonical tables".to_string(),
                    )
                })?;
                import_cityobjects_batch(batch, projection, state)?;
            }
            CanonicalTable::CityObjectChildren => {
                import_cityobject_children_batch(batch, self.state_mut()?)?;
            }
        }
        Ok(())
    }

    fn state_mut(&mut self) -> Result<&mut ImportState> {
        self.state.as_mut().ok_or_else(|| {
            Error::Unsupported(
                "metadata table must arrive before other canonical tables".to_string(),
            )
        })
    }
}

fn ensure_required_tables_seen(seen_tables: &BTreeSet<CanonicalTable>) -> Result<()> {
    for table in canonical_table_order()
        .iter()
        .copied()
        .filter(|table| table.is_required())
    {
        if !seen_tables.contains(&table) {
            return Err(Error::Unsupported(format!(
                "stream or package is missing required '{}' table",
                table.as_str()
            )));
        }
    }
    Ok(())
}

fn index_unique_ids(ids: &UInt64Array, id_name: &str, label: &str) -> Result<HashMap<u64, usize>> {
    let mut row_by_id = HashMap::with_capacity(ids.len());
    let mut previous = None;
    for row in 0..ids.len() {
        let id = ids.value(row);
        ensure_strictly_increasing_u64(previous, id, id_name)?;
        previous = Some(id);
        if row_by_id.insert(id, row).is_some() {
            return Err(Error::Conversion(format!("duplicate {label} row {id}")));
        }
    }
    Ok(row_by_id)
}

fn index_grouped_ids(ids: &UInt64Array, id_name: &str) -> Result<HashMap<u64, Range<usize>>> {
    let mut rows_by_id = HashMap::new();
    let mut previous = None;
    let mut range_start = 0_usize;
    for row in 0..ids.len() {
        let id = ids.value(row);
        if let Some(previous_id) = previous {
            if id < previous_id {
                return Err(Error::Conversion(format!(
                    "{id_name} must be non-decreasing in canonical order, found {id} after {previous_id}"
                )));
            }
            if id != previous_id {
                rows_by_id.insert(previous_id, range_start..row);
                range_start = row;
            }
        }
        previous = Some(id);
    }
    if let Some(last_id) = previous {
        rows_by_id.insert(last_id, range_start..ids.len());
    }
    Ok(rows_by_id)
}

fn grouped_row_range<V>(rows: Option<&GroupedBatchView<V>>, id: u64) -> Option<&Range<usize>> {
    rows.and_then(|rows| rows.rows_by_id.get(&id))
}

fn bind_u32_list_column(batch: &RecordBatch, name: &str) -> Result<U32ListColumnView> {
    let list = downcast_required::<ListArray>(batch, name)?.clone();
    let values = required_downcast::<UInt32Array>(list.values().as_ref(), "u32")?.clone();
    Ok(U32ListColumnView { list, values })
}

fn bind_u64_list_column(batch: &RecordBatch, name: &str) -> Result<U64ListColumnView> {
    let list = downcast_required::<ListArray>(batch, name)?.clone();
    let values = required_downcast::<UInt64Array>(list.values().as_ref(), "u64")?.clone();
    Ok(U64ListColumnView { list, values })
}

fn bind_boundary_batch_view(batch: &RecordBatch, id_name: &str) -> Result<BoundaryBatchView> {
    Ok(BoundaryBatchView {
        id: downcast_required::<UInt64Array>(batch, id_name)?.clone(),
        vertex_indices: bind_u32_list_column(batch, "vertex_indices")?,
        line_offsets: bind_u32_list_column(batch, "line_offsets")?,
        ring_offsets: bind_u32_list_column(batch, "ring_offsets")?,
        surface_offsets: bind_u32_list_column(batch, "surface_offsets")?,
        shell_offsets: bind_u32_list_column(batch, "shell_offsets")?,
        solid_offsets: bind_u32_list_column(batch, "solid_offsets")?,
    })
}

fn bind_indexed_semantic_batch_view(
    batch: &RecordBatch,
    ordinal_name: &str,
) -> Result<IndexedSemanticBatchView> {
    Ok(IndexedSemanticBatchView {
        semantic_id: downcast_required::<UInt64Array>(batch, "semantic_id")?.clone(),
        ordinal: downcast_required::<UInt32Array>(batch, ordinal_name)?.clone(),
    })
}

fn bind_template_semantic_batch_view(batch: &RecordBatch) -> Result<TemplateSemanticBatchView> {
    Ok(TemplateSemanticBatchView {
        primitive_type: downcast_required::<StringArray>(batch, "primitive_type")?.clone(),
        primitive_ordinal: downcast_required::<UInt32Array>(batch, "primitive_ordinal")?.clone(),
        semantic_id: downcast_required::<UInt64Array>(batch, "semantic_id")?.clone(),
    })
}

fn bind_geometry_surface_material_batch_view(
    batch: &RecordBatch,
) -> Result<GeometrySurfaceMaterialBatchView> {
    Ok(GeometrySurfaceMaterialBatchView {
        theme: downcast_required::<StringArray>(batch, "theme")?.clone(),
        surface_ordinal: downcast_required::<UInt32Array>(batch, "surface_ordinal")?.clone(),
        material_id: downcast_required::<UInt64Array>(batch, "material_id")?.clone(),
    })
}

fn bind_template_material_batch_view(batch: &RecordBatch) -> Result<TemplateMaterialBatchView> {
    Ok(TemplateMaterialBatchView {
        primitive_type: downcast_required::<StringArray>(batch, "primitive_type")?.clone(),
        primitive_ordinal: downcast_required::<UInt32Array>(batch, "primitive_ordinal")?.clone(),
        theme: downcast_required::<StringArray>(batch, "theme")?.clone(),
        material_id: downcast_required::<UInt64Array>(batch, "material_id")?.clone(),
    })
}

fn bind_ring_texture_batch_view(batch: &RecordBatch) -> Result<RingTextureBatchView> {
    Ok(RingTextureBatchView {
        surface_ordinal: downcast_required::<UInt32Array>(batch, "surface_ordinal")?.clone(),
        ring_ordinal: downcast_required::<UInt32Array>(batch, "ring_ordinal")?.clone(),
        theme: downcast_required::<StringArray>(batch, "theme")?.clone(),
        texture_id: downcast_required::<UInt64Array>(batch, "texture_id")?.clone(),
        uv_indices: bind_u64_list_column(batch, "uv_indices")?,
    })
}

fn initialize_model_from_metadata(
    header: &CityArrowHeader,
    projection: &ProjectionLayout,
    metadata: &RecordBatch,
) -> Result<ImportState> {
    let kind = CityModelType::try_from(read_string_scalar(metadata, "citymodel_kind", 0)?)?;
    let mut model = OwnedCityModel::new(kind);
    let empty_geometry_handles = HashMap::new();

    let metadata_row = read_metadata_row(metadata, projection)?;
    if metadata_row.citymodel_id != header.citymodel_id {
        return Err(Error::Conversion(format!(
            "metadata citymodel_id '{}' does not match stream/package header '{}'",
            metadata_row.citymodel_id, header.citymodel_id
        )));
    }
    if metadata_row.cityjson_version != header.cityjson_version {
        return Err(Error::Conversion(format!(
            "metadata cityjson_version '{}' does not match stream/package header '{}'",
            metadata_row.cityjson_version, header.cityjson_version
        )));
    }
    match kind {
        CityModelType::CityJSONFeature if metadata_row.feature_root_id.is_none() => {
            return Err(Error::Conversion(
                "metadata feature_root_id is required for CityJSONFeature".to_string(),
            ));
        }
        CityModelType::CityJSON if metadata_row.feature_root_id.is_some() => {
            return Err(Error::Conversion(
                "metadata feature_root_id is only valid for CityJSONFeature".to_string(),
            ));
        }
        _ => {}
    }
    apply_metadata_row(&mut model, &metadata_row, &empty_geometry_handles)?;

    Ok(ImportState {
        model,
        pending_feature_root_id: metadata_row.feature_root_id.clone(),
        semantic_handle_by_id: HashMap::new(),
        material_handle_by_id: HashMap::new(),
        texture_handle_by_id: HashMap::new(),
        template_handle_by_id: HashMap::new(),
        geometry_handle_by_id: HashMap::new(),
        cityobject_handle_by_ix: Vec::new(),
        pending_geometry_attachments: Vec::new(),
        fully_reserved: false,
    })
}

fn reserve_model_import(state: &mut ImportState, capacities: CityModelCapacities) -> Result<()> {
    if state.fully_reserved {
        return Ok(());
    }
    state.model.reserve_import(capacities).map_err(Error::from)
}

fn reserve_parts_import_state(state: &mut ImportState, parts: &CityModelArrowParts) -> Result<()> {
    let cityobject_count = parts.cityobjects.num_rows();
    let semantics_count = parts.semantics.as_ref().map_or(0, RecordBatch::num_rows);
    let materials_count = parts.materials.as_ref().map_or(0, RecordBatch::num_rows);
    let textures_count = parts.textures.as_ref().map_or(0, RecordBatch::num_rows);
    let template_geometry_count = parts
        .template_geometries
        .as_ref()
        .map_or(0, RecordBatch::num_rows);
    let geometry_count = parts.geometries.num_rows()
        + parts
            .geometry_instances
            .as_ref()
            .map_or(0, RecordBatch::num_rows);

    state
        .model
        .reserve_import(CityModelCapacities {
            cityobjects: cityobject_count,
            vertices: parts.vertices.num_rows(),
            semantics: semantics_count,
            materials: materials_count,
            textures: textures_count,
            geometries: geometry_count,
            template_vertices: parts
                .template_vertices
                .as_ref()
                .map_or(0, RecordBatch::num_rows),
            template_geometries: template_geometry_count,
            uv_coordinates: parts
                .texture_vertices
                .as_ref()
                .map_or(0, RecordBatch::num_rows),
        })
        .map_err(Error::from)?;
    state.semantic_handle_by_id.reserve(semantics_count);
    state.material_handle_by_id.reserve(materials_count);
    state.texture_handle_by_id.reserve(textures_count);
    state.template_handle_by_id.reserve(template_geometry_count);
    state.geometry_handle_by_id.reserve(geometry_count);
    state.cityobject_handle_by_ix.resize(cityobject_count, None);
    state
        .pending_geometry_attachments
        .resize_with(cityobject_count, Vec::new);
    state.fully_reserved = true;
    Ok(())
}

fn ensure_cityobject_slots_for_ix(state: &mut ImportState, max_cityobject_ix: u64) -> Result<()> {
    let slot_len = usize::try_from(max_cityobject_ix)
        .map_err(|_| Error::Conversion("cityobject_ix does not fit in memory".to_string()))?
        .checked_add(1)
        .ok_or_else(|| Error::Conversion("cityobject slot count overflow".to_string()))?;
    if state.cityobject_handle_by_ix.len() < slot_len {
        state.cityobject_handle_by_ix.resize(slot_len, None);
    }
    if state.pending_geometry_attachments.len() < slot_len {
        state
            .pending_geometry_attachments
            .resize_with(slot_len, Vec::new);
    }
    Ok(())
}

fn import_semantics_batch(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
    model: &mut OwnedCityModel,
) -> Result<HashMap<u64, cityjson::prelude::SemanticHandle>> {
    let empty_geometry_handles = HashMap::new();
    let mut semantic_handle_by_id = HashMap::with_capacity(batch.num_rows());
    let columns = bind_semantic_columns(batch, projection)?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let semantic_id = columns.semantic_id.value(row);
        ensure_strictly_increasing_u64(previous_id, semantic_id, "semantic_id")?;
        previous_id = Some(semantic_id);
        let mut semantic =
            OwnedSemantic::new(parse_semantic_type(columns.semantic_type.value(row)));
        let projected = projected_attributes_from_array(
            projection.semantic_attributes.as_ref(),
            columns.attributes,
            row,
            &empty_geometry_handles,
        )?;
        if !projected.is_empty() {
            *semantic.attributes_mut() = projected;
        }
        semantic_handle_by_id.insert(semantic_id, model.add_semantic(semantic)?);
    }
    Ok(semantic_handle_by_id)
}

fn import_semantic_child_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    let parents = downcast_required::<UInt64Array>(batch, "parent_semantic_id")?;
    let ordinals = downcast_required::<UInt32Array>(batch, "child_ordinal")?;
    let children = downcast_required::<UInt64Array>(batch, "child_semantic_id")?;
    for row in 0..batch.num_rows() {
        let parent_semantic_id = parents.value(row);
        let child_ordinal = ordinals.value(row);
        let child_semantic_id = children.value(row);
        let parent = *state
            .semantic_handle_by_id
            .get(&parent_semantic_id)
            .ok_or_else(|| {
                Error::Conversion(format!(
                    "missing semantic {parent_semantic_id} for child relation"
                ))
            })?;
        let child = *state
            .semantic_handle_by_id
            .get(&child_semantic_id)
            .ok_or_else(|| {
                Error::Conversion(format!(
                    "missing semantic {child_semantic_id} for child relation"
                ))
            })?;
        state
            .model
            .get_semantic_mut(parent)
            .ok_or_else(|| Error::Conversion("semantic parent handle missing".to_string()))?
            .children_mut()
            .push(child);
        state
            .model
            .get_semantic_mut(child)
            .ok_or_else(|| Error::Conversion("semantic child handle missing".to_string()))?
            .set_parent(parent);
        let _ = child_ordinal;
    }
    Ok(())
}

fn import_transform_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    let row = read_transform_row(batch)?;
    state.model.transform_mut().set_scale(row.scale);
    state.model.transform_mut().set_translate(row.translate);
    Ok(())
}

fn import_extensions_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    for row in 0..batch.num_rows() {
        state.model.extensions_mut().add(Extension::new(
            read_string_scalar(batch, "extension_name", row)?,
            read_large_string_scalar(batch, "uri", row)?,
            read_string_optional(batch, "version", row)?.unwrap_or_default(),
        ));
    }
    Ok(())
}

fn import_vertex_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    reserve_model_import(
        state,
        CityModelCapacities {
            vertices: batch.num_rows(),
            ..CityModelCapacities::default()
        },
    )?;
    let columns = bind_vertex_columns(batch, "vertex_id")?;
    let mut previous_id = None;
    let mut vertices = Vec::with_capacity(batch.num_rows());
    for row in 0..batch.num_rows() {
        let vertex_id = columns.vertex_id.value(row);
        ensure_strictly_increasing_u64(previous_id, vertex_id, "vertex_id")?;
        previous_id = Some(vertex_id);
        vertices.push(cityjson::v2_0::RealWorldCoordinate::new(
            columns.x.value(row),
            columns.y.value(row),
            columns.z.value(row),
        ));
    }
    let _ = state.model.add_vertices(&vertices)?;
    Ok(())
}

fn import_template_vertex_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    reserve_model_import(
        state,
        CityModelCapacities {
            template_vertices: batch.num_rows(),
            ..CityModelCapacities::default()
        },
    )?;
    let columns = bind_vertex_columns(batch, "template_vertex_id")?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let template_vertex_id = columns.vertex_id.value(row);
        ensure_strictly_increasing_u64(previous_id, template_vertex_id, "template_vertex_id")?;
        previous_id = Some(template_vertex_id);
        state
            .model
            .add_template_vertex(cityjson::v2_0::RealWorldCoordinate::new(
                columns.x.value(row),
                columns.y.value(row),
                columns.z.value(row),
            ))?;
    }
    Ok(())
}

fn import_texture_vertex_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    reserve_model_import(
        state,
        CityModelCapacities {
            uv_coordinates: batch.num_rows(),
            ..CityModelCapacities::default()
        },
    )?;
    let columns = bind_uv_columns(batch)?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let uv_id = columns.uv_id.value(row);
        ensure_strictly_increasing_u64(previous_id, uv_id, "uv_id")?;
        previous_id = Some(uv_id);
        state.model.add_uv_coordinate(UVCoordinate::new(
            columns.u.value(row),
            columns.v.value(row),
        ))?;
    }
    Ok(())
}

fn import_materials_batch(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
    model: &mut OwnedCityModel,
) -> Result<HashMap<u64, cityjson::prelude::MaterialHandle>> {
    let mut material_handle_by_id = HashMap::with_capacity(batch.num_rows());
    let columns = bind_material_columns(batch, projection)?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let material_id = columns.material_id.value(row);
        ensure_strictly_increasing_u64(previous_id, material_id, "material_id")?;
        previous_id = Some(material_id);
        let mut material = OwnedMaterial::new(columns.name.value(row).to_string());
        material.set_ambient_intensity(
            read_f64_array_optional(columns.ambient_intensity, row).map(decode_payload_f32),
        );
        material.set_diffuse_color(
            read_list_f64_array_optional::<3>(columns.diffuse_color, row)?.map(rgb_from_components),
        );
        material.set_emissive_color(
            read_list_f64_array_optional::<3>(columns.emissive_color, row)?
                .map(rgb_from_components),
        );
        material.set_specular_color(
            read_list_f64_array_optional::<3>(columns.specular_color, row)?
                .map(rgb_from_components),
        );
        material
            .set_shininess(read_f64_array_optional(columns.shininess, row).map(decode_payload_f32));
        material.set_transparency(
            read_f64_array_optional(columns.transparency, row).map(decode_payload_f32),
        );
        material.set_is_smooth(read_bool_array_optional(columns.is_smooth, row));
        material_handle_by_id.insert(material_id, model.add_material(material)?);
    }
    Ok(material_handle_by_id)
}

fn import_textures_batch(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
    model: &mut OwnedCityModel,
) -> Result<HashMap<u64, cityjson::prelude::TextureHandle>> {
    let mut texture_handle_by_id = HashMap::with_capacity(batch.num_rows());
    let columns = bind_texture_columns(batch, projection)?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let texture_id = columns.texture_id.value(row);
        ensure_strictly_increasing_u64(previous_id, texture_id, "texture_id")?;
        previous_id = Some(texture_id);
        let mut texture = OwnedTexture::new(
            columns.image_uri.value(row).to_string(),
            parse_image_type(columns.image_type.value(row))?,
        );
        texture.set_wrap_mode(
            read_large_string_array_optional(columns.wrap_mode, row)
                .as_deref()
                .map(parse_wrap_mode)
                .transpose()?,
        );
        texture.set_texture_type(
            read_large_string_array_optional(columns.texture_type, row)
                .as_deref()
                .map(parse_texture_mapping_type)
                .transpose()?,
        );
        texture.set_border_color(
            read_list_f64_array_optional::<4>(columns.border_color, row)?.map(rgba_from_components),
        );
        texture_handle_by_id.insert(texture_id, model.add_texture(texture)?);
    }
    Ok(texture_handle_by_id)
}

fn import_template_geometries_batch(
    batch: &RecordBatch,
    state: &mut ImportState,
    grouped_rows: &PartBatchViews,
) -> Result<()> {
    reserve_model_import(
        state,
        CityModelCapacities {
            template_geometries: batch.num_rows(),
            ..CityModelCapacities::default()
        },
    )?;
    state.template_handle_by_id.reserve(batch.num_rows());
    let columns = bind_template_geometry_columns(batch)?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let template_geometry_id = columns.template_geometry_id.value(row);
        ensure_strictly_increasing_u64(previous_id, template_geometry_id, "template_geometry_id")?;
        previous_id = Some(template_geometry_id);
        let boundary_row = grouped_rows
            .template_boundaries
            .as_ref()
            .and_then(|rows| rows.row_by_id.get(&template_geometry_id).copied())
            .ok_or_else(|| {
                Error::Conversion(format!(
                    "missing boundary row for template geometry {template_geometry_id}"
                ))
            })?;
        let boundary = grouped_rows
            .template_boundaries
            .as_ref()
            .expect("checked above")
            .view
            .payload(boundary_row)?;
        let geometry = Geometry::from_stored_parts(StoredGeometryParts {
            type_geometry: parse_geometry_type(columns.geometry_type.value(row))?,
            lod: (!columns.lod.is_null(row))
                .then(|| columns.lod.value(row))
                .map(parse_lod)
                .transpose()?,
            boundaries: Some(boundary_from_payload(
                &boundary,
                columns.geometry_type.value(row),
            )?),
            semantics: build_template_semantic_map(
                columns.geometry_type.value(row),
                &boundary,
                grouped_rows.template_semantics.as_ref(),
                template_geometry_id,
                &state.semantic_handle_by_id,
            )?,
            materials: build_template_material_maps(
                columns.geometry_type.value(row),
                &boundary,
                grouped_rows.template_materials.as_ref(),
                template_geometry_id,
                &state.material_handle_by_id,
            )?,
            textures: build_template_texture_maps(
                columns.geometry_type.value(row),
                &boundary,
                grouped_rows.template_ring_textures.as_ref(),
                template_geometry_id,
                &state.texture_handle_by_id,
            )?,
            instance: None,
        });
        state.template_handle_by_id.insert(
            template_geometry_id,
            state.model.add_geometry_template(geometry)?,
        );
    }
    Ok(())
}

fn import_boundary_geometries_batch(
    batch: &RecordBatch,
    state: &mut ImportState,
    grouped_rows: &PartBatchViews,
) -> Result<()> {
    reserve_model_import(
        state,
        CityModelCapacities {
            geometries: batch.num_rows(),
            ..CityModelCapacities::default()
        },
    )?;
    state.geometry_handle_by_id.reserve(batch.num_rows());
    let columns = bind_geometry_columns(batch)?;
    if batch.num_rows() > 0 {
        ensure_cityobject_slots_for_ix(state, columns.cityobject_ix.value(batch.num_rows() - 1))?;
    }
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let geometry_id = columns.geometry_id.value(row);
        ensure_strictly_increasing_u64(previous_id, geometry_id, "geometry_id")?;
        previous_id = Some(geometry_id);
        let boundary_row = grouped_rows
            .boundaries
            .as_ref()
            .and_then(|rows| rows.row_by_id.get(&geometry_id).copied())
            .ok_or_else(|| {
                Error::Conversion(format!("missing boundary row for geometry {geometry_id}"))
            })?;
        let boundary = grouped_rows
            .boundaries
            .as_ref()
            .expect("checked above")
            .view
            .payload(boundary_row)?;
        let geometry = Geometry::from_stored_parts(StoredGeometryParts {
            type_geometry: parse_geometry_type(columns.geometry_type.value(row))?,
            lod: (!columns.lod.is_null(row))
                .then(|| columns.lod.value(row))
                .map(parse_lod)
                .transpose()?,
            boundaries: Some(boundary_from_payload(
                &boundary,
                columns.geometry_type.value(row),
            )?),
            semantics: build_semantic_map(
                columns.geometry_type.value(row),
                &boundary,
                grouped_rows.surface_semantics.as_ref(),
                grouped_rows.point_semantics.as_ref(),
                grouped_rows.linestring_semantics.as_ref(),
                geometry_id,
                &state.semantic_handle_by_id,
            )?,
            materials: build_material_maps(
                columns.geometry_type.value(row),
                &boundary,
                grouped_rows.surface_materials.as_ref(),
                geometry_id,
                &state.material_handle_by_id,
            )?,
            textures: build_texture_maps(
                columns.geometry_type.value(row),
                &boundary,
                grouped_rows.ring_textures.as_ref(),
                geometry_id,
                &state.texture_handle_by_id,
            )?,
            instance: None,
        });
        insert_unique_geometry_handle(
            &mut state.geometry_handle_by_id,
            geometry_id,
            state.model.add_geometry(geometry)?,
        )?;
        push_pending_geometry_attachment(
            state,
            columns.cityobject_ix.value(row),
            columns.geometry_ordinal.value(row),
            geometry_id,
        )?;
    }
    Ok(())
}

fn import_instance_geometries_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    reserve_model_import(
        state,
        CityModelCapacities {
            geometries: batch.num_rows(),
            ..CityModelCapacities::default()
        },
    )?;
    state.geometry_handle_by_id.reserve(batch.num_rows());
    let columns = bind_geometry_instance_columns(batch)?;
    if batch.num_rows() > 0 {
        ensure_cityobject_slots_for_ix(state, columns.cityobject_ix.value(batch.num_rows() - 1))?;
    }
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let geometry_id = columns.geometry_id.value(row);
        ensure_strictly_increasing_u64(previous_id, geometry_id, "geometry_instance_id")?;
        previous_id = Some(geometry_id);
        let template = *state
            .template_handle_by_id
            .get(&columns.template_geometry_id.value(row))
            .ok_or_else(|| {
                Error::Conversion(format!(
                    "missing template geometry {}",
                    columns.template_geometry_id.value(row)
                ))
            })?;
        let reference_point =
            u32::try_from(columns.reference_point_vertex_id.value(row)).map_err(|_| {
                Error::Conversion(format!(
                    "reference point vertex id {} does not fit into u32",
                    columns.reference_point_vertex_id.value(row)
                ))
            })?;
        let geometry = Geometry::from_stored_parts(StoredGeometryParts {
            type_geometry: GeometryType::GeometryInstance,
            lod: (!columns.lod.is_null(row))
                .then(|| columns.lod.value(row))
                .map(parse_lod)
                .transpose()?,
            boundaries: None,
            semantics: None,
            materials: None,
            textures: None,
            instance: Some(StoredGeometryInstance {
                template,
                reference_point: cityjson::v2_0::VertexIndex::new(reference_point),
                transformation: read_fixed_size_list_array_optional::<16>(
                    columns.transform_matrix,
                    "transform_matrix",
                    row,
                )?
                .map(cityjson::v2_0::AffineTransform3D::from)
                .unwrap_or_default(),
            }),
        });
        insert_unique_geometry_handle(
            &mut state.geometry_handle_by_id,
            geometry_id,
            state.model.add_geometry(geometry)?,
        )?;
        push_pending_geometry_attachment(
            state,
            columns.cityobject_ix.value(row),
            columns.geometry_ordinal.value(row),
            geometry_id,
        )?;
    }
    Ok(())
}

fn insert_unique_geometry_handle(
    handles: &mut HashMap<u64, cityjson::prelude::GeometryHandle>,
    geometry_id: u64,
    handle: cityjson::prelude::GeometryHandle,
) -> Result<()> {
    if handles.insert(geometry_id, handle).is_some() {
        return Err(Error::Conversion(format!(
            "duplicate geometry id {geometry_id}"
        )));
    }
    Ok(())
}

fn ensure_slot<T: Default>(slots: &mut Vec<T>, index: usize) {
    if slots.len() <= index {
        slots.resize_with(index + 1, T::default);
    }
}

fn push_pending_geometry_attachment(
    state: &mut ImportState,
    cityobject_ix: u64,
    geometry_ordinal: u32,
    geometry_id: u64,
) -> Result<()> {
    let cityobject_ix = usize::try_from(cityobject_ix)
        .map_err(|_| Error::Conversion("cityobject_ix does not fit in memory".to_string()))?;
    ensure_slot(&mut state.pending_geometry_attachments, cityobject_ix);
    let attachments = &mut state.pending_geometry_attachments[cityobject_ix];
    if let Some((last_ordinal, last_geometry_id)) = attachments.last()
        && (geometry_ordinal < *last_ordinal
            || (geometry_ordinal == *last_ordinal && geometry_id <= *last_geometry_id))
    {
        return Err(Error::Conversion(format!(
            "geometry attachment order for cityobject_ix {cityobject_ix} is not strictly increasing"
        )));
    }
    attachments.push((geometry_ordinal, geometry_id));
    Ok(())
}

fn register_cityobject_handle(
    state: &mut ImportState,
    cityobject_ix: u64,
    handle: cityjson::prelude::CityObjectHandle,
) -> Result<()> {
    let cityobject_ix = usize::try_from(cityobject_ix)
        .map_err(|_| Error::Conversion("cityobject_ix does not fit in memory".to_string()))?;
    ensure_slot(&mut state.cityobject_handle_by_ix, cityobject_ix);
    let slot = &mut state.cityobject_handle_by_ix[cityobject_ix];
    if slot.replace(handle).is_some() {
        return Err(Error::Conversion(format!(
            "duplicate cityobject_ix {cityobject_ix}"
        )));
    }
    Ok(())
}

fn cityobject_handle(
    state: &ImportState,
    cityobject_ix: u64,
) -> Result<cityjson::prelude::CityObjectHandle> {
    let cityobject_ix = usize::try_from(cityobject_ix)
        .map_err(|_| Error::Conversion("cityobject_ix does not fit in memory".to_string()))?;
    state
        .cityobject_handle_by_ix
        .get(cityobject_ix)
        .and_then(|handle| *handle)
        .ok_or_else(|| Error::Conversion(format!("missing cityobject_ix {cityobject_ix}")))
}

fn import_cityobjects_batch(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
    state: &mut ImportState,
) -> Result<()> {
    reserve_model_import(
        state,
        CityModelCapacities {
            cityobjects: batch.num_rows(),
            ..CityModelCapacities::default()
        },
    )?;
    let columns = bind_cityobject_columns(batch, projection)?;
    if batch.num_rows() > 0 {
        ensure_cityobject_slots_for_ix(state, columns.cityobject_ix.value(batch.num_rows() - 1))?;
    }
    let mut previous_ix = None;
    for row in 0..batch.num_rows() {
        let object_index = columns.cityobject_ix.value(row);
        ensure_strictly_increasing_u64(previous_ix, object_index, "cityobject_ix")?;
        previous_ix = Some(object_index);
        let object_id = columns.cityobject_id.value(row).to_string();
        let mut object = CityObject::new(
            CityObjectIdentifier::new(object_id.clone()),
            columns
                .object_type
                .value(row)
                .parse::<CityObjectType<_>>()?,
        );
        if let Some(extent) = read_fixed_size_list_array_optional::<6>(
            columns.geographical_extent,
            "geographical_extent",
            row,
        )? {
            object.set_geographical_extent(Some(BBox::from(extent)));
        }
        let projected_attributes = projected_attributes_from_array(
            projection.cityobject_attributes.as_ref(),
            columns.attributes,
            row,
            &state.geometry_handle_by_id,
        )?;
        if !projected_attributes.is_empty() {
            *object.attributes_mut() = projected_attributes;
        }
        let projected_extra = projected_attributes_from_array(
            projection.cityobject_extra.as_ref(),
            columns.extra,
            row,
            &state.geometry_handle_by_id,
        )?;
        if !projected_extra.is_empty() {
            *object.extra_mut() = projected_extra;
        }
        let handle = state.model.cityobjects_mut().add(object)?;
        register_cityobject_handle(state, object_index, handle)?;
    }
    Ok(())
}

fn attach_cityobject_geometries(state: &mut ImportState) -> Result<()> {
    for (cityobject_ix, attachments) in state.pending_geometry_attachments.iter_mut().enumerate() {
        if attachments.is_empty() {
            continue;
        }
        let object = state
            .cityobject_handle_by_ix
            .get(cityobject_ix)
            .and_then(|handle| *handle)
            .ok_or_else(|| Error::Conversion(format!("missing cityobject_ix {cityobject_ix}")))?;
        let object = state
            .model
            .cityobjects_mut()
            .get_mut(object)
            .ok_or_else(|| Error::Conversion("missing cityobject handle".to_string()))?;
        for (_, geometry_id) in attachments.iter() {
            let geometry = state
                .geometry_handle_by_id
                .get(geometry_id)
                .copied()
                .ok_or_else(|| Error::Conversion(format!("missing geometry {geometry_id}")))?;
            object.add_geometry(geometry);
        }
    }
    Ok(())
}

fn apply_feature_root_id(model: &mut OwnedCityModel, feature_root_id: Option<&str>) -> Result<()> {
    let Some(feature_root_id) = feature_root_id else {
        return Ok(());
    };
    let handle = model
        .cityobjects()
        .iter()
        .find_map(|(handle, cityobject)| (cityobject.id() == feature_root_id).then_some(handle))
        .ok_or_else(|| {
            Error::Conversion(format!(
                "feature_root_id does not resolve to a CityObject: {feature_root_id}"
            ))
        })?;
    model.set_id(Some(handle));
    Ok(())
}

fn import_cityobject_children_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    let parents = downcast_required::<UInt64Array>(batch, "parent_cityobject_ix")?;
    let children = downcast_required::<UInt64Array>(batch, "child_cityobject_ix")?;
    for row in 0..batch.num_rows() {
        let parent = cityobject_handle(state, parents.value(row))?;
        let child = cityobject_handle(state, children.value(row))?;
        state
            .model
            .cityobjects_mut()
            .get_mut(parent)
            .ok_or_else(|| Error::Conversion("missing parent handle".to_string()))?
            .add_child(child);
        state
            .model
            .cityobjects_mut()
            .get_mut(child)
            .ok_or_else(|| Error::Conversion("missing child handle".to_string()))?
            .add_parent(parent);
    }
    Ok(())
}

fn reject_unsupported_modules(model: &OwnedCityModel) -> Result<()> {
    for (_, geometry) in model.iter_geometries() {
        if geometry.textures().is_some() {
            ensure_surface_backed_geometry(*geometry.type_geometry(), "geometry textures")?;
        }
    }
    for (_, geometry) in model.iter_geometry_templates() {
        if geometry.instance().is_some() {
            return Err(Error::Unsupported(
                "geometry instances in template geometry pool".to_string(),
            ));
        }
        if geometry.textures().is_some() {
            ensure_surface_backed_geometry(
                *geometry.type_geometry(),
                "template geometry textures",
            )?;
        }
    }
    Ok(())
}

fn infer_citymodel_id(model: &OwnedCityModel) -> String {
    model
        .metadata()
        .and_then(|metadata| metadata.identifier().map(ToString::to_string))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_CITYMODEL_ID.to_string())
}

fn usize_to_u32(value: usize, label: &str) -> Result<u32> {
    u32::try_from(value)
        .map_err(|_| Error::Conversion(format!("{label} {value} does not fit into u32")))
}

fn usize_to_i32(value: usize, label: &str) -> Result<i32> {
    i32::try_from(value)
        .map_err(|_| Error::Conversion(format!("{label} {value} does not fit into i32")))
}

#[allow(clippy::cast_possible_truncation)]
fn decode_payload_f32(value: f64) -> f32 {
    value as f32
}

trait RawPartsHandle {
    fn raw_parts(self) -> (u32, u16);
}

impl RawPartsHandle for cityjson::prelude::GeometryHandle {
    fn raw_parts(self) -> (u32, u16) {
        self.raw_parts()
    }
}

impl RawPartsHandle for cityjson::prelude::GeometryTemplateHandle {
    fn raw_parts(self) -> (u32, u16) {
        self.raw_parts()
    }
}

impl RawPartsHandle for cityjson::prelude::SemanticHandle {
    fn raw_parts(self) -> (u32, u16) {
        self.raw_parts()
    }
}

impl RawPartsHandle for cityjson::prelude::MaterialHandle {
    fn raw_parts(self) -> (u32, u16) {
        self.raw_parts()
    }
}

impl RawPartsHandle for cityjson::prelude::TextureHandle {
    fn raw_parts(self) -> (u32, u16) {
        self.raw_parts()
    }
}

fn raw_id_from_handle(handle: impl RawPartsHandle) -> u64 {
    let (index, generation) = handle.raw_parts();
    (u64::from(index) << 16) | u64::from(generation)
}

fn metadata_row(model: &OwnedCityModel, header: &CityArrowHeader) -> MetadataRow {
    let metadata = model.metadata();
    MetadataRow {
        citymodel_id: header.citymodel_id.clone(),
        cityjson_version: header.cityjson_version.clone(),
        citymodel_kind: model.type_citymodel().to_string(),
        feature_root_id: model.id().and_then(|handle| {
            model
                .cityobjects()
                .get(handle)
                .map(|cityobject| cityobject.id().to_string())
        }),
        identifier: metadata.and_then(|item| item.identifier().map(ToString::to_string)),
        title: metadata.and_then(Metadata::title).map(ToString::to_string),
        reference_system: metadata
            .and_then(|item| item.reference_system().map(ToString::to_string)),
        geographical_extent: metadata
            .and_then(Metadata::geographical_extent)
            .map(|bbox| bbox.as_slice().try_into().expect("bbox is 6 long")),
        reference_date: metadata
            .and_then(Metadata::reference_date)
            .map(ToString::to_string),
        default_material_theme: model.default_material_theme().map(ToString::to_string),
        default_texture_theme: model.default_texture_theme().map(ToString::to_string),
        point_of_contact: metadata
            .and_then(Metadata::point_of_contact)
            .map(|contact| MetadataContactRow {
                contact_name: contact.contact_name().to_string(),
                email_address: contact.email_address().to_string(),
                role: contact.role().map(|value| value.to_string()),
                website: contact.website().clone(),
                contact_type: contact.contact_type().map(|value| value.to_string()),
                phone: contact.phone().clone(),
                organization: contact.organization().clone(),
                address: contact.address().cloned(),
            }),
        root_extra: cloned_attributes(model.extra()),
        metadata_extra: metadata.and_then(Metadata::extra).cloned(),
    }
}

fn cityobject_ix_map(model: &OwnedCityModel) -> HashMap<cityjson::prelude::CityObjectHandle, u64> {
    model
        .cityobjects()
        .iter()
        .enumerate()
        .map(|(index, (handle, _))| {
            (
                handle,
                u64::try_from(index).expect("cityobject index fits into u64"),
            )
        })
        .collect()
}

fn extensions_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
) -> Result<Option<RecordBatch>> {
    let Some(extensions) = model.extensions() else {
        return Ok(None);
    };
    if extensions.is_empty() {
        return Ok(None);
    }
    Ok(Some(RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(
                extensions
                    .iter()
                    .map(|extension| Some(extension.name().clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(LargeStringArray::from(
                extensions
                    .iter()
                    .map(|extension| Some(extension.url().clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                extensions
                    .iter()
                    .map(|extension| Some(extension.version().clone()))
                    .collect::<Vec<_>>(),
            )),
        ],
    )?))
}

fn vertices_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
) -> Result<RecordBatch> {
    vertex_batch_from_coordinates(schema, model.vertices().as_slice(), "vertex_id")
}

fn cityobjects_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
    projection: &ProjectionLayout,
) -> Result<RecordBatch> {
    let mut cityobject_id = Vec::with_capacity(model.cityobjects().len());
    let mut cityobject_index = Vec::with_capacity(model.cityobjects().len());
    let mut object_type = Vec::with_capacity(model.cityobjects().len());
    let mut geographical_extent: Vec<Option<[f64; 6]>> =
        Vec::with_capacity(model.cityobjects().len());
    let mut attributes = Vec::with_capacity(model.cityobjects().len());
    let mut extra = Vec::with_capacity(model.cityobjects().len());

    for (index, (_, object)) in model.cityobjects().iter().enumerate() {
        cityobject_id.push(Some(object.id().to_string()));
        cityobject_index.push(u64::try_from(index).expect("cityobject index fits into u64"));
        object_type.push(Some(object.type_cityobject().to_string()));
        geographical_extent.push(
            object
                .geographical_extent()
                .map(|bbox| bbox.as_slice().try_into().expect("bbox is 6 long")),
        );
        attributes.push(non_empty_attributes(object.attributes()));
        extra.push(non_empty_attributes(object.extra()));
    }

    let mut fields = SchemaFieldLookup::new(schema);
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(LargeStringArray::from(cityobject_id)),
        Arc::new(UInt64Array::from(cityobject_index)),
        Arc::new(StringArray::from(object_type)),
        Arc::new(fixed_size_f64_array(
            &fields.field("geographical_extent")?,
            6,
            geographical_extent,
        )?),
    ];

    if let Some(spec) = projection.cityobject_attributes.as_ref() {
        arrays.push(projected_struct_array_from_attributes(
            &fields.field("attributes")?,
            spec,
            &attributes,
        )?);
    }
    if let Some(spec) = projection.cityobject_extra.as_ref() {
        arrays.push(projected_struct_array_from_attributes(
            &fields.field("extra")?,
            spec,
            &extra,
        )?);
    }

    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn cityobject_children_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
) -> Result<Option<RecordBatch>> {
    let cityobject_ix_map = cityobject_ix_map(model);
    let mut parent_cityobject_ix = Vec::new();
    let mut child_ordinal = Vec::new();
    let mut child_cityobject_ix = Vec::new();
    for (parent_handle, object) in model.cityobjects().iter() {
        let parent_ix = cityobject_ix_map
            .get(&parent_handle)
            .copied()
            .unwrap_or_default();
        if let Some(children) = object.children() {
            for (ordinal, child) in children.iter().enumerate() {
                if let Some(child_ix) = cityobject_ix_map.get(child).copied() {
                    parent_cityobject_ix.push(parent_ix);
                    child_ordinal.push(usize_to_u32(ordinal, "child ordinal")?);
                    child_cityobject_ix.push(child_ix);
                }
            }
        }
    }
    if parent_cityobject_ix.is_empty() {
        return Ok(None);
    }
    Ok(Some(RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(parent_cityobject_ix)),
            Arc::new(UInt32Array::from(child_ordinal)),
            Arc::new(UInt64Array::from(child_cityobject_ix)),
        ],
    )?))
}

fn template_vertices_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
) -> Result<Option<RecordBatch>> {
    if model.template_vertices().as_slice().is_empty() {
        return Ok(None);
    }
    Ok(Some(vertex_batch_from_coordinates(
        schema,
        model.template_vertices().as_slice(),
        "template_vertex_id",
    )?))
}

fn semantics_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
    projection: &ProjectionLayout,
) -> Result<Option<RecordBatch>> {
    if model.semantic_count() == 0 {
        return Ok(None);
    }
    let semantic_type = model
        .iter_semantics()
        .map(|(_, semantic)| Some(encode_semantic_type(semantic.type_semantic())))
        .collect::<Vec<_>>();
    let semantic_id = model
        .iter_semantics()
        .map(|(handle, _)| raw_id_from_handle(handle))
        .collect::<Vec<_>>();
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(UInt64Array::from(semantic_id)),
        Arc::new(StringArray::from(semantic_type)),
    ];
    if let Some(spec) = projection.semantic_attributes.as_ref() {
        let attrs = model
            .iter_semantics()
            .map(|(_, semantic)| semantic.attributes())
            .collect::<Vec<_>>();
        arrays.push(projected_struct_array_from_attributes(
            &field_from_schema(schema, "attributes")?,
            spec,
            &attrs,
        )?);
    }
    Ok(Some(RecordBatch::try_new(schema.clone(), arrays)?))
}

fn semantic_children_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
) -> Result<Option<RecordBatch>> {
    let mut parent_semantic_id = Vec::new();
    let mut child_ordinal = Vec::new();
    let mut child_semantic_id = Vec::new();
    for (handle, semantic) in model.iter_semantics() {
        if let Some(children) = semantic.children() {
            let parent_id = raw_id_from_handle(handle);
            for (ordinal, child) in children.iter().enumerate() {
                let child_id = raw_id_from_handle(*child);
                parent_semantic_id.push(parent_id);
                child_ordinal.push(usize_to_u32(ordinal, "child ordinal")?);
                child_semantic_id.push(child_id);
            }
        }
    }
    if parent_semantic_id.is_empty() {
        return Ok(None);
    }
    Ok(Some(RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(parent_semantic_id)),
            Arc::new(UInt32Array::from(child_ordinal)),
            Arc::new(UInt64Array::from(child_semantic_id)),
        ],
    )?))
}

fn materials_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
    projection: &ProjectionLayout,
) -> Result<Option<RecordBatch>> {
    if model.material_count() == 0 {
        return Ok(None);
    }
    let mut material_id = Vec::with_capacity(model.material_count());
    let mut name = Vec::with_capacity(model.material_count());
    let mut ambient_intensity = Vec::with_capacity(model.material_count());
    let mut diffuse_color = Vec::with_capacity(model.material_count());
    let mut emissive_color = Vec::with_capacity(model.material_count());
    let mut specular_color = Vec::with_capacity(model.material_count());
    let mut shininess = Vec::with_capacity(model.material_count());
    let mut transparency = Vec::with_capacity(model.material_count());
    let mut is_smooth = Vec::with_capacity(model.material_count());

    for (handle, material) in model.iter_materials() {
        material_id.push(raw_id_from_handle(handle));
        name.push(Some(material.name().clone()));
        ambient_intensity.push(material.ambient_intensity().map(f64::from));
        diffuse_color.push(material.diffuse_color().map(rgb_to_components));
        emissive_color.push(material.emissive_color().map(rgb_to_components));
        specular_color.push(material.specular_color().map(rgb_to_components));
        shininess.push(material.shininess().map(f64::from));
        transparency.push(material.transparency().map(f64::from));
        is_smooth.push(material.is_smooth());
    }

    let mut arrays: Vec<ArrayRef> = vec![Arc::new(UInt64Array::from(material_id))];
    if let Some(specs) = &projection.material_payload {
        for spec in &specs.fields {
            arrays.push(match spec.name.as_str() {
                FIELD_MATERIAL_NAME => Arc::new(LargeStringArray::from(name.clone())) as ArrayRef,
                FIELD_MATERIAL_AMBIENT_INTENSITY => {
                    Arc::new(Float64Array::from(ambient_intensity.clone())) as ArrayRef
                }
                FIELD_MATERIAL_DIFFUSE_COLOR => Arc::new(list_f64_array(
                    &Arc::new(spec.to_arrow_field()),
                    diffuse_color
                        .iter()
                        .map(|row| row.map(|value| value.into_iter().collect()))
                        .collect::<Vec<_>>(),
                )?) as ArrayRef,
                FIELD_MATERIAL_EMISSIVE_COLOR => Arc::new(list_f64_array(
                    &Arc::new(spec.to_arrow_field()),
                    emissive_color
                        .iter()
                        .map(|row| row.map(|value| value.into_iter().collect()))
                        .collect::<Vec<_>>(),
                )?) as ArrayRef,
                FIELD_MATERIAL_SPECULAR_COLOR => Arc::new(list_f64_array(
                    &Arc::new(spec.to_arrow_field()),
                    specular_color
                        .iter()
                        .map(|row| row.map(|value| value.into_iter().collect()))
                        .collect::<Vec<_>>(),
                )?) as ArrayRef,
                FIELD_MATERIAL_SHININESS => {
                    Arc::new(Float64Array::from(shininess.clone())) as ArrayRef
                }
                FIELD_MATERIAL_TRANSPARENCY => {
                    Arc::new(Float64Array::from(transparency.clone())) as ArrayRef
                }
                FIELD_MATERIAL_IS_SMOOTH => {
                    Arc::new(::arrow::array::BooleanArray::from(is_smooth.clone())) as ArrayRef
                }
                other => {
                    return Err(Error::Conversion(format!(
                        "unsupported material projection column {other}"
                    )));
                }
            });
        }
    }

    Ok(Some(RecordBatch::try_new(schema.clone(), arrays)?))
}

fn textures_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
    projection: &ProjectionLayout,
) -> Result<Option<RecordBatch>> {
    if model.texture_count() == 0 {
        return Ok(None);
    }
    let mut texture_id = Vec::with_capacity(model.texture_count());
    let mut image_uri = Vec::with_capacity(model.texture_count());
    let mut image_type = Vec::with_capacity(model.texture_count());
    let mut wrap_mode = Vec::with_capacity(model.texture_count());
    let mut texture_type = Vec::with_capacity(model.texture_count());
    let mut border_color = Vec::with_capacity(model.texture_count());

    for (handle, texture) in model.iter_textures() {
        texture_id.push(raw_id_from_handle(handle));
        image_uri.push(Some(texture.image().clone()));
        image_type.push(Some(texture.image_type().to_string()));
        wrap_mode.push(texture.wrap_mode().map(|value| value.to_string()));
        texture_type.push(texture.texture_type().map(|value| value.to_string()));
        border_color.push(texture.border_color().map(rgba_to_components));
    }

    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(UInt64Array::from(texture_id)),
        Arc::new(LargeStringArray::from(image_uri)),
    ];
    if let Some(specs) = &projection.texture_payload {
        for spec in &specs.fields {
            arrays.push(match spec.name.as_str() {
                FIELD_TEXTURE_IMAGE_TYPE => {
                    Arc::new(LargeStringArray::from(image_type.clone())) as ArrayRef
                }
                FIELD_TEXTURE_WRAP_MODE => {
                    Arc::new(LargeStringArray::from(wrap_mode.clone())) as ArrayRef
                }
                FIELD_TEXTURE_TEXTURE_TYPE => {
                    Arc::new(LargeStringArray::from(texture_type.clone())) as ArrayRef
                }
                FIELD_TEXTURE_BORDER_COLOR => Arc::new(list_f64_array(
                    &Arc::new(spec.to_arrow_field()),
                    border_color
                        .iter()
                        .map(|row| row.map(|value| value.into_iter().collect()))
                        .collect::<Vec<_>>(),
                )?) as ArrayRef,
                other => {
                    return Err(Error::Conversion(format!(
                        "unsupported texture projection column {other}"
                    )));
                }
            });
        }
    }
    Ok(Some(RecordBatch::try_new(schema.clone(), arrays)?))
}

fn texture_vertices_batch_from_model(
    schema: &Arc<::arrow::datatypes::Schema>,
    model: &OwnedCityModel,
) -> Result<Option<RecordBatch>> {
    if model.vertices_texture().as_slice().is_empty() {
        return Ok(None);
    }
    Ok(Some(RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                (0..model.vertices_texture().as_slice().len())
                    .map(|index| index as u64)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float32Array::from(
                model
                    .vertices_texture()
                    .as_slice()
                    .iter()
                    .map(cityjson::v2_0::UVCoordinate::u)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float32Array::from(
                model
                    .vertices_texture()
                    .as_slice()
                    .iter()
                    .map(cityjson::v2_0::UVCoordinate::v)
                    .collect::<Vec<_>>(),
            )),
        ],
    )?))
}

fn vertex_batch_from_coordinates(
    schema: &Arc<::arrow::datatypes::Schema>,
    coordinates: &[cityjson::v2_0::RealWorldCoordinate],
    _id_name: &str,
) -> Result<RecordBatch> {
    let count = coordinates.len();
    let mut ids = MutableBuffer::new(count * std::mem::size_of::<u64>());
    let mut x = MutableBuffer::new(count * std::mem::size_of::<f64>());
    let mut y = MutableBuffer::new(count * std::mem::size_of::<f64>());
    let mut z = MutableBuffer::new(count * std::mem::size_of::<f64>());

    for (index, coordinate) in coordinates.iter().enumerate() {
        ids.push(index as u64);
        x.push(coordinate.x());
        y.push(coordinate.y());
        z.push(coordinate.z());
    }

    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::new(ScalarBuffer::from(ids), None)),
            Arc::new(Float64Array::new(ScalarBuffer::from(x), None)),
            Arc::new(Float64Array::new(ScalarBuffer::from(y), None)),
            Arc::new(Float64Array::new(ScalarBuffer::from(z), None)),
        ],
    )
    .map_err(Error::from)
}

fn encode_semantic_type(
    semantic_type: &SemanticType<cityjson::prelude::OwnedStringStorage>,
) -> String {
    match semantic_type {
        SemanticType::Default => "Default".to_string(),
        SemanticType::RoofSurface => "RoofSurface".to_string(),
        SemanticType::GroundSurface => "GroundSurface".to_string(),
        SemanticType::WallSurface => "WallSurface".to_string(),
        SemanticType::ClosureSurface => "ClosureSurface".to_string(),
        SemanticType::OuterCeilingSurface => "OuterCeilingSurface".to_string(),
        SemanticType::OuterFloorSurface => "OuterFloorSurface".to_string(),
        SemanticType::Window => "Window".to_string(),
        SemanticType::Door => "Door".to_string(),
        SemanticType::InteriorWallSurface => "InteriorWallSurface".to_string(),
        SemanticType::CeilingSurface => "CeilingSurface".to_string(),
        SemanticType::FloorSurface => "FloorSurface".to_string(),
        SemanticType::WaterSurface => "WaterSurface".to_string(),
        SemanticType::WaterGroundSurface => "WaterGroundSurface".to_string(),
        SemanticType::WaterClosureSurface => "WaterClosureSurface".to_string(),
        SemanticType::TrafficArea => "TrafficArea".to_string(),
        SemanticType::AuxiliaryTrafficArea => "AuxiliaryTrafficArea".to_string(),
        SemanticType::TransportationMarking => "TransportationMarking".to_string(),
        SemanticType::TransportationHole => "TransportationHole".to_string(),
        SemanticType::Extension(value) => value.clone(),
        other => other.to_string(),
    }
}

fn cloned_attributes(
    attributes: Option<&cityjson::v2_0::OwnedAttributes>,
) -> Option<cityjson::v2_0::OwnedAttributes> {
    attributes
        .cloned()
        .filter(|attributes| !attributes.is_empty())
}

fn non_empty_attributes(
    attributes: Option<&cityjson::v2_0::OwnedAttributes>,
) -> Option<&cityjson::v2_0::OwnedAttributes> {
    attributes.filter(|attributes| !attributes.is_empty())
}

fn metadata_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    row: MetadataRow,
    projection: &ProjectionLayout,
) -> Result<RecordBatch> {
    let MetadataRow {
        citymodel_id,
        cityjson_version,
        citymodel_kind,
        feature_root_id,
        identifier,
        title,
        reference_system,
        geographical_extent,
        reference_date,
        default_material_theme,
        default_texture_theme,
        point_of_contact,
        root_extra,
        metadata_extra,
    } = row;
    let mut fields = SchemaFieldLookup::new(schema);
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(LargeStringArray::from(vec![Some(citymodel_id)])),
        Arc::new(StringArray::from(vec![Some(cityjson_version)])),
        Arc::new(StringArray::from(vec![Some(citymodel_kind)])),
        Arc::new(LargeStringArray::from(vec![feature_root_id])),
        Arc::new(LargeStringArray::from(vec![identifier])),
        Arc::new(LargeStringArray::from(vec![title])),
        Arc::new(LargeStringArray::from(vec![reference_system])),
        Arc::new(fixed_size_f64_array(
            &fields.field("geographical_extent")?,
            6,
            vec![geographical_extent],
        )?),
        Arc::new(StringArray::from(vec![reference_date])),
        Arc::new(StringArray::from(vec![default_material_theme])),
        Arc::new(StringArray::from(vec![default_texture_theme])),
        point_of_contact_array(
            &fields.field("point_of_contact")?,
            point_of_contact.as_ref(),
            projection.metadata_point_of_contact_address.as_ref(),
        )?,
    ];
    if let Some(spec) = projection.root_extra.as_ref() {
        arrays.push(projected_struct_array_from_attributes(
            &fields.field("root_extra")?,
            spec,
            &[root_extra.as_ref()],
        )?);
    }
    if let Some(spec) = projection.metadata_extra.as_ref() {
        arrays.push(projected_struct_array_from_attributes(
            &fields.field("metadata_extra")?,
            spec,
            &[metadata_extra.as_ref()],
        )?);
    }
    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn transform_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    row: TransformRow,
) -> Result<RecordBatch> {
    let mut fields = SchemaFieldLookup::new(schema);
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(fixed_size_f64_array(
                &fields.field("scale")?,
                3,
                vec![Some(row.scale)],
            )?),
            Arc::new(fixed_size_f64_array(
                &fields.field("translate")?,
                3,
                vec![Some(row.translate)],
            )?),
        ],
    )
    .map_err(Error::from)
}

fn geometries_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometryTableBuffer,
) -> Result<RecordBatch> {
    let GeometryTableBuffer {
        geometry_id,
        cityobject_ix,
        geometry_ordinal,
        geometry_type,
        lod,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(geometry_id)),
            Arc::new(UInt64Array::from(cityobject_ix)),
            Arc::new(UInt32Array::from(geometry_ordinal)),
            Arc::new(StringArray::from(
                geometry_type.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(lod)),
        ],
    )
    .map_err(Error::from)
}

fn geometry_boundaries_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometryBoundaryTableBuffer,
) -> Result<RecordBatch> {
    let GeometryBoundaryTableBuffer {
        geometry_id,
        vertex_indices,
        line_offsets,
        ring_offsets,
        surface_offsets,
        shell_offsets,
        solid_offsets,
    } = rows;
    let mut fields = SchemaFieldLookup::new(schema);
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(geometry_id)),
            Arc::new(vertex_indices.into_array(&fields.field("vertex_indices")?)?),
            Arc::new(line_offsets.into_array(&fields.field("line_offsets")?)?),
            Arc::new(ring_offsets.into_array(&fields.field("ring_offsets")?)?),
            Arc::new(surface_offsets.into_array(&fields.field("surface_offsets")?)?),
            Arc::new(shell_offsets.into_array(&fields.field("shell_offsets")?)?),
            Arc::new(solid_offsets.into_array(&fields.field("solid_offsets")?)?),
        ],
    )
    .map_err(Error::from)
}

fn geometry_instances_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometryInstanceTableBuffer,
) -> Result<RecordBatch> {
    let GeometryInstanceTableBuffer {
        geometry_id,
        cityobject_ix,
        geometry_ordinal,
        lod,
        template_geometry_id,
        reference_point_vertex_id,
        transform_matrix,
    } = rows;
    let mut fields = SchemaFieldLookup::new(schema);
    let arrays: Vec<ArrayRef> = vec![
        Arc::new(UInt64Array::from(geometry_id)),
        Arc::new(UInt64Array::from(cityobject_ix)),
        Arc::new(UInt32Array::from(geometry_ordinal)),
        Arc::new(StringArray::from(lod)),
        Arc::new(UInt64Array::from(template_geometry_id)),
        Arc::new(UInt64Array::from(reference_point_vertex_id)),
        Arc::new(fixed_size_f64_array(
            &fields.field("transform_matrix")?,
            16,
            transform_matrix,
        )?),
    ];
    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn template_geometries_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: TemplateGeometryTableBuffer,
) -> Result<RecordBatch> {
    let TemplateGeometryTableBuffer {
        template_geometry_id,
        geometry_type,
        lod,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(template_geometry_id)),
            Arc::new(StringArray::from(
                geometry_type.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(lod)),
        ],
    )
    .map_err(Error::from)
}

fn template_geometry_boundaries_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: TemplateGeometryBoundaryTableBuffer,
) -> Result<RecordBatch> {
    let TemplateGeometryBoundaryTableBuffer {
        template_geometry_id,
        vertex_indices,
        line_offsets,
        ring_offsets,
        surface_offsets,
        shell_offsets,
        solid_offsets,
    } = rows;
    let mut fields = SchemaFieldLookup::new(schema);
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(template_geometry_id)),
            Arc::new(vertex_indices.into_array(&fields.field("vertex_indices")?)?),
            Arc::new(line_offsets.into_array(&fields.field("line_offsets")?)?),
            Arc::new(ring_offsets.into_array(&fields.field("ring_offsets")?)?),
            Arc::new(surface_offsets.into_array(&fields.field("surface_offsets")?)?),
            Arc::new(shell_offsets.into_array(&fields.field("shell_offsets")?)?),
            Arc::new(solid_offsets.into_array(&fields.field("solid_offsets")?)?),
        ],
    )
    .map_err(Error::from)
}

fn geometry_surface_semantics_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometrySurfaceSemanticTableBuffer,
) -> Result<RecordBatch> {
    let GeometrySurfaceSemanticTableBuffer {
        geometry_id,
        surface_ordinal,
        semantic_id,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(geometry_id)),
            Arc::new(UInt32Array::from(surface_ordinal)),
            Arc::new(UInt64Array::from(semantic_id)),
        ],
    )
    .map_err(Error::from)
}

fn geometry_point_semantics_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometryPointSemanticTableBuffer,
) -> Result<RecordBatch> {
    let GeometryPointSemanticTableBuffer {
        geometry_id,
        point_ordinal,
        semantic_id,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(geometry_id)),
            Arc::new(UInt32Array::from(point_ordinal)),
            Arc::new(UInt64Array::from(semantic_id)),
        ],
    )
    .map_err(Error::from)
}

fn geometry_linestring_semantics_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometryLinestringSemanticTableBuffer,
) -> Result<RecordBatch> {
    let GeometryLinestringSemanticTableBuffer {
        geometry_id,
        linestring_ordinal,
        semantic_id,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(geometry_id)),
            Arc::new(UInt32Array::from(linestring_ordinal)),
            Arc::new(UInt64Array::from(semantic_id)),
        ],
    )
    .map_err(Error::from)
}

fn template_geometry_semantics_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: TemplateGeometrySemanticTableBuffer,
) -> Result<RecordBatch> {
    let TemplateGeometrySemanticTableBuffer {
        template_geometry_id,
        primitive_type,
        primitive_ordinal,
        semantic_id,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(template_geometry_id)),
            Arc::new(StringArray::from(
                primitive_type.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(primitive_ordinal)),
            Arc::new(UInt64Array::from(semantic_id)),
        ],
    )
    .map_err(Error::from)
}

fn geometry_surface_materials_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometrySurfaceMaterialTableBuffer,
) -> Result<RecordBatch> {
    let GeometrySurfaceMaterialTableBuffer {
        geometry_id,
        surface_ordinal,
        theme,
        material_id,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(geometry_id)),
            Arc::new(UInt32Array::from(surface_ordinal)),
            Arc::new(StringArray::from(
                theme.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(material_id)),
        ],
    )
    .map_err(Error::from)
}

fn template_geometry_materials_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: TemplateGeometryMaterialTableBuffer,
) -> Result<RecordBatch> {
    let TemplateGeometryMaterialTableBuffer {
        template_geometry_id,
        primitive_type,
        primitive_ordinal,
        theme,
        material_id,
    } = rows;
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(template_geometry_id)),
            Arc::new(StringArray::from(
                primitive_type.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(primitive_ordinal)),
            Arc::new(StringArray::from(
                theme.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(material_id)),
        ],
    )
    .map_err(Error::from)
}

fn geometry_ring_textures_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: GeometryRingTextureTableBuffer,
) -> Result<RecordBatch> {
    let GeometryRingTextureTableBuffer {
        geometry_id,
        surface_ordinal,
        ring_ordinal,
        theme,
        texture_id,
        uv_indices,
    } = rows;
    let mut fields = SchemaFieldLookup::new(schema);
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(geometry_id)),
            Arc::new(UInt32Array::from(surface_ordinal)),
            Arc::new(UInt32Array::from(ring_ordinal)),
            Arc::new(StringArray::from(
                theme.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(texture_id)),
            Arc::new(uv_indices.into_array(&fields.field("uv_indices")?)?),
        ],
    )
    .map_err(Error::from)
}

fn template_geometry_ring_textures_batch(
    schema: &Arc<::arrow::datatypes::Schema>,
    rows: TemplateGeometryRingTextureTableBuffer,
) -> Result<RecordBatch> {
    let TemplateGeometryRingTextureTableBuffer {
        template_geometry_id,
        surface_ordinal,
        ring_ordinal,
        theme,
        texture_id,
        uv_indices,
    } = rows;
    let mut fields = SchemaFieldLookup::new(schema);
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(template_geometry_id)),
            Arc::new(UInt32Array::from(surface_ordinal)),
            Arc::new(UInt32Array::from(ring_ordinal)),
            Arc::new(StringArray::from(
                theme.into_iter().map(Some).collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(texture_id)),
            Arc::new(uv_indices.into_array(&fields.field("uv_indices")?)?),
        ],
    )
    .map_err(Error::from)
}
