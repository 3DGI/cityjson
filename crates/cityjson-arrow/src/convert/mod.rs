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
use arrow::array::{
    Array, ArrayRef, BooleanArray, FixedSizeListArray, Float64Array, Int64Array,
    LargeStringArray, ListArray, NullArray, RecordBatch, StringArray, StructArray, UInt32Array,
    UInt64Array,
};
use arrow::datatypes::{DataType, FieldRef};
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use cityjson::CityModelType;
use cityjson::v2_0::geometry::{MaterialThemesView, TextureThemesView};
use cityjson::v2_0::{
    AttributeValue, BBox, Boundary, CRS, CityModelIdentifier, CityObject, CityObjectIdentifier,
    CityObjectType, Contact, ContactRole, ContactType, Extension, Geometry, GeometryType,
    ImageType, LoD, MaterialMap, Metadata, OwnedAttributeValue, OwnedCityModel, OwnedMaterial,
    OwnedSemantic, OwnedTexture, RGB, RGBA, SemanticMap, SemanticType, StoredGeometryInstance,
    StoredGeometryParts, TextureMap, TextureType, ThemeName, UVCoordinate, VertexIndexVec,
    WrapMode,
};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{Read, Write as IoWrite};
use std::sync::Arc;

const DEFAULT_CITYMODEL_ID: &str = "citymodel";
const FIELD_MATERIAL_NAME: &str = "payload.name";
const FIELD_MATERIAL_AMBIENT_INTENSITY: &str = "payload.ambient_intensity";
const FIELD_MATERIAL_DIFFUSE_COLOR: &str = "payload.diffuse_color_json";
const FIELD_MATERIAL_EMISSIVE_COLOR: &str = "payload.emissive_color_json";
const FIELD_MATERIAL_SPECULAR_COLOR: &str = "payload.specular_color_json";
const FIELD_MATERIAL_SHININESS: &str = "payload.shininess";
const FIELD_MATERIAL_TRANSPARENCY: &str = "payload.transparency";
const FIELD_MATERIAL_IS_SMOOTH: &str = "payload.is_smooth";
const FIELD_TEXTURE_IMAGE_TYPE: &str = "payload.image_type";
const FIELD_TEXTURE_WRAP_MODE: &str = "payload.wrap_mode";
const FIELD_TEXTURE_TEXTURE_TYPE: &str = "payload.texture_type";
const FIELD_TEXTURE_BORDER_COLOR: &str = "payload.border_color_json";
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
struct MetadataRow {
    citymodel_id: String,
    cityjson_version: String,
    citymodel_kind: String,
    identifier: Option<String>,
    title: Option<String>,
    reference_system: Option<String>,
    geographical_extent: Option<[f64; 6]>,
    reference_date: Option<String>,
    default_material_theme: Option<String>,
    default_texture_theme: Option<String>,
    point_of_contact_json: Option<String>,
    root_extra: Option<cityjson::v2_0::OwnedAttributes>,
    metadata_extra: Option<cityjson::v2_0::OwnedAttributes>,
}

#[derive(Debug, Clone, Copy)]
struct TransformRow {
    scale: [f64; 3],
    translate: [f64; 3],
}

#[derive(Debug, Clone)]
struct ExtensionRow {
    extension_name: String,
    uri: String,
    version: Option<String>,
}

#[derive(Debug, Clone)]
struct VertexRow {
    vertex_id: u64,
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone)]
struct TemplateVertexRow {
    template_vertex_id: u64,
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone)]
struct CityObjectRow {
    cityobject_id: String,
    cityobject_ix: u64,
    object_type: String,
    geographical_extent: Option<[f64; 6]>,
    attributes: Option<cityjson::v2_0::OwnedAttributes>,
    extra: Option<cityjson::v2_0::OwnedAttributes>,
}

#[derive(Debug, Clone)]
struct CityObjectChildRow {
    parent_cityobject_ix: u64,
    child_ordinal: u32,
    child_cityobject_ix: u64,
}

#[derive(Debug, Clone)]
struct GeometryRow {
    geometry_id: u64,
    cityobject_ix: u64,
    geometry_ordinal: u32,
    geometry_type: String,
    lod: Option<String>,
}

#[derive(Debug, Clone)]
struct GeometryBoundaryRow {
    geometry_id: u64,
    vertex_indices: Vec<u64>,
    line_lengths: Option<Vec<u32>>,
    ring_lengths: Option<Vec<u32>>,
    surface_lengths: Option<Vec<u32>>,
    shell_lengths: Option<Vec<u32>>,
    solid_lengths: Option<Vec<u32>>,
}

#[derive(Debug, Clone)]
struct GeometryInstanceRow {
    geometry_id: u64,
    cityobject_ix: u64,
    geometry_ordinal: u32,
    lod: Option<String>,
    template_geometry_id: u64,
    reference_point_vertex_id: u64,
    transform_matrix: Option<[f64; 16]>,
}

#[derive(Debug, Clone)]
struct TemplateGeometryRow {
    template_geometry_id: u64,
    geometry_type: String,
    lod: Option<String>,
}

#[derive(Debug, Clone)]
struct TemplateGeometryBoundaryRow {
    template_geometry_id: u64,
    vertex_indices: Vec<u64>,
    line_lengths: Option<Vec<u32>>,
    ring_lengths: Option<Vec<u32>>,
    surface_lengths: Option<Vec<u32>>,
    shell_lengths: Option<Vec<u32>>,
    solid_lengths: Option<Vec<u32>>,
}

#[derive(Debug, Clone)]
struct SemanticRow {
    semantic_id: u64,
    semantic_type: String,
    attributes: Option<cityjson::v2_0::OwnedAttributes>,
}

#[derive(Debug, Clone)]
struct SemanticChildRow {
    parent_semantic_id: u64,
    child_ordinal: u32,
    child_semantic_id: u64,
}

#[derive(Debug, Clone)]
struct GeometrySurfaceSemanticRow {
    geometry_id: u64,
    surface_ordinal: u32,
    semantic_id: Option<u64>,
}

#[derive(Debug, Clone)]
struct GeometryPointSemanticRow {
    geometry_id: u64,
    point_ordinal: u32,
    semantic_id: Option<u64>,
}

#[derive(Debug, Clone)]
struct GeometryLinestringSemanticRow {
    geometry_id: u64,
    linestring_ordinal: u32,
    semantic_id: Option<u64>,
}

#[derive(Debug, Clone)]
struct TemplateGeometrySemanticRow {
    template_geometry_id: u64,
    primitive_type: String,
    primitive_ordinal: u32,
    semantic_id: Option<u64>,
}

#[derive(Debug, Clone)]
struct MaterialRow {
    material_id: u64,
    name: String,
    ambient_intensity: Option<f64>,
    diffuse_color: Option<String>,
    emissive_color: Option<String>,
    specular_color: Option<String>,
    shininess: Option<f64>,
    transparency: Option<f64>,
    is_smooth: Option<bool>,
}

#[derive(Debug, Clone)]
struct GeometrySurfaceMaterialRow {
    geometry_id: u64,
    surface_ordinal: u32,
    theme: String,
    material_id: u64,
}

#[derive(Debug, Clone)]
struct TemplateGeometryMaterialRow {
    template_geometry_id: u64,
    primitive_type: String,
    primitive_ordinal: u32,
    theme: String,
    material_id: u64,
}

#[derive(Debug, Clone)]
struct TextureRow {
    texture_id: u64,
    image_uri: String,
    image_type: String,
    wrap_mode: Option<String>,
    texture_type: Option<String>,
    border_color: Option<String>,
}

#[derive(Debug, Clone)]
struct TextureVertexRow {
    uv_id: u64,
    u: f64,
    v: f64,
}

#[derive(Debug, Clone)]
struct GeometryRingTextureRow {
    geometry_id: u64,
    surface_ordinal: u32,
    ring_ordinal: u32,
    theme: String,
    texture_id: u64,
    uv_indices: Vec<u64>,
}

#[derive(Debug, Clone)]
struct TemplateGeometryRingTextureRow {
    template_geometry_id: u64,
    surface_ordinal: u32,
    ring_ordinal: u32,
    theme: String,
    texture_id: u64,
    uv_indices: Vec<u64>,
}

struct ExportedGeometryRows {
    geometries: Vec<GeometryRow>,
    boundaries: Vec<GeometryBoundaryRow>,
    instances: Vec<GeometryInstanceRow>,
    surface_semantics: Vec<GeometrySurfaceSemanticRow>,
    point_semantics: Vec<GeometryPointSemanticRow>,
    linestring_semantics: Vec<GeometryLinestringSemanticRow>,
    surface_materials: Vec<GeometrySurfaceMaterialRow>,
    ring_textures: Vec<GeometryRingTextureRow>,
}

struct ExportedTemplateGeometryRows {
    geometries: Vec<TemplateGeometryRow>,
    boundaries: Vec<TemplateGeometryBoundaryRow>,
    semantics: Vec<TemplateGeometrySemanticRow>,
    materials: Vec<TemplateGeometryMaterialRow>,
    ring_textures: Vec<TemplateGeometryRingTextureRow>,
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
    geometry_id_map: HashMap<cityjson::prelude::GeometryHandle, u64>,
    template_geometry_id_map: HashMap<cityjson::prelude::GeometryTemplateHandle, u64>,
    semantic_id_map: HashMap<cityjson::prelude::SemanticHandle, u64>,
    material_id_map: HashMap<cityjson::prelude::MaterialHandle, u64>,
    texture_id_map: HashMap<cityjson::prelude::TextureHandle, u64>,
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
    geometry_id_map: &'a HashMap<cityjson::prelude::GeometryHandle, u64>,
    semantic_id_map: &'a HashMap<cityjson::prelude::SemanticHandle, u64>,
    material_id_map: &'a HashMap<cityjson::prelude::MaterialHandle, u64>,
    texture_id_map: &'a HashMap<cityjson::prelude::TextureHandle, u64>,
    template_geometry_id_map: &'a HashMap<cityjson::prelude::GeometryTemplateHandle, u64>,
}

struct TemplateGeometryExportContext<'a> {
    template_geometries: &'a HashMap<cityjson::prelude::GeometryTemplateHandle, u64>,
    semantics: &'a HashMap<cityjson::prelude::SemanticHandle, u64>,
    materials: &'a HashMap<cityjson::prelude::MaterialHandle, u64>,
    textures: &'a HashMap<cityjson::prelude::TextureHandle, u64>,
}

type GroupedRows<T> = HashMap<u64, Vec<T>>;

struct ImportState {
    model: OwnedCityModel,
    semantic_handle_by_id: HashMap<u64, cityjson::prelude::SemanticHandle>,
    material_handle_by_id: HashMap<u64, cityjson::prelude::MaterialHandle>,
    texture_handle_by_id: HashMap<u64, cityjson::prelude::TextureHandle>,
    template_handle_by_id: HashMap<u64, cityjson::prelude::GeometryTemplateHandle>,
    geometry_handle_by_id: HashMap<u64, cityjson::prelude::GeometryHandle>,
    cityobject_handle_by_ix: Vec<Option<cityjson::prelude::CityObjectHandle>>,
    pending_geometry_attachments: Vec<Vec<(u32, u64)>>,
}

#[derive(Default)]
struct PartRowGroups {
    boundaries: HashMap<u64, GeometryBoundaryRow>,
    template_boundaries: HashMap<u64, TemplateGeometryBoundaryRow>,
    surface_semantics: GroupedRows<GeometrySurfaceSemanticRow>,
    point_semantics: GroupedRows<GeometryPointSemanticRow>,
    linestring_semantics: GroupedRows<GeometryLinestringSemanticRow>,
    template_semantics: GroupedRows<TemplateGeometrySemanticRow>,
    surface_materials: GroupedRows<GeometrySurfaceMaterialRow>,
    template_materials: GroupedRows<TemplateGeometryMaterialRow>,
    ring_textures: GroupedRows<GeometryRingTextureRow>,
    template_ring_textures: GroupedRows<TemplateGeometryRingTextureRow>,
}

pub(crate) struct IncrementalDecoder {
    header: CityArrowHeader,
    projection: ProjectionLayout,
    schemas: CanonicalSchemaSet,
    state: Option<ImportState>,
    grouped_rows: PartRowGroups,
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
    u: &'a Float64Array,
    v: &'a Float64Array,
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
    diffuse_color: Option<&'a LargeStringArray>,
    emissive_color: Option<&'a LargeStringArray>,
    specular_color: Option<&'a LargeStringArray>,
    shininess: Option<&'a Float64Array>,
    transparency: Option<&'a Float64Array>,
    is_smooth: Option<&'a arrow::array::BooleanArray>,
}

struct TextureColumns<'a> {
    texture_id: &'a UInt64Array,
    image_uri: &'a LargeStringArray,
    image_type: &'a LargeStringArray,
    wrap_mode: Option<&'a LargeStringArray>,
    texture_type: Option<&'a LargeStringArray>,
    border_color: Option<&'a LargeStringArray>,
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

    let geometry_rows = geometry_rows(
        context.model,
        &context.geometry_id_map,
        &context.semantic_id_map,
        &context.material_id_map,
        &context.texture_id_map,
        &context.template_geometry_id_map,
    )?;
    let template_geometry_rows =
        template_geometry_rows(context.model, &context.template_geometry_id_map)?;
    let geometry = export_geometry_batches(&context, &geometry_rows, &template_geometry_rows)?;
    push_optional_batch(
        sink,
        CanonicalTable::TemplateVertices,
        geometry.template_vertices,
    )?;

    let semantics = export_semantic_batches(&context, &geometry_rows, &template_geometry_rows)?;
    let appearance = export_appearance_batches(&context, &geometry_rows, &template_geometry_rows)?;

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
            CityArrowPackageVersion::V3Alpha1,
            citymodel_id,
            model
                .version()
                .unwrap_or(cityjson::CityJSONVersion::V2_0)
                .to_string(),
        ),
        projection: projection.clone(),
        schemas: canonical_schema_set(&projection),
        geometry_id_map: geometry_id_map(model),
        template_geometry_id_map: template_geometry_id_map(model),
        semantic_id_map: semantic_id_map(model),
        material_id_map: material_id_map(model),
        texture_id_map: texture_id_map(model),
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
        metadata_row(context.model, &context.header)?,
        &context.projection,
        &context.geometry_id_map,
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
        extensions: optional_batch(extension_rows(context.model), |rows| {
            extensions_batch(&context.schemas.extensions, rows)
        })?,
        vertices: vertices_batch(&context.schemas.vertices, &vertex_rows(context.model))?,
        cityobjects: cityobjects_batch(
            &context.schemas.cityobjects,
            &cityobject_rows(context.model, &context.projection, &context.geometry_id_map)?,
            &context.projection,
            &context.geometry_id_map,
        )?,
        cityobject_children: optional_batch(cityobject_child_rows(context.model), |rows| {
            cityobject_children_batch(&context.schemas.cityobject_children, rows)
        })?,
    })
}

fn export_geometry_batches(
    context: &ExportContext<'_>,
    geometry_rows: &ExportedGeometryRows,
    template_geometry_rows: &ExportedTemplateGeometryRows,
) -> Result<ExportGeometryBatches> {
    Ok(ExportGeometryBatches {
        geometries: geometries_batch(&context.schemas.geometries, &geometry_rows.geometries)?,
        geometry_boundaries: geometry_boundaries_batch(
            &context.schemas.geometry_boundaries,
            &geometry_rows.boundaries,
        )?,
        geometry_instances: optional_batch_ref(&geometry_rows.instances, |rows| {
            geometry_instances_batch(&context.schemas.geometry_instances, rows)
        })?,
        template_vertices: optional_batch(template_vertex_rows(context.model), |rows| {
            template_vertices_batch(&context.schemas.template_vertices, &rows)
        })?,
        template_geometries: optional_batch_ref(&template_geometry_rows.geometries, |rows| {
            template_geometries_batch(&context.schemas.template_geometries, rows)
        })?,
        template_geometry_boundaries: optional_batch_ref(
            &template_geometry_rows.boundaries,
            |rows| {
                template_geometry_boundaries_batch(
                    &context.schemas.template_geometry_boundaries,
                    rows,
                )
            },
        )?,
    })
}

fn export_semantic_batches(
    context: &ExportContext<'_>,
    geometry_rows: &ExportedGeometryRows,
    template_geometry_rows: &ExportedTemplateGeometryRows,
) -> Result<ExportSemanticBatches> {
    let semantic_rows =
        semantic_rows(context.model, &context.projection, &context.geometry_id_map)?;

    Ok(ExportSemanticBatches {
        semantics: optional_batch(semantic_rows, |rows| {
            semantics_batch(
                &context.schemas.semantics,
                &rows,
                &context.projection,
                &context.geometry_id_map,
            )
        })?,
        semantic_children: optional_batch(
            semantic_child_rows(context.model, &context.semantic_id_map),
            |rows| semantic_children_batch(&context.schemas.semantic_children, rows),
        )?,
        geometry_surface_semantics: optional_batch_ref(&geometry_rows.surface_semantics, |rows| {
            geometry_surface_semantics_batch(&context.schemas.geometry_surface_semantics, rows)
        })?,
        geometry_point_semantics: optional_batch_ref(&geometry_rows.point_semantics, |rows| {
            geometry_point_semantics_batch(&context.schemas.geometry_point_semantics, rows)
        })?,
        geometry_linestring_semantics: optional_batch_ref(
            &geometry_rows.linestring_semantics,
            |rows| {
                geometry_linestring_semantics_batch(
                    &context.schemas.geometry_linestring_semantics,
                    rows,
                )
            },
        )?,
        template_geometry_semantics: optional_batch_ref(
            &template_geometry_rows.semantics,
            |rows| {
                template_geometry_semantics_batch(
                    &context.schemas.template_geometry_semantics,
                    rows,
                )
            },
        )?,
    })
}

