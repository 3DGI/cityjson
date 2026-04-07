use arrow::datatypes::{DataType, Field, Fields, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;

pub const PACKAGE_SCHEMA_ID: &str = "cityarrow.package.v3alpha1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CityArrowPackageVersion {
    #[serde(rename = "cityarrow.package.v3alpha1")]
    V3Alpha1,
}

impl CityArrowPackageVersion {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        PACKAGE_SCHEMA_ID
    }
}

impl Display for CityArrowPackageVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageVersionParseError {
    found: String,
}

impl PackageVersionParseError {
    #[must_use]
    pub fn new(found: impl Into<String>) -> Self {
        Self {
            found: found.into(),
        }
    }
}

impl Display for PackageVersionParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported cityarrow package schema id: {}", self.found)
    }
}

impl std::error::Error for PackageVersionParseError {}

impl FromStr for CityArrowPackageVersion {
    type Err = PackageVersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            PACKAGE_SCHEMA_ID => Ok(Self::V3Alpha1),
            other => Err(PackageVersionParseError::new(other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityArrowHeader {
    pub package_version: CityArrowPackageVersion,
    pub citymodel_id: String,
    pub cityjson_version: String,
}

impl CityArrowHeader {
    #[must_use]
    pub fn new(
        package_version: CityArrowPackageVersion,
        citymodel_id: impl Into<String>,
        cityjson_version: impl Into<String>,
    ) -> Self {
        Self {
            package_version,
            citymodel_id: citymodel_id.into(),
            cityjson_version: cityjson_version.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectedValueSpec {
    Null,
    Boolean,
    UInt64,
    Int64,
    Float64,
    Utf8,
    GeometryRef,
    List {
        item_nullable: bool,
        item: Box<ProjectedValueSpec>,
    },
    Struct(ProjectedStructSpec),
}

impl ProjectedValueSpec {
    #[must_use]
    pub fn to_arrow_type(&self) -> DataType {
        match self {
            Self::Null => DataType::Null,
            Self::Boolean => DataType::Boolean,
            Self::UInt64 | Self::GeometryRef => DataType::UInt64,
            Self::Int64 => DataType::Int64,
            Self::Float64 => DataType::Float64,
            Self::Utf8 => DataType::LargeUtf8,
            Self::List {
                item_nullable,
                item,
            } => DataType::List(Arc::new(Field::new_list_field(
                item.to_arrow_type(),
                *item_nullable,
            ))),
            Self::Struct(fields) => DataType::Struct(fields.to_arrow_fields()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectedStructSpec {
    pub fields: Vec<ProjectedFieldSpec>,
}

impl ProjectedStructSpec {
    #[must_use]
    pub fn new(fields: Vec<ProjectedFieldSpec>) -> Self {
        Self { fields }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    #[must_use]
    pub fn to_arrow_fields(&self) -> Fields {
        self.fields
            .iter()
            .map(ProjectedFieldSpec::to_arrow_field)
            .map(Arc::new)
            .collect()
    }

    #[must_use]
    pub fn field(&self, name: &str) -> Option<&ProjectedFieldSpec> {
        self.fields.iter().find(|field| field.name == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectedFieldSpec {
    pub name: String,
    pub value: ProjectedValueSpec,
    pub nullable: bool,
}

impl ProjectedFieldSpec {
    #[must_use]
    pub fn new(name: impl Into<String>, value: ProjectedValueSpec, nullable: bool) -> Self {
        Self {
            name: name.into(),
            value,
            nullable,
        }
    }

    #[must_use]
    pub fn to_arrow_field(&self) -> Field {
        Field::new(self.name.clone(), self.value.to_arrow_type(), self.nullable)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionLayout {
    pub root_extra: Option<ProjectedStructSpec>,
    pub metadata_extra: Option<ProjectedStructSpec>,
    pub metadata_point_of_contact_address: Option<ProjectedStructSpec>,
    pub cityobject_attributes: Option<ProjectedStructSpec>,
    pub cityobject_extra: Option<ProjectedStructSpec>,
    pub geometry_extra: Option<ProjectedStructSpec>,
    pub semantic_attributes: Option<ProjectedStructSpec>,
    pub material_payload: Option<ProjectedStructSpec>,
    pub texture_payload: Option<ProjectedStructSpec>,
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct CityModelArrowParts {
    pub header: CityArrowHeader,
    pub projection: ProjectionLayout,
    pub metadata: RecordBatch,
    pub transform: Option<RecordBatch>,
    pub extensions: Option<RecordBatch>,
    pub vertices: RecordBatch,
    pub cityobjects: RecordBatch,
    pub cityobject_children: Option<RecordBatch>,
    pub geometries: RecordBatch,
    pub geometry_boundaries: RecordBatch,
    pub geometry_instances: Option<RecordBatch>,
    pub template_vertices: Option<RecordBatch>,
    pub template_geometries: Option<RecordBatch>,
    pub template_geometry_boundaries: Option<RecordBatch>,
    pub semantics: Option<RecordBatch>,
    pub semantic_children: Option<RecordBatch>,
    pub geometry_surface_semantics: Option<RecordBatch>,
    pub geometry_point_semantics: Option<RecordBatch>,
    pub geometry_linestring_semantics: Option<RecordBatch>,
    pub template_geometry_semantics: Option<RecordBatch>,
    pub materials: Option<RecordBatch>,
    pub geometry_surface_materials: Option<RecordBatch>,
    pub template_geometry_materials: Option<RecordBatch>,
    pub textures: Option<RecordBatch>,
    pub texture_vertices: Option<RecordBatch>,
    pub geometry_ring_textures: Option<RecordBatch>,
    pub template_geometry_ring_textures: Option<RecordBatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageTableRef {
    pub name: String,
    pub offset: u64,
    pub length: u64,
    pub rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    pub package_schema: CityArrowPackageVersion,
    pub cityjson_version: String,
    pub citymodel_id: String,
    pub projection: ProjectionLayout,
    pub tables: Vec<PackageTableRef>,
}

impl PackageManifest {
    #[must_use]
    pub fn new(
        citymodel_id: impl Into<String>,
        cityjson_version: impl Into<String>,
        projection: ProjectionLayout,
    ) -> Self {
        Self {
            package_schema: CityArrowPackageVersion::V3Alpha1,
            cityjson_version: cityjson_version.into(),
            citymodel_id: citymodel_id.into(),
            projection,
            tables: Vec::new(),
        }
    }
}

impl From<&PackageManifest> for CityArrowHeader {
    fn from(value: &PackageManifest) -> Self {
        Self::new(
            value.package_schema,
            value.citymodel_id.clone(),
            value.cityjson_version.clone(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSchemaSet {
    pub metadata: SchemaRef,
    pub transform: SchemaRef,
    pub extensions: SchemaRef,
    pub vertices: SchemaRef,
    pub cityobjects: SchemaRef,
    pub cityobject_children: SchemaRef,
    pub geometries: SchemaRef,
    pub geometry_boundaries: SchemaRef,
    pub geometry_instances: SchemaRef,
    pub template_vertices: SchemaRef,
    pub template_geometries: SchemaRef,
    pub template_geometry_boundaries: SchemaRef,
    pub semantics: SchemaRef,
    pub semantic_children: SchemaRef,
    pub geometry_surface_semantics: SchemaRef,
    pub geometry_point_semantics: SchemaRef,
    pub geometry_linestring_semantics: SchemaRef,
    pub template_geometry_semantics: SchemaRef,
    pub materials: SchemaRef,
    pub geometry_surface_materials: SchemaRef,
    pub template_geometry_materials: SchemaRef,
    pub textures: SchemaRef,
    pub texture_vertices: SchemaRef,
    pub geometry_ring_textures: SchemaRef,
    pub template_geometry_ring_textures: SchemaRef,
}

impl Default for CanonicalSchemaSet {
    fn default() -> Self {
        canonical_schema_set(&ProjectionLayout::default())
    }
}

#[must_use]
pub fn canonical_schema_set(layout: &ProjectionLayout) -> CanonicalSchemaSet {
    CanonicalSchemaSet {
        metadata: schema_ref(metadata_fields(layout)),
        transform: schema_ref(transform_fields()),
        extensions: schema_ref(extensions_fields()),
        vertices: schema_ref(vertices_fields()),
        cityobjects: schema_ref(cityobjects_fields(layout)),
        cityobject_children: schema_ref(cityobject_children_fields()),
        geometries: schema_ref(geometries_fields(layout)),
        geometry_boundaries: schema_ref(geometry_boundaries_fields()),
        geometry_instances: schema_ref(geometry_instances_fields(layout)),
        template_vertices: schema_ref(template_vertices_fields()),
        template_geometries: schema_ref(template_geometries_fields(layout)),
        template_geometry_boundaries: schema_ref(template_geometry_boundaries_fields()),
        semantics: schema_ref(semantics_fields(layout)),
        semantic_children: schema_ref(semantic_children_fields()),
        geometry_surface_semantics: schema_ref(geometry_surface_semantics_fields()),
        geometry_point_semantics: schema_ref(geometry_point_semantics_fields()),
        geometry_linestring_semantics: schema_ref(geometry_linestring_semantics_fields()),
        template_geometry_semantics: schema_ref(template_geometry_semantics_fields()),
        materials: schema_ref(materials_fields(layout)),
        geometry_surface_materials: schema_ref(geometry_surface_materials_fields()),
        template_geometry_materials: schema_ref(template_geometry_materials_fields()),
        textures: schema_ref(textures_fields(layout)),
        texture_vertices: schema_ref(texture_vertices_fields()),
        geometry_ring_textures: schema_ref(geometry_ring_textures_fields()),
        template_geometry_ring_textures: schema_ref(template_geometry_ring_textures_fields()),
    }
}

fn schema_ref(fields: Vec<Field>) -> SchemaRef {
    Arc::new(Schema::new(fields))
}

fn fixed_size_list_field(
    name: &str,
    item_type: DataType,
    item_nullable: bool,
    size: i32,
    nullable: bool,
) -> Field {
    Field::new(
        name,
        DataType::FixedSizeList(
            Arc::new(Field::new_list_field(item_type, item_nullable)),
            size,
        ),
        nullable,
    )
}

fn list_field(name: &str, item_type: DataType, item_nullable: bool, nullable: bool) -> Field {
    Field::new(
        name,
        DataType::List(Arc::new(Field::new_list_field(item_type, item_nullable))),
        nullable,
    )
}

fn projected_struct_field(name: &str, layout: Option<&ProjectedStructSpec>) -> Option<Field> {
    layout.map(|layout| Field::new(name, DataType::Struct(layout.to_arrow_fields()), true))
}

fn point_of_contact_field(layout: &ProjectionLayout) -> Field {
    let mut fields = vec![
        Field::new("contact_name", DataType::LargeUtf8, false),
        Field::new("email_address", DataType::LargeUtf8, false),
        Field::new("role", DataType::Utf8, true),
        Field::new("website", DataType::LargeUtf8, true),
        Field::new("contact_type", DataType::Utf8, true),
        Field::new("phone", DataType::LargeUtf8, true),
        Field::new("organization", DataType::LargeUtf8, true),
    ];
    if let Some(field) =
        projected_struct_field("address", layout.metadata_point_of_contact_address.as_ref())
    {
        fields.push(field);
    }
    Field::new(
        "point_of_contact",
        DataType::Struct(fields.into_iter().map(Arc::new).collect()),
        true,
    )
}

fn projected_payload_fields(layout: Option<&ProjectedStructSpec>) -> Vec<Field> {
    layout
        .map(|layout| {
            layout
                .fields
                .iter()
                .map(ProjectedFieldSpec::to_arrow_field)
                .collect()
        })
        .unwrap_or_default()
}

fn metadata_fields(layout: &ProjectionLayout) -> Vec<Field> {
    let mut fields = vec![
        Field::new("citymodel_id", DataType::LargeUtf8, false),
        Field::new("cityjson_version", DataType::Utf8, false),
        Field::new("citymodel_kind", DataType::Utf8, false),
        Field::new("feature_root_id", DataType::LargeUtf8, true),
        Field::new("identifier", DataType::LargeUtf8, true),
        Field::new("title", DataType::LargeUtf8, true),
        Field::new("reference_system", DataType::LargeUtf8, true),
        fixed_size_list_field("geographical_extent", DataType::Float64, false, 6, true),
        Field::new("reference_date", DataType::Utf8, true),
        Field::new("default_material_theme", DataType::Utf8, true),
        Field::new("default_texture_theme", DataType::Utf8, true),
        point_of_contact_field(layout),
    ];
    if let Some(field) = projected_struct_field("root_extra", layout.root_extra.as_ref()) {
        fields.push(field);
    }
    if let Some(field) = projected_struct_field("metadata_extra", layout.metadata_extra.as_ref()) {
        fields.push(field);
    }
    fields
}

fn transform_fields() -> Vec<Field> {
    vec![
        fixed_size_list_field("scale", DataType::Float64, false, 3, false),
        fixed_size_list_field("translate", DataType::Float64, false, 3, false),
    ]
}

fn extensions_fields() -> Vec<Field> {
    vec![
        Field::new("extension_name", DataType::Utf8, false),
        Field::new("uri", DataType::LargeUtf8, false),
        Field::new("version", DataType::Utf8, true),
    ]
}

fn vertices_fields() -> Vec<Field> {
    vec![
        Field::new("vertex_id", DataType::UInt64, false),
        Field::new("x", DataType::Float64, false),
        Field::new("y", DataType::Float64, false),
        Field::new("z", DataType::Float64, false),
    ]
}

fn cityobjects_fields(layout: &ProjectionLayout) -> Vec<Field> {
    let mut fields = vec![
        Field::new("cityobject_id", DataType::LargeUtf8, false),
        Field::new("cityobject_ix", DataType::UInt64, false),
        Field::new("object_type", DataType::Utf8, false),
        fixed_size_list_field("geographical_extent", DataType::Float64, false, 6, true),
    ];
    if let Some(field) = projected_struct_field("attributes", layout.cityobject_attributes.as_ref())
    {
        fields.push(field);
    }
    if let Some(field) = projected_struct_field("extra", layout.cityobject_extra.as_ref()) {
        fields.push(field);
    }
    fields
}

fn cityobject_children_fields() -> Vec<Field> {
    vec![
        Field::new("parent_cityobject_ix", DataType::UInt64, false),
        Field::new("child_ordinal", DataType::UInt32, false),
        Field::new("child_cityobject_ix", DataType::UInt64, false),
    ]
}

fn geometries_fields(layout: &ProjectionLayout) -> Vec<Field> {
    let mut fields = vec![
        Field::new("geometry_id", DataType::UInt64, false),
        Field::new("cityobject_ix", DataType::UInt64, false),
        Field::new("geometry_ordinal", DataType::UInt32, false),
        Field::new("geometry_type", DataType::Utf8, false),
        Field::new("lod", DataType::Utf8, true),
    ];
    if let Some(field) = projected_struct_field("extra", layout.geometry_extra.as_ref()) {
        fields.push(field);
    }
    fields
}

fn geometry_boundaries_fields() -> Vec<Field> {
    vec![
        Field::new("geometry_id", DataType::UInt64, false),
        list_field("vertex_indices", DataType::UInt64, false, false),
        list_field("line_lengths", DataType::UInt32, false, true),
        list_field("ring_lengths", DataType::UInt32, false, true),
        list_field("surface_lengths", DataType::UInt32, false, true),
        list_field("shell_lengths", DataType::UInt32, false, true),
        list_field("solid_lengths", DataType::UInt32, false, true),
    ]
}

fn geometry_instances_fields(layout: &ProjectionLayout) -> Vec<Field> {
    let mut fields = vec![
        Field::new("geometry_id", DataType::UInt64, false),
        Field::new("cityobject_ix", DataType::UInt64, false),
        Field::new("geometry_ordinal", DataType::UInt32, false),
        Field::new("lod", DataType::Utf8, true),
        Field::new("template_geometry_id", DataType::UInt64, false),
        Field::new("reference_point_vertex_id", DataType::UInt64, false),
        fixed_size_list_field("transform_matrix", DataType::Float64, false, 16, true),
    ];
    if let Some(field) = projected_struct_field("extra", layout.geometry_extra.as_ref()) {
        fields.push(field);
    }
    fields
}

fn template_vertices_fields() -> Vec<Field> {
    vec![
        Field::new("template_vertex_id", DataType::UInt64, false),
        Field::new("x", DataType::Float64, false),
        Field::new("y", DataType::Float64, false),
        Field::new("z", DataType::Float64, false),
    ]
}

fn template_geometries_fields(layout: &ProjectionLayout) -> Vec<Field> {
    let mut fields = vec![
        Field::new("template_geometry_id", DataType::UInt64, false),
        Field::new("geometry_type", DataType::Utf8, false),
        Field::new("lod", DataType::Utf8, true),
    ];
    if let Some(field) = projected_struct_field("extra", layout.geometry_extra.as_ref()) {
        fields.push(field);
    }
    fields
}

fn template_geometry_boundaries_fields() -> Vec<Field> {
    vec![
        Field::new("template_geometry_id", DataType::UInt64, false),
        list_field("vertex_indices", DataType::UInt64, false, false),
        list_field("line_lengths", DataType::UInt32, false, true),
        list_field("ring_lengths", DataType::UInt32, false, true),
        list_field("surface_lengths", DataType::UInt32, false, true),
        list_field("shell_lengths", DataType::UInt32, false, true),
        list_field("solid_lengths", DataType::UInt32, false, true),
    ]
}

fn semantics_fields(layout: &ProjectionLayout) -> Vec<Field> {
    let mut fields = vec![
        Field::new("semantic_id", DataType::UInt64, false),
        Field::new("semantic_type", DataType::Utf8, false),
    ];
    if let Some(field) = projected_struct_field("attributes", layout.semantic_attributes.as_ref()) {
        fields.push(field);
    }
    fields
}

fn semantic_children_fields() -> Vec<Field> {
    vec![
        Field::new("parent_semantic_id", DataType::UInt64, false),
        Field::new("child_ordinal", DataType::UInt32, false),
        Field::new("child_semantic_id", DataType::UInt64, false),
    ]
}

fn geometry_surface_semantics_fields() -> Vec<Field> {
    vec![
        Field::new("geometry_id", DataType::UInt64, false),
        Field::new("surface_ordinal", DataType::UInt32, false),
        Field::new("semantic_id", DataType::UInt64, true),
    ]
}

fn geometry_point_semantics_fields() -> Vec<Field> {
    vec![
        Field::new("geometry_id", DataType::UInt64, false),
        Field::new("point_ordinal", DataType::UInt32, false),
        Field::new("semantic_id", DataType::UInt64, true),
    ]
}

fn geometry_linestring_semantics_fields() -> Vec<Field> {
    vec![
        Field::new("geometry_id", DataType::UInt64, false),
        Field::new("linestring_ordinal", DataType::UInt32, false),
        Field::new("semantic_id", DataType::UInt64, true),
    ]
}

fn template_geometry_semantics_fields() -> Vec<Field> {
    vec![
        Field::new("template_geometry_id", DataType::UInt64, false),
        Field::new("primitive_type", DataType::Utf8, false),
        Field::new("primitive_ordinal", DataType::UInt32, false),
        Field::new("semantic_id", DataType::UInt64, true),
    ]
}

fn materials_fields(layout: &ProjectionLayout) -> Vec<Field> {
    let mut fields = vec![Field::new("material_id", DataType::UInt64, false)];
    fields.extend(projected_payload_fields(layout.material_payload.as_ref()));
    fields
}

fn geometry_surface_materials_fields() -> Vec<Field> {
    vec![
        Field::new("geometry_id", DataType::UInt64, false),
        Field::new("surface_ordinal", DataType::UInt32, false),
        Field::new("theme", DataType::Utf8, false),
        Field::new("material_id", DataType::UInt64, false),
    ]
}

fn template_geometry_materials_fields() -> Vec<Field> {
    vec![
        Field::new("template_geometry_id", DataType::UInt64, false),
        Field::new("primitive_type", DataType::Utf8, false),
        Field::new("primitive_ordinal", DataType::UInt32, false),
        Field::new("theme", DataType::Utf8, false),
        Field::new("material_id", DataType::UInt64, false),
    ]
}

fn textures_fields(layout: &ProjectionLayout) -> Vec<Field> {
    let mut fields = vec![
        Field::new("texture_id", DataType::UInt64, false),
        Field::new("image_uri", DataType::LargeUtf8, false),
    ];
    fields.extend(projected_payload_fields(layout.texture_payload.as_ref()));
    fields
}

fn texture_vertices_fields() -> Vec<Field> {
    vec![
        Field::new("uv_id", DataType::UInt64, false),
        Field::new("u", DataType::Float64, false),
        Field::new("v", DataType::Float64, false),
    ]
}

fn geometry_ring_textures_fields() -> Vec<Field> {
    vec![
        Field::new("geometry_id", DataType::UInt64, false),
        Field::new("surface_ordinal", DataType::UInt32, false),
        Field::new("ring_ordinal", DataType::UInt32, false),
        Field::new("theme", DataType::Utf8, false),
        Field::new("texture_id", DataType::UInt64, false),
        list_field("uv_indices", DataType::UInt64, false, false),
    ]
}

fn template_geometry_ring_textures_fields() -> Vec<Field> {
    vec![
        Field::new("template_geometry_id", DataType::UInt64, false),
        Field::new("surface_ordinal", DataType::UInt32, false),
        Field::new("ring_ordinal", DataType::UInt32, false),
        Field::new("theme", DataType::Utf8, false),
        Field::new("texture_id", DataType::UInt64, false),
        list_field("uv_indices", DataType::UInt64, false, false),
    ]
}
