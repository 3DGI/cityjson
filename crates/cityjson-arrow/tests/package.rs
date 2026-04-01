#[path = "support/mod.rs"]
mod support;

use std::sync::Arc;

use arrow::array::{
    Array, ArrayData, ArrayRef, BinaryArray, FixedSizeListArray, Float64Array, LargeStringArray,
    ListArray, StringArray, UInt32Array, UInt64Array,
};
use arrow::buffer::{NullBuffer, OffsetBuffer};
use arrow::datatypes::{DataType, Field, SchemaRef};
use arrow::record_batch::RecordBatch;
use cityarrow::package::{read_package_ipc_dir, write_package_ipc_dir};
use cityarrow::schema::{
    CanonicalSchemaSet, CityArrowHeader, CityArrowPackageVersion, CityModelArrowParts,
    PackageTableEncoding, ProjectedFieldSpec, ProjectedValueType, ProjectionLayout,
    canonical_schema_set,
};
use cityparquet::package::{read_package_dir, write_package_dir};
use tempfile::tempdir;

fn array_ref<T: Array + 'static>(array: T) -> ArrayRef {
    Arc::new(array) as ArrayRef
}

fn fixed_size_list_3(values: Vec<[f64; 3]>) -> ArrayRef {
    fixed_size_list(values, 3)
}

fn fixed_size_list_16(values: Vec<[f64; 16]>) -> ArrayRef {
    fixed_size_list(values, 16)
}

fn fixed_size_list_6(values: Vec<[f64; 6]>) -> ArrayRef {
    fixed_size_list(values, 6)
}

