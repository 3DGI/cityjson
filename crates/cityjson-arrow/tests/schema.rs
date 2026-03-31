use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayData, ArrayRef, FixedSizeListArray, Float64Array, LargeStringArray, StringArray,
};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use cityarrow::package::write_package_dir;
use cityarrow::schema::{
    CityArrowHeader, CityArrowPackageVersion, CityModelArrowParts, PackageManifest, PackageTables,
    ProjectedFieldSpec, ProjectedValueType, ProjectionLayout, canonical_schema_set,
};
use tempfile::tempdir;

const CANONICAL_SCHEMA_LOCK: &str = r#"metadata
  citymodel_id: LargeUtf8!
  cityjson_version: Utf8!
  citymodel_kind: Utf8!
  identifier: LargeUtf8?
  title: LargeUtf8?
  reference_system: LargeUtf8?
  geographical_extent: FixedSizeList<Float64!, 6>?

transform
  citymodel_id: LargeUtf8!
  scale: FixedSizeList<Float64!, 3>!
  translate: FixedSizeList<Float64!, 3>!

extensions
  citymodel_id: LargeUtf8!
  extension_name: Utf8!
  uri: LargeUtf8!
  version: Utf8?

vertices
  citymodel_id: LargeUtf8!
  vertex_id: UInt64!
  x: Float64!
  y: Float64!
  z: Float64!

cityobjects
  citymodel_id: LargeUtf8!
  cityobject_id: LargeUtf8!
  cityobject_ix: UInt64!
  object_type: Utf8!
  geographical_extent: FixedSizeList<Float64!, 6>?

cityobject_children
  citymodel_id: LargeUtf8!
  parent_cityobject_id: LargeUtf8!
  child_ordinal: UInt32!
  child_cityobject_id: LargeUtf8!

geometries
  citymodel_id: LargeUtf8!
  geometry_id: UInt64!
  cityobject_id: LargeUtf8!
  geometry_ordinal: UInt32!
  geometry_type: Utf8!
  lod: Utf8?

geometry_boundaries
  citymodel_id: LargeUtf8!
  geometry_id: UInt64!
  vertex_indices: List<UInt64!>!
  line_lengths: List<UInt32!>?
  ring_lengths: List<UInt32!>?
  surface_lengths: List<UInt32!>?
  shell_lengths: List<UInt32!>?
  solid_lengths: List<UInt32!>?

geometry_instances
  citymodel_id: LargeUtf8!
  geometry_id: UInt64!
  cityobject_id: LargeUtf8!
  geometry_ordinal: UInt32!
  lod: Utf8?
  template_geometry_id: UInt64!
  reference_point_vertex_id: UInt64!
  transform_matrix: FixedSizeList<Float64!, 16>?

template_vertices
  citymodel_id: LargeUtf8!
  template_vertex_id: UInt64!
  x: Float64!
  y: Float64!
  z: Float64!

template_geometries
  citymodel_id: LargeUtf8!
  template_geometry_id: UInt64!
  geometry_type: Utf8!
  lod: Utf8?

template_geometry_boundaries
  citymodel_id: LargeUtf8!
  template_geometry_id: UInt64!
  vertex_indices: List<UInt64!>!
  line_lengths: List<UInt32!>?
  ring_lengths: List<UInt32!>?
  surface_lengths: List<UInt32!>?
  shell_lengths: List<UInt32!>?
  solid_lengths: List<UInt32!>?

semantics
  citymodel_id: LargeUtf8!
  semantic_id: UInt64!
  semantic_type: Utf8!

semantic_children
  citymodel_id: LargeUtf8!
  parent_semantic_id: UInt64!
  child_ordinal: UInt32!
  child_semantic_id: UInt64!

geometry_surface_semantics
  citymodel_id: LargeUtf8!
  geometry_id: UInt64!
  surface_ordinal: UInt32!
  semantic_id: UInt64?

geometry_point_semantics
  citymodel_id: LargeUtf8!
  geometry_id: UInt64!
  point_ordinal: UInt32!
  semantic_id: UInt64?

geometry_linestring_semantics
  citymodel_id: LargeUtf8!
  geometry_id: UInt64!
  linestring_ordinal: UInt32!
  semantic_id: UInt64?