fn export_appearance_batches(
    context: &ExportContext<'_>,
    geometry_rows: &ExportedGeometryRows,
    template_geometry_rows: &ExportedTemplateGeometryRows,
) -> Result<ExportAppearanceBatches> {
    Ok(ExportAppearanceBatches {
        materials: optional_batch(material_rows(context.model), |rows| {
            materials_batch(&context.schemas.materials, &rows, &context.projection)
        })?,
        geometry_surface_materials: optional_batch_ref(&geometry_rows.surface_materials, |rows| {
            geometry_surface_materials_batch(&context.schemas.geometry_surface_materials, rows)
        })?,
        template_geometry_materials: optional_batch_ref(
            &template_geometry_rows.materials,
            |rows| {
                template_geometry_materials_batch(
                    &context.schemas.template_geometry_materials,
                    rows,
                )
            },
        )?,
        textures: optional_batch(texture_rows(context.model), |rows| {
            textures_batch(&context.schemas.textures, &rows, &context.projection)
        })?,
        texture_vertices: optional_batch(texture_vertex_rows(context.model), |rows| {
            texture_vertices_batch(&context.schemas.texture_vertices, rows)
        })?,
        geometry_ring_textures: optional_batch_ref(&geometry_rows.ring_textures, |rows| {
            geometry_ring_textures_batch(&context.schemas.geometry_ring_textures, rows)
        })?,
        template_geometry_ring_textures: optional_batch_ref(
            &template_geometry_rows.ring_textures,
            |rows| {
                template_geometry_ring_textures_batch(
                    &context.schemas.template_geometry_ring_textures,
                    rows,
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
    for (table, batch) in collect_tables(parts) {
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
            grouped_rows: PartRowGroups::default(),
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
                let projection = self.projection.clone();
                let state = self.state_mut()?;
                let handles = import_semantics_batch(batch, &projection, &mut state.model)?;
                state.semantic_handle_by_id = handles;
            }
            CanonicalTable::SemanticChildren => {
                import_semantic_child_batch(batch, self.state_mut()?)?;
            }
            CanonicalTable::Materials => {
                let projection = self.projection.clone();
                let state = self.state_mut()?;
                let handles = import_materials_batch(batch, &projection, &mut state.model)?;
                state.material_handle_by_id = handles;
            }
            CanonicalTable::Textures => {
                let projection = self.projection.clone();
                let state = self.state_mut()?;
                let handles = import_textures_batch(batch, &projection, &mut state.model)?;
                state.texture_handle_by_id = handles;
            }
            CanonicalTable::TemplateGeometryBoundaries => {
                extend_unique_rows(
                    &mut self.grouped_rows.template_boundaries,
                    read_template_geometry_boundary_rows(batch)?,
                    |row| row.template_geometry_id,
                    "template geometry boundary",
                )?;
            }
            CanonicalTable::TemplateGeometrySemantics => {
                extend_grouped_rows(
                    &mut self.grouped_rows.template_semantics,
                    read_template_geometry_semantic_rows(batch)?,
                    |row| row.template_geometry_id,
                );
            }
            CanonicalTable::TemplateGeometryMaterials => {
                extend_grouped_rows(
                    &mut self.grouped_rows.template_materials,
                    read_template_geometry_material_rows(batch)?,
                    |row| row.template_geometry_id,
                );
            }
            CanonicalTable::TemplateGeometryRingTextures => {
                extend_grouped_rows(
                    &mut self.grouped_rows.template_ring_textures,
                    read_template_geometry_ring_texture_rows(batch)?,
                    |row| row.template_geometry_id,
                );
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
                extend_unique_rows(
                    &mut self.grouped_rows.boundaries,
                    read_geometry_boundary_rows(batch)?,
                    |row| row.geometry_id,
                    "geometry boundary",
                )?;
            }
            CanonicalTable::GeometrySurfaceSemantics => {
                extend_grouped_rows(
                    &mut self.grouped_rows.surface_semantics,
                    read_geometry_surface_semantic_rows(batch)?,
                    |row| row.geometry_id,
                );
            }
            CanonicalTable::GeometryPointSemantics => {
                extend_grouped_rows(
                    &mut self.grouped_rows.point_semantics,
                    read_geometry_point_semantic_rows(batch)?,
                    |row| row.geometry_id,
                );
            }
            CanonicalTable::GeometryLinestringSemantics => {
                extend_grouped_rows(
                    &mut self.grouped_rows.linestring_semantics,
                    read_geometry_linestring_semantic_rows(batch)?,
                    |row| row.geometry_id,
                );
            }
            CanonicalTable::GeometrySurfaceMaterials => {
                extend_grouped_rows(
                    &mut self.grouped_rows.surface_materials,
                    read_geometry_surface_material_rows(batch)?,
                    |row| row.geometry_id,
                );
            }
            CanonicalTable::GeometryRingTextures => {
                extend_grouped_rows(
                    &mut self.grouped_rows.ring_textures,
                    read_geometry_ring_texture_rows(batch)?,
                    |row| row.geometry_id,
                );
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
                let projection = self.projection.clone();
                let state = self.state_mut()?;
                import_cityobjects_batch(batch, &projection, state)?;
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

fn extend_unique_rows<T, FKey>(
    target: &mut HashMap<u64, T>,
    rows: Vec<T>,
    key: FKey,
    label: &str,
) -> Result<()>
where
    FKey: Fn(&T) -> u64,
{
    for row in rows {
        let row_id = key(&row);
        if target.insert(row_id, row).is_some() {
            return Err(Error::Conversion(format!("duplicate {label} row {row_id}")));
        }
    }
    Ok(())
}

fn extend_grouped_rows<T, FKey>(target: &mut GroupedRows<T>, rows: Vec<T>, key: FKey)
where
    FKey: Fn(&T) -> u64,
{
    for row in rows {
        target.entry(key(&row)).or_default().push(row);
    }
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
    apply_metadata_row(
        &mut model,
        &metadata_row,
        &empty_geometry_handles,
    )?;

    Ok(ImportState {
        model,
        semantic_handle_by_id: HashMap::new(),
        material_handle_by_id: HashMap::new(),
        texture_handle_by_id: HashMap::new(),
        template_handle_by_id: HashMap::new(),
        geometry_handle_by_id: HashMap::new(),
        cityobject_handle_by_ix: Vec::new(),
        pending_geometry_attachments: Vec::new(),
    })
}

fn import_semantics_batch(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
    model: &mut OwnedCityModel,
) -> Result<HashMap<u64, cityjson::prelude::SemanticHandle>> {
    let mut semantic_handle_by_id = HashMap::new();
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
            &HashMap::new(),
        )?;
        for (key, value) in projected.iter() {
            semantic
                .attributes_mut()
                .insert(key.clone(), value.clone());
        }
        semantic_handle_by_id.insert(semantic_id, model.add_semantic(semantic)?);
    }
    Ok(semantic_handle_by_id)
}

fn import_semantic_child_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    for row in read_semantic_child_rows(batch)? {
        let parent = *state
            .semantic_handle_by_id
            .get(&row.parent_semantic_id)
            .ok_or_else(|| {
                Error::Conversion(format!(
                    "missing semantic {} for child relation",
                    row.parent_semantic_id
                ))
            })?;
        let child = *state
            .semantic_handle_by_id
            .get(&row.child_semantic_id)
            .ok_or_else(|| {
                Error::Conversion(format!(
                    "missing semantic {} for child relation",
                    row.child_semantic_id
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
    for row in read_extension_rows(batch)? {
        state.model.extensions_mut().add(Extension::new(
            row.extension_name,
            row.uri,
            row.version.unwrap_or_default(),
        ));
    }
    Ok(())
}

fn import_vertex_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    for row in read_vertex_rows(batch)? {
        state
            .model
            .add_vertex(cityjson::v2_0::RealWorldCoordinate::new(
                row.x, row.y, row.z,
            ))?;
    }
    Ok(())
}

fn import_template_vertex_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
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
    let columns = bind_uv_columns(batch)?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let uv_id = columns.uv_id.value(row);
        ensure_strictly_increasing_u64(previous_id, uv_id, "uv_id")?;
        previous_id = Some(uv_id);
        state.model.add_uv_coordinate(UVCoordinate::new(
            f64_to_f32_preserving_cast(columns.u.value(row))?,
            f64_to_f32_preserving_cast(columns.v.value(row))?,
        ))?;
    }
    Ok(())
}

fn import_materials_batch(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
    model: &mut OwnedCityModel,
) -> Result<HashMap<u64, cityjson::prelude::MaterialHandle>> {
    let mut material_handle_by_id = HashMap::new();
    let columns = bind_material_columns(batch, projection)?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let material_id = columns.material_id.value(row);
        ensure_strictly_increasing_u64(previous_id, material_id, "material_id")?;
        previous_id = Some(material_id);
        let mut material = OwnedMaterial::new(columns.name.value(row).to_string());
        material.set_ambient_intensity(
            read_f64_array_optional(columns.ambient_intensity, row)
                .map(f64_to_f32_preserving_cast)
                .transpose()?,
        );
        material.set_diffuse_color(
            read_large_string_array_optional(columns.diffuse_color, row)
                .as_deref()
                .map(parse_rgb_json)
                .transpose()?,
        );
        material.set_emissive_color(
            read_large_string_array_optional(columns.emissive_color, row)
                .as_deref()
                .map(parse_rgb_json)
                .transpose()?,
        );
        material.set_specular_color(
            read_large_string_array_optional(columns.specular_color, row)
                .as_deref()
                .map(parse_rgb_json)
                .transpose()?,
        );
        material.set_shininess(
            read_f64_array_optional(columns.shininess, row)
                .map(f64_to_f32_preserving_cast)
                .transpose()?,
        );
        material.set_transparency(
            read_f64_array_optional(columns.transparency, row)
                .map(f64_to_f32_preserving_cast)
                .transpose()?,
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
    let mut texture_handle_by_id = HashMap::new();
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
            read_large_string_array_optional(columns.border_color, row)
                .as_deref()
                .map(parse_rgba_json)
                .transpose()?,
        );
        texture_handle_by_id.insert(texture_id, model.add_texture(texture)?);
    }
    Ok(texture_handle_by_id)
}

fn import_template_geometries_batch(
    batch: &RecordBatch,
    state: &mut ImportState,
    grouped_rows: &PartRowGroups,
) -> Result<()> {
    let columns = bind_template_geometry_columns(batch)?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let template_geometry_id = columns.template_geometry_id.value(row);
        ensure_strictly_increasing_u64(previous_id, template_geometry_id, "template_geometry_id")?;
        previous_id = Some(template_geometry_id);
        let boundary = grouped_rows
            .template_boundaries
            .get(&template_geometry_id)
            .ok_or_else(|| {
                Error::Conversion(format!(
                    "missing boundary row for template geometry {template_geometry_id}"
                ))
            })?;
        let geometry = Geometry::from_stored_parts(StoredGeometryParts {
            type_geometry: parse_geometry_type(columns.geometry_type.value(row))?,
            lod: (!columns.lod.is_null(row))
                .then(|| columns.lod.value(row))
                .map(parse_lod)
                .transpose()?,
            boundaries: Some(template_boundary_from_row(
                boundary,
                columns.geometry_type.value(row),
            )?),
            semantics: build_template_semantic_map(
                columns.geometry_type.value(row),
                boundary,
                grouped_rows.template_semantics.get(&template_geometry_id),
                &state.semantic_handle_by_id,
            )?,
            materials: build_template_material_maps(
                columns.geometry_type.value(row),
                boundary,
                grouped_rows.template_materials.get(&template_geometry_id),
                &state.material_handle_by_id,
            )?,
            textures: build_template_texture_maps(
                columns.geometry_type.value(row),
                boundary,
                grouped_rows
                    .template_ring_textures
                    .get(&template_geometry_id),
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
    grouped_rows: &PartRowGroups,
) -> Result<()> {
    let columns = bind_geometry_columns(batch)?;
    let mut previous_id = None;
    for row in 0..batch.num_rows() {
        let geometry_id = columns.geometry_id.value(row);
        ensure_strictly_increasing_u64(previous_id, geometry_id, "geometry_id")?;
        previous_id = Some(geometry_id);
        let boundary = grouped_rows.boundaries.get(&geometry_id).ok_or_else(|| {
            Error::Conversion(format!("missing boundary row for geometry {geometry_id}"))
        })?;
        let geometry = Geometry::from_stored_parts(StoredGeometryParts {
            type_geometry: parse_geometry_type(columns.geometry_type.value(row))?,
            lod: (!columns.lod.is_null(row))
                .then(|| columns.lod.value(row))
                .map(parse_lod)
                .transpose()?,
            boundaries: Some(boundary_from_row(
                boundary,
                columns.geometry_type.value(row),
            )?),
            semantics: build_semantic_map(
                columns.geometry_type.value(row),
                boundary,
                grouped_rows.surface_semantics.get(&geometry_id),
                grouped_rows.point_semantics.get(&geometry_id),
                grouped_rows.linestring_semantics.get(&geometry_id),
                &state.semantic_handle_by_id,
            )?,
            materials: build_material_maps(
                columns.geometry_type.value(row),
                boundary,
                grouped_rows.surface_materials.get(&geometry_id),
                &state.material_handle_by_id,
            )?,
            textures: build_texture_maps(
                columns.geometry_type.value(row),
                boundary,
                grouped_rows.ring_textures.get(&geometry_id),
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
    let columns = bind_geometry_instance_columns(batch)?;
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
    let columns = bind_cityobject_columns(batch, projection)?;
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
        for (key, value) in projected_attributes.iter() {
            object.attributes_mut().insert(key.clone(), value.clone());
        }
        let projected_extra = projected_attributes_from_array(
            projection.cityobject_extra.as_ref(),
            columns.extra,
            row,
            &state.geometry_handle_by_id,
        )?;
        for (key, value) in projected_extra.iter() {
            object.extra_mut().insert(key.clone(), value.clone());
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

fn import_cityobject_children_batch(batch: &RecordBatch, state: &mut ImportState) -> Result<()> {
    for row in read_cityobject_child_rows(batch)? {
        let parent = cityobject_handle(state, row.parent_cityobject_ix)?;
        let child = cityobject_handle(state, row.child_cityobject_ix)?;
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

fn f64_to_f32_preserving_cast(value: f64) -> Result<f32> {
    value.to_string().parse::<f32>().map_err(|error| {
        Error::Conversion(format!(
            "failed to narrow f64 value {value} to f32: {error}"
        ))
    })
}

fn geometry_id_map(model: &OwnedCityModel) -> HashMap<cityjson::prelude::GeometryHandle, u64> {
    model
        .iter_geometries()
        .enumerate()
        .map(|(index, (handle, _))| (handle, index as u64))
        .collect()
}

fn template_geometry_id_map(
    model: &OwnedCityModel,
) -> HashMap<cityjson::prelude::GeometryTemplateHandle, u64> {
    model
        .iter_geometry_templates()
        .enumerate()
        .map(|(index, (handle, _))| (handle, index as u64))
        .collect()
}

fn semantic_id_map(model: &OwnedCityModel) -> HashMap<cityjson::prelude::SemanticHandle, u64> {
    model
        .iter_semantics()
        .enumerate()
        .map(|(index, (handle, _))| (handle, index as u64))
        .collect()
}

fn material_id_map(model: &OwnedCityModel) -> HashMap<cityjson::prelude::MaterialHandle, u64> {
    model
        .iter_materials()
        .enumerate()
        .map(|(index, (handle, _))| (handle, index as u64))
        .collect()
}

fn texture_id_map(model: &OwnedCityModel) -> HashMap<cityjson::prelude::TextureHandle, u64> {
    model
        .iter_textures()
        .enumerate()
        .map(|(index, (handle, _))| (handle, index as u64))
        .collect()
}

fn discover_projection_layout(model: &OwnedCityModel) -> Result<ProjectionLayout> {
    Ok(ProjectionLayout {
        root_extra: discover_optional_attribute_projection(model.extra())?,
        metadata_extra: discover_optional_attribute_projection(
            model.metadata().and_then(Metadata::extra),
        )?,
        cityobject_attributes: discover_attribute_projection(
            model
                .cityobjects()
                .iter()
                .filter_map(|(_, object)| object.attributes()),
        )?,
        cityobject_extra: discover_attribute_projection(
            model
                .cityobjects()
                .iter()
                .filter_map(|(_, object)| object.extra()),
        )?,
        geometry_extra: None,
        semantic_attributes: discover_attribute_projection(
            model
                .iter_semantics()
                .filter_map(|(_, semantic)| semantic.attributes()),
        )?,
        material_payload: (model.material_count() > 0).then(canonical_material_projection),
        texture_payload: (model.texture_count() > 0).then(canonical_texture_projection),
    })
}

fn canonical_material_projection() -> ProjectedStructSpec {
    ProjectedStructSpec::new(vec![
        ProjectedFieldSpec::new(FIELD_MATERIAL_NAME, ProjectedValueSpec::Utf8, false),
        ProjectedFieldSpec::new(
            FIELD_MATERIAL_AMBIENT_INTENSITY,
            ProjectedValueSpec::Float64,
            true,
        ),
        ProjectedFieldSpec::new(FIELD_MATERIAL_DIFFUSE_COLOR, ProjectedValueSpec::Utf8, true),
        ProjectedFieldSpec::new(FIELD_MATERIAL_EMISSIVE_COLOR, ProjectedValueSpec::Utf8, true),
        ProjectedFieldSpec::new(FIELD_MATERIAL_SPECULAR_COLOR, ProjectedValueSpec::Utf8, true),
        ProjectedFieldSpec::new(FIELD_MATERIAL_SHININESS, ProjectedValueSpec::Float64, true),
        ProjectedFieldSpec::new(FIELD_MATERIAL_TRANSPARENCY, ProjectedValueSpec::Float64, true),
        ProjectedFieldSpec::new(FIELD_MATERIAL_IS_SMOOTH, ProjectedValueSpec::Boolean, true),
    ])
}

fn canonical_texture_projection() -> ProjectedStructSpec {
    ProjectedStructSpec::new(vec![
        ProjectedFieldSpec::new(FIELD_TEXTURE_IMAGE_TYPE, ProjectedValueSpec::Utf8, false),
        ProjectedFieldSpec::new(FIELD_TEXTURE_WRAP_MODE, ProjectedValueSpec::Utf8, true),
        ProjectedFieldSpec::new(FIELD_TEXTURE_TEXTURE_TYPE, ProjectedValueSpec::Utf8, true),
        ProjectedFieldSpec::new(FIELD_TEXTURE_BORDER_COLOR, ProjectedValueSpec::Utf8, true),
    ])
}

fn validate_appearance_projection_layout(layout: &ProjectionLayout) -> Result<()> {
    let supported_material = canonical_material_projection()
        .fields
        .into_iter()
        .map(|spec| spec.name)
        .collect::<BTreeSet<_>>();
    if let Some(specs) = &layout.material_payload {
        for spec in &specs.fields {
            if !supported_material.contains(&spec.name) {
                return Err(Error::Unsupported(format!(
                    "material payload column {}",
                    spec.name
                )));
            }
        }
    }

    let supported_texture = canonical_texture_projection()
        .fields
        .into_iter()
        .map(|spec| spec.name)
        .collect::<BTreeSet<_>>();
    if let Some(specs) = &layout.texture_payload {
        for spec in &specs.fields {
            if !supported_texture.contains(&spec.name) {
                return Err(Error::Unsupported(format!(
                    "texture payload column {}",
                    spec.name
                )));
            }
        }
    }
    Ok(())
}

fn discover_optional_attribute_projection(
    attributes: Option<&cityjson::v2_0::OwnedAttributes>,
) -> Result<Option<ProjectedStructSpec>> {
    match attributes {
        Some(attributes) => discover_attribute_projection(std::iter::once(attributes)),
        None => Ok(None),
    }
}

fn discover_attribute_projection<'a, I>(attributes: I) -> Result<Option<ProjectedStructSpec>>
where
    I: IntoIterator<Item = &'a cityjson::v2_0::OwnedAttributes>,
{
    let mut layout = ProjectedStructSpec::new(Vec::new());
    let mut seen_rows = 0_usize;

    for attrs in attributes {
        merge_attribute_map_into_spec(&mut layout, attrs, seen_rows)?;
        seen_rows += 1;
    }

    if seen_rows == 0 || layout.is_empty() {
        Ok(None)
    } else {
        sort_projected_struct(&mut layout);
        Ok(Some(layout))
    }
}

fn merge_attribute_map_into_spec(
    spec: &mut ProjectedStructSpec,
    attributes: &cityjson::v2_0::OwnedAttributes,
    seen_rows: usize,
) -> Result<()> {
    let present = attributes.keys().cloned().collect::<BTreeSet<_>>();
    for field in &mut spec.fields {
        if !present.contains(&field.name) {
            field.nullable = true;
        }
    }

    for (key, value) in attributes.iter() {
        if let Some(field) = spec.fields.iter_mut().find(|field| field.name == *key) {
            merge_projected_field(field, value)?;
        } else {
            spec.fields.push(ProjectedFieldSpec::new(
                key.clone(),
                infer_projected_value_spec(value)?,
                seen_rows > 0 || matches!(value, AttributeValue::Null),
            ));
        }
    }

    sort_projected_struct(spec);
    Ok(())
}

fn merge_projected_field(field: &mut ProjectedFieldSpec, value: &OwnedAttributeValue) -> Result<()> {
    if matches!(value, AttributeValue::Null) {
        field.nullable = true;
        return Ok(());
    }

    let inferred = infer_projected_value_spec(value)?;
    field.value = merge_projected_value_specs(field.value.clone(), inferred)?;
    Ok(())
}

fn infer_projected_value_spec(value: &OwnedAttributeValue) -> Result<ProjectedValueSpec> {
    Ok(match value {
        AttributeValue::Null => ProjectedValueSpec::Null,
        AttributeValue::Bool(_) => ProjectedValueSpec::Boolean,
        AttributeValue::Unsigned(_) => ProjectedValueSpec::UInt64,
        AttributeValue::Integer(_) => ProjectedValueSpec::Int64,
        AttributeValue::Float(_) => ProjectedValueSpec::Float64,
        AttributeValue::String(_) => ProjectedValueSpec::Utf8,
        AttributeValue::Geometry(_) => ProjectedValueSpec::GeometryRef,
        AttributeValue::Vec(values) => {
            let mut item_nullable = false;
            let mut item_spec = ProjectedValueSpec::Null;
            let mut has_non_null = false;
            for item in values {
                if matches!(item, AttributeValue::Null) {
                    item_nullable = true;
                    continue;
                }
                let inferred = infer_projected_value_spec(item)?;
                item_spec = if has_non_null {
                    merge_projected_value_specs(item_spec, inferred)?
                } else {
                    inferred
                };
                has_non_null = true;
            }
            ProjectedValueSpec::List {
                item_nullable,
                item: Box::new(item_spec),
            }
        }
        AttributeValue::Map(values) => {
            let mut fields = ProjectedStructSpec::new(Vec::new());
            let mut attributes = cityjson::v2_0::OwnedAttributes::default();
            for (key, value) in values {
                attributes.insert(key.clone(), value.clone());
            }
            merge_attribute_map_into_spec(&mut fields, &attributes, 0)?;
            ProjectedValueSpec::Struct(fields)
        }
        other => {
            return Err(Error::Unsupported(format!(
                "unsupported attribute value variant {other}"
            )));
        }
    })
}

fn merge_projected_value_specs(
    current: ProjectedValueSpec,
    incoming: ProjectedValueSpec,
) -> Result<ProjectedValueSpec> {
    Ok(match (current, incoming) {
        (ProjectedValueSpec::Null, other) | (other, ProjectedValueSpec::Null) => other,
        (ProjectedValueSpec::Boolean, ProjectedValueSpec::Boolean) => ProjectedValueSpec::Boolean,
        (ProjectedValueSpec::UInt64, ProjectedValueSpec::UInt64) => ProjectedValueSpec::UInt64,
        (ProjectedValueSpec::Int64, ProjectedValueSpec::Int64) => ProjectedValueSpec::Int64,
        (ProjectedValueSpec::Float64, ProjectedValueSpec::Float64) => ProjectedValueSpec::Float64,
        (ProjectedValueSpec::Utf8, ProjectedValueSpec::Utf8) => ProjectedValueSpec::Utf8,
        (ProjectedValueSpec::GeometryRef, ProjectedValueSpec::GeometryRef) => {
            ProjectedValueSpec::GeometryRef
        }
        (
            ProjectedValueSpec::List {
                item_nullable: left_nullable,
                item: left_item,
            },
            ProjectedValueSpec::List {
                item_nullable: right_nullable,
                item: right_item,
            },
        ) => ProjectedValueSpec::List {
            item_nullable: left_nullable || right_nullable,
            item: Box::new(merge_projected_value_specs(*left_item, *right_item)?),
        },
        (ProjectedValueSpec::Struct(left), ProjectedValueSpec::Struct(right)) => {
            ProjectedValueSpec::Struct(merge_projected_struct_specs(left, right)?)
        }
        (left, right) => {
            return Err(Error::Conversion(format!(
                "incompatible projected attribute shapes: {:?} versus {:?}",
                left, right
            )));
        }
    })
}

fn merge_projected_struct_specs(
    mut left: ProjectedStructSpec,
    right: ProjectedStructSpec,
) -> Result<ProjectedStructSpec> {
    let right_names = right
        .fields
        .iter()
        .map(|field| field.name.clone())
        .collect::<BTreeSet<_>>();
    for field in &mut left.fields {
        if !right_names.contains(&field.name) {
            field.nullable = true;
        }
    }

    for incoming in right.fields {
        if let Some(existing) = left.fields.iter_mut().find(|field| field.name == incoming.name) {
            existing.nullable |= incoming.nullable;
            existing.value = merge_projected_value_specs(existing.value.clone(), incoming.value)?;
        } else {
            let mut incoming = incoming;
            incoming.nullable = true;
            left.fields.push(incoming);
        }
    }

    sort_projected_struct(&mut left);
    Ok(left)
}

fn sort_projected_struct(spec: &mut ProjectedStructSpec) {
    spec.fields.sort_by(|left, right| left.name.cmp(&right.name));
    for field in &mut spec.fields {
        if let ProjectedValueSpec::Struct(child) = &mut field.value {
            sort_projected_struct(child);
        } else if let ProjectedValueSpec::List { item, .. } = &mut field.value
            && let ProjectedValueSpec::Struct(child) = item.as_mut()
        {
            sort_projected_struct(child);
        }
    }
}

fn metadata_row(
    model: &OwnedCityModel,
    header: &CityArrowHeader,
) -> Result<MetadataRow> {
    let metadata = model.metadata();
    Ok(MetadataRow {
        citymodel_id: header.citymodel_id.clone(),
        cityjson_version: header.cityjson_version.clone(),
        citymodel_kind: model.type_citymodel().to_string(),
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
        point_of_contact_json: metadata
            .and_then(Metadata::point_of_contact)
            .map(contact_to_json)
            .transpose()?,
        root_extra: cloned_attributes(model.extra()),
        metadata_extra: metadata.and_then(Metadata::extra).cloned(),
    })
}

fn extension_rows(model: &OwnedCityModel) -> Vec<ExtensionRow> {
    model
        .extensions()
        .into_iter()
        .flat_map(|extensions| extensions.iter())
        .map(|extension| ExtensionRow {
            extension_name: extension.name().clone(),
            uri: extension.url().clone(),
            version: Some(extension.version().clone()),
        })
        .collect()
}

fn material_rows(model: &OwnedCityModel) -> Vec<MaterialRow> {
    model
        .iter_materials()
        .enumerate()
        .map(|(index, (_, material))| MaterialRow {
            material_id: index as u64,
            name: material.name().clone(),
            ambient_intensity: material.ambient_intensity().map(f64::from),
            diffuse_color: material.diffuse_color().map(rgb_to_json),
            emissive_color: material.emissive_color().map(rgb_to_json),
            specular_color: material.specular_color().map(rgb_to_json),
            shininess: material.shininess().map(f64::from),
            transparency: material.transparency().map(f64::from),
            is_smooth: material.is_smooth(),
        })
        .collect()
}

fn texture_rows(model: &OwnedCityModel) -> Vec<TextureRow> {
    model
        .iter_textures()
        .enumerate()
        .map(|(index, (_, texture))| TextureRow {
            texture_id: index as u64,
            image_uri: texture.image().clone(),
            image_type: texture.image_type().to_string(),
            wrap_mode: texture.wrap_mode().map(|value| value.to_string()),
            texture_type: texture.texture_type().map(|value| value.to_string()),
            border_color: texture.border_color().map(rgba_to_json),
        })
        .collect()
}

fn texture_vertex_rows(model: &OwnedCityModel) -> Vec<TextureVertexRow> {
    model
        .vertices_texture()
        .as_slice()
        .iter()
        .enumerate()
        .map(|(index, coordinate)| TextureVertexRow {
            uv_id: index as u64,
            u: f64::from(coordinate.u()),
            v: f64::from(coordinate.v()),
        })
        .collect()
}

fn vertex_rows(model: &OwnedCityModel) -> Vec<VertexRow> {
    model
        .vertices()
        .as_slice()
        .iter()
        .enumerate()
        .map(|(index, coordinate)| VertexRow {
            vertex_id: index as u64,
            x: coordinate.x(),
            y: coordinate.y(),
            z: coordinate.z(),
        })
        .collect()
}

fn cityobject_rows(
    model: &OwnedCityModel,
    _projection: &ProjectionLayout,
    _geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<Vec<CityObjectRow>> {
    model
        .cityobjects()
        .iter()
        .enumerate()
        .map(|(index, (_, object))| {
            Ok(CityObjectRow {
                cityobject_id: object.id().to_string(),
                cityobject_ix: index as u64,
                object_type: object.type_cityobject().to_string(),
                geographical_extent: object
                    .geographical_extent()
                    .map(|bbox| bbox.as_slice().try_into().expect("bbox is 6 long")),
                attributes: cloned_attributes(object.attributes()),
                extra: cloned_attributes(object.extra()),
            })
        })
        .collect()
}

fn cityobject_child_rows(model: &OwnedCityModel) -> Vec<CityObjectChildRow> {
    let cityobject_ix_map = cityobject_ix_map(model);
    let mut rows = Vec::new();
    for (parent_handle, object) in model.cityobjects().iter() {
        let parent_cityobject_ix = cityobject_ix_map
            .get(&parent_handle)
            .copied()
            .unwrap_or_default();
        if let Some(children) = object.children() {
            for (ordinal, child) in children.iter().enumerate() {
                if let Some(child_cityobject_ix) = cityobject_ix_map.get(child).copied() {
                    rows.push(CityObjectChildRow {
                        parent_cityobject_ix,
                        child_ordinal: usize_to_u32(ordinal, "child ordinal")
                            .expect("child ordinal fits into u32"),
                        child_cityobject_ix,
                    });
                }
            }
        }
    }
    rows
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

fn geometry_rows(
    model: &OwnedCityModel,
    geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
    semantic_id_map: &HashMap<cityjson::prelude::SemanticHandle, u64>,
    material_id_map: &HashMap<cityjson::prelude::MaterialHandle, u64>,
    texture_id_map: &HashMap<cityjson::prelude::TextureHandle, u64>,
    template_geometry_id_map: &HashMap<cityjson::prelude::GeometryTemplateHandle, u64>,
) -> Result<ExportedGeometryRows> {
    let mut exported = ExportedGeometryRows {
        geometries: Vec::new(),
        boundaries: Vec::new(),
        instances: Vec::new(),
        surface_semantics: Vec::new(),
        point_semantics: Vec::new(),
        linestring_semantics: Vec::new(),
        surface_materials: Vec::new(),
        ring_textures: Vec::new(),
    };
    let context = GeometryExportContext {
        model,
        geometry_id_map,
        semantic_id_map,
        material_id_map,
        texture_id_map,
        template_geometry_id_map,
    };

    for (cityobject_ix, (_, object)) in model.cityobjects().iter().enumerate() {
        if let Some(geometries) = object.geometry() {
            for (ordinal, geometry_handle) in geometries.iter().enumerate() {
                append_geometry_rows(
                    &context,
                    u64::try_from(cityobject_ix).expect("cityobject index fits into u64"),
                    *geometry_handle,
                    ordinal,
                    &mut exported,
                )?;
            }
        }
    }

    Ok(exported)
}

fn append_geometry_rows(
    context: &GeometryExportContext<'_>,
    cityobject_ix: u64,
    geometry_handle: cityjson::prelude::GeometryHandle,
    ordinal: usize,
    exported: &mut ExportedGeometryRows,
) -> Result<()> {
    let geometry_id = *context
        .geometry_id_map
        .get(&geometry_handle)
        .ok_or_else(|| Error::Conversion("geometry handle missing from id map".to_string()))?;
    let geometry = context.model.get_geometry(geometry_handle).ok_or_else(|| {
        Error::Conversion(format!("missing geometry for handle {geometry_handle:?}"))
    })?;
    if *geometry.type_geometry() == GeometryType::GeometryInstance {
        return append_geometry_instance_row(
            context,
            cityobject_ix,
            geometry_handle,
            geometry_id,
            geometry,
            ordinal,
            exported,
        );
    }
    append_boundary_geometry_rows(
        context,
        cityobject_ix,
        geometry_id,
        geometry,
        ordinal,
        exported,
    )
}

fn append_geometry_instance_row(
    context: &GeometryExportContext<'_>,
    cityobject_ix: u64,
    geometry_handle: cityjson::prelude::GeometryHandle,
    geometry_id: u64,
    geometry: &Geometry<u32, cityjson::prelude::OwnedStringStorage>,
    ordinal: usize,
    exported: &mut ExportedGeometryRows,
) -> Result<()> {
    let instance = geometry.instance().ok_or_else(|| {
        Error::Conversion("geometry instance missing instance payload".to_string())
    })?;
    let template_geometry_id = *context
        .template_geometry_id_map
        .get(&instance.template())
        .ok_or_else(|| {
            Error::Conversion(format!(
                "missing template id for instance geometry {geometry_handle:?}"
            ))
        })?;
    exported.instances.push(GeometryInstanceRow {
        geometry_id,
        cityobject_ix,
        geometry_ordinal: usize_to_u32(ordinal, "geometry ordinal")?,
        lod: geometry.lod().map(ToString::to_string),
        template_geometry_id,
        reference_point_vertex_id: u64::from(instance.reference_point().value()),
        transform_matrix: Some(instance.transformation().into()),
    });
    Ok(())
}

fn append_boundary_geometry_rows(
    context: &GeometryExportContext<'_>,
    cityobject_ix: u64,
    geometry_id: u64,
    geometry: &Geometry<u32, cityjson::prelude::OwnedStringStorage>,
    ordinal: usize,
    exported: &mut ExportedGeometryRows,
) -> Result<()> {
    let boundary = geometry.boundaries().ok_or_else(|| {
        Error::Conversion("boundary-carrying geometry missing boundaries".to_string())
    })?;
    let boundary_row = geometry_boundary_row(geometry_id, *geometry.type_geometry(), boundary);
    append_geometry_semantic_rows(
        geometry_id,
        geometry,
        &boundary_row,
        context.semantic_id_map,
        exported,
    )?;
    exported.surface_materials.extend(geometry_material_rows(
        geometry_id,
        *geometry.type_geometry(),
        &boundary_row,
        geometry.materials(),
        context.material_id_map,
    )?);
    exported.ring_textures.extend(geometry_ring_texture_rows(
        geometry_id,
        *geometry.type_geometry(),
        &boundary_row,
        geometry.textures(),
        context.texture_id_map,
    )?);
    exported.geometries.push(GeometryRow {
        geometry_id,
        cityobject_ix,
        geometry_ordinal: usize_to_u32(ordinal, "geometry ordinal")?,
        geometry_type: geometry.type_geometry().to_string(),
        lod: geometry.lod().map(ToString::to_string),
    });
    exported.boundaries.push(boundary_row);
    Ok(())
}

fn append_geometry_semantic_rows(
    geometry_id: u64,
    geometry: &Geometry<u32, cityjson::prelude::OwnedStringStorage>,
    boundary_row: &GeometryBoundaryRow,
    semantic_id_map: &HashMap<cityjson::prelude::SemanticHandle, u64>,
    exported: &mut ExportedGeometryRows,
) -> Result<()> {
    let Some(semantics) = geometry.semantics() else {
        return Ok(());
    };
    match geometry.type_geometry() {
        GeometryType::MultiPoint => {
            if semantics.points().len() != boundary_row.vertex_indices.len() {
                return Err(Error::Conversion(format!(
                    "point semantic row count {} does not match point count {}",
                    semantics.points().len(),
                    boundary_row.vertex_indices.len()
                )));
            }
            for (point_ordinal, semantic_id) in semantics.points().iter().enumerate() {
                exported.point_semantics.push(GeometryPointSemanticRow {
                    geometry_id,
                    point_ordinal: usize_to_u32(point_ordinal, "point ordinal")?,
                    semantic_id: semantic_id
                        .and_then(|handle| semantic_id_map.get(&handle).copied()),
                });
            }
        }
        GeometryType::MultiLineString => {
            let linestring_count =
                required_lengths(boundary_row.line_lengths.as_ref(), "line_lengths")?.len();
            if semantics.linestrings().len() != linestring_count {
                return Err(Error::Conversion(format!(
                    "linestring semantic row count {} does not match linestring count {}",
                    semantics.linestrings().len(),
                    linestring_count
                )));
            }
            for (linestring_ordinal, semantic_id) in semantics.linestrings().iter().enumerate() {
                exported
                    .linestring_semantics
                    .push(GeometryLinestringSemanticRow {
                        geometry_id,
                        linestring_ordinal: usize_to_u32(linestring_ordinal, "linestring ordinal")?,
                        semantic_id: semantic_id
                            .and_then(|handle| semantic_id_map.get(&handle).copied()),
                    });
            }
        }
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => {
            for (surface_ordinal, semantic_id) in semantics.surfaces().iter().enumerate() {
                exported.surface_semantics.push(GeometrySurfaceSemanticRow {
                    geometry_id,
                    surface_ordinal: usize_to_u32(surface_ordinal, "surface ordinal")?,
                    semantic_id: semantic_id
                        .and_then(|handle| semantic_id_map.get(&handle).copied()),
                });
            }
        }
        GeometryType::GeometryInstance => {
            return Err(Error::Unsupported("geometry instances".to_string()));
        }
        _ => return Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct RingLayout {
    start: usize,
    len: usize,
    surface_ordinal: u32,
    ring_ordinal: u32,
}

fn geometry_material_rows(
    geometry_id: u64,
    geometry_type: GeometryType,
    boundary_row: &GeometryBoundaryRow,
    materials: Option<MaterialThemesView<'_, u32, cityjson::prelude::OwnedStringStorage>>,
    material_id_map: &HashMap<cityjson::prelude::MaterialHandle, u64>,
) -> Result<Vec<GeometrySurfaceMaterialRow>> {
    let Some(materials) = materials else {
        return Ok(Vec::new());
    };
    let mut surface_rows = Vec::new();

    for (theme, map) in materials.iter() {
        match geometry_type {
            GeometryType::MultiSurface
            | GeometryType::CompositeSurface
            | GeometryType::Solid
            | GeometryType::MultiSolid
            | GeometryType::CompositeSolid => {
                let surface_count = surface_count(boundary_row);
                if map.surfaces().len() != surface_count {
                    return Err(Error::Conversion(format!(
                        "material theme {} has {} surface assignments, expected {}",
                        theme,
                        map.surfaces().len(),
                        surface_count
                    )));
                }
                for (surface_ordinal, material_handle) in map.surfaces().iter().enumerate() {
                    let Some(material_handle) = material_handle else {
                        continue;
                    };
                    let material_id = *material_id_map.get(material_handle).ok_or_else(|| {
                        Error::Conversion("material handle missing from id map".to_string())
                    })?;
                    surface_rows.push(GeometrySurfaceMaterialRow {
                        geometry_id,
                        surface_ordinal: usize_to_u32(surface_ordinal, "surface ordinal")?,
                        theme: theme.as_ref().to_string(),
                        material_id,
                    });
                }
            }
            GeometryType::GeometryInstance
            | GeometryType::MultiPoint
            | GeometryType::MultiLineString => {
                return Err(Error::Unsupported("geometry materials".to_string()));
            }
            _ => return Err(Error::Unsupported("unsupported geometry type".to_string())),
        }
    }

    Ok(surface_rows)
}

fn geometry_ring_texture_rows(
    geometry_id: u64,
    geometry_type: GeometryType,
    boundary_row: &GeometryBoundaryRow,
    textures: Option<TextureThemesView<'_, u32, cityjson::prelude::OwnedStringStorage>>,
    texture_id_map: &HashMap<cityjson::prelude::TextureHandle, u64>,
) -> Result<Vec<GeometryRingTextureRow>> {
    let Some(textures) = textures else {
        return Ok(Vec::new());
    };
    ensure_surface_backed_geometry(geometry_type, "geometry textures")?;
    let ring_layouts = ring_layouts(boundary_row)?;
    let mut rows = Vec::new();

    for (theme, map) in textures.iter() {
        if map.rings().len() != ring_layouts.len() {
            return Err(Error::Conversion(format!(
                "texture theme {} has {} rings, expected {}",
                theme,
                map.rings().len(),
                ring_layouts.len()
            )));
        }
        if map.vertices().len() != boundary_row.vertex_indices.len() {
            return Err(Error::Conversion(format!(
                "texture theme {} has {} uv assignments, expected {}",
                theme,
                map.vertices().len(),
                boundary_row.vertex_indices.len()
            )));
        }
        let ring_textures = map.ring_textures();
        for (ring_index, layout) in ring_layouts.iter().enumerate() {
            let Some(texture_handle) = ring_textures[ring_index] else {
                continue;
            };
            let texture_id = *texture_id_map.get(&texture_handle).ok_or_else(|| {
                Error::Conversion("texture handle missing from id map".to_string())
            })?;
            let uv_indices = map.vertices()[layout.start..layout.start + layout.len]
                .iter()
                .map(|value: &Option<cityjson::v2_0::VertexIndex<u32>>| {
                    value
                        .map(|uv: cityjson::v2_0::VertexIndex<u32>| u64::from(uv.value()))
                        .ok_or_else(|| {
                            Error::Conversion(format!(
                                "textured ring {ring_index} for theme {theme} contains missing uv indices"
                            ))
                        })
                })
                .collect::<Result<Vec<_>>>()?;
            rows.push(GeometryRingTextureRow {
                geometry_id,
                surface_ordinal: layout.surface_ordinal,
                ring_ordinal: layout.ring_ordinal,
                theme: theme.as_ref().to_string(),
                texture_id,
                uv_indices,
            });
        }
    }

    Ok(rows)
}

fn template_geometry_ring_texture_rows(
    template_geometry_id: u64,
    geometry_type: GeometryType,
    boundary_row: &TemplateGeometryBoundaryRow,
    textures: &TextureThemesView<'_, u32, cityjson::prelude::OwnedStringStorage>,
    texture_id_map: &HashMap<cityjson::prelude::TextureHandle, u64>,
) -> Result<Vec<TemplateGeometryRingTextureRow>> {
    ensure_surface_backed_geometry(geometry_type, "template geometry textures")?;
    let ring_layouts = template_ring_layouts(boundary_row)?;
    let mut rows = Vec::new();

    for (theme, map) in textures.iter() {
        if map.rings().len() != ring_layouts.len() {
            return Err(Error::Conversion(format!(
                "template geometry texture theme {} has {} rings, expected {}",
                theme,
                map.rings().len(),
                ring_layouts.len()
            )));
        }
        if map.vertices().len() != boundary_row.vertex_indices.len() {
            return Err(Error::Conversion(format!(
                "template geometry texture theme {} has {} uv assignments, expected {}",
                theme,
                map.vertices().len(),
                boundary_row.vertex_indices.len()
            )));
        }
        let ring_textures = map.ring_textures();
        for (ring_index, layout) in ring_layouts.iter().enumerate() {
            let Some(texture_handle) = ring_textures[ring_index] else {
                continue;
            };
            let texture_id = *texture_id_map.get(&texture_handle).ok_or_else(|| {
                Error::Conversion("texture handle missing from id map".to_string())
            })?;
            let uv_indices = map.vertices()[layout.start..layout.start + layout.len]
                .iter()
                .map(|value: &Option<cityjson::v2_0::VertexIndex<u32>>| {
                    value
                        .map(|uv: cityjson::v2_0::VertexIndex<u32>| u64::from(uv.value()))
                        .ok_or_else(|| {
                            Error::Conversion(format!(
                                "template textured ring {ring_index} for theme {theme} contains missing uv indices"
                            ))
                        })
                })
                .collect::<Result<Vec<_>>>()?;
            rows.push(TemplateGeometryRingTextureRow {
                template_geometry_id,
                surface_ordinal: layout.surface_ordinal,
                ring_ordinal: layout.ring_ordinal,
                theme: theme.as_ref().to_string(),
                texture_id,
                uv_indices,
            });
        }
    }

    Ok(rows)
}

fn ensure_surface_backed_geometry(geometry_type: GeometryType, feature: &str) -> Result<()> {
    match geometry_type {
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => Ok(()),
        GeometryType::MultiPoint | GeometryType::MultiLineString => {
            Err(Error::Unsupported(feature.to_string()))
        }
        GeometryType::GeometryInstance => Err(Error::Unsupported(feature.to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

fn ring_layouts(boundary_row: &GeometryBoundaryRow) -> Result<Vec<RingLayout>> {
    let ring_lengths = required_lengths(boundary_row.ring_lengths.as_ref(), "ring_lengths")?;
    let surface_lengths =
        required_lengths(boundary_row.surface_lengths.as_ref(), "surface_lengths")?;
    let mut layouts = Vec::with_capacity(ring_lengths.len());
    let mut vertex_start = 0_usize;
    let mut ring_index = 0_usize;

    for (surface_ordinal, ring_count) in surface_lengths.iter().enumerate() {
        for ring_ordinal in 0..*ring_count {
            let len = *ring_lengths.get(ring_index).ok_or_else(|| {
                Error::Conversion(format!(
                    "surface topology references missing ring {ring_index}"
                ))
            })? as usize;
            layouts.push(RingLayout {
                start: vertex_start,
                len,
                surface_ordinal: usize_to_u32(surface_ordinal, "surface ordinal")?,
                ring_ordinal,
            });
            vertex_start += len;
            ring_index += 1;
        }
    }

    if ring_index != ring_lengths.len() {
        return Err(Error::Conversion(format!(
            "ring topology consumed {} rings, but {} ring lengths are present",
            ring_index,
            ring_lengths.len()
        )));
    }
    if vertex_start != boundary_row.vertex_indices.len() {
        return Err(Error::Conversion(format!(
            "ring topology consumed {} vertices, but {} boundary vertices are present",
            vertex_start,
            boundary_row.vertex_indices.len()
        )));
    }

    Ok(layouts)
}

fn template_ring_layouts(boundary_row: &TemplateGeometryBoundaryRow) -> Result<Vec<RingLayout>> {
    let ring_lengths = required_lengths(boundary_row.ring_lengths.as_ref(), "ring_lengths")?;
    let surface_lengths =
        required_lengths(boundary_row.surface_lengths.as_ref(), "surface_lengths")?;
    let mut layouts = Vec::with_capacity(ring_lengths.len());
    let mut vertex_start = 0_usize;
    let mut ring_index = 0_usize;

    for (surface_ordinal, ring_count) in surface_lengths.iter().enumerate() {
        for ring_ordinal in 0..*ring_count {
            let len = *ring_lengths.get(ring_index).ok_or_else(|| {
                Error::Conversion(format!(
                    "surface topology references missing ring {ring_index}"
                ))
            })? as usize;
            layouts.push(RingLayout {
                start: vertex_start,
                len,
                surface_ordinal: usize_to_u32(surface_ordinal, "surface ordinal")?,
                ring_ordinal,
            });
            vertex_start += len;
            ring_index += 1;
        }
    }

    if ring_index != ring_lengths.len() {
        return Err(Error::Conversion(format!(
            "ring topology consumed {} rings, but {} ring lengths are present",
            ring_index,
            ring_lengths.len()
        )));
    }
    if vertex_start != boundary_row.vertex_indices.len() {
        return Err(Error::Conversion(format!(
            "ring topology consumed {} vertices, but {} boundary vertices are present",
            vertex_start,
            boundary_row.vertex_indices.len()
        )));
    }

    Ok(layouts)
}

fn rgb_to_json(value: RGB) -> String {
    JsonValue::Array(
        value
            .to_array()
            .into_iter()
            .map(|component| {
                JsonValue::Number(
                    JsonNumber::from_f64(f64::from(component)).expect("finite rgb component"),
                )
            })
            .collect(),
    )
    .to_string()
}

fn rgba_to_json(value: RGBA) -> String {
    JsonValue::Array(
        value
            .to_array()
            .into_iter()
            .map(|component| {
                JsonValue::Number(
                    JsonNumber::from_f64(f64::from(component)).expect("finite rgba component"),
                )
            })
            .collect(),
    )
    .to_string()
}

#[derive(Debug, Clone)]
struct FlattenedBoundary {
    vertex_indices: Vec<u64>,
    line_lengths: Option<Vec<u32>>,
    ring_lengths: Option<Vec<u32>>,
    surface_lengths: Option<Vec<u32>>,
    shell_lengths: Option<Vec<u32>>,
    solid_lengths: Option<Vec<u32>>,
}

fn geometry_boundary_row(
    geometry_id: u64,
    geometry_type: GeometryType,
    boundary: &Boundary<u32>,
) -> GeometryBoundaryRow {
    let payload = flatten_boundary(geometry_type, boundary);
    GeometryBoundaryRow {
        geometry_id,
        vertex_indices: payload.vertex_indices,
        line_lengths: payload.line_lengths,
        ring_lengths: payload.ring_lengths,
        surface_lengths: payload.surface_lengths,
        shell_lengths: payload.shell_lengths,
        solid_lengths: payload.solid_lengths,
    }
}

fn template_geometry_rows(
    model: &OwnedCityModel,
    template_geometry_id_map: &HashMap<cityjson::prelude::GeometryTemplateHandle, u64>,
) -> Result<ExportedTemplateGeometryRows> {
    let semantic_id_map = semantic_id_map(model);
    let material_id_map = material_id_map(model);
    let texture_id_map = texture_id_map(model);
    let context = TemplateGeometryExportContext {
        template_geometries: template_geometry_id_map,
        semantics: &semantic_id_map,
        materials: &material_id_map,
        textures: &texture_id_map,
    };
    let mut exported = ExportedTemplateGeometryRows {
        geometries: Vec::new(),
        boundaries: Vec::new(),
        semantics: Vec::new(),
        materials: Vec::new(),
        ring_textures: Vec::new(),
    };
    for (handle, geometry) in model.iter_geometry_templates() {
        append_template_geometry_rows(&context, handle, geometry, &mut exported)?;
    }
    Ok(exported)
}

fn append_template_geometry_rows(
    context: &TemplateGeometryExportContext<'_>,
    handle: cityjson::prelude::GeometryTemplateHandle,
    geometry: &Geometry<u32, cityjson::prelude::OwnedStringStorage>,
    exported: &mut ExportedTemplateGeometryRows,
) -> Result<()> {
    let template_geometry_id = *context.template_geometries.get(&handle).ok_or_else(|| {
        Error::Conversion("template geometry handle missing from id map".to_string())
    })?;
    let boundary = geometry
        .boundaries()
        .ok_or_else(|| Error::Conversion("template geometry missing boundaries".to_string()))?;
    let boundary_row =
        template_geometry_boundary_row(template_geometry_id, *geometry.type_geometry(), boundary);
    exported.geometries.push(TemplateGeometryRow {
        template_geometry_id,
        geometry_type: geometry.type_geometry().to_string(),
        lod: geometry.lod().map(ToString::to_string),
    });
    append_template_semantic_rows(
        template_geometry_id,
        geometry,
        &boundary_row,
        context.semantics,
        exported,
    )?;
    append_template_material_rows(
        template_geometry_id,
        geometry,
        &boundary_row,
        context.materials,
        exported,
    )?;
    if let Some(textures) = geometry.textures() {
        exported
            .ring_textures
            .extend(template_geometry_ring_texture_rows(
                template_geometry_id,
                *geometry.type_geometry(),
                &boundary_row,
                &textures,
                context.textures,
            )?);
    }
    exported.boundaries.push(boundary_row);
    Ok(())
}

fn append_template_semantic_rows(
    template_geometry_id: u64,
    geometry: &Geometry<u32, cityjson::prelude::OwnedStringStorage>,
    boundary_row: &TemplateGeometryBoundaryRow,
    semantic_id_map: &HashMap<cityjson::prelude::SemanticHandle, u64>,
    exported: &mut ExportedTemplateGeometryRows,
) -> Result<()> {
    let Some(semantics) = geometry.semantics() else {
        return Ok(());
    };
    match geometry.type_geometry() {
        GeometryType::MultiPoint => {
            if semantics.points().len() != boundary_row.vertex_indices.len() {
                return Err(Error::Conversion(format!(
                    "template geometry {} has {} point semantics, expected {}",
                    template_geometry_id,
                    semantics.points().len(),
                    boundary_row.vertex_indices.len()
                )));
            }
            for (primitive_ordinal, semantic_id) in semantics.points().iter().enumerate() {
                exported.semantics.push(TemplateGeometrySemanticRow {
                    template_geometry_id,
                    primitive_type: PRIMITIVE_TYPE_POINT.to_string(),
                    primitive_ordinal: usize_to_u32(primitive_ordinal, "primitive ordinal")?,
                    semantic_id: semantic_id
                        .and_then(|handle| semantic_id_map.get(&handle).copied()),
                });
            }
        }
        GeometryType::MultiLineString => {
            let linestring_count =
                required_lengths(boundary_row.line_lengths.as_ref(), "line_lengths")?.len();
            if semantics.linestrings().len() != linestring_count {
                return Err(Error::Conversion(format!(
                    "template geometry {} has {} linestring semantics, expected {}",
                    template_geometry_id,
                    semantics.linestrings().len(),
                    linestring_count
                )));
            }
            for (primitive_ordinal, semantic_id) in semantics.linestrings().iter().enumerate() {
                exported.semantics.push(TemplateGeometrySemanticRow {
                    template_geometry_id,
                    primitive_type: PRIMITIVE_TYPE_LINESTRING.to_string(),
                    primitive_ordinal: usize_to_u32(primitive_ordinal, "primitive ordinal")?,
                    semantic_id: semantic_id
                        .and_then(|handle| semantic_id_map.get(&handle).copied()),
                });
            }
        }
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => {
            for (primitive_ordinal, semantic_id) in semantics.surfaces().iter().enumerate() {
                exported.semantics.push(TemplateGeometrySemanticRow {
                    template_geometry_id,
                    primitive_type: PRIMITIVE_TYPE_SURFACE.to_string(),
                    primitive_ordinal: usize_to_u32(primitive_ordinal, "primitive ordinal")?,
                    semantic_id: semantic_id
                        .and_then(|handle| semantic_id_map.get(&handle).copied()),
                });
            }
        }
        GeometryType::GeometryInstance => {
            return Err(Error::Unsupported("geometry instances".to_string()));
        }
        _ => return Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
    Ok(())
}

fn append_template_material_rows(
    template_geometry_id: u64,
    geometry: &Geometry<u32, cityjson::prelude::OwnedStringStorage>,
    boundary_row: &TemplateGeometryBoundaryRow,
    material_id_map: &HashMap<cityjson::prelude::MaterialHandle, u64>,
    exported: &mut ExportedTemplateGeometryRows,
) -> Result<()> {
    let Some(materials) = geometry.materials() else {
        return Ok(());
    };
    for (theme, map) in materials.iter() {
        match geometry.type_geometry() {
            GeometryType::MultiPoint => {
                if map.points().len() != boundary_row.vertex_indices.len() {
                    return Err(Error::Conversion(format!(
                        "template geometry {} material theme {} has {} point assignments, expected {}",
                        template_geometry_id,
                        theme,
                        map.points().len(),
                        boundary_row.vertex_indices.len()
                    )));
                }
                for (primitive_ordinal, material_handle) in map.points().iter().enumerate() {
                    let Some(material_handle) = material_handle else {
                        continue;
                    };
                    exported.materials.push(TemplateGeometryMaterialRow {
                        template_geometry_id,
                        primitive_type: PRIMITIVE_TYPE_POINT.to_string(),
                        primitive_ordinal: usize_to_u32(primitive_ordinal, "primitive ordinal")?,
                        theme: theme.as_ref().to_string(),
                        material_id: *material_id_map.get(material_handle).ok_or_else(|| {
                            Error::Conversion("material handle missing from id map".to_string())
                        })?,
                    });
                }
            }
            GeometryType::MultiLineString => {
                let linestring_count =
                    required_lengths(boundary_row.line_lengths.as_ref(), "line_lengths")?.len();
                if map.linestrings().len() != linestring_count {
                    return Err(Error::Conversion(format!(
                        "template geometry {} material theme {} has {} linestring assignments, expected {}",
                        template_geometry_id,
                        theme,
                        map.linestrings().len(),
                        linestring_count
                    )));
                }
                for (primitive_ordinal, material_handle) in map.linestrings().iter().enumerate() {
                    let Some(material_handle) = material_handle else {
                        continue;
                    };
                    exported.materials.push(TemplateGeometryMaterialRow {
                        template_geometry_id,
                        primitive_type: PRIMITIVE_TYPE_LINESTRING.to_string(),
                        primitive_ordinal: usize_to_u32(primitive_ordinal, "primitive ordinal")?,
                        theme: theme.as_ref().to_string(),
                        material_id: *material_id_map.get(material_handle).ok_or_else(|| {
                            Error::Conversion("material handle missing from id map".to_string())
                        })?,
                    });
                }
            }
            GeometryType::MultiSurface
            | GeometryType::CompositeSurface
            | GeometryType::Solid
            | GeometryType::MultiSolid
            | GeometryType::CompositeSolid => {
                let surface_count = template_surface_count(boundary_row);
                if map.surfaces().len() != surface_count {
                    return Err(Error::Conversion(format!(
                        "template geometry {} material theme {} has {} surface assignments, expected {}",
                        template_geometry_id,
                        theme,
                        map.surfaces().len(),
                        surface_count
                    )));
                }
                for (primitive_ordinal, material_handle) in map.surfaces().iter().enumerate() {
                    let Some(material_handle) = material_handle else {
                        continue;
                    };
                    exported.materials.push(TemplateGeometryMaterialRow {
                        template_geometry_id,
                        primitive_type: PRIMITIVE_TYPE_SURFACE.to_string(),
                        primitive_ordinal: usize_to_u32(primitive_ordinal, "primitive ordinal")?,
                        theme: theme.as_ref().to_string(),
                        material_id: *material_id_map.get(material_handle).ok_or_else(|| {
                            Error::Conversion("material handle missing from id map".to_string())
                        })?,
                    });
                }
            }
            GeometryType::GeometryInstance => {
                return Err(Error::Unsupported("geometry materials".to_string()));
            }
            _ => return Err(Error::Unsupported("unsupported geometry type".to_string())),
        }
    }
    Ok(())
}

fn template_vertex_rows(model: &OwnedCityModel) -> Vec<TemplateVertexRow> {
    model
        .template_vertices()
        .as_slice()
        .iter()
        .enumerate()
        .map(|(index, coordinate)| TemplateVertexRow {
            template_vertex_id: index as u64,
            x: coordinate.x(),
            y: coordinate.y(),
            z: coordinate.z(),
        })
        .collect()
}

fn template_geometry_boundary_row(
    template_geometry_id: u64,
    geometry_type: GeometryType,
    boundary: &Boundary<u32>,
) -> TemplateGeometryBoundaryRow {
    let payload = flatten_boundary(geometry_type, boundary);
    TemplateGeometryBoundaryRow {
        template_geometry_id,
        vertex_indices: payload.vertex_indices,
        line_lengths: payload.line_lengths,
        ring_lengths: payload.ring_lengths,
        surface_lengths: payload.surface_lengths,
        shell_lengths: payload.shell_lengths,
        solid_lengths: payload.solid_lengths,
    }
}

fn flatten_boundary(geometry_type: GeometryType, boundary: &Boundary<u32>) -> FlattenedBoundary {
    let vertices = boundary
        .vertices_raw()
        .iter()
        .copied()
        .map(u64::from)
        .collect();
    let ring_lengths = offsets_to_lengths(&boundary.rings_raw(), boundary.vertices_raw().len());
    let surface_lengths = offsets_to_lengths(&boundary.surfaces_raw(), boundary.rings_raw().len());
    let shell_lengths = offsets_to_lengths(&boundary.shells_raw(), boundary.surfaces_raw().len());
    let solid_lengths = offsets_to_lengths(&boundary.solids_raw(), boundary.shells_raw().len());

    let (line_lengths, ring_lengths, surface_lengths, shell_lengths, solid_lengths) =
        match geometry_type {
            GeometryType::MultiPoint => (None, None, None, None, None),
            GeometryType::MultiLineString => (Some(ring_lengths), None, None, None, None),
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                (None, Some(ring_lengths), Some(surface_lengths), None, None)
            }
            GeometryType::Solid => (
                None,
                Some(ring_lengths),
                Some(surface_lengths),
                Some(shell_lengths),
                None,
            ),
            GeometryType::MultiSolid | GeometryType::CompositeSolid => (
                None,
                Some(ring_lengths),
                Some(surface_lengths),
                Some(shell_lengths),
                Some(solid_lengths),
            ),
            GeometryType::GeometryInstance => unreachable!("instances rejected earlier"),
            _ => unreachable!("unsupported geometry type rejected earlier"),
        };

    FlattenedBoundary {
        vertex_indices: vertices,
        line_lengths,
        ring_lengths,
        surface_lengths,
        shell_lengths,
        solid_lengths,
    }
}

fn semantic_rows(
    model: &OwnedCityModel,
    _projection: &ProjectionLayout,
    _geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<Vec<SemanticRow>> {
    model
        .iter_semantics()
        .enumerate()
        .map(|(index, (_, semantic))| {
            Ok(SemanticRow {
                semantic_id: index as u64,
                semantic_type: encode_semantic_type(semantic.type_semantic()),
                attributes: cloned_attributes(semantic.attributes()),
            })
        })
        .collect()
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

fn semantic_child_rows(
    model: &OwnedCityModel,
    semantic_id_map: &HashMap<cityjson::prelude::SemanticHandle, u64>,
) -> Vec<SemanticChildRow> {
    let mut rows = Vec::new();
    for (handle, semantic) in model.iter_semantics() {
        if let Some(children) = semantic.children() {
            let parent_id = semantic_id_map.get(&handle).copied().unwrap_or_default();
            for (ordinal, child) in children.iter().enumerate() {
                if let Some(child_id) = semantic_id_map.get(child).copied() {
                    rows.push(SemanticChildRow {
                        parent_semantic_id: parent_id,
                        child_ordinal: usize_to_u32(ordinal, "child ordinal")
                            .expect("child ordinal fits into u32"),
                        child_semantic_id: child_id,
                    });
                }
            }
        }
    }
    rows
}

fn offsets_to_lengths(raw: &cityjson::v2_0::RawVertexView<'_, u32>, child_len: usize) -> Vec<u32> {
    let raw = &**raw;
    if raw.is_empty() {
        return Vec::new();
    }
    let mut lengths = Vec::with_capacity(raw.len());
    for window in raw.windows(2) {
        lengths.push(window[1] - window[0]);
    }
    lengths.push(
        usize_to_u32(child_len, "child length").expect("child length fits into u32")
            - raw[raw.len() - 1],
    );
    lengths
}

fn cloned_attributes(
    attributes: Option<&cityjson::v2_0::OwnedAttributes>,
) -> Option<cityjson::v2_0::OwnedAttributes> {
    attributes.cloned().filter(|attributes| !attributes.is_empty())
}

fn attribute_to_json(
    value: &OwnedAttributeValue,
    geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<JsonValue> {
    Ok(match value {
        AttributeValue::Null => JsonValue::Null,
        AttributeValue::Bool(value) => JsonValue::Bool(*value),
        AttributeValue::Unsigned(value) => JsonValue::Number(JsonNumber::from(*value)),
        AttributeValue::Integer(value) => JsonValue::Number(JsonNumber::from(*value)),
        AttributeValue::Float(value) => JsonNumber::from_f64(*value)
            .map(JsonValue::Number)
            .ok_or_else(|| Error::Conversion(format!("cannot encode float attribute {value}")))?,
        AttributeValue::String(value) => JsonValue::String(value.clone()),
        AttributeValue::Vec(values) => JsonValue::Array(
            values
                .iter()
                .map(|value| attribute_to_json(value, geometry_id_map))
                .collect::<Result<Vec<_>>>()?,
        ),
        AttributeValue::Map(values) => JsonValue::Object(
            values
                .iter()
                .map(|(key, value)| Ok((key.clone(), attribute_to_json(value, geometry_id_map)?)))
                .collect::<Result<JsonMap<_, _>>>()?,
        ),
        AttributeValue::Geometry(handle) => {
            let geometry_id = geometry_id_map.get(handle).copied().ok_or_else(|| {
                Error::Conversion("attribute geometry handle missing from map".to_string())
            })?;
            let mut object = JsonMap::new();
            object.insert(
                "__cityarrow_geometry_id".to_string(),
                JsonValue::Number(JsonNumber::from(geometry_id)),
            );
            JsonValue::Object(object)
        }
        _ => {
            return Err(Error::Unsupported(
                "unsupported attribute value variant".to_string(),
            ));
        }
    })
}

fn json_to_attribute(
    value: &JsonValue,
    geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<OwnedAttributeValue> {
    Ok(match value {
        JsonValue::Null => AttributeValue::Null,
        JsonValue::Bool(value) => AttributeValue::Bool(*value),
        JsonValue::Number(value) => {
            if let Some(unsigned) = value.as_u64() {
                AttributeValue::Unsigned(unsigned)
            } else if let Some(integer) = value.as_i64() {
                AttributeValue::Integer(integer)
            } else {
                AttributeValue::Float(
                    value.as_f64().ok_or_else(|| {
                        Error::Conversion("failed to decode json number".to_string())
                    })?,
                )
            }
        }
        JsonValue::String(value) => AttributeValue::String(value.clone()),
        JsonValue::Array(values) => AttributeValue::Vec(
            values
                .iter()
                .map(|value| json_to_attribute(value, geometry_handles))
                .collect::<Result<Vec<_>>>()?,
        ),
        JsonValue::Object(values) => {
            if values.len() == 1 && values.contains_key("__cityarrow_geometry_id") {
                let geometry_id = values["__cityarrow_geometry_id"].as_u64().ok_or_else(|| {
                    Error::Conversion("invalid geometry id attribute payload".to_string())
                })?;
                let handle = geometry_handles.get(&geometry_id).copied().ok_or_else(|| {
                    Error::Conversion(format!(
                        "missing geometry handle {geometry_id} for attribute reconstruction"
                    ))
                })?;
                AttributeValue::Geometry(handle)
            } else {
                AttributeValue::Map(
                    values
                        .iter()
                        .map(|(key, value)| {
                            Ok((key.clone(), json_to_attribute(value, geometry_handles)?))
                        })
                        .collect::<Result<HashMap<_, _>>>()?,
                )
            }
        }
    })
}

fn metadata_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    row: MetadataRow,
    projection: &ProjectionLayout,
    geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<RecordBatch> {
    let MetadataRow {
        citymodel_id,
        cityjson_version,
        citymodel_kind,
        identifier,
        title,
        reference_system,
        geographical_extent,
        reference_date,
        default_material_theme,
        default_texture_theme,
        point_of_contact_json,
        root_extra,
        metadata_extra,
    } = row;
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(LargeStringArray::from(vec![Some(citymodel_id)])),
        Arc::new(StringArray::from(vec![Some(cityjson_version)])),
        Arc::new(StringArray::from(vec![Some(citymodel_kind)])),
        Arc::new(LargeStringArray::from(vec![identifier])),
        Arc::new(LargeStringArray::from(vec![title])),
        Arc::new(LargeStringArray::from(vec![reference_system])),
        Arc::new(fixed_size_f64_array(
            &field_from_schema(schema, "geographical_extent")?,
            6,
            vec![geographical_extent],
        )?),
        Arc::new(StringArray::from(vec![reference_date])),
        Arc::new(StringArray::from(vec![default_material_theme])),
        Arc::new(StringArray::from(vec![default_texture_theme])),
        Arc::new(LargeStringArray::from(vec![point_of_contact_json])),
    ];
    if let Some(spec) = projection.root_extra.as_ref() {
        arrays.push(projected_struct_array_from_attributes(
            &field_from_schema(schema, "root_extra")?,
            spec,
            &[root_extra.as_ref()],
            geometry_id_map,
        )?);
    }
    if let Some(spec) = projection.metadata_extra.as_ref() {
        arrays.push(projected_struct_array_from_attributes(
            &field_from_schema(schema, "metadata_extra")?,
            spec,
            &[metadata_extra.as_ref()],
            geometry_id_map,
        )?);
    }
    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn transform_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    row: TransformRow,
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(fixed_size_f64_array(
                &field_from_schema(schema, "scale")?,
                3,
                vec![Some(row.scale)],
            )?),
            Arc::new(fixed_size_f64_array(
                &field_from_schema(schema, "translate")?,
                3,
                vec![Some(row.translate)],
            )?),
        ],
    )
    .map_err(Error::from)
}

fn extensions_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: Vec<ExtensionRow>,
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.extension_name.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.uri.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.into_iter().map(|row| row.version).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn vertices_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[VertexRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.vertex_id).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.x).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.y).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.z).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn cityobjects_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[CityObjectRow],
    projection: &ProjectionLayout,
    geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<RecordBatch> {
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| Some(row.cityobject_id.clone()))
                .collect::<Vec<_>>(),
        )),
        Arc::new(UInt64Array::from(
            rows.iter().map(|row| row.cityobject_ix).collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            rows.iter()
                .map(|row| Some(row.object_type.clone()))
                .collect::<Vec<_>>(),
        )),
        Arc::new(fixed_size_f64_array(
            &field_from_schema(schema, "geographical_extent")?,
            6,
            rows.iter().map(|row| row.geographical_extent).collect(),
        )?),
    ];

    if let Some(spec) = projection.cityobject_attributes.as_ref() {
        arrays.push(projected_struct_array_from_attributes(
            &field_from_schema(schema, "attributes")?,
            spec,
            &rows.iter().map(|row| row.attributes.as_ref()).collect::<Vec<_>>(),
            geometry_id_map,
        )?);
    }
    if let Some(spec) = projection.cityobject_extra.as_ref() {
        arrays.push(projected_struct_array_from_attributes(
            &field_from_schema(schema, "extra")?,
            spec,
            &rows.iter().map(|row| row.extra.as_ref()).collect::<Vec<_>>(),
            geometry_id_map,
        )?);
    }

    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn cityobject_children_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: Vec<CityObjectChildRow>,
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.parent_cityobject_ix)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter().map(|row| row.child_ordinal).collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.into_iter()
                    .map(|row| row.child_cityobject_ix)
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn geometries_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[GeometryRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.geometry_id).collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.cityobject_ix).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter()
                    .map(|row| row.geometry_ordinal)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.geometry_type.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter().map(|row| row.lod.clone()).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn geometry_boundaries_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[GeometryBoundaryRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.geometry_id).collect::<Vec<_>>(),
            )),
            Arc::new(list_u64_array(
                &field_from_schema(schema, "vertex_indices")?,
                rows.iter()
                    .map(|row| Some(row.vertex_indices.clone()))
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                &field_from_schema(schema, "line_lengths")?,
                rows.iter()
                    .map(|row| row.line_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                &field_from_schema(schema, "ring_lengths")?,
                rows.iter()
                    .map(|row| row.ring_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                &field_from_schema(schema, "surface_lengths")?,
                rows.iter()
                    .map(|row| row.surface_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                &field_from_schema(schema, "shell_lengths")?,
                rows.iter()
                    .map(|row| row.shell_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                &field_from_schema(schema, "solid_lengths")?,
                rows.iter()
                    .map(|row| row.solid_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
        ],
    )
    .map_err(Error::from)
}

fn geometry_instances_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[GeometryInstanceRow],
) -> Result<RecordBatch> {
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(UInt64Array::from(
            rows.iter().map(|row| row.geometry_id).collect::<Vec<_>>(),
        )),
        Arc::new(UInt64Array::from(
            rows.iter().map(|row| row.cityobject_ix).collect::<Vec<_>>(),
        )),
        Arc::new(UInt32Array::from(
            rows.iter()
                .map(|row| row.geometry_ordinal)
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            rows.iter().map(|row| row.lod.clone()).collect::<Vec<_>>(),
        )),
        Arc::new(UInt64Array::from(
            rows.iter()
                .map(|row| row.template_geometry_id)
                .collect::<Vec<_>>(),
        )),
        Arc::new(UInt64Array::from(
            rows.iter()
                .map(|row| row.reference_point_vertex_id)
                .collect::<Vec<_>>(),
        )),
        Arc::new(fixed_size_f64_array(
            &field_from_schema(schema, "transform_matrix")?,
            16,
            rows.iter().map(|row| row.transform_matrix).collect(),
        )?),
    ];
    RecordBatch::try_new(schema.clone(), arrays.split_off(0)).map_err(Error::from)
}

fn template_vertices_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[TemplateVertexRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.template_vertex_id)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.x).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.y).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.z).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn template_geometries_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[TemplateGeometryRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.template_geometry_id)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.geometry_type.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter().map(|row| row.lod.clone()).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn template_geometry_boundaries_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[TemplateGeometryBoundaryRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.template_geometry_id)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(list_u64_array(
                &field_from_schema(schema, "vertex_indices")?,
                rows.iter()
                    .map(|row| Some(row.vertex_indices.clone()))
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                &field_from_schema(schema, "line_lengths")?,
                rows.iter()
                    .map(|row| row.line_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                &field_from_schema(schema, "ring_lengths")?,
                rows.iter()
                    .map(|row| row.ring_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                &field_from_schema(schema, "surface_lengths")?,
                rows.iter()
                    .map(|row| row.surface_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                &field_from_schema(schema, "shell_lengths")?,
                rows.iter()
                    .map(|row| row.shell_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                &field_from_schema(schema, "solid_lengths")?,
                rows.iter()
                    .map(|row| row.solid_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
        ],
    )
    .map_err(Error::from)
}

fn semantics_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[SemanticRow],
    projection: &ProjectionLayout,
    geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<RecordBatch> {
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(UInt64Array::from(
            rows.iter().map(|row| row.semantic_id).collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            rows.iter()
                .map(|row| Some(row.semantic_type.clone()))
                .collect::<Vec<_>>(),
        )),
    ];
    if let Some(spec) = projection.semantic_attributes.as_ref() {
        arrays.push(projected_struct_array_from_attributes(
            &field_from_schema(schema, "attributes")?,
            spec,
            &rows.iter().map(|row| row.attributes.as_ref()).collect::<Vec<_>>(),
            geometry_id_map,
        )?);
    }
    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn semantic_children_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: Vec<SemanticChildRow>,
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.parent_semantic_id)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter().map(|row| row.child_ordinal).collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.into_iter()
                    .map(|row| row.child_semantic_id)
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn geometry_surface_semantics_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[GeometrySurfaceSemanticRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.geometry_id).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter()
                    .map(|row| row.surface_ordinal)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.semantic_id).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn geometry_point_semantics_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[GeometryPointSemanticRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.geometry_id).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter().map(|row| row.point_ordinal).collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.semantic_id).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn geometry_linestring_semantics_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[GeometryLinestringSemanticRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.geometry_id).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter()
                    .map(|row| row.linestring_ordinal)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.semantic_id).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn template_geometry_semantics_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[TemplateGeometrySemanticRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.template_geometry_id)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.primitive_type.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter()
                    .map(|row| row.primitive_ordinal)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.semantic_id).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn materials_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[MaterialRow],
    projection: &ProjectionLayout,
) -> Result<RecordBatch> {
    let mut arrays: Vec<ArrayRef> = vec![Arc::new(UInt64Array::from(
        rows.iter().map(|row| row.material_id).collect::<Vec<_>>(),
    ))];
    if let Some(specs) = &projection.material_payload {
        for spec in &specs.fields {
            arrays.push(material_payload_array(spec, rows)?);
        }
    }
    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn geometry_surface_materials_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[GeometrySurfaceMaterialRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.geometry_id).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter()
                    .map(|row| row.surface_ordinal)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.theme.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.material_id).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn template_geometry_materials_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[TemplateGeometryMaterialRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.template_geometry_id)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.primitive_type.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter()
                    .map(|row| row.primitive_ordinal)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.theme.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.material_id).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn textures_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[TextureRow],
    projection: &ProjectionLayout,
) -> Result<RecordBatch> {
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(UInt64Array::from(
            rows.iter().map(|row| row.texture_id).collect::<Vec<_>>(),
        )),
        Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| Some(row.image_uri.clone()))
                .collect::<Vec<_>>(),
        )),
    ];
    if let Some(specs) = &projection.texture_payload {
        for spec in &specs.fields {
            arrays.push(texture_payload_array(spec, rows)?);
        }
    }
    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn texture_vertices_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: Vec<TextureVertexRow>,
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.uv_id).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.u).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.into_iter().map(|row| row.v).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn geometry_ring_textures_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[GeometryRingTextureRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.geometry_id).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter()
                    .map(|row| row.surface_ordinal)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter().map(|row| row.ring_ordinal).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.theme.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.texture_id).collect::<Vec<_>>(),
            )),
            Arc::new(list_u64_array(
                &field_from_schema(schema, "uv_indices")?,
                rows.iter()
                    .map(|row| Some(row.uv_indices.clone()))
                    .collect::<Vec<_>>(),
            )?),
        ],
    )
    .map_err(Error::from)
}