fn fixed_size_list<const N: usize>(values: Vec<[f64; N]>, width: i32) -> ArrayRef {
    let flat: Vec<f64> = values
        .into_iter()
        .flat_map(std::iter::IntoIterator::into_iter)
        .collect();
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

fn usize_to_i32(value: usize) -> i32 {
    i32::try_from(value).expect("test offset should fit in i32")
}

fn u64_list(values: Vec<Option<Vec<u64>>>) -> ArrayRef {
    let mut offsets = vec![0_i32];
    let mut flat = Vec::new();
    let mut validity = Vec::with_capacity(values.len());
    for value in values {
        if let Some(items) = value {
            flat.extend(items);
            offsets.push(usize_to_i32(flat.len()));
            validity.push(true);
        } else {
            offsets.push(usize_to_i32(flat.len()));
            validity.push(false);
        }
    }

    let field = Arc::new(Field::new("item", DataType::UInt64, false));
    let values = Arc::new(UInt64Array::from(flat)) as ArrayRef;
    let nulls = validity
        .iter()
        .all(|valid| *valid)
        .then(|| NullBuffer::from(validity));
    Arc::new(ListArray::new(
        field,
        OffsetBuffer::new(offsets.into()),
        values,
        nulls,
    )) as ArrayRef
}

fn u32_list(values: Vec<Option<Vec<u32>>>) -> ArrayRef {
    let mut offsets = vec![0_i32];
    let mut flat = Vec::new();
    let mut validity = Vec::with_capacity(values.len());
    for value in values {
        if let Some(items) = value {
            flat.extend(items);
            offsets.push(usize_to_i32(flat.len()));
            validity.push(true);
        } else {
            offsets.push(usize_to_i32(flat.len()));
            validity.push(false);
        }
    }

    let field = Arc::new(Field::new("item", DataType::UInt32, false));
    let values = Arc::new(UInt32Array::from(flat)) as ArrayRef;
    let nulls = validity
        .iter()
        .all(|valid| *valid)
        .then(|| NullBuffer::from(validity));
    Arc::new(ListArray::new(
        field,
        OffsetBuffer::new(offsets.into()),
        values,
        nulls,
    )) as ArrayRef
}

fn batch(schema: &SchemaRef, columns: Vec<ArrayRef>) -> RecordBatch {
    RecordBatch::try_new(schema.clone(), columns).expect("record batch")
}

struct SampleTables {
    metadata: RecordBatch,
    transform: RecordBatch,
    extensions: RecordBatch,
    vertices: RecordBatch,
    cityobjects: RecordBatch,
    cityobject_children: RecordBatch,
    geometries: RecordBatch,
    geometry_boundaries: RecordBatch,
    geometry_instances: RecordBatch,
    template_vertices: RecordBatch,
    template_geometries: RecordBatch,
    template_geometry_boundaries: RecordBatch,
    semantics: RecordBatch,
    geometry_surface_semantics: RecordBatch,
    materials: RecordBatch,
    textures: RecordBatch,
    texture_vertices: RecordBatch,
    geometry_ring_textures: RecordBatch,
}

fn sample_parts() -> CityModelArrowParts {
    let projection = sample_projection();
    let schemas = canonical_schema_set(&projection);
    let tables = sample_tables(&schemas);

    CityModelArrowParts {
        header: CityArrowHeader::new(CityArrowPackageVersion::V1Alpha1, "sample-citymodel", "2.0"),
        projection,
        metadata: tables.metadata,
        transform: Some(tables.transform),
        extensions: Some(tables.extensions),
        vertices: tables.vertices,
        cityobjects: tables.cityobjects,
        cityobject_children: Some(tables.cityobject_children),
        geometries: tables.geometries,
        geometry_boundaries: tables.geometry_boundaries,
        geometry_instances: Some(tables.geometry_instances),
        template_vertices: Some(tables.template_vertices),
        template_geometries: Some(tables.template_geometries),
        template_geometry_boundaries: Some(tables.template_geometry_boundaries),
        semantics: Some(tables.semantics),
        semantic_children: None,
        geometry_surface_semantics: Some(tables.geometry_surface_semantics),
        geometry_point_semantics: None,
        geometry_linestring_semantics: None,
        template_geometry_semantics: None,
        materials: Some(tables.materials),
        geometry_surface_materials: None,
        geometry_point_materials: None,
        geometry_linestring_materials: None,
        template_geometry_materials: None,
        textures: Some(tables.textures),
        texture_vertices: Some(tables.texture_vertices),
        geometry_ring_textures: Some(tables.geometry_ring_textures),
        template_geometry_ring_textures: None,
    }
}

fn sample_projection() -> ProjectionLayout {
    ProjectionLayout {
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
    }
}

fn sample_tables(schemas: &CanonicalSchemaSet) -> SampleTables {
    let (metadata, transform, extensions, vertices, cityobjects, cityobject_children) =
        sample_core_tables(schemas);
    let (
        geometries,
        geometry_boundaries,
        geometry_instances,
        template_vertices,
        template_geometries,
        template_geometry_boundaries,
    ) = sample_geometry_tables(schemas);
    let (semantics, geometry_surface_semantics) = sample_semantic_tables(schemas);
    let (materials, textures, texture_vertices, geometry_ring_textures) =
        sample_appearance_tables(schemas);

    SampleTables {
        metadata,
        transform,
        extensions,
        vertices,
        cityobjects,
        cityobject_children,
        geometries,
        geometry_boundaries,
        geometry_instances,
        template_vertices,
        template_geometries,
        template_geometry_boundaries,
        semantics,
        geometry_surface_semantics,
        materials,
        textures,
        texture_vertices,
        geometry_ring_textures,
    }
}

fn sample_core_tables(
    schemas: &CanonicalSchemaSet,
) -> (
    RecordBatch,
    RecordBatch,
    RecordBatch,
    RecordBatch,
    RecordBatch,
    RecordBatch,
) {
    let metadata = batch(
        &schemas.metadata,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            array_ref(StringArray::from(vec!["2.0"])),
            array_ref(StringArray::from(vec!["CityJSON"])),
            array_ref(LargeStringArray::from(vec![Some("sample-identifier")])),
            array_ref(LargeStringArray::from(vec![Some("Sample dataset")])),
            array_ref(LargeStringArray::from(vec![Some("EPSG:7415")])),
            fixed_size_list_6(vec![[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]]),
            array_ref(LargeStringArray::from(vec![Some("metadata-extra")])),
        ],
    );
    let transform = batch(
        &schemas.transform,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            fixed_size_list_3(vec![[1.0, 1.0, 1.0]]),
            fixed_size_list_3(vec![[10.0, 20.0, 30.0]]),
        ],
    );
    let extensions = batch(
        &schemas.extensions,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            array_ref(StringArray::from(vec!["demo"])),
            array_ref(LargeStringArray::from(vec!["https://example.test/demo"])),
            array_ref(StringArray::from(vec![Some("1.0")])),
        ],
    );
    let vertices = batch(
        &schemas.vertices,
        vec![
            array_ref(LargeStringArray::from(vec![
                "sample-citymodel",
                "sample-citymodel",
            ])),
            array_ref(UInt64Array::from(vec![0_u64, 1_u64])),
            array_ref(Float64Array::from(vec![10.0, 11.0])),
            array_ref(Float64Array::from(vec![20.0, 21.0])),
            array_ref(Float64Array::from(vec![30.0, 31.0])),
        ],
    );
    let cityobjects = batch(
        &schemas.cityobjects,
        vec![
            array_ref(LargeStringArray::from(vec![
                "sample-citymodel",
                "sample-citymodel",
            ])),
            array_ref(LargeStringArray::from(vec![
                "building-1",
                "building-1-part",
            ])),
            array_ref(UInt64Array::from(vec![0_u64, 1_u64])),
            array_ref(StringArray::from(vec!["Building", "BuildingPart"])),
            fixed_size_list_6(vec![
                [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            ]),
            array_ref(Float64Array::from(vec![Some(12.5), None])),
            array_ref(LargeStringArray::from(vec![
                Some("survey"),
                Some("archive"),
            ])),
        ],
    );
    let cityobject_children = batch(
        &schemas.cityobject_children,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            array_ref(LargeStringArray::from(vec!["building-1"])),
            array_ref(UInt32Array::from(vec![0_u32])),
            array_ref(LargeStringArray::from(vec!["building-1-part"])),
        ],
    );
    (
        metadata,
        transform,
        extensions,
        vertices,
        cityobjects,
        cityobject_children,
    )
}