template_geometry_semantics
  citymodel_id: LargeUtf8!
  template_geometry_id: UInt64!
  primitive_type: Utf8!
  primitive_ordinal: UInt32!
  semantic_id: UInt64?

materials
  citymodel_id: LargeUtf8!
  material_id: UInt64!

geometry_surface_materials
  citymodel_id: LargeUtf8!
  geometry_id: UInt64!
  surface_ordinal: UInt32!
  theme: Utf8!
  material_id: UInt64!

geometry_point_materials
  citymodel_id: LargeUtf8!
  geometry_id: UInt64!
  point_ordinal: UInt32!
  theme: Utf8!
  material_id: UInt64!

geometry_linestring_materials
  citymodel_id: LargeUtf8!
  geometry_id: UInt64!
  linestring_ordinal: UInt32!
  theme: Utf8!
  material_id: UInt64!

template_geometry_materials
  citymodel_id: LargeUtf8!
  template_geometry_id: UInt64!
  primitive_type: Utf8!
  primitive_ordinal: UInt32!
  theme: Utf8!
  material_id: UInt64!

textures
  citymodel_id: LargeUtf8!
  texture_id: UInt64!
  image_uri: LargeUtf8!

texture_vertices
  citymodel_id: LargeUtf8!
  uv_id: UInt64!
  u: Float64!
  v: Float64!

geometry_ring_textures
  citymodel_id: LargeUtf8!
  geometry_id: UInt64!
  surface_ordinal: UInt32!
  ring_ordinal: UInt32!
  theme: Utf8!
  texture_id: UInt64!
  uv_indices: List<UInt64!>!

template_geometry_ring_textures
  citymodel_id: LargeUtf8!
  template_geometry_id: UInt64!
  surface_ordinal: UInt32!
  ring_ordinal: UInt32!
  theme: Utf8!
  texture_id: UInt64!
  uv_indices: List<UInt64!>!"#;

const MANIFEST_LOCK: &str = r#"{
  "package_schema": "cityarrow.package.v1alpha1",
  "cityjson_version": "2.0",
  "citymodel_id": "schema-lock-citymodel",
  "tables": {
    "metadata": "metadata.parquet",
    "transform": "transform.parquet",
    "extensions": "extensions.parquet",
    "vertices": "vertices.parquet",
    "cityobjects": "cityobjects.parquet",
    "cityobject_children": "cityobject_children.parquet",
    "geometries": "geometries.parquet",
    "geometry_boundaries": "geometry_boundaries.parquet",
    "geometry_instances": "geometry_instances.parquet",
    "template_vertices": "template_vertices.parquet",
    "template_geometries": "template_geometries.parquet",
    "template_geometry_boundaries": "template_geometry_boundaries.parquet",
    "semantics": "semantics.parquet",
    "semantic_children": "semantic_children.parquet",
    "geometry_surface_semantics": "geometry_surface_semantics.parquet",
    "geometry_point_semantics": "geometry_point_semantics.parquet",
    "geometry_linestring_semantics": "geometry_linestring_semantics.parquet",
    "template_geometry_semantics": "template_geometry_semantics.parquet",
    "materials": "materials.parquet",
    "geometry_surface_materials": "geometry_surface_materials.parquet",
    "geometry_point_materials": "geometry_point_materials.parquet",
    "geometry_linestring_materials": "geometry_linestring_materials.parquet",
    "template_geometry_materials": "template_geometry_materials.parquet",
    "textures": "textures.parquet",
    "texture_vertices": "texture_vertices.parquet",
    "geometry_ring_textures": "geometry_ring_textures.parquet",
    "template_geometry_ring_textures": "template_geometry_ring_textures.parquet"
  }
}"#;

fn field<'a>(schema: &'a Schema, index: usize) -> &'a Field {
    schema.field(index)
}

fn fixed_size_list<const N: usize>(values: Vec<[f64; N]>, width: i32) -> ArrayRef {
    let flat: Vec<f64> = values.into_iter().flat_map(|row| row.into_iter()).collect();
    let values = Float64Array::from(flat).into_data();
    let data_type = DataType::FixedSizeList(
        Arc::new(Field::new_list_field(DataType::Float64, false)),
        width,
    );
    let data = unsafe {
        ArrayData::builder(data_type)
            .len(values.len() / N)
            .add_child_data(values)
            .build_unchecked()
    };
    Arc::new(FixedSizeListArray::from(data)) as ArrayRef
}

