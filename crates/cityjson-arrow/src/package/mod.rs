use crate::error::{Error, Result};
use crate::schema::{
    CanonicalSchemaSet, PackageTableEncoding, ProjectedFieldSpec, ProjectedValueType,
    ProjectionLayout, canonical_schema_set,
};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use arrow_select::concat::concat_batches;
use std::path::{Path, PathBuf};

mod read;
mod write;

pub use read::{read_package_ipc, read_package_ipc_dir};
pub use write::{write_package_ipc, write_package_ipc_dir};

pub const MANIFEST_FILE_NAME: &str = "manifest.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonicalTable {
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
        self.file_name_for(PackageTableEncoding::Parquet)
    }

    #[must_use]
    pub fn file_name_for(self, encoding: PackageTableEncoding) -> &'static str {
        match encoding {
            PackageTableEncoding::Parquet => match self {
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
            },
            PackageTableEncoding::ArrowIpcFile => match self {
                Self::Metadata => "metadata.arrow",
                Self::Transform => "transform.arrow",
                Self::Extensions => "extensions.arrow",
                Self::Vertices => "vertices.arrow",
                Self::CityObjects => "cityobjects.arrow",
                Self::CityObjectChildren => "cityobject_children.arrow",
                Self::Geometries => "geometries.arrow",
                Self::GeometryBoundaries => "geometry_boundaries.arrow",
                Self::GeometryInstances => "geometry_instances.arrow",
                Self::TemplateVertices => "template_vertices.arrow",
                Self::TemplateGeometries => "template_geometries.arrow",
                Self::TemplateGeometryBoundaries => "template_geometry_boundaries.arrow",
                Self::Semantics => "semantics.arrow",
                Self::SemanticChildren => "semantic_children.arrow",
                Self::GeometrySurfaceSemantics => "geometry_surface_semantics.arrow",
                Self::GeometryPointSemantics => "geometry_point_semantics.arrow",
                Self::GeometryLinestringSemantics => "geometry_linestring_semantics.arrow",
                Self::TemplateGeometrySemantics => "template_geometry_semantics.arrow",
                Self::Materials => "materials.arrow",
                Self::GeometrySurfaceMaterials => "geometry_surface_materials.arrow",
                Self::GeometryPointMaterials => "geometry_point_materials.arrow",
                Self::GeometryLinestringMaterials => "geometry_linestring_materials.arrow",
                Self::TemplateGeometryMaterials => "template_geometry_materials.arrow",
                Self::Textures => "textures.arrow",
                Self::TextureVertices => "texture_vertices.arrow",
                Self::GeometryRingTextures => "geometry_ring_textures.arrow",
                Self::TemplateGeometryRingTextures => "template_geometry_ring_textures.arrow",
            },
        }
    }
}

#[must_use]
pub fn package_manifest_path(dir: &Path) -> PathBuf {
    dir.join(MANIFEST_FILE_NAME)
}

#[must_use]
pub fn package_table_path_for_encoding(
    dir: &Path,
    table: CanonicalTable,
    encoding: PackageTableEncoding,
) -> PathBuf {
    dir.join(table.file_name_for(encoding))
}

#[must_use]
pub fn expected_schema_set(layout: &ProjectionLayout) -> CanonicalSchemaSet {
    canonical_schema_set(layout)
}

/// Concatenates a sequence of record batches that all share the same schema.
///
/// # Errors
///
/// Returns an error when Arrow cannot concatenate the provided batches.
pub fn concat_record_batches(schema: &SchemaRef, batches: &[RecordBatch]) -> Result<RecordBatch> {
    let batch_refs: Vec<&RecordBatch> = batches.iter().collect();
    concat_batches(schema, batch_refs).map_err(Error::from)
}

fn projected_field_spec(field: &Field) -> Result<ProjectedFieldSpec> {
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

/// Infers projected field specifications from the trailing fields of a schema.
///
/// # Errors
///
/// Returns an error when a trailing field uses an unsupported Arrow type.
pub fn infer_tail_projection(
    schema: &Schema,
    start_index: usize,
) -> Result<Vec<ProjectedFieldSpec>> {
    let mut fields = Vec::with_capacity(schema.fields().len().saturating_sub(start_index));
    for field in schema.fields().iter().skip(start_index) {
        fields.push(projected_field_spec(field)?);
    }
    Ok(fields)
}

/// Infers projected attribute and extra columns from a cityobjects schema.
///
/// # Errors
///
/// Returns an error when a projected field does not belong to either supported
/// projection namespace.
pub fn infer_cityobject_projections(
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

/// Infers projected semantic attribute columns from a semantics schema.
///
/// # Errors
///
/// Returns an error when a projected field uses an unsupported Arrow type.
pub fn infer_semantic_projection(schema: &Schema) -> Result<Vec<ProjectedFieldSpec>> {
    infer_tail_projection(schema, 3)
}

/// Infers projected material payload columns from a materials schema.
///
/// # Errors
///
/// Returns an error when a projected field uses an unsupported Arrow type.
pub fn infer_material_projection(schema: &Schema) -> Result<Vec<ProjectedFieldSpec>> {
    infer_tail_projection(schema, 2)
}

/// Infers projected texture payload columns from a textures schema.
///
/// # Errors
///
/// Returns an error when a projected field uses an unsupported Arrow type.
pub fn infer_texture_projection(schema: &Schema) -> Result<Vec<ProjectedFieldSpec>> {
    infer_tail_projection(schema, 3)
}

/// Validates that a table schema matches the canonical schema for a table.
///
/// # Errors
///
/// Returns an error when the actual schema differs from the expected canonical
/// schema.
pub fn validate_schema(
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

#[must_use]
pub fn table_path_from_manifest(dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        dir.join(path)
    }
}