fn template_geometry_ring_textures_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[TemplateGeometryRingTextureRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.template_geometry_id)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter()
                    .map(|row| row.surface_ordinal)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter().map(|row| row.ring_ordinal).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.theme.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.texture_id).collect::<Vec<_>>(),
            )),
            Arc::new(list_u64_array(
                &field_from_schema(schema, "uv_indices")?,
                rows.iter()
                    .map(|row| Some(row.uv_indices.clone()))
                    .collect::<Vec<_>>(),
            )?),
        ],
    )
    .map_err(Error::from)
}

fn material_payload_array(spec: &ProjectedFieldSpec, rows: &[MaterialRow]) -> Result<ArrayRef> {
    Ok(match spec.name.as_str() {
        FIELD_MATERIAL_NAME => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| Some(row.name.clone()))
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_MATERIAL_AMBIENT_INTENSITY => Arc::new(Float64Array::from(
            rows.iter()
                .map(|row| row.ambient_intensity)
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_MATERIAL_DIFFUSE_COLOR => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.diffuse_color.clone())
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_MATERIAL_EMISSIVE_COLOR => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.emissive_color.clone())
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_MATERIAL_SPECULAR_COLOR => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.specular_color.clone())
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_MATERIAL_SHININESS => Arc::new(Float64Array::from(
            rows.iter().map(|row| row.shininess).collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_MATERIAL_TRANSPARENCY => Arc::new(Float64Array::from(
            rows.iter().map(|row| row.transparency).collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_MATERIAL_IS_SMOOTH => Arc::new(arrow::array::BooleanArray::from(
            rows.iter().map(|row| row.is_smooth).collect::<Vec<_>>(),
        )) as ArrayRef,
        other => {
            return Err(Error::Conversion(format!(
                "unsupported material projection column {other}"
            )));
        }
    })
}

fn texture_payload_array(spec: &ProjectedFieldSpec, rows: &[TextureRow]) -> Result<ArrayRef> {
    Ok(match spec.name.as_str() {
        FIELD_TEXTURE_IMAGE_TYPE => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| Some(row.image_type.clone()))
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_TEXTURE_WRAP_MODE => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.wrap_mode.clone())
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_TEXTURE_TEXTURE_TYPE => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.texture_type.clone())
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_TEXTURE_BORDER_COLOR => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.border_color.clone())
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        other => {
            return Err(Error::Conversion(format!(
                "unsupported texture projection column {other}"
            )));
        }
    })
}