fn fixed_size_list_6(values: Vec<[f64; 6]>) -> ArrayRef {
    fixed_size_list(values, 6)
}

fn metadata_batch(schema: &SchemaRef) -> RecordBatch {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(vec!["schema-lock-citymodel"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["2.0"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["CityJSON"])) as ArrayRef,
            Arc::new(LargeStringArray::from(vec![Option::<&str>::None])) as ArrayRef,
            Arc::new(LargeStringArray::from(vec![Option::<&str>::None])) as ArrayRef,
            Arc::new(LargeStringArray::from(vec![Option::<&str>::None])) as ArrayRef,
            fixed_size_list_6(vec![[0.0, 0.0, 0.0, 0.0, 0.0, 0.0]]),
        ],
    )
    .expect("metadata batch")
}

fn empty_batch(schema: &SchemaRef) -> RecordBatch {
    RecordBatch::new_empty(schema.clone())
}

fn schema_lock_parts() -> CityModelArrowParts {
    let projection = ProjectionLayout::default();
    let schemas = canonical_schema_set(&projection);

    CityModelArrowParts {
        header: CityArrowHeader::new(
            CityArrowPackageVersion::V1Alpha1,
            "schema-lock-citymodel",
            "2.0",
        ),
        projection,
        metadata: metadata_batch(&schemas.metadata),
        transform: Some(empty_batch(&schemas.transform)),
        extensions: Some(empty_batch(&schemas.extensions)),
        vertices: empty_batch(&schemas.vertices),
        cityobjects: empty_batch(&schemas.cityobjects),
        cityobject_children: Some(empty_batch(&schemas.cityobject_children)),
        geometries: empty_batch(&schemas.geometries),
        geometry_boundaries: empty_batch(&schemas.geometry_boundaries),
        geometry_instances: Some(empty_batch(&schemas.geometry_instances)),
        template_vertices: Some(empty_batch(&schemas.template_vertices)),
        template_geometries: Some(empty_batch(&schemas.template_geometries)),
        template_geometry_boundaries: Some(empty_batch(&schemas.template_geometry_boundaries)),
        semantics: Some(empty_batch(&schemas.semantics)),
        semantic_children: Some(empty_batch(&schemas.semantic_children)),
        geometry_surface_semantics: Some(empty_batch(&schemas.geometry_surface_semantics)),
        geometry_point_semantics: Some(empty_batch(&schemas.geometry_point_semantics)),
        geometry_linestring_semantics: Some(empty_batch(&schemas.geometry_linestring_semantics)),
        template_geometry_semantics: Some(empty_batch(&schemas.template_geometry_semantics)),
        materials: Some(empty_batch(&schemas.materials)),
        geometry_surface_materials: Some(empty_batch(&schemas.geometry_surface_materials)),
        geometry_point_materials: Some(empty_batch(&schemas.geometry_point_materials)),
        geometry_linestring_materials: Some(empty_batch(&schemas.geometry_linestring_materials)),
        template_geometry_materials: Some(empty_batch(&schemas.template_geometry_materials)),
        textures: Some(empty_batch(&schemas.textures)),
        texture_vertices: Some(empty_batch(&schemas.texture_vertices)),
        geometry_ring_textures: Some(empty_batch(&schemas.geometry_ring_textures)),
        template_geometry_ring_textures: Some(empty_batch(
            &schemas.template_geometry_ring_textures,
        )),
    }
}

fn format_nested_type(field: &Field) -> String {
    format!(
        "{}{}",
        format_data_type(field.data_type()),
        if field.is_nullable() { "?" } else { "!" }
    )
}

fn format_data_type(data_type: &DataType) -> String {
    match data_type {
        DataType::Boolean => "Boolean".to_string(),
        DataType::UInt32 => "UInt32".to_string(),
        DataType::UInt64 => "UInt64".to_string(),
        DataType::Int64 => "Int64".to_string(),
        DataType::Float64 => "Float64".to_string(),
        DataType::Utf8 => "Utf8".to_string(),
        DataType::LargeUtf8 => "LargeUtf8".to_string(),
        DataType::Binary => "Binary".to_string(),
        DataType::List(child) => format!("List<{}>", format_nested_type(child)),
        DataType::FixedSizeList(child, size) => {
            format!("FixedSizeList<{}, {}>", format_nested_type(child), size)
        }
        other => format!("{other:?}"),
    }
}