fn sample_geometry_tables(
    schemas: &CanonicalSchemaSet,
) -> (
    RecordBatch,
    RecordBatch,
    RecordBatch,
    RecordBatch,
    RecordBatch,
    RecordBatch,
) {
    let geometries = batch(
        &schemas.geometries,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            array_ref(UInt64Array::from(vec![0_u64])),
            array_ref(LargeStringArray::from(vec!["building-1"])),
            array_ref(UInt32Array::from(vec![0_u32])),
            array_ref(StringArray::from(vec!["MultiSurface"])),
            array_ref(StringArray::from(vec![Some("2.0")])),
            array_ref(BinaryArray::from_iter_values([b"mesh".as_slice()])),
        ],
    );
    let geometry_boundaries = batch(
        &schemas.geometry_boundaries,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            array_ref(UInt64Array::from(vec![0_u64])),
            u64_list(vec![Some(vec![0_u64, 1_u64, 0_u64])]),
            u32_list(vec![None]),
            u32_list(vec![Some(vec![3_u32])]),
            u32_list(vec![Some(vec![1_u32])]),
            u32_list(vec![None]),
            u32_list(vec![None]),
        ],
    );
    let geometry_instances = batch(
        &schemas.geometry_instances,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            array_ref(UInt64Array::from(vec![1_u64])),
            array_ref(LargeStringArray::from(vec!["building-1-part"])),
            array_ref(UInt32Array::from(vec![0_u32])),
            array_ref(StringArray::from(vec![Some("1.0")])),
            array_ref(UInt64Array::from(vec![0_u64])),
            array_ref(UInt64Array::from(vec![1_u64])),
            fixed_size_list_16(vec![[
                1.0, 0.0, 0.0, 2.0, 0.0, 1.0, 0.0, 3.0, 0.0, 0.0, 1.0, 4.0, 0.0, 0.0, 0.0, 1.0,
            ]]),
            array_ref(BinaryArray::from_iter_values([b"instance-mesh".as_slice()])),
        ],
    );
    let template_vertices = batch(
        &schemas.template_vertices,
        vec![
            array_ref(LargeStringArray::from(vec![
                "sample-citymodel",
                "sample-citymodel",
            ])),
            array_ref(UInt64Array::from(vec![0_u64, 1_u64])),
            array_ref(Float64Array::from(vec![0.0, 1.0])),
            array_ref(Float64Array::from(vec![0.0, 0.0])),
            array_ref(Float64Array::from(vec![0.0, 0.0])),
        ],
    );
    let template_geometries = batch(
        &schemas.template_geometries,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            array_ref(UInt64Array::from(vec![0_u64])),
            array_ref(StringArray::from(vec!["MultiPoint"])),
            array_ref(StringArray::from(vec![Some("1.0")])),
            array_ref(BinaryArray::from_iter_values([b"template-mesh".as_slice()])),
        ],
    );
    let template_geometry_boundaries = batch(
        &schemas.template_geometry_boundaries,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            array_ref(UInt64Array::from(vec![0_u64])),
            u64_list(vec![Some(vec![0_u64, 1_u64])]),
            u32_list(vec![None]),
            u32_list(vec![None]),
            u32_list(vec![None]),
            u32_list(vec![None]),
            u32_list(vec![None]),
        ],
    );
    (
        geometries,
        geometry_boundaries,
        geometry_instances,
        template_vertices,
        template_geometries,
        template_geometry_boundaries,
    )
}