fn optional_batch<T, F>(rows: Vec<T>, build: F) -> Result<Option<RecordBatch>>
where
    F: FnOnce(Vec<T>) -> Result<RecordBatch>,
{
    if rows.is_empty() {
        Ok(None)
    } else {
        build(rows).map(Some)
    }
}

fn optional_batch_ref<T, F>(rows: &[T], build: F) -> Result<Option<RecordBatch>>
where
    F: FnOnce(&[T]) -> Result<RecordBatch>,
{
    if rows.is_empty() {
        Ok(None)
    } else {
        build(rows).map(Some)
    }
}

fn field_from_schema(schema: &Arc<arrow::datatypes::Schema>, name: &str) -> Result<FieldRef> {
    Ok(Arc::new(schema.field_with_name(name)?.clone()))
}

fn fixed_size_f64_array<const N: usize>(
    field: &FieldRef,
    size: i32,
    rows: Vec<Option<[f64; N]>>,
) -> Result<FixedSizeListArray> {
    let mut flat = Vec::with_capacity(rows.len() * N);
    let mut validity = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(values) = row {
            flat.extend(values);
            validity.push(true);
        } else {
            flat.extend(std::iter::repeat_n(0.0, N));
            validity.push(false);
        }
    }
    let values: ArrayRef = Arc::new(Float64Array::from(flat));
    let nulls = if validity.iter().all(|item| *item) {
        None
    } else {
        Some(NullBuffer::from(validity))
    };
    FixedSizeListArray::try_new(fixed_list_child_field(field)?, size, values, nulls)
        .map_err(Error::from)
}