fn schema_snapshot() -> String {
    let schemas = canonical_schema_set(&ProjectionLayout::default());
    let tables = [
        ("metadata", schemas.metadata.as_ref()),
        ("transform", schemas.transform.as_ref()),
        ("extensions", schemas.extensions.as_ref()),
        ("vertices", schemas.vertices.as_ref()),
        ("cityobjects", schemas.cityobjects.as_ref()),
        ("cityobject_children", schemas.cityobject_children.as_ref()),
        ("geometries", schemas.geometries.as_ref()),
        ("geometry_boundaries", schemas.geometry_boundaries.as_ref()),
        ("geometry_instances", schemas.geometry_instances.as_ref()),
        ("template_vertices", schemas.template_vertices.as_ref()),
        ("template_geometries", schemas.template_geometries.as_ref()),
        (
            "template_geometry_boundaries",
            schemas.template_geometry_boundaries.as_ref(),
        ),
        ("semantics", schemas.semantics.as_ref()),
        ("semantic_children", schemas.semantic_children.as_ref()),
        (
            "geometry_surface_semantics",
            schemas.geometry_surface_semantics.as_ref(),
        ),
        (
            "geometry_point_semantics",
            schemas.geometry_point_semantics.as_ref(),
        ),
        (
            "geometry_linestring_semantics",
            schemas.geometry_linestring_semantics.as_ref(),
        ),
        (
            "template_geometry_semantics",
            schemas.template_geometry_semantics.as_ref(),
        ),
        ("materials", schemas.materials.as_ref()),
        (
            "geometry_surface_materials",
            schemas.geometry_surface_materials.as_ref(),
        ),
        (
            "geometry_point_materials",
            schemas.geometry_point_materials.as_ref(),
        ),
        (
            "geometry_linestring_materials",
            schemas.geometry_linestring_materials.as_ref(),
        ),
        (
            "template_geometry_materials",
            schemas.template_geometry_materials.as_ref(),
        ),
        ("textures", schemas.textures.as_ref()),
        ("texture_vertices", schemas.texture_vertices.as_ref()),
        (
            "geometry_ring_textures",
            schemas.geometry_ring_textures.as_ref(),
        ),
        (
            "template_geometry_ring_textures",
            schemas.template_geometry_ring_textures.as_ref(),
        ),
    ];

    let mut snapshot = String::new();
    for (index, (table_name, schema)) in tables.iter().enumerate() {
        if index > 0 {
            snapshot.push('\n');
        }
        writeln!(&mut snapshot, "{table_name}").unwrap();
        for field in schema.fields() {
            writeln!(
                &mut snapshot,
                "  {}: {}{}",
                field.name(),
                format_data_type(field.data_type()),
                if field.is_nullable() { "?" } else { "!" }
            )
            .unwrap();
        }
    }
    snapshot.trim_end().to_string()
}

#[test]
fn package_manifest_roundtrips_and_keeps_schema_id() {
    let mut manifest = PackageManifest::new("rotterdam-sample", "2.0");
    manifest.tables.metadata = Some(PathBuf::from("metadata.parquet"));
    manifest.tables.vertices = Some(PathBuf::from("vertices.parquet"));
    manifest.tables.cityobjects = Some(PathBuf::from("cityobjects.parquet"));
    manifest.views.insert(
        "surfaces".to_string(),
        PathBuf::from("views/surfaces.geoparquet"),
    );

    assert_eq!(manifest.package_schema, CityArrowPackageVersion::V1Alpha1);
    assert_eq!(
        manifest.package_schema.to_string(),
        "cityarrow.package.v1alpha1"
    );

    let json = serde_json::to_string(&manifest).expect("manifest should serialize");
    assert!(json.contains("cityarrow.package.v1alpha1"));

    let roundtrip: PackageManifest =
        serde_json::from_str(&json).expect("manifest should deserialize");
    assert_eq!(manifest, roundtrip);
}

