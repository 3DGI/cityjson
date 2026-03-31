use std::collections::BTreeMap;
use std::path::PathBuf;

use arrow::datatypes::DataType;
use cityarrow::schema::{
    CityArrowHeader, CityArrowPackageVersion, PackageManifest, PackageTables, ProjectedFieldSpec,
    ProjectedValueType, ProjectionLayout, canonical_schema_set,
};

fn field<'a>(schema: &'a arrow::datatypes::Schema, index: usize) -> &'a arrow::datatypes::Field {
    schema.field(index)
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
    assert_eq!(field(instance_schema, 7).is_nullable(), true);
}

#[test]
fn projected_fields_are_appended_to_the_owner_table() {
    let layout = ProjectionLayout {
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
        ..ProjectionLayout::default()
    };

    let schemas = canonical_schema_set(&layout);

    let cityobjects = schemas.cityobjects.as_ref();
    assert_eq!(field(cityobjects, 5).name(), "attributes.height");
    assert_eq!(field(cityobjects, 5).data_type(), &DataType::Float64);
    assert!(field(cityobjects, 5).is_nullable());
    assert_eq!(field(cityobjects, 6).name(), "extra.source");
    assert_eq!(field(cityobjects, 6).data_type(), &DataType::LargeUtf8);

    let geometries = schemas.geometries.as_ref();
    assert_eq!(field(geometries, 6).name(), "extra.mesh");
    assert_eq!(field(geometries, 6).data_type(), &DataType::Binary);
    assert!(!field(geometries, 6).is_nullable());
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