fn list_u64_array(field: &FieldRef, rows: Vec<Option<Vec<u64>>>) -> Result<ListArray> {
    let mut offsets = vec![0_i32];
    let mut flat: Vec<u64> = Vec::new();
    let mut validity = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(values) = row {
            flat.extend(&values);
            offsets.push(usize_to_i32(flat.len(), "list offset")?);
            validity.push(true);
        } else {
            offsets.push(usize_to_i32(flat.len(), "list offset")?);
            validity.push(false);
        }
    }
    let values: ArrayRef = Arc::new(UInt64Array::from(flat));
    let nulls = if validity.iter().all(|item| *item) {
        None
    } else {
        Some(NullBuffer::from(validity))
    };
    ListArray::try_new(
        list_child_field(field)?,
        OffsetBuffer::new(ScalarBuffer::from(offsets)),
        values,
        nulls,
    )
    .map_err(Error::from)
}

fn list_u32_array(field: &FieldRef, rows: Vec<Option<Vec<u32>>>) -> Result<ListArray> {
    let mut offsets = vec![0_i32];
    let mut flat: Vec<u32> = Vec::new();
    let mut validity = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(values) = row {
            flat.extend(&values);
            offsets.push(usize_to_i32(flat.len(), "list offset")?);
            validity.push(true);
        } else {
            offsets.push(usize_to_i32(flat.len(), "list offset")?);
            validity.push(false);
        }
    }
    let values: ArrayRef = Arc::new(UInt32Array::from(flat));
    let nulls = if validity.iter().all(|item| *item) {
        None
    } else {
        Some(NullBuffer::from(validity))
    };
    ListArray::try_new(
        list_child_field(field)?,
        OffsetBuffer::new(ScalarBuffer::from(offsets)),
        values,
        nulls,
    )
    .map_err(Error::from)
}

fn fixed_list_child_field(field: &FieldRef) -> Result<FieldRef> {
    match field.data_type() {
        DataType::FixedSizeList(child, _) => Ok(child.clone()),
        other => Err(Error::Conversion(format!(
            "expected fixed size list field, found {other:?}"
        ))),
    }
}

fn list_child_field(field: &FieldRef) -> Result<FieldRef> {
    match field.data_type() {
        DataType::List(child) => Ok(child.clone()),
        other => Err(Error::Conversion(format!(
            "expected list field, found {other:?}"
        ))),
    }
}