#[test]
fn cityarrow_header_is_derived_from_the_package_manifest() {
    let manifest = PackageManifest::new("rotterdam-sample", "2.0");
    let header = CityArrowHeader::from(&manifest);

    assert_eq!(header.package_version, CityArrowPackageVersion::V1Alpha1);
    assert_eq!(header.citymodel_id, "rotterdam-sample");
    assert_eq!(header.cityjson_version, "2.0");
}

#[test]
fn package_tables_empty_is_detected() {
    assert!(PackageTables::default().is_empty());
}

#[test]
fn canonical_metadata_schema_matches_the_documented_layout() {
    let schemas = canonical_schema_set(&ProjectionLayout::default());
    let schema = schemas.metadata.as_ref();

    assert_eq!(schema.fields().len(), 7);
    assert_eq!(field(schema, 0).name(), "citymodel_id");
    assert_eq!(field(schema, 0).data_type(), &DataType::LargeUtf8);
    assert!(!field(schema, 0).is_nullable());

    assert_eq!(field(schema, 1).name(), "cityjson_version");
    assert_eq!(field(schema, 1).data_type(), &DataType::Utf8);
    assert_eq!(field(schema, 2).name(), "citymodel_kind");
    assert_eq!(field(schema, 2).data_type(), &DataType::Utf8);

    assert_eq!(field(schema, 6).name(), "geographical_extent");
    match field(schema, 6).data_type() {
        DataType::FixedSizeList(child, size) => {
            assert_eq!(*size, 6);
            assert_eq!(child.data_type(), &DataType::Float64);
            assert!(!child.is_nullable());
        }
        other => panic!("unexpected metadata extent type: {other:?}"),
    }
}

#[test]
fn boundary_and_instance_tables_use_the_expected_norms() {
    let schemas = canonical_schema_set(&ProjectionLayout::default());

    let geometry_schema = schemas.geometry_boundaries.as_ref();
    assert_eq!(field(geometry_schema, 2).name(), "vertex_indices");
    match field(geometry_schema, 2).data_type() {
        DataType::List(child) => assert_eq!(child.data_type(), &DataType::UInt64),
        other => panic!("unexpected vertex_indices type: {other:?}"),
    }
    assert_eq!(field(geometry_schema, 3).name(), "line_lengths");
    assert!(field(geometry_schema, 3).is_nullable());

    let instance_schema = schemas.geometry_instances.as_ref();
    assert_eq!(field(instance_schema, 5).name(), "template_geometry_id");
    assert_eq!(
        field(instance_schema, 6).name(),
        "reference_point_vertex_id"
    );
    assert_eq!(field(instance_schema, 7).name(), "transform_matrix");
    assert!(field(instance_schema, 7).is_nullable());
}

#[test]
fn projected_fields_are_appended_to_owner_tables() {
    let layout = ProjectionLayout {
        metadata_extra: vec![ProjectedFieldSpec::new(
            "extra.note",
            ProjectedValueType::LargeUtf8,
            true,
        )],
        cityobject_attributes: vec![ProjectedFieldSpec::new(
            "attributes.height",
            ProjectedValueType::Float64,
            true,
        )],
        cityobject_extra: vec![ProjectedFieldSpec::new(
            "extra.source",
            ProjectedValueType::LargeUtf8,
            true,
        )],
        geometry_extra: vec![ProjectedFieldSpec::new(
            "extra.mesh",
            ProjectedValueType::WkbBinary,
            false,
        )],
        semantic_attributes: vec![ProjectedFieldSpec::new(
            "attributes.label",
            ProjectedValueType::LargeUtf8,
            true,
        )],
        material_payload: vec![ProjectedFieldSpec::new(
            "payload.kind",
            ProjectedValueType::LargeUtf8,
            true,
        )],
        texture_payload: vec![ProjectedFieldSpec::new(
            "payload.mode",
            ProjectedValueType::LargeUtf8,
            true,
        )],
    };

    let schemas = canonical_schema_set(&layout);

    let metadata = schemas.metadata.as_ref();
    assert_eq!(field(metadata, 7).name(), "extra.note");
    assert_eq!(field(metadata, 7).data_type(), &DataType::LargeUtf8);
    assert!(field(metadata, 7).is_nullable());

    let cityobjects = schemas.cityobjects.as_ref();
    assert_eq!(field(cityobjects, 5).name(), "attributes.height");
    assert_eq!(field(cityobjects, 5).data_type(), &DataType::Float64);
    assert!(field(cityobjects, 5).is_nullable());
    assert_eq!(field(cityobjects, 6).name(), "extra.source");
    assert_eq!(field(cityobjects, 6).data_type(), &DataType::LargeUtf8);
    assert!(field(cityobjects, 6).is_nullable());

    let geometries = schemas.geometries.as_ref();
    assert_eq!(field(geometries, 6).name(), "extra.mesh");
    assert_eq!(field(geometries, 6).data_type(), &DataType::Binary);
    assert!(!field(geometries, 6).is_nullable());

    let semantics = schemas.semantics.as_ref();
    assert_eq!(field(semantics, 3).name(), "attributes.label");
    assert_eq!(field(semantics, 3).data_type(), &DataType::LargeUtf8);
    assert!(field(semantics, 3).is_nullable());

    let materials = schemas.materials.as_ref();
    assert_eq!(field(materials, 2).name(), "payload.kind");
    assert_eq!(field(materials, 2).data_type(), &DataType::LargeUtf8);
    assert!(field(materials, 2).is_nullable());

    let textures = schemas.textures.as_ref();
    assert_eq!(field(textures, 3).name(), "payload.mode");
    assert_eq!(field(textures, 3).data_type(), &DataType::LargeUtf8);
    assert!(field(textures, 3).is_nullable());
}