fn sample_semantic_tables(schemas: &CanonicalSchemaSet) -> (RecordBatch, RecordBatch) {
    let semantics = batch(
        &schemas.semantics,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            array_ref(UInt64Array::from(vec![0_u64])),
            array_ref(StringArray::from(vec!["RoofSurface"])),
            array_ref(LargeStringArray::from(vec![Some("roof")])),
        ],
    );
    let geometry_surface_semantics = batch(
        &schemas.geometry_surface_semantics,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            array_ref(UInt64Array::from(vec![0_u64])),
            array_ref(UInt32Array::from(vec![0_u32])),
            array_ref(UInt64Array::from(vec![0_u64])),
        ],
    );
    (semantics, geometry_surface_semantics)
}

fn sample_appearance_tables(
    schemas: &CanonicalSchemaSet,
) -> (RecordBatch, RecordBatch, RecordBatch, RecordBatch) {
    let materials = batch(
        &schemas.materials,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            array_ref(UInt64Array::from(vec![0_u64])),
            array_ref(LargeStringArray::from(vec![Some("brick")])),
        ],
    );
    let textures = batch(
        &schemas.textures,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            array_ref(UInt64Array::from(vec![0_u64])),
            array_ref(LargeStringArray::from(vec!["textures/roof.png"])),
            array_ref(LargeStringArray::from(vec![Some("tile")])),
        ],
    );
    let texture_vertices = batch(
        &schemas.texture_vertices,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            array_ref(UInt64Array::from(vec![0_u64])),
            array_ref(Float64Array::from(vec![0.0])),
            array_ref(Float64Array::from(vec![0.0])),
        ],
    );
    let geometry_ring_textures = batch(
        &schemas.geometry_ring_textures,
        vec![
            array_ref(LargeStringArray::from(vec!["sample-citymodel"])),
            array_ref(UInt64Array::from(vec![0_u64])),
            array_ref(UInt32Array::from(vec![0_u32])),
            array_ref(UInt32Array::from(vec![0_u32])),
            array_ref(StringArray::from(vec!["default"])),
            array_ref(UInt64Array::from(vec![0_u64])),
            u64_list(vec![Some(vec![0_u64, 0_u64, 0_u64])]),
        ],
    );
    (
        materials,
        textures,
        texture_vertices,
        geometry_ring_textures,
    )
}

#[test]
fn package_directory_roundtrips_canonical_tables() {
    let parts = sample_parts();
    let dir = tempdir().expect("temp dir");

    let manifest = write_package_dir(dir.path(), &parts).expect("package write");
    assert_eq!(manifest.citymodel_id, "sample-citymodel");
    assert!(manifest.tables.metadata.is_some());
    assert!(manifest.tables.vertices.is_some());
    assert!(manifest.tables.geometries.is_some());
    assert!(manifest.tables.geometry_instances.is_some());
    assert!(manifest.tables.template_geometries.is_some());
    assert!(manifest.tables.geometry_ring_textures.is_some());

    let roundtrip = read_package_dir(dir.path()).expect("package read");
    support::assert_parts_eq(&parts, &roundtrip);
}

#[test]
fn ipc_package_directory_roundtrips_canonical_tables() {
    let parts = sample_parts();
    let dir = tempdir().expect("temp dir");

    let manifest = write_package_ipc_dir(dir.path(), &parts).expect("ipc package write");
    assert_eq!(manifest.citymodel_id, "sample-citymodel");
    assert_eq!(manifest.table_encoding, PackageTableEncoding::ArrowIpcFile);
    assert_eq!(
        manifest.tables.metadata.as_deref(),
        Some(std::path::Path::new("metadata.arrow"))
    );
    assert_eq!(
        manifest.tables.geometries.as_deref(),
        Some(std::path::Path::new("geometries.arrow"))
    );

    let explicit_roundtrip = read_package_ipc_dir(dir.path()).expect("ipc package read");
    support::assert_parts_eq(&parts, &explicit_roundtrip);

    let error = read_package_dir(dir.path()).expect_err("parquet reader should reject ipc data");
    assert!(matches!(error, cityarrow::error::Error::Unsupported(_)));
}

#[test]
fn package_writer_rejects_schema_mismatches() {
    let mut parts = sample_parts();
    parts.projection.cityobject_attributes = vec![ProjectedFieldSpec::new(
        "attributes.height",
        ProjectedValueType::Int64,
        true,
    )];

    let dir = tempdir().expect("temp dir");
    let error = write_package_dir(dir.path(), &parts).expect_err("schema mismatch should fail");
    let message = error.to_string();
    assert!(message.contains("Schema mismatch") || message.contains("expected schema"));
}