fn projected_struct_array_from_attributes(
    field: &FieldRef,
    spec: &ProjectedStructSpec,
    rows: &[Option<&cityjson::v2_0::OwnedAttributes>],
    geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<ArrayRef> {
    let values = rows
        .iter()
        .map(|row| row.map(|attributes| OwnedAttributeValue::Map(attributes_to_hash_map(attributes))))
        .collect::<Vec<_>>();
    let value_refs = values.iter().map(Option::as_ref).collect::<Vec<_>>();
    projected_value_array(
        field,
        &ProjectedValueSpec::Struct(spec.clone()),
        &value_refs,
        geometry_id_map,
    )
}

fn attributes_to_hash_map(
    attributes: &cityjson::v2_0::OwnedAttributes,
) -> HashMap<String, OwnedAttributeValue> {
    attributes
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn projected_value_array(
    field: &FieldRef,
    spec: &ProjectedValueSpec,
    values: &[Option<&OwnedAttributeValue>],
    geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<ArrayRef> {
    Ok(match spec {
        ProjectedValueSpec::Null => {
            for value in values {
                if let Some(value) = value
                    && !matches!(value, AttributeValue::Null)
                {
                    return Err(Error::Conversion(format!(
                        "expected null projected value, found {value}"
                    )));
                }
            }
            Arc::new(NullArray::new(values.len())) as ArrayRef
        }
        ProjectedValueSpec::Boolean => Arc::new(BooleanArray::from(
            values
                .iter()
                .map(|value| match value {
                    None | Some(AttributeValue::Null) => Ok(None),
                    Some(AttributeValue::Bool(value)) => Ok(Some(*value)),
                    Some(other) => Err(Error::Conversion(format!(
                        "expected bool projected value, found {other}"
                    ))),
                })
                .collect::<Result<Vec<_>>>()?,
        )) as ArrayRef,
        ProjectedValueSpec::UInt64 => Arc::new(UInt64Array::from(
            values
                .iter()
                .map(|value| match value {
                    None | Some(AttributeValue::Null) => Ok(None),
                    Some(AttributeValue::Unsigned(value)) => Ok(Some(*value)),
                    Some(other) => Err(Error::Conversion(format!(
                        "expected u64 projected value, found {other}"
                    ))),
                })
                .collect::<Result<Vec<_>>>()?,
        )) as ArrayRef,
        ProjectedValueSpec::Int64 => Arc::new(Int64Array::from(
            values
                .iter()
                .map(|value| match value {
                    None | Some(AttributeValue::Null) => Ok(None),
                    Some(AttributeValue::Integer(value)) => Ok(Some(*value)),
                    Some(other) => Err(Error::Conversion(format!(
                        "expected i64 projected value, found {other}"
                    ))),
                })
                .collect::<Result<Vec<_>>>()?,
        )) as ArrayRef,
        ProjectedValueSpec::Float64 => Arc::new(Float64Array::from(
            values
                .iter()
                .map(|value| match value {
                    None | Some(AttributeValue::Null) => Ok(None),
                    Some(AttributeValue::Float(value)) => Ok(Some(*value)),
                    Some(other) => Err(Error::Conversion(format!(
                        "expected f64 projected value, found {other}"
                    ))),
                })
                .collect::<Result<Vec<_>>>()?,
        )) as ArrayRef,
        ProjectedValueSpec::Utf8 => Arc::new(LargeStringArray::from(
            values
                .iter()
                .map(|value| match value {
                    None | Some(AttributeValue::Null) => Ok(None),
                    Some(AttributeValue::String(value)) => Ok(Some(value.clone())),
                    Some(other) => Err(Error::Conversion(format!(
                        "expected string projected value, found {other}"
                    ))),
                })
                .collect::<Result<Vec<_>>>()?,
        )) as ArrayRef,
        ProjectedValueSpec::GeometryRef => Arc::new(UInt64Array::from(
            values
                .iter()
                .map(|value| match value {
                    None | Some(AttributeValue::Null) => Ok(None),
                    Some(AttributeValue::Geometry(handle)) => geometry_id_map
                        .get(handle)
                        .copied()
                        .map(Some)
                        .ok_or_else(|| {
                            Error::Conversion(
                                "attribute geometry handle missing from id map".to_string(),
                            )
                        }),
                    Some(other) => Err(Error::Conversion(format!(
                        "expected geometry reference projected value, found {other}"
                    ))),
                })
                .collect::<Result<Vec<_>>>()?,
        )) as ArrayRef,
        ProjectedValueSpec::List {
            item_nullable: _,
            item,
        } => {
            let mut offsets = vec![0_i32];
            let mut flattened = Vec::new();
            let mut validity = Vec::with_capacity(values.len());
            for value in values {
                match value {
                    None | Some(AttributeValue::Null) => {
                        offsets.push(usize_to_i32(flattened.len(), "projected list offset")?);
                        validity.push(false);
                    }
                    Some(AttributeValue::Vec(items)) => {
                        flattened.extend(items.iter().map(Some));
                        offsets.push(usize_to_i32(flattened.len(), "projected list offset")?);
                        validity.push(true);
                    }
                    Some(other) => {
                        return Err(Error::Conversion(format!(
                            "expected list projected value, found {other}"
                        )));
                    }
                }
            }
            let child_field = list_child_field(field)?;
            let child_values = projected_value_array(&child_field, item, &flattened, geometry_id_map)?;
            let nulls = if validity.iter().all(|item| *item) {
                None
            } else {
                Some(NullBuffer::from(validity))
            };
            Arc::new(ListArray::try_new(
                child_field,
                OffsetBuffer::new(ScalarBuffer::from(offsets)),
                child_values,
                nulls,
            )?) as ArrayRef
        }
        ProjectedValueSpec::Struct(spec) => {
            let mut validity = Vec::with_capacity(values.len());
            let child_fields = spec.to_arrow_fields();
            let mut child_arrays = Vec::with_capacity(spec.fields.len());

            for child_spec in &spec.fields {
                let child_values = values
                    .iter()
                    .map(|value| match value {
                        None | Some(AttributeValue::Null) => Ok(None),
                        Some(AttributeValue::Map(map)) => Ok(map.get(&child_spec.name)),
                        Some(other) => Err(Error::Conversion(format!(
                            "expected struct projected value, found {other}"
                        ))),
                    })
                    .collect::<Result<Vec<_>>>()?;
                child_arrays.push(projected_value_array(
                    &Arc::new(child_spec.to_arrow_field()),
                    &child_spec.value,
                    &child_values,
                    geometry_id_map,
                )?);
            }

            for value in values {
                validity.push(matches!(value, Some(AttributeValue::Map(_))));
            }
            let nulls = if validity.iter().all(|item| *item) {
                None
            } else {
                Some(NullBuffer::from(validity))
            };
            Arc::new(StructArray::try_new(child_fields, child_arrays, nulls)?) as ArrayRef
        }
    })
}

fn projected_attributes_from_array(
    spec: Option<&ProjectedStructSpec>,
    array: Option<&StructArray>,
    row: usize,
    geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<cityjson::v2_0::OwnedAttributes> {
    let mut attributes = cityjson::v2_0::OwnedAttributes::default();
    let (Some(spec), Some(array)) = (spec, array) else {
        return Ok(attributes);
    };
    if array.is_null(row) {
        return Ok(attributes);
    }
    for (index, field_spec) in spec.fields.iter().enumerate() {
        let value = projected_value_from_array(
            array.column(index).as_ref(),
            &field_spec.value,
            row,
            geometry_handles,
        )?;
        attributes.insert(field_spec.name.clone(), value);
    }
    Ok(attributes)
}

fn projected_value_from_array(
    array: &dyn Array,
    spec: &ProjectedValueSpec,
    row: usize,
    geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<OwnedAttributeValue> {
    if array.is_null(row) {
        return Ok(AttributeValue::Null);
    }

    Ok(match spec {
        ProjectedValueSpec::Null => AttributeValue::Null,
        ProjectedValueSpec::Boolean => {
            AttributeValue::Bool(required_downcast::<BooleanArray>(array, "bool")?.value(row))
        }
        ProjectedValueSpec::UInt64 => {
            AttributeValue::Unsigned(required_downcast::<UInt64Array>(array, "u64")?.value(row))
        }
        ProjectedValueSpec::Int64 => {
            AttributeValue::Integer(required_downcast::<Int64Array>(array, "i64")?.value(row))
        }
        ProjectedValueSpec::Float64 => {
            AttributeValue::Float(required_downcast::<Float64Array>(array, "f64")?.value(row))
        }
        ProjectedValueSpec::Utf8 => AttributeValue::String(
            required_downcast::<LargeStringArray>(array, "large_utf8")?
                .value(row)
                .to_string(),
        ),
        ProjectedValueSpec::GeometryRef => {
            let id = required_downcast::<UInt64Array>(array, "geometry_ref")?.value(row);
            AttributeValue::Geometry(*geometry_handles.get(&id).ok_or_else(|| {
                Error::Conversion(format!("missing geometry handle for projected geometry id {id}"))
            })?)
        }
        ProjectedValueSpec::List { item, .. } => {
            let list = required_downcast::<ListArray>(array, "list")?;
            let offsets = list.value_offsets();
            let start = usize::try_from(offsets[row]).expect("offset fits into usize");
            let end = usize::try_from(offsets[row + 1]).expect("offset fits into usize");
            let values = (start..end)
                .map(|index| projected_value_from_array(list.values().as_ref(), item, index, geometry_handles))
                .collect::<Result<Vec<_>>>()?;
            AttributeValue::Vec(values)
        }
        ProjectedValueSpec::Struct(spec) => AttributeValue::Map(
            attributes_to_hash_map(&projected_attributes_from_array(
                Some(spec),
                Some(required_downcast::<StructArray>(array, "struct")?),
                row,
                geometry_handles,
            )?),
        ),
    })
}

fn required_downcast<'a, T: 'static>(
    array: &'a dyn Array,
    expected: &str,
) -> Result<&'a T> {
    array.as_any().downcast_ref::<T>().ok_or_else(|| {
        Error::Conversion(format!("expected projected array type {expected}"))
    })
}

fn read_metadata_row(batch: &RecordBatch, projection: &ProjectionLayout) -> Result<MetadataRow> {
    let empty_geometry_handles = HashMap::new();
    Ok(MetadataRow {
        citymodel_id: read_large_string_scalar(batch, "citymodel_id", 0)?,
        cityjson_version: read_string_scalar(batch, "cityjson_version", 0)?,
        citymodel_kind: read_string_scalar(batch, "citymodel_kind", 0)?,
        identifier: read_large_string_optional(batch, "identifier", 0)?,
        title: read_large_string_optional(batch, "title", 0)?,
        reference_system: read_large_string_optional(batch, "reference_system", 0)?,
        geographical_extent: read_fixed_size_f64_optional::<6>(batch, "geographical_extent", 0)?,
        reference_date: read_string_optional(batch, "reference_date", 0)?,
        default_material_theme: read_string_optional(batch, "default_material_theme", 0)?,
        default_texture_theme: read_string_optional(batch, "default_texture_theme", 0)?,
        point_of_contact_json: read_large_string_optional(batch, "point_of_contact_json", 0)?,
        root_extra: {
            let array = projection
                .root_extra
                .as_ref()
                .map(|_| downcast_required::<StructArray>(batch, "root_extra"))
                .transpose()?;
            let attributes = projected_attributes_from_array(
                projection.root_extra.as_ref(),
                array,
                0,
                &empty_geometry_handles,
            )?;
            (!attributes.is_empty()).then_some(attributes)
        },
        metadata_extra: {
            let array = projection
                .metadata_extra
                .as_ref()
                .map(|_| downcast_required::<StructArray>(batch, "metadata_extra"))
                .transpose()?;
            let attributes = projected_attributes_from_array(
                projection.metadata_extra.as_ref(),
                array,
                0,
                &empty_geometry_handles,
            )?;
            (!attributes.is_empty()).then_some(attributes)
        },
    })
}

fn read_transform_row(batch: &RecordBatch) -> Result<TransformRow> {
    Ok(TransformRow {
        scale: read_fixed_size_f64_required::<3>(batch, "scale", 0)?,
        translate: read_fixed_size_f64_required::<3>(batch, "translate", 0)?,
    })
}

fn read_extension_rows(batch: &RecordBatch) -> Result<Vec<ExtensionRow>> {
    (0..batch.num_rows())
        .map(|row| {
            Ok(ExtensionRow {
                extension_name: read_string_scalar(batch, "extension_name", row)?,
                uri: read_large_string_scalar(batch, "uri", row)?,
                version: read_string_optional(batch, "version", row)?,
            })
        })
        .collect()
}

fn read_vertex_rows(batch: &RecordBatch) -> Result<Vec<VertexRow>> {
    let vertex_ids = downcast_required::<UInt64Array>(batch, "vertex_id")?;
    let xs = downcast_required::<Float64Array>(batch, "x")?;
    let ys = downcast_required::<Float64Array>(batch, "y")?;
    let zs = downcast_required::<Float64Array>(batch, "z")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(VertexRow {
                vertex_id: vertex_ids.value(row),
                x: xs.value(row),
                y: ys.value(row),
                z: zs.value(row),
            })
        })
        .collect()
}

fn read_cityobject_child_rows(batch: &RecordBatch) -> Result<Vec<CityObjectChildRow>> {
    let parents = downcast_required::<UInt64Array>(batch, "parent_cityobject_ix")?;
    let ordinals = downcast_required::<UInt32Array>(batch, "child_ordinal")?;
    let children = downcast_required::<UInt64Array>(batch, "child_cityobject_ix")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(CityObjectChildRow {
                parent_cityobject_ix: parents.value(row),
                child_ordinal: ordinals.value(row),
                child_cityobject_ix: children.value(row),
            })
        })
        .collect()
}

fn read_geometry_boundary_rows(batch: &RecordBatch) -> Result<Vec<GeometryBoundaryRow>> {
    let geometry_ids = downcast_required::<UInt64Array>(batch, "geometry_id")?;
    let vertices = downcast_required::<ListArray>(batch, "vertex_indices")?;
    let lines = downcast_required::<ListArray>(batch, "line_lengths")?;
    let rings = downcast_required::<ListArray>(batch, "ring_lengths")?;
    let surfaces = downcast_required::<ListArray>(batch, "surface_lengths")?;
    let shells = downcast_required::<ListArray>(batch, "shell_lengths")?;
    let solids = downcast_required::<ListArray>(batch, "solid_lengths")?;

    (0..batch.num_rows())
        .map(|row| {
            Ok(GeometryBoundaryRow {
                geometry_id: geometry_ids.value(row),
                vertex_indices: list_u64_value(vertices, row)?,
                line_lengths: list_u32_optional_value(lines, row)?,
                ring_lengths: list_u32_optional_value(rings, row)?,
                surface_lengths: list_u32_optional_value(surfaces, row)?,
                shell_lengths: list_u32_optional_value(shells, row)?,
                solid_lengths: list_u32_optional_value(solids, row)?,
            })
        })
        .collect()
}

fn read_template_geometry_boundary_rows(
    batch: &RecordBatch,
) -> Result<Vec<TemplateGeometryBoundaryRow>> {
    let geometry_ids = downcast_required::<UInt64Array>(batch, "template_geometry_id")?;
    let vertices = downcast_required::<ListArray>(batch, "vertex_indices")?;
    let lines = downcast_required::<ListArray>(batch, "line_lengths")?;
    let rings = downcast_required::<ListArray>(batch, "ring_lengths")?;
    let surfaces = downcast_required::<ListArray>(batch, "surface_lengths")?;
    let shells = downcast_required::<ListArray>(batch, "shell_lengths")?;
    let solids = downcast_required::<ListArray>(batch, "solid_lengths")?;

    (0..batch.num_rows())
        .map(|row| {
            Ok(TemplateGeometryBoundaryRow {
                template_geometry_id: geometry_ids.value(row),
                vertex_indices: list_u64_value(vertices, row)?,
                line_lengths: list_u32_optional_value(lines, row)?,
                ring_lengths: list_u32_optional_value(rings, row)?,
                surface_lengths: list_u32_optional_value(surfaces, row)?,
                shell_lengths: list_u32_optional_value(shells, row)?,
                solid_lengths: list_u32_optional_value(solids, row)?,
            })
        })
        .collect()
}

fn read_semantic_child_rows(batch: &RecordBatch) -> Result<Vec<SemanticChildRow>> {
    let parents = downcast_required::<UInt64Array>(batch, "parent_semantic_id")?;
    let ordinals = downcast_required::<UInt32Array>(batch, "child_ordinal")?;
    let children = downcast_required::<UInt64Array>(batch, "child_semantic_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(SemanticChildRow {
                parent_semantic_id: parents.value(row),
                child_ordinal: ordinals.value(row),
                child_semantic_id: children.value(row),
            })
        })
        .collect()
}

fn read_geometry_surface_semantic_rows(
    batch: &RecordBatch,
) -> Result<Vec<GeometrySurfaceSemanticRow>> {
    let geometries = downcast_required::<UInt64Array>(batch, "geometry_id")?;
    let ordinals = downcast_required::<UInt32Array>(batch, "surface_ordinal")?;
    let semantics = downcast_required::<UInt64Array>(batch, "semantic_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(GeometrySurfaceSemanticRow {
                geometry_id: geometries.value(row),
                surface_ordinal: ordinals.value(row),
                semantic_id: if semantics.is_null(row) {
                    None
                } else {
                    Some(semantics.value(row))
                },
            })
        })
        .collect()
}

fn read_geometry_point_semantic_rows(batch: &RecordBatch) -> Result<Vec<GeometryPointSemanticRow>> {
    let geometries = downcast_required::<UInt64Array>(batch, "geometry_id")?;
    let ordinals = downcast_required::<UInt32Array>(batch, "point_ordinal")?;
    let semantics = downcast_required::<UInt64Array>(batch, "semantic_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(GeometryPointSemanticRow {
                geometry_id: geometries.value(row),
                point_ordinal: ordinals.value(row),
                semantic_id: if semantics.is_null(row) {
                    None
                } else {
                    Some(semantics.value(row))
                },
            })
        })
        .collect()
}

fn read_geometry_linestring_semantic_rows(
    batch: &RecordBatch,
) -> Result<Vec<GeometryLinestringSemanticRow>> {
    let geometries = downcast_required::<UInt64Array>(batch, "geometry_id")?;
    let ordinals = downcast_required::<UInt32Array>(batch, "linestring_ordinal")?;
    let semantics = downcast_required::<UInt64Array>(batch, "semantic_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(GeometryLinestringSemanticRow {
                geometry_id: geometries.value(row),
                linestring_ordinal: ordinals.value(row),
                semantic_id: if semantics.is_null(row) {
                    None
                } else {
                    Some(semantics.value(row))
                },
            })
        })
        .collect()
}

fn read_template_geometry_semantic_rows(
    batch: &RecordBatch,
) -> Result<Vec<TemplateGeometrySemanticRow>> {
    let geometries = downcast_required::<UInt64Array>(batch, "template_geometry_id")?;
    let ordinals = downcast_required::<UInt32Array>(batch, "primitive_ordinal")?;
    let semantics = downcast_required::<UInt64Array>(batch, "semantic_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(TemplateGeometrySemanticRow {
                template_geometry_id: geometries.value(row),
                primitive_type: read_string_scalar(batch, "primitive_type", row)?,
                primitive_ordinal: ordinals.value(row),
                semantic_id: if semantics.is_null(row) {
                    None
                } else {
                    Some(semantics.value(row))
                },
            })
        })
        .collect()
}

fn read_geometry_surface_material_rows(
    batch: &RecordBatch,
) -> Result<Vec<GeometrySurfaceMaterialRow>> {
    let geometries = downcast_required::<UInt64Array>(batch, "geometry_id")?;
    let ordinals = downcast_required::<UInt32Array>(batch, "surface_ordinal")?;
    let materials = downcast_required::<UInt64Array>(batch, "material_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(GeometrySurfaceMaterialRow {
                geometry_id: geometries.value(row),
                surface_ordinal: ordinals.value(row),
                theme: read_string_scalar(batch, "theme", row)?,
                material_id: materials.value(row),
            })
        })
        .collect()
}

fn read_template_geometry_material_rows(
    batch: &RecordBatch,
) -> Result<Vec<TemplateGeometryMaterialRow>> {
    let geometries = downcast_required::<UInt64Array>(batch, "template_geometry_id")?;
    let ordinals = downcast_required::<UInt32Array>(batch, "primitive_ordinal")?;
    let materials = downcast_required::<UInt64Array>(batch, "material_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(TemplateGeometryMaterialRow {
                template_geometry_id: geometries.value(row),
                primitive_type: read_string_scalar(batch, "primitive_type", row)?,
                primitive_ordinal: ordinals.value(row),
                theme: read_string_scalar(batch, "theme", row)?,
                material_id: materials.value(row),
            })
        })
        .collect()
}

fn read_geometry_ring_texture_rows(batch: &RecordBatch) -> Result<Vec<GeometryRingTextureRow>> {
    let geometries = downcast_required::<UInt64Array>(batch, "geometry_id")?;
    let surface_ordinals = downcast_required::<UInt32Array>(batch, "surface_ordinal")?;
    let ring_ordinals = downcast_required::<UInt32Array>(batch, "ring_ordinal")?;
    let textures = downcast_required::<UInt64Array>(batch, "texture_id")?;
    let uv_indices = downcast_required::<ListArray>(batch, "uv_indices")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(GeometryRingTextureRow {
                geometry_id: geometries.value(row),
                surface_ordinal: surface_ordinals.value(row),
                ring_ordinal: ring_ordinals.value(row),
                theme: read_string_scalar(batch, "theme", row)?,
                texture_id: textures.value(row),
                uv_indices: list_u64_value(uv_indices, row)?,
            })
        })
        .collect()
}

fn read_template_geometry_ring_texture_rows(
    batch: &RecordBatch,
) -> Result<Vec<TemplateGeometryRingTextureRow>> {
    let geometries = downcast_required::<UInt64Array>(batch, "template_geometry_id")?;
    let surface_ordinals = downcast_required::<UInt32Array>(batch, "surface_ordinal")?;
    let ring_ordinals = downcast_required::<UInt32Array>(batch, "ring_ordinal")?;
    let textures = downcast_required::<UInt64Array>(batch, "texture_id")?;
    let uv_indices = downcast_required::<ListArray>(batch, "uv_indices")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(TemplateGeometryRingTextureRow {
                template_geometry_id: geometries.value(row),
                surface_ordinal: surface_ordinals.value(row),
                ring_ordinal: ring_ordinals.value(row),
                theme: read_string_scalar(batch, "theme", row)?,
                texture_id: textures.value(row),
                uv_indices: list_u64_value(uv_indices, row)?,
            })
        })
        .collect()
}