#[test]
fn package_manifest_supports_empty_views_and_tables() {
    let manifest = PackageManifest {
        package_schema: CityArrowPackageVersion::V1Alpha1,
        cityjson_version: "2.0".to_string(),
        citymodel_id: "sample".to_string(),
        tables: PackageTables::default(),
        views: BTreeMap::new(),
    };

    let json = serde_json::to_string(&manifest).unwrap();
    assert!(!json.contains("views"));
    assert!(!json.contains("tables"));
}

#[test]
fn canonical_schema_set_avoids_union_and_map_types() {
    let schemas = canonical_schema_set(&ProjectionLayout::default());
    let all_schemas = [
        schemas.metadata.as_ref(),
        schemas.transform.as_ref(),
        schemas.extensions.as_ref(),
        schemas.vertices.as_ref(),
        schemas.cityobjects.as_ref(),
        schemas.cityobject_children.as_ref(),
        schemas.geometries.as_ref(),
        schemas.geometry_boundaries.as_ref(),
        schemas.geometry_instances.as_ref(),
        schemas.template_vertices.as_ref(),
        schemas.template_geometries.as_ref(),
        schemas.template_geometry_boundaries.as_ref(),
        schemas.semantics.as_ref(),
        schemas.semantic_children.as_ref(),
        schemas.geometry_surface_semantics.as_ref(),
        schemas.geometry_point_semantics.as_ref(),
        schemas.geometry_linestring_semantics.as_ref(),
        schemas.template_geometry_semantics.as_ref(),
        schemas.materials.as_ref(),
        schemas.geometry_surface_materials.as_ref(),
        schemas.geometry_point_materials.as_ref(),
        schemas.geometry_linestring_materials.as_ref(),
        schemas.template_geometry_materials.as_ref(),
        schemas.textures.as_ref(),
        schemas.texture_vertices.as_ref(),
        schemas.geometry_ring_textures.as_ref(),
        schemas.template_geometry_ring_textures.as_ref(),
    ];

    for schema in all_schemas {
        for field in schema.flattened_fields() {
            assert!(
                !matches!(
                    field.data_type(),
                    DataType::Union(_, _) | DataType::Map(_, _)
                ),
                "canonical schema must not use Arrow Union or Map types"
            );
        }
    }
}

#[test]
fn canonical_schema_set_matches_locked_snapshot() {
    assert_eq!(schema_snapshot(), CANONICAL_SCHEMA_LOCK);
}

#[test]
fn package_writer_manifest_matches_locked_snapshot() {
    let parts = schema_lock_parts();
    let dir = tempdir().expect("temp dir");

    let manifest = write_package_dir(dir.path(), &parts).expect("package write");
    let json = serde_json::to_string_pretty(&manifest).expect("manifest json");

    assert_eq!(json, MANIFEST_LOCK);
}
