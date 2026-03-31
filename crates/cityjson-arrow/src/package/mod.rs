use crate::error::{Error, Result};
use crate::schema::{
    CanonicalSchemaSet, ProjectedFieldSpec, ProjectedValueType, ProjectionLayout,
    canonical_schema_set,
};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use arrow_select::concat::concat_batches;
use std::path::{Path, PathBuf};

pub mod read;
pub mod write;

pub use read::{read_package, read_package_dir};
pub use write::{write_package, write_package_dir};

pub(crate) const MANIFEST_FILE_NAME: &str = "manifest.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CanonicalTable {
    Metadata,
    Transform,
    Extensions,
    Vertices,
    CityObjects,
    CityObjectChildren,
    Geometries,
    GeometryBoundaries,
    GeometryInstances,
    TemplateVertices,
    TemplateGeometries,
    TemplateGeometryBoundaries,
    Semantics,
    SemanticChildren,
    GeometrySurfaceSemantics,
    GeometryPointSemantics,
    GeometryLinestringSemantics,
    TemplateGeometrySemantics,
    Materials,
    GeometrySurfaceMaterials,
    GeometryPointMaterials,
    GeometryLinestringMaterials,
    TemplateGeometryMaterials,
    Textures,
    TextureVertices,
    GeometryRingTextures,
    TemplateGeometryRingTextures,
}

impl CanonicalTable {
    #[must_use]
    pub fn file_name(self) -> &'static str {
        match self {
            Self::Metadata => "metadata.parquet",
            Self::Transform => "transform.parquet",
            Self::Extensions => "extensions.parquet",
            Self::Vertices => "vertices.parquet",
            Self::CityObjects => "cityobjects.parquet",
            Self::CityObjectChildren => "cityobject_children.parquet",
            Self::Geometries => "geometries.parquet",
            Self::GeometryBoundaries => "geometry_boundaries.parquet",
            Self::GeometryInstances => "geometry_instances.parquet",
            Self::TemplateVertices => "template_vertices.parquet",
            Self::TemplateGeometries => "template_geometries.parquet",
            Self::TemplateGeometryBoundaries => "template_geometry_boundaries.parquet",
            Self::Semantics => "semantics.parquet",
            Self::SemanticChildren => "semantic_children.parquet",
            Self::GeometrySurfaceSemantics => "geometry_surface_semantics.parquet",
            Self::GeometryPointSemantics => "geometry_point_semantics.parquet",
            Self::GeometryLinestringSemantics => "geometry_linestring_semantics.parquet",
            Self::TemplateGeometrySemantics => "template_geometry_semantics.parquet",
            Self::Materials => "materials.parquet",
            Self::GeometrySurfaceMaterials => "geometry_surface_materials.parquet",
            Self::GeometryPointMaterials => "geometry_point_materials.parquet",
            Self::GeometryLinestringMaterials => "geometry_linestring_materials.parquet",
            Self::TemplateGeometryMaterials => "template_geometry_materials.parquet",
            Self::Textures => "textures.parquet",
            Self::TextureVertices => "texture_vertices.parquet",
            Self::GeometryRingTextures => "geometry_ring_textures.parquet",
            Self::TemplateGeometryRingTextures => "template_geometry_ring_textures.parquet",
        }
    }
}

#[must_use]
pub(crate) fn package_manifest_path(dir: &Path) -> PathBuf {
    dir.join(MANIFEST_FILE_NAME)
}

#[must_use]
pub(crate) fn package_table_path(dir: &Path, table: CanonicalTable) -> PathBuf {
    dir.join(table.file_name())
}

#[must_use]
pub(crate) fn expected_schema_set(layout: &ProjectionLayout) -> CanonicalSchemaSet {
    canonical_schema_set(layout)
}

pub(crate) fn concat_record_batches(
    schema: &SchemaRef,
    batches: &[RecordBatch],
) -> Result<RecordBatch> {
    let batch_refs: Vec<&RecordBatch> = batches.iter().collect();
    concat_batches(schema, batch_refs).map_err(Error::from)
}

pub(crate) fn projected_field_spec(field: &Field) -> Result<ProjectedFieldSpec> {
    let data_type = match field.data_type() {
        DataType::Boolean => ProjectedValueType::Boolean,
        DataType::UInt64 => {
            if field.name().ends_with("_geometry_id") {
                ProjectedValueType::GeometryId
            } else {
                ProjectedValueType::UInt64
            }
        }
        DataType::Int64 => ProjectedValueType::Int64,
        DataType::Float64 => ProjectedValueType::Float64,
        DataType::LargeUtf8 => ProjectedValueType::LargeUtf8,
        DataType::Binary => ProjectedValueType::WkbBinary,
        other => {
            return Err(Error::Unsupported(format!(
                "projected field {} uses unsupported Arrow type {:?}",
                field.name(),
                other
            )));
        }
    };

    Ok(ProjectedFieldSpec::new(
        field.name(),
        data_type,
        field.is_nullable(),
    ))
}

pub(crate) fn infer_tail_projection(
    schema: &Schema,
    start_index: usize,
) -> Result<Vec<ProjectedFieldSpec>> {
    let mut fields = Vec::with_capacity(schema.fields().len().saturating_sub(start_index));
    for field in schema.fields().iter().skip(start_index) {
        fields.push(projected_field_spec(field)?);
    }
    Ok(fields)
}

pub(crate) fn infer_cityobject_projections(
    schema: &Schema,
) -> Result<(Vec<ProjectedFieldSpec>, Vec<ProjectedFieldSpec>)> {
    let mut attributes = Vec::new();
    let mut extra = Vec::new();

    for field in schema.fields().iter().skip(5) {
        let spec = projected_field_spec(field)?;
        if field.name().starts_with("attr__") || field.name().starts_with("attributes.") {
            attributes.push(spec);
        } else if field.name().starts_with("extra__") || field.name().starts_with("extra.") {
            extra.push(spec);
        } else {
            return Err(Error::Unsupported(format!(
                "unexpected projected field {} in cityobjects table",
                field.name()
            )));
        }
    }

    Ok((attributes, extra))
}

pub(crate) fn infer_semantic_projection(schema: &Schema) -> Result<Vec<ProjectedFieldSpec>> {
    infer_tail_projection(schema, 3)
}

pub(crate) fn infer_material_projection(schema: &Schema) -> Result<Vec<ProjectedFieldSpec>> {
    infer_tail_projection(schema, 2)
}

pub(crate) fn infer_texture_projection(schema: &Schema) -> Result<Vec<ProjectedFieldSpec>> {
    infer_tail_projection(schema, 3)
}

pub(crate) fn validate_schema(
    expected: impl AsRef<Schema>,
    actual: impl AsRef<Schema>,
    table: CanonicalTable,
) -> Result<()> {
    let expected = expected.as_ref();
    let actual = actual.as_ref();

    if expected != actual {
        return Err(Error::SchemaMismatch {
            expected: format!("{}: {:?}", table.file_name(), expected),
            found: format!("{}: {:?}", table.file_name(), actual),
        });
    }
    Ok(())
}

pub(crate) fn table_path_from_manifest(dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        dir.join(path)
    }
}