fn apply_metadata_row(
    model: &mut OwnedCityModel,
    row: &MetadataRow,
    geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<()> {
    if let Some(identifier) = &row.identifier {
        model
            .metadata_mut()
            .set_identifier(CityModelIdentifier::new(identifier.clone()));
    }
    if let Some(title) = &row.title {
        model.metadata_mut().set_title(title.clone());
    }
    if let Some(reference_system) = &row.reference_system {
        model
            .metadata_mut()
            .set_reference_system(CRS::new(reference_system.clone()));
    }
    if let Some(extent) = row.geographical_extent {
        model
            .metadata_mut()
            .set_geographical_extent(BBox::from(extent));
    }

    if let Some(reference_date) = &row.reference_date {
        model
            .metadata_mut()
            .set_reference_date(cityjson::v2_0::Date::new(reference_date.clone()));
    }
    if let Some(theme) = &row.default_material_theme {
        model.set_default_material_theme(Some(ThemeName::new(theme.clone())));
    }
    if let Some(theme) = &row.default_texture_theme {
        model.set_default_texture_theme(Some(ThemeName::new(theme.clone())));
    }
    if let Some(value) = &row.point_of_contact_json {
        let json: JsonValue = serde_json::from_str(value)?;
        let contact = contact_from_json(&json, geometry_handles)?;
        model.metadata_mut().set_point_of_contact(Some(contact));
    }
    if let Some(extra) = &row.root_extra {
        for (key, value) in extra.iter() {
            model.extra_mut().insert(key.clone(), value.clone());
        }
    }
    if let Some(extra) = &row.metadata_extra {
        for (key, value) in extra.iter() {
            model
                .metadata_mut()
                .extra_mut()
                .insert(key.clone(), value.clone());
        }
    }

    Ok(())
}

fn build_semantic_map(
    geometry_type: &str,
    boundary: &GeometryBoundaryRow,
    surface_rows: Option<&Vec<GeometrySurfaceSemanticRow>>,
    point_rows: Option<&Vec<GeometryPointSemanticRow>>,
    linestring_rows: Option<&Vec<GeometryLinestringSemanticRow>>,
    handles: &HashMap<u64, cityjson::prelude::SemanticHandle>,
) -> Result<Option<SemanticMap<u32>>> {
    match parse_geometry_type(geometry_type)? {
        GeometryType::MultiPoint => {
            let Some(rows) = point_rows else {
                return Ok(None);
            };
            if rows.is_empty() {
                return Ok(None);
            }
            let point_count = boundary.vertex_indices.len();
            if rows.len() != point_count {
                return Err(Error::Conversion(format!(
                    "point semantic row count {} does not match point count {}",
                    rows.len(),
                    point_count
                )));
            }
            let mut map = SemanticMap::new();
            for (expected_ordinal, row) in rows.iter().enumerate() {
                let actual_ordinal =
                    usize::try_from(row.point_ordinal).expect("u32 point ordinal fits into usize");
                if actual_ordinal != expected_ordinal {
                    return Err(Error::Conversion(format!(
                        "point semantic ordinal {} is out of order, expected {}",
                        row.point_ordinal, expected_ordinal
                    )));
                }
                map.add_point(row.semantic_id.and_then(|id| handles.get(&id).copied()));
            }
            Ok(Some(map))
        }
        GeometryType::MultiLineString => {
            let Some(rows) = linestring_rows else {
                return Ok(None);
            };
            if rows.is_empty() {
                return Ok(None);
            }
            let linestring_count =
                required_lengths(boundary.line_lengths.as_ref(), "line_lengths")?.len();
            if rows.len() != linestring_count {
                return Err(Error::Conversion(format!(
                    "linestring semantic row count {} does not match linestring count {}",
                    rows.len(),
                    linestring_count
                )));
            }
            let mut map = SemanticMap::new();
            for (expected_ordinal, row) in rows.iter().enumerate() {
                let actual_ordinal = usize::try_from(row.linestring_ordinal)
                    .expect("u32 linestring ordinal fits into usize");
                if actual_ordinal != expected_ordinal {
                    return Err(Error::Conversion(format!(
                        "linestring semantic ordinal {} is out of order, expected {}",
                        row.linestring_ordinal, expected_ordinal
                    )));
                }
                map.add_linestring(row.semantic_id.and_then(|id| handles.get(&id).copied()));
            }
            Ok(Some(map))
        }
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => {
            let Some(rows) = surface_rows else {
                return Ok(None);
            };
            if rows.is_empty() {
                return Ok(None);
            }
            let surface_count = surface_count(boundary);
            if rows.len() != surface_count {
                return Err(Error::Conversion(format!(
                    "surface semantic row count {} does not match surface count {}",
                    rows.len(),
                    surface_count
                )));
            }
            let mut map = SemanticMap::new();
            for (expected_ordinal, row) in rows.iter().enumerate() {
                let actual_ordinal = usize::try_from(row.surface_ordinal)
                    .expect("u32 surface ordinal fits into usize");
                if actual_ordinal != expected_ordinal {
                    return Err(Error::Conversion(format!(
                        "surface semantic ordinal {} is out of order, expected {}",
                        row.surface_ordinal, expected_ordinal
                    )));
                }
                map.add_surface(row.semantic_id.and_then(|id| handles.get(&id).copied()));
            }
            Ok(Some(map))
        }
        GeometryType::GeometryInstance => Err(Error::Unsupported("geometry instances".to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

fn build_material_maps(
    geometry_type: &str,
    boundary: &GeometryBoundaryRow,
    surface_rows: Option<&Vec<GeometrySurfaceMaterialRow>>,
    handles: &HashMap<u64, cityjson::prelude::MaterialHandle>,
) -> Result<Option<MaterialThemeMaps>> {
    match parse_geometry_type(geometry_type)? {
        GeometryType::MultiPoint | GeometryType::MultiLineString => Ok(None),
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => surface_rows.map_or(Ok(None), |rows| {
            grouped_material_maps(
                rows,
                surface_count(boundary),
                |row| Ok((row.theme.clone(), row.surface_ordinal, row.material_id)),
                MaterialMap::add_surface,
                |row, count| {
                    format!(
                        "material assignment surface ordinal {} exceeds surface count {}",
                        row.surface_ordinal, count
                    )
                },
                |row| {
                    format!(
                        "duplicate material assignment for theme {} surface {}",
                        row.theme, row.surface_ordinal
                    )
                },
                handles,
            )
            .map(Some)
        }),
        GeometryType::GeometryInstance => Err(Error::Unsupported("geometry materials".to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

fn grouped_material_maps<R, FFields, FAppend, FExceeds, FDuplicate>(
    rows: &[R],
    primitive_count: usize,
    fields: FFields,
    append: FAppend,
    exceeds_message: FExceeds,
    duplicate_message: FDuplicate,
    handles: &HashMap<u64, cityjson::prelude::MaterialHandle>,
) -> Result<MaterialThemeMaps>
where
    FFields: Fn(&R) -> Result<(String, u32, u64)>,
    FAppend: Fn(&mut MaterialMap<u32>, Option<cityjson::prelude::MaterialHandle>),
    FExceeds: Fn(&R, usize) -> String,
    FDuplicate: Fn(&R) -> String,
{
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let mut grouped = BTreeMap::<String, Vec<Option<cityjson::prelude::MaterialHandle>>>::new();
    for row in rows {
        let (theme, ordinal, id) = fields(row)?;
        let ordinal = usize::try_from(ordinal).expect("u32 ordinal fits into usize");
        if ordinal >= primitive_count {
            return Err(Error::Conversion(exceeds_message(row, primitive_count)));
        }
        let material = *handles
            .get(&id)
            .ok_or_else(|| Error::Conversion(format!("missing material {id}")))?;
        let entries = grouped
            .entry(theme)
            .or_insert_with(|| vec![None; primitive_count]);
        if entries[ordinal].is_some() {
            return Err(Error::Conversion(duplicate_message(row)));
        }
        entries[ordinal] = Some(material);
    }
    Ok(grouped
        .into_iter()
        .map(|(theme, values)| {
            let mut map = MaterialMap::new();
            for value in values {
                append(&mut map, value);
            }
            (ThemeName::new(theme), map)
        })
        .collect())
}

fn build_ordered_template_semantic_map(
    rows: &[TemplateGeometrySemanticRow],
    expected_primitive_type: &str,
    expected_count: usize,
    count_label: &str,
    handles: &HashMap<u64, cityjson::prelude::SemanticHandle>,
) -> Result<Option<SemanticMap<u32>>> {
    if rows.is_empty() {
        return Ok(None);
    }
    if rows.len() != expected_count {
        return Err(Error::Conversion(format!(
            "template {count_label} semantic row count {} does not match {count_label} count {}",
            rows.len(),
            expected_count
        )));
    }

    let mut map = SemanticMap::new();
    for (expected_ordinal, row) in rows.iter().enumerate() {
        if row.primitive_type != expected_primitive_type {
            return Err(Error::Conversion(format!(
                "template {count_label} semantic row has unexpected primitive type {}",
                row.primitive_type
            )));
        }
        let actual_ordinal =
            usize::try_from(row.primitive_ordinal).expect("u32 primitive ordinal fits into usize");
        if actual_ordinal != expected_ordinal {
            return Err(Error::Conversion(format!(
                "template {count_label} semantic ordinal {} is out of order, expected {}",
                row.primitive_ordinal, expected_ordinal
            )));
        }
        let handle = row.semantic_id.and_then(|id| handles.get(&id).copied());
        match expected_primitive_type {
            PRIMITIVE_TYPE_POINT => map.add_point(handle),
            PRIMITIVE_TYPE_LINESTRING => map.add_linestring(handle),
            PRIMITIVE_TYPE_SURFACE => map.add_surface(handle),
            other => {
                return Err(Error::Conversion(format!(
                    "unsupported template semantic primitive type {other}"
                )));
            }
        }
    }

    Ok(Some(map))
}

fn build_template_semantic_map(
    geometry_type: &str,
    boundary: &TemplateGeometryBoundaryRow,
    rows: Option<&Vec<TemplateGeometrySemanticRow>>,
    handles: &HashMap<u64, cityjson::prelude::SemanticHandle>,
) -> Result<Option<SemanticMap<u32>>> {
    let Some(rows) = rows else {
        return Ok(None);
    };
    match parse_geometry_type(geometry_type)? {
        GeometryType::MultiPoint => build_ordered_template_semantic_map(
            rows,
            PRIMITIVE_TYPE_POINT,
            boundary.vertex_indices.len(),
            "point",
            handles,
        ),
        GeometryType::MultiLineString => build_ordered_template_semantic_map(
            rows,
            PRIMITIVE_TYPE_LINESTRING,
            required_lengths(boundary.line_lengths.as_ref(), "line_lengths")?.len(),
            "linestring",
            handles,
        ),
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => build_ordered_template_semantic_map(
            rows,
            PRIMITIVE_TYPE_SURFACE,
            template_surface_count(boundary),
            "surface",
            handles,
        ),
        GeometryType::GeometryInstance => Err(Error::Unsupported("geometry instances".to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

fn build_template_material_maps(
    geometry_type: &str,
    boundary: &TemplateGeometryBoundaryRow,
    rows: Option<&Vec<TemplateGeometryMaterialRow>>,
    handles: &HashMap<u64, cityjson::prelude::MaterialHandle>,
) -> Result<Option<MaterialThemeMaps>> {
    let Some(rows) = rows else {
        return Ok(None);
    };
    if rows.is_empty() {
        return Ok(None);
    }
    match parse_geometry_type(geometry_type)? {
        GeometryType::MultiPoint => grouped_material_maps(
            rows,
            boundary.vertex_indices.len(),
            |row| {
                if row.primitive_type != PRIMITIVE_TYPE_POINT {
                    return Err(Error::Conversion(format!(
                        "template point material row has unexpected primitive type {}",
                        row.primitive_type
                    )));
                }
                Ok((row.theme.clone(), row.primitive_ordinal, row.material_id))
            },
            MaterialMap::add_point,
            |row, count| {
                format!(
                    "template material assignment point ordinal {} exceeds point count {}",
                    row.primitive_ordinal, count
                )
            },
            |row| {
                format!(
                    "duplicate template material assignment for theme {} point {}",
                    row.theme, row.primitive_ordinal
                )
            },
            handles,
        )
        .map(Some),
        GeometryType::MultiLineString => grouped_material_maps(
            rows,
            required_lengths(boundary.line_lengths.as_ref(), "line_lengths")?.len(),
            |row| {
                if row.primitive_type != PRIMITIVE_TYPE_LINESTRING {
                    return Err(Error::Conversion(format!(
                        "template linestring material row has unexpected primitive type {}",
                        row.primitive_type
                    )));
                }
                Ok((row.theme.clone(), row.primitive_ordinal, row.material_id))
            },
            MaterialMap::add_linestring,
            |row, count| {
                format!(
                    "template material assignment linestring ordinal {} exceeds linestring count {}",
                    row.primitive_ordinal, count
                )
            },
            |row| {
                format!(
                    "duplicate template material assignment for theme {} linestring {}",
                    row.theme, row.primitive_ordinal
                )
            },
            handles,
        )
        .map(Some),
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => grouped_material_maps(
            rows,
            template_surface_count(boundary),
            |row| {
                if row.primitive_type != PRIMITIVE_TYPE_SURFACE {
                    return Err(Error::Conversion(format!(
                        "template surface material row has unexpected primitive type {}",
                        row.primitive_type
                    )));
                }
                Ok((row.theme.clone(), row.primitive_ordinal, row.material_id))
            },
            MaterialMap::add_surface,
            |row, count| {
                format!(
                    "template material assignment surface ordinal {} exceeds surface count {}",
                    row.primitive_ordinal, count
                )
            },
            |row| {
                format!(
                    "duplicate template material assignment for theme {} surface {}",
                    row.theme, row.primitive_ordinal
                )
            },
            handles,
        )
        .map(Some),
        GeometryType::GeometryInstance => Err(Error::Unsupported("geometry materials".to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

fn build_template_texture_maps(
    geometry_type: &str,
    boundary: &TemplateGeometryBoundaryRow,
    rows: Option<&Vec<TemplateGeometryRingTextureRow>>,
    handles: &HashMap<u64, cityjson::prelude::TextureHandle>,
) -> Result<Option<TextureThemeMaps>> {
    let Some(rows) = rows else {
        return Ok(None);
    };
    if rows.is_empty() {
        return Ok(None);
    }
    match parse_geometry_type(geometry_type)? {
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => {
            let ring_layouts = template_ring_layouts(boundary)?;
            let ring_lookup = ring_layouts
                .iter()
                .enumerate()
                .map(|(index, layout)| ((layout.surface_ordinal, layout.ring_ordinal), index))
                .collect::<HashMap<_, _>>();
            let mut maps = BTreeMap::<String, TextureMap<u32>>::new();

            for row in rows {
                let layout_index = *ring_lookup
                    .get(&(row.surface_ordinal, row.ring_ordinal))
                    .ok_or_else(|| {
                        Error::Conversion(format!(
                            "missing template ring layout for surface {} ring {}",
                            row.surface_ordinal, row.ring_ordinal
                        ))
                    })?;
                let layout = ring_layouts[layout_index];
                if row.uv_indices.len() != layout.len {
                    return Err(Error::Conversion(format!(
                        "template texture assignment for theme {} surface {} ring {} has {} uv indices, expected {}",
                        row.theme,
                        row.surface_ordinal,
                        row.ring_ordinal,
                        row.uv_indices.len(),
                        layout.len
                    )));
                }
                let texture = *handles.get(&row.texture_id).ok_or_else(|| {
                    Error::Conversion(format!("missing texture {}", row.texture_id))
                })?;
                if !maps.contains_key(&row.theme) {
                    maps.insert(row.theme.clone(), empty_texture_map(&ring_layouts)?);
                }
                let map = maps
                    .get_mut(&row.theme)
                    .expect("texture theme map must exist");
                if map.ring_textures()[layout_index].is_some() {
                    return Err(Error::Conversion(format!(
                        "duplicate template texture assignment for theme {} surface {} ring {}",
                        row.theme, row.surface_ordinal, row.ring_ordinal
                    )));
                }
                if !map.set_ring_texture(layout_index, Some(texture)) {
                    return Err(Error::Conversion("missing texture ring slot".to_string()));
                }
                let slice = map.vertices_mut()[layout.start..layout.start + layout.len].iter_mut();
                for (slot, uv_id) in slice.zip(&row.uv_indices) {
                    let uv_id = u32::try_from(*uv_id).map_err(|_| {
                        Error::Conversion(format!("uv index {uv_id} does not fit into u32"))
                    })?;
                    *slot = Some(cityjson::v2_0::VertexIndex::new(uv_id));
                }
            }

            Ok(Some(
                maps.into_iter()
                    .map(|(theme, map)| (ThemeName::new(theme), map))
                    .collect(),
            ))
        }
        GeometryType::MultiPoint | GeometryType::MultiLineString => {
            Err(Error::Unsupported("geometry textures".to_string()))
        }
        GeometryType::GeometryInstance => Err(Error::Unsupported("geometry textures".to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

fn build_texture_maps(
    geometry_type: &str,
    boundary: &GeometryBoundaryRow,
    rows: Option<&Vec<GeometryRingTextureRow>>,
    handles: &HashMap<u64, cityjson::prelude::TextureHandle>,
) -> Result<Option<TextureThemeMaps>> {
    let Some(rows) = rows else {
        return Ok(None);
    };
    if rows.is_empty() {
        return Ok(None);
    }
    match parse_geometry_type(geometry_type)? {
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => {
            let ring_layouts = ring_layouts(boundary)?;
            let ring_lookup = ring_layouts
                .iter()
                .enumerate()
                .map(|(index, layout)| ((layout.surface_ordinal, layout.ring_ordinal), index))
                .collect::<HashMap<_, _>>();
            let mut maps = BTreeMap::<String, TextureMap<u32>>::new();

            for row in rows {
                let layout_index = *ring_lookup
                    .get(&(row.surface_ordinal, row.ring_ordinal))
                    .ok_or_else(|| {
                        Error::Conversion(format!(
                            "missing ring layout for surface {} ring {}",
                            row.surface_ordinal, row.ring_ordinal
                        ))
                    })?;
                let layout = ring_layouts[layout_index];
                if row.uv_indices.len() != layout.len {
                    return Err(Error::Conversion(format!(
                        "texture assignment for theme {} surface {} ring {} has {} uv indices, expected {}",
                        row.theme,
                        row.surface_ordinal,
                        row.ring_ordinal,
                        row.uv_indices.len(),
                        layout.len
                    )));
                }
                let texture = *handles.get(&row.texture_id).ok_or_else(|| {
                    Error::Conversion(format!("missing texture {}", row.texture_id))
                })?;
                if !maps.contains_key(&row.theme) {
                    maps.insert(row.theme.clone(), empty_texture_map(&ring_layouts)?);
                }
                let map = maps
                    .get_mut(&row.theme)
                    .expect("texture theme map must exist");
                if map.ring_textures()[layout_index].is_some() {
                    return Err(Error::Conversion(format!(
                        "duplicate texture assignment for theme {} surface {} ring {}",
                        row.theme, row.surface_ordinal, row.ring_ordinal
                    )));
                }
                if !map.set_ring_texture(layout_index, Some(texture)) {
                    return Err(Error::Conversion("missing texture ring slot".to_string()));
                }
                let slice = map.vertices_mut()[layout.start..layout.start + layout.len].iter_mut();
                for (slot, uv_id) in slice.zip(&row.uv_indices) {
                    let uv_id = u32::try_from(*uv_id).map_err(|_| {
                        Error::Conversion(format!("uv index {uv_id} does not fit into u32"))
                    })?;
                    *slot = Some(cityjson::v2_0::VertexIndex::new(uv_id));
                }
            }

            Ok(Some(
                maps.into_iter()
                    .map(|(theme, map)| (ThemeName::new(theme), map))
                    .collect(),
            ))
        }
        GeometryType::MultiPoint | GeometryType::MultiLineString => {
            Err(Error::Unsupported("geometry textures".to_string()))
        }
        GeometryType::GeometryInstance => Err(Error::Unsupported("geometry textures".to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

fn boundary_from_row(row: &GeometryBoundaryRow, geometry_type: &str) -> Result<Boundary<u32>> {
    boundary_from_parts(
        &row.vertex_indices,
        row.line_lengths.as_ref(),
        row.ring_lengths.as_ref(),
        row.surface_lengths.as_ref(),
        row.shell_lengths.as_ref(),
        row.solid_lengths.as_ref(),
        geometry_type,
    )
}

fn template_boundary_from_row(
    row: &TemplateGeometryBoundaryRow,
    geometry_type: &str,
) -> Result<Boundary<u32>> {
    boundary_from_parts(
        &row.vertex_indices,
        row.line_lengths.as_ref(),
        row.ring_lengths.as_ref(),
        row.surface_lengths.as_ref(),
        row.shell_lengths.as_ref(),
        row.solid_lengths.as_ref(),
        geometry_type,
    )
}

fn boundary_from_parts(
    vertex_indices: &[u64],
    line_lengths: Option<&Vec<u32>>,
    ring_lengths: Option<&Vec<u32>>,
    surface_lengths: Option<&Vec<u32>>,
    shell_lengths: Option<&Vec<u32>>,
    solid_lengths: Option<&Vec<u32>>,
    geometry_type: &str,
) -> Result<Boundary<u32>> {
    let vertices = vertex_indices
        .iter()
        .map(|value| {
            u32::try_from(*value).map_err(|_| {
                Error::Conversion(format!("vertex index {value} does not fit into u32"))
            })
        })
        .collect::<Result<Vec<_>>>()?
        .to_vertex_indices();

    let boundary = match parse_geometry_type(geometry_type)? {
        GeometryType::MultiPoint => Boundary::from_parts(vertices, vec![], vec![], vec![], vec![])?,
        GeometryType::MultiLineString => Boundary::from_parts(
            vertices,
            lengths_to_offsets(required_lengths(line_lengths, "line_lengths")?)?,
            vec![],
            vec![],
            vec![],
        )?,
        GeometryType::MultiSurface | GeometryType::CompositeSurface => Boundary::from_parts(
            vertices,
            lengths_to_offsets(required_lengths(ring_lengths, "ring_lengths")?)?,
            lengths_to_offsets(required_lengths(surface_lengths, "surface_lengths")?)?,
            vec![],
            vec![],
        )?,
        GeometryType::Solid => Boundary::from_parts(
            vertices,
            lengths_to_offsets(required_lengths(ring_lengths, "ring_lengths")?)?,
            lengths_to_offsets(required_lengths(surface_lengths, "surface_lengths")?)?,
            lengths_to_offsets(required_lengths(shell_lengths, "shell_lengths")?)?,
            vec![],
        )?,
        GeometryType::MultiSolid | GeometryType::CompositeSolid => Boundary::from_parts(
            vertices,
            lengths_to_offsets(required_lengths(ring_lengths, "ring_lengths")?)?,
            lengths_to_offsets(required_lengths(surface_lengths, "surface_lengths")?)?,
            lengths_to_offsets(required_lengths(shell_lengths, "shell_lengths")?)?,
            lengths_to_offsets(required_lengths(solid_lengths, "solid_lengths")?)?,
        )?,
        GeometryType::GeometryInstance => {
            return Err(Error::Unsupported("geometry instances".to_string()));
        }
        _ => {
            return Err(Error::Unsupported("unsupported geometry type".to_string()));
        }
    };
    Ok(boundary)
}

fn required_lengths<'a>(value: Option<&'a Vec<u32>>, name: &str) -> Result<&'a [u32]> {
    value
        .map(Vec::as_slice)
        .ok_or_else(|| Error::Conversion(format!("missing required {name}")))
}

fn lengths_to_offsets(lengths: &[u32]) -> Result<Vec<cityjson::v2_0::VertexIndex<u32>>> {
    if lengths.is_empty() {
        return Ok(Vec::<u32>::new().to_vertex_indices());
    }
    let mut offsets = Vec::with_capacity(lengths.len());
    let mut total = 0_u32;
    offsets.push(0);
    for length in &lengths[..lengths.len() - 1] {
        total = total
            .checked_add(*length)
            .ok_or_else(|| Error::Conversion("length offsets overflow u32".to_string()))?;
        offsets.push(total);
    }
    Ok(offsets.to_vertex_indices())
}

fn surface_count(row: &GeometryBoundaryRow) -> usize {
    match row.surface_lengths.as_ref() {
        Some(lengths) => lengths.len(),
        None => 0,
    }
}

fn template_surface_count(row: &TemplateGeometryBoundaryRow) -> usize {
    row.surface_lengths.as_ref().map_or(0, std::vec::Vec::len)
}

fn has_projection_field(specs: Option<&ProjectedStructSpec>, name: &str) -> bool {
    specs
        .map(|specs| specs.fields.iter().any(|spec| spec.name == name))
        .unwrap_or(false)
}

fn parse_geometry_type(value: &str) -> Result<GeometryType> {
    value.parse().map_err(Error::from)
}

fn parse_lod(value: &str) -> Result<LoD> {
    Ok(match value {
        "0" => LoD::LoD0,
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
        other => {
            return Err(Error::Conversion(format!("unsupported lod string {other}")));
        }
    })
}

fn parse_image_type(value: &str) -> Result<ImageType> {
    Ok(match value {
        "PNG" => ImageType::Png,
        "JPG" => ImageType::Jpg,
        other => return Err(Error::Conversion(format!("unsupported image type {other}"))),
    })
}

fn parse_wrap_mode(value: &str) -> Result<WrapMode> {
    Ok(match value {
        "wrap" => WrapMode::Wrap,
        "mirror" => WrapMode::Mirror,
        "clamp" => WrapMode::Clamp,
        "border" => WrapMode::Border,
        "none" => WrapMode::None,
        other => return Err(Error::Conversion(format!("unsupported wrap mode {other}"))),
    })
}

fn parse_texture_mapping_type(value: &str) -> Result<TextureType> {
    Ok(match value {
        "unknown" => TextureType::Unknown,
        "specific" => TextureType::Specific,
        "typical" => TextureType::Typical,
        other => {
            return Err(Error::Conversion(format!(
                "unsupported texture type {other}"
            )));
        }
    })
}

fn parse_rgb_json(value: &str) -> Result<RGB> {
    let json: JsonValue = serde_json::from_str(value)?;
    let array = json
        .as_array()
        .ok_or_else(|| Error::Conversion("rgb payload must be a JSON array".to_string()))?;
    if array.len() != 3 {
        return Err(Error::Conversion(format!(
            "rgb payload must contain 3 elements, found {}",
            array.len()
        )));
    }
    let mut values = [0.0_f32; 3];
    for (index, component) in array.iter().enumerate() {
        values[index] = component
            .as_f64()
            .ok_or_else(|| Error::Conversion("rgb component must be numeric".to_string()))?
            .to_string()
            .parse::<f32>()
            .map_err(|error| Error::Conversion(format!("invalid rgb component: {error}")))?;
    }
    Ok(RGB::from(values))
}

fn parse_rgba_json(value: &str) -> Result<RGBA> {
    let json: JsonValue = serde_json::from_str(value)?;
    let array = json
        .as_array()
        .ok_or_else(|| Error::Conversion("rgba payload must be a JSON array".to_string()))?;
    if array.len() != 4 {
        return Err(Error::Conversion(format!(
            "rgba payload must contain 4 elements, found {}",
            array.len()
        )));
    }
    let mut values = [0.0_f32; 4];
    for (index, component) in array.iter().enumerate() {
        values[index] = component
            .as_f64()
            .ok_or_else(|| Error::Conversion("rgba component must be numeric".to_string()))?
            .to_string()
            .parse::<f32>()
            .map_err(|error| Error::Conversion(format!("invalid rgba component: {error}")))?;
    }
    Ok(RGBA::from(values))
}

fn parse_semantic_type(value: &str) -> SemanticType<cityjson::prelude::OwnedStringStorage> {
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
        other if other.starts_with('+') => SemanticType::Extension(other.to_string()),
        other => SemanticType::Extension(other.to_string()),
    }
}

fn contact_from_json(
    value: &JsonValue,
    geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<Contact<cityjson::prelude::OwnedStringStorage>> {
    let object = value.as_object().ok_or_else(|| {
        Error::Conversion("pointOfContact must be encoded as a JSON object".to_string())
    })?;
    let mut contact = Contact::new();
    if let Some(value) = object.get("contactName").and_then(JsonValue::as_str) {
        contact.set_contact_name(value.to_string());
    }
    if let Some(value) = object.get("emailAddress").and_then(JsonValue::as_str) {
        contact.set_email_address(value.to_string());
    }
    if let Some(value) = object.get("role").and_then(JsonValue::as_str) {
        contact.set_role(Some(parse_contact_role(value)?));
    }
    if let Some(value) = object.get("website").and_then(JsonValue::as_str) {
        contact.set_website(Some(value.to_string()));
    }
    if let Some(value) = object.get("type").and_then(JsonValue::as_str) {
        contact.set_contact_type(Some(parse_contact_type(value)?));
    }
    if let Some(value) = object.get("phone").and_then(JsonValue::as_str) {
        contact.set_phone(Some(value.to_string()));
    }
    if let Some(value) = object.get("organization").and_then(JsonValue::as_str) {
        contact.set_organization(Some(value.to_string()));
    }
    if let Some(address) = object.get("address") {
        match json_to_attribute(address, geometry_handles)? {
            AttributeValue::Map(map) => {
                contact.set_address(Some(map.into()));
            }
            other => {
                return Err(Error::Conversion(format!(
                    "pointOfContact address must decode to an attribute map, found {other}"
                )));
            }
        }
    }
    Ok(contact)
}

fn parse_contact_role(value: &str) -> Result<ContactRole> {
    Ok(match value {
        "Author" => ContactRole::Author,
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
        other => {
            return Err(Error::Conversion(format!(
                "unsupported metadata contact role {other}"
            )));
        }
    })
}

fn parse_contact_type(value: &str) -> Result<ContactType> {
    Ok(match value {
        "Individual" => ContactType::Individual,
        "Organization" => ContactType::Organization,
        other => {
            return Err(Error::Conversion(format!(
                "unsupported metadata contact type {other}"
            )));
        }
    })
}

fn contact_to_json(contact: &Contact<cityjson::prelude::OwnedStringStorage>) -> Result<String> {
    let mut object = JsonMap::new();
    object.insert(
        "contactName".to_string(),
        JsonValue::String(contact.contact_name().to_string()),
    );
    object.insert(
        "emailAddress".to_string(),
        JsonValue::String(contact.email_address().to_string()),
    );
    if let Some(role) = contact.role() {
        object.insert("role".to_string(), JsonValue::String(role.to_string()));
    }
    if let Some(value) = contact.website().as_ref() {
        object.insert("website".to_string(), JsonValue::String(value.clone()));
    }
    if let Some(kind) = contact.contact_type() {
        object.insert("type".to_string(), JsonValue::String(kind.to_string()));
    }
    if let Some(value) = contact.phone().as_ref() {
        object.insert("phone".to_string(), JsonValue::String(value.clone()));
    }
    if let Some(value) = contact.organization().as_ref() {
        object.insert("organization".to_string(), JsonValue::String(value.clone()));
    }
    if let Some(address) = contact.address() {
        object.insert(
            "address".to_string(),
            attribute_to_json(
                &AttributeValue::Map(
                    address
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect::<HashMap<_, _>>(),
                ),
                &HashMap::new(),
            )?,
        );
    }
    Ok(JsonValue::Object(object).to_string())
}

fn read_large_string_scalar(batch: &RecordBatch, name: &str, row: usize) -> Result<String> {
    let array = downcast_required::<LargeStringArray>(batch, name)?;
    Ok(array.value(row).to_string())
}

fn read_large_string_optional(
    batch: &RecordBatch,
    name: &str,
    row: usize,
) -> Result<Option<String>> {
    let array = downcast_required::<LargeStringArray>(batch, name)?;
    Ok((!array.is_null(row)).then(|| array.value(row).to_string()))
}

fn read_string_scalar(batch: &RecordBatch, name: &str, row: usize) -> Result<String> {
    let array = downcast_required::<StringArray>(batch, name)?;
    Ok(array.value(row).to_string())
}

fn read_string_optional(batch: &RecordBatch, name: &str, row: usize) -> Result<Option<String>> {
    let array = downcast_required::<StringArray>(batch, name)?;
    Ok((!array.is_null(row)).then(|| array.value(row).to_string()))
}

fn read_large_string_array_optional(
    array: Option<&LargeStringArray>,
    row: usize,
) -> Option<String> {
    array.and_then(|array| (!array.is_null(row)).then(|| array.value(row).to_string()))
}

fn read_f64_array_optional(array: Option<&Float64Array>, row: usize) -> Option<f64> {
    array.and_then(|array| (!array.is_null(row)).then(|| array.value(row)))
}

fn read_bool_array_optional(
    array: Option<&arrow::array::BooleanArray>,
    row: usize,
) -> Option<bool> {
    array.and_then(|array| (!array.is_null(row)).then(|| array.value(row)))
}

fn read_fixed_size_f64_required<const N: usize>(
    batch: &RecordBatch,
    name: &str,
    row: usize,
) -> Result<[f64; N]> {
    read_fixed_size_f64_optional::<N>(batch, name, row)?
        .ok_or_else(|| Error::Conversion(format!("missing required fixed-size list {name}")))
}

fn read_fixed_size_f64_optional<const N: usize>(
    batch: &RecordBatch,
    name: &str,
    row: usize,
) -> Result<Option<[f64; N]>> {
    let array = downcast_required::<FixedSizeListArray>(batch, name)?;
    if array.is_null(row) {
        return Ok(None);
    }
    let values = array.value(row);
    let values = values
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| Error::Conversion(format!("fixed-size list {name} does not contain f64")))?;
    let slice = values.values().as_ref();
    Ok(Some(slice.try_into().map_err(|_| {
        Error::Conversion(format!("fixed-size list {name} does not have length {N}"))
    })?))
}

fn read_fixed_size_list_array_optional<const N: usize>(
    array: &FixedSizeListArray,
    name: &str,
    row: usize,
) -> Result<Option<[f64; N]>> {
    if array.is_null(row) {
        return Ok(None);
    }
    let values = array.value(row);
    let values = values
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| Error::Conversion(format!("fixed-size list {name} does not contain f64")))?;
    let slice = values.values().as_ref();
    Ok(Some(slice.try_into().map_err(|_| {
        Error::Conversion(format!("fixed-size list {name} does not have length {N}"))
    })?))
}

fn ensure_strictly_increasing_u64(
    previous: Option<u64>,
    current: u64,
    field_name: &str,
) -> Result<()> {
    if let Some(previous) = previous
        && current <= previous
    {
        return Err(Error::Conversion(format!(
            "{field_name} must be strictly increasing in canonical order, found {current} after {previous}"
        )));
    }
    Ok(())
}

fn bind_vertex_columns<'a>(batch: &'a RecordBatch, id_name: &str) -> Result<VertexColumns<'a>> {
    Ok(VertexColumns {
        vertex_id: downcast_required::<UInt64Array>(batch, id_name)?,
        x: downcast_required::<Float64Array>(batch, "x")?,
        y: downcast_required::<Float64Array>(batch, "y")?,
        z: downcast_required::<Float64Array>(batch, "z")?,
    })
}

fn bind_uv_columns(batch: &RecordBatch) -> Result<UvColumns<'_>> {
    Ok(UvColumns {
        uv_id: downcast_required::<UInt64Array>(batch, "uv_id")?,
        u: downcast_required::<Float64Array>(batch, "u")?,
        v: downcast_required::<Float64Array>(batch, "v")?,
    })
}

fn bind_semantic_columns<'a>(
    batch: &'a RecordBatch,
    projection: &ProjectionLayout,
) -> Result<SemanticColumns<'a>> {
    Ok(SemanticColumns {
        semantic_id: downcast_required::<UInt64Array>(batch, "semantic_id")?,
        semantic_type: downcast_required::<StringArray>(batch, "semantic_type")?,
        attributes: projection
            .semantic_attributes
            .as_ref()
            .map(|_| downcast_required::<StructArray>(batch, "attributes"))
            .transpose()?,
    })
}

fn bind_material_columns<'a>(
    batch: &'a RecordBatch,
    projection: &ProjectionLayout,
) -> Result<MaterialColumns<'a>> {
    Ok(MaterialColumns {
        material_id: downcast_required::<UInt64Array>(batch, "material_id")?,
        name: downcast_required::<LargeStringArray>(batch, FIELD_MATERIAL_NAME)?,
        ambient_intensity: has_projection_field(
            projection.material_payload.as_ref(),
            FIELD_MATERIAL_AMBIENT_INTENSITY,
        )
        .then(|| downcast_required::<Float64Array>(batch, FIELD_MATERIAL_AMBIENT_INTENSITY))
        .transpose()?,
        diffuse_color: has_projection_field(
            projection.material_payload.as_ref(),
            FIELD_MATERIAL_DIFFUSE_COLOR,
        )
        .then(|| downcast_required::<LargeStringArray>(batch, FIELD_MATERIAL_DIFFUSE_COLOR))
        .transpose()?,
        emissive_color: has_projection_field(
            projection.material_payload.as_ref(),
            FIELD_MATERIAL_EMISSIVE_COLOR,
        )
        .then(|| downcast_required::<LargeStringArray>(batch, FIELD_MATERIAL_EMISSIVE_COLOR))
        .transpose()?,
        specular_color: has_projection_field(
            projection.material_payload.as_ref(),
            FIELD_MATERIAL_SPECULAR_COLOR,
        )
        .then(|| downcast_required::<LargeStringArray>(batch, FIELD_MATERIAL_SPECULAR_COLOR))
        .transpose()?,
        shininess: has_projection_field(projection.material_payload.as_ref(), FIELD_MATERIAL_SHININESS)
            .then(|| downcast_required::<Float64Array>(batch, FIELD_MATERIAL_SHININESS))
            .transpose()?,
        transparency: has_projection_field(
            projection.material_payload.as_ref(),
            FIELD_MATERIAL_TRANSPARENCY,
        )
        .then(|| downcast_required::<Float64Array>(batch, FIELD_MATERIAL_TRANSPARENCY))
        .transpose()?,
        is_smooth: has_projection_field(projection.material_payload.as_ref(), FIELD_MATERIAL_IS_SMOOTH)
            .then(|| {
                downcast_required::<arrow::array::BooleanArray>(batch, FIELD_MATERIAL_IS_SMOOTH)
            })
            .transpose()?,
    })
}

fn bind_texture_columns<'a>(
    batch: &'a RecordBatch,
    projection: &ProjectionLayout,
) -> Result<TextureColumns<'a>> {
    Ok(TextureColumns {
        texture_id: downcast_required::<UInt64Array>(batch, "texture_id")?,
        image_uri: downcast_required::<LargeStringArray>(batch, "image_uri")?,
        image_type: downcast_required::<LargeStringArray>(batch, FIELD_TEXTURE_IMAGE_TYPE)?,
        wrap_mode: has_projection_field(projection.texture_payload.as_ref(), FIELD_TEXTURE_WRAP_MODE)
            .then(|| downcast_required::<LargeStringArray>(batch, FIELD_TEXTURE_WRAP_MODE))
            .transpose()?,
        texture_type: has_projection_field(projection.texture_payload.as_ref(), FIELD_TEXTURE_TEXTURE_TYPE)
            .then(|| downcast_required::<LargeStringArray>(batch, FIELD_TEXTURE_TEXTURE_TYPE))
            .transpose()?,
        border_color: has_projection_field(projection.texture_payload.as_ref(), FIELD_TEXTURE_BORDER_COLOR)
            .then(|| downcast_required::<LargeStringArray>(batch, FIELD_TEXTURE_BORDER_COLOR))
            .transpose()?,
    })
}

fn bind_template_geometry_columns(batch: &RecordBatch) -> Result<TemplateGeometryColumns<'_>> {
    Ok(TemplateGeometryColumns {
        template_geometry_id: downcast_required::<UInt64Array>(batch, "template_geometry_id")?,
        geometry_type: downcast_required::<StringArray>(batch, "geometry_type")?,
        lod: downcast_required::<StringArray>(batch, "lod")?,
    })
}

fn bind_geometry_columns(batch: &RecordBatch) -> Result<GeometryColumns<'_>> {
    Ok(GeometryColumns {
        geometry_id: downcast_required::<UInt64Array>(batch, "geometry_id")?,
        cityobject_ix: downcast_required::<UInt64Array>(batch, "cityobject_ix")?,
        geometry_ordinal: downcast_required::<UInt32Array>(batch, "geometry_ordinal")?,
        geometry_type: downcast_required::<StringArray>(batch, "geometry_type")?,
        lod: downcast_required::<StringArray>(batch, "lod")?,
    })
}

fn bind_geometry_instance_columns(batch: &RecordBatch) -> Result<GeometryInstanceColumns<'_>> {
    Ok(GeometryInstanceColumns {
        geometry_id: downcast_required::<UInt64Array>(batch, "geometry_id")?,
        cityobject_ix: downcast_required::<UInt64Array>(batch, "cityobject_ix")?,
        geometry_ordinal: downcast_required::<UInt32Array>(batch, "geometry_ordinal")?,
        lod: downcast_required::<StringArray>(batch, "lod")?,
        template_geometry_id: downcast_required::<UInt64Array>(batch, "template_geometry_id")?,
        reference_point_vertex_id: downcast_required::<UInt64Array>(
            batch,
            "reference_point_vertex_id",
        )?,
        transform_matrix: downcast_required::<FixedSizeListArray>(batch, "transform_matrix")?,
    })
}

fn bind_cityobject_columns<'a>(
    batch: &'a RecordBatch,
    projection: &ProjectionLayout,
) -> Result<CityObjectColumns<'a>> {
    Ok(CityObjectColumns {
        cityobject_id: downcast_required::<LargeStringArray>(batch, "cityobject_id")?,
        cityobject_ix: downcast_required::<UInt64Array>(batch, "cityobject_ix")?,
        object_type: downcast_required::<StringArray>(batch, "object_type")?,
        geographical_extent: downcast_required::<FixedSizeListArray>(batch, "geographical_extent")?,
        attributes: projection
            .cityobject_attributes
            .as_ref()
            .map(|_| downcast_required::<StructArray>(batch, "attributes"))
            .transpose()?,
        extra: projection
            .cityobject_extra
            .as_ref()
            .map(|_| downcast_required::<StructArray>(batch, "extra"))
            .transpose()?,
    })
}

fn list_u64_value(array: &ListArray, row: usize) -> Result<Vec<u64>> {
    let values = array.value(row);
    let values = values
        .as_any()
        .downcast_ref::<UInt64Array>()
        .ok_or_else(|| Error::Conversion("list child is not u64".to_string()))?;
    Ok(values.values().to_vec())
}

fn list_u32_optional_value(array: &ListArray, row: usize) -> Result<Option<Vec<u32>>> {
    if array.is_null(row) {
        return Ok(None);
    }
    let values = array.value(row);
    let values = values
        .as_any()
        .downcast_ref::<UInt32Array>()
        .ok_or_else(|| Error::Conversion("list child is not u32".to_string()))?;
    Ok(Some(values.values().to_vec()))
}

fn downcast_required<'a, T: Array + 'static>(batch: &'a RecordBatch, name: &str) -> Result<&'a T> {
    batch
        .column_by_name(name)
        .ok_or_else(|| Error::MissingField(name.to_string()))?
        .as_any()
        .downcast_ref::<T>()
        .ok_or_else(|| Error::Conversion(format!("field {name} has unexpected array type")))
}
