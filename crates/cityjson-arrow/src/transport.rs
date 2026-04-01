use crate::error::{Error, Result};
use crate::schema::{CanonicalSchemaSet, CityModelArrowParts, PackageManifest};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use std::collections::HashMap;

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
    TemplateGeometryMaterials,
    Textures,
    TextureVertices,
    GeometryRingTextures,
    TemplateGeometryRingTextures,
}

impl CanonicalTable {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Metadata => "metadata",
            Self::Transform => "transform",
            Self::Extensions => "extensions",
            Self::Vertices => "vertices",
            Self::CityObjects => "cityobjects",
            Self::CityObjectChildren => "cityobject_children",
            Self::Geometries => "geometries",
            Self::GeometryBoundaries => "geometry_boundaries",
            Self::GeometryInstances => "geometry_instances",
            Self::TemplateVertices => "template_vertices",
            Self::TemplateGeometries => "template_geometries",
            Self::TemplateGeometryBoundaries => "template_geometry_boundaries",
            Self::Semantics => "semantics",
            Self::SemanticChildren => "semantic_children",
            Self::GeometrySurfaceSemantics => "geometry_surface_semantics",
            Self::GeometryPointSemantics => "geometry_point_semantics",
            Self::GeometryLinestringSemantics => "geometry_linestring_semantics",
            Self::TemplateGeometrySemantics => "template_geometry_semantics",
            Self::Materials => "materials",
            Self::GeometrySurfaceMaterials => "geometry_surface_materials",
            Self::TemplateGeometryMaterials => "template_geometry_materials",
            Self::Textures => "textures",
            Self::TextureVertices => "texture_vertices",
            Self::GeometryRingTextures => "geometry_ring_textures",
            Self::TemplateGeometryRingTextures => "template_geometry_ring_textures",
        }
    }

    /// # Errors
    ///
    /// Returns an error when `name` does not match a canonical table id.
    pub fn parse(name: &str) -> Result<Self> {
        match name {
            "metadata" => Ok(Self::Metadata),
            "transform" => Ok(Self::Transform),
            "extensions" => Ok(Self::Extensions),
            "vertices" => Ok(Self::Vertices),
            "cityobjects" => Ok(Self::CityObjects),
            "cityobject_children" => Ok(Self::CityObjectChildren),
            "geometries" => Ok(Self::Geometries),
            "geometry_boundaries" => Ok(Self::GeometryBoundaries),
            "geometry_instances" => Ok(Self::GeometryInstances),
            "template_vertices" => Ok(Self::TemplateVertices),
            "template_geometries" => Ok(Self::TemplateGeometries),
            "template_geometry_boundaries" => Ok(Self::TemplateGeometryBoundaries),
            "semantics" => Ok(Self::Semantics),
            "semantic_children" => Ok(Self::SemanticChildren),
            "geometry_surface_semantics" => Ok(Self::GeometrySurfaceSemantics),
            "geometry_point_semantics" => Ok(Self::GeometryPointSemantics),
            "geometry_linestring_semantics" => Ok(Self::GeometryLinestringSemantics),
            "template_geometry_semantics" => Ok(Self::TemplateGeometrySemantics),
            "materials" => Ok(Self::Materials),
            "geometry_surface_materials" => Ok(Self::GeometrySurfaceMaterials),
            "template_geometry_materials" => Ok(Self::TemplateGeometryMaterials),
            "textures" => Ok(Self::Textures),
            "texture_vertices" => Ok(Self::TextureVertices),
            "geometry_ring_textures" => Ok(Self::GeometryRingTextures),
            "template_geometry_ring_textures" => Ok(Self::TemplateGeometryRingTextures),
            other => Err(Error::Unsupported(format!(
                "unknown canonical table '{other}' in package manifest"
            ))),
        }
    }
}

#[must_use]
pub fn collect_tables(parts: &CityModelArrowParts) -> Vec<(CanonicalTable, RecordBatch)> {
    let mut tables = vec![
        (CanonicalTable::Metadata, parts.metadata.clone()),
        (CanonicalTable::Vertices, parts.vertices.clone()),
        (CanonicalTable::CityObjects, parts.cityobjects.clone()),
        (CanonicalTable::Geometries, parts.geometries.clone()),
        (
            CanonicalTable::GeometryBoundaries,
            parts.geometry_boundaries.clone(),
        ),
    ];

    extend_optional_tables(&mut tables, parts);
    tables
}

fn extend_optional_tables(
    tables: &mut Vec<(CanonicalTable, RecordBatch)>,
    parts: &CityModelArrowParts,
) {
    for (table, batch) in [
        (CanonicalTable::Transform, parts.transform.clone()),
        (CanonicalTable::Extensions, parts.extensions.clone()),
        (
            CanonicalTable::CityObjectChildren,
            parts.cityobject_children.clone(),
        ),
        (
            CanonicalTable::GeometryInstances,
            parts.geometry_instances.clone(),
        ),
        (
            CanonicalTable::TemplateVertices,
            parts.template_vertices.clone(),
        ),
        (
            CanonicalTable::TemplateGeometries,
            parts.template_geometries.clone(),
        ),
        (
            CanonicalTable::TemplateGeometryBoundaries,
            parts.template_geometry_boundaries.clone(),
        ),
        (CanonicalTable::Semantics, parts.semantics.clone()),
        (
            CanonicalTable::SemanticChildren,
            parts.semantic_children.clone(),
        ),
        (
            CanonicalTable::GeometrySurfaceSemantics,
            parts.geometry_surface_semantics.clone(),
        ),
        (
            CanonicalTable::GeometryPointSemantics,
            parts.geometry_point_semantics.clone(),
        ),
        (
            CanonicalTable::GeometryLinestringSemantics,
            parts.geometry_linestring_semantics.clone(),
        ),
        (
            CanonicalTable::TemplateGeometrySemantics,
            parts.template_geometry_semantics.clone(),
        ),
        (CanonicalTable::Materials, parts.materials.clone()),
        (
            CanonicalTable::GeometrySurfaceMaterials,
            parts.geometry_surface_materials.clone(),
        ),
        (
            CanonicalTable::TemplateGeometryMaterials,
            parts.template_geometry_materials.clone(),
        ),
        (CanonicalTable::Textures, parts.textures.clone()),
        (
            CanonicalTable::TextureVertices,
            parts.texture_vertices.clone(),
        ),
        (
            CanonicalTable::GeometryRingTextures,
            parts.geometry_ring_textures.clone(),
        ),
        (
            CanonicalTable::TemplateGeometryRingTextures,
            parts.template_geometry_ring_textures.clone(),
        ),
    ] {
        push_optional(tables, table, batch);
    }
}

fn push_optional(
    tables: &mut Vec<(CanonicalTable, RecordBatch)>,
    table: CanonicalTable,
    batch: Option<RecordBatch>,
) {
    if let Some(batch) = batch {
        tables.push((table, batch));
    }
}

pub fn build_parts<S: std::hash::BuildHasher>(
    manifest: &PackageManifest,
    mut tables: HashMap<CanonicalTable, RecordBatch, S>,
) -> Result<CityModelArrowParts> {
    Ok(CityModelArrowParts {
        header: manifest.into(),
        projection: manifest.projection.clone(),
        metadata: required_table(&mut tables, CanonicalTable::Metadata)?,
        transform: tables.remove(&CanonicalTable::Transform),
        extensions: tables.remove(&CanonicalTable::Extensions),
        vertices: required_table(&mut tables, CanonicalTable::Vertices)?,
        cityobjects: required_table(&mut tables, CanonicalTable::CityObjects)?,
        cityobject_children: tables.remove(&CanonicalTable::CityObjectChildren),
        geometries: required_table(&mut tables, CanonicalTable::Geometries)?,
        geometry_boundaries: required_table(&mut tables, CanonicalTable::GeometryBoundaries)?,
        geometry_instances: tables.remove(&CanonicalTable::GeometryInstances),
        template_vertices: tables.remove(&CanonicalTable::TemplateVertices),
        template_geometries: tables.remove(&CanonicalTable::TemplateGeometries),
        template_geometry_boundaries: tables.remove(&CanonicalTable::TemplateGeometryBoundaries),
        semantics: tables.remove(&CanonicalTable::Semantics),
        semantic_children: tables.remove(&CanonicalTable::SemanticChildren),
        geometry_surface_semantics: tables.remove(&CanonicalTable::GeometrySurfaceSemantics),
        geometry_point_semantics: tables.remove(&CanonicalTable::GeometryPointSemantics),
        geometry_linestring_semantics: tables.remove(&CanonicalTable::GeometryLinestringSemantics),
        template_geometry_semantics: tables.remove(&CanonicalTable::TemplateGeometrySemantics),
        materials: tables.remove(&CanonicalTable::Materials),
        geometry_surface_materials: tables.remove(&CanonicalTable::GeometrySurfaceMaterials),
        template_geometry_materials: tables.remove(&CanonicalTable::TemplateGeometryMaterials),
        textures: tables.remove(&CanonicalTable::Textures),
        texture_vertices: tables.remove(&CanonicalTable::TextureVertices),
        geometry_ring_textures: tables.remove(&CanonicalTable::GeometryRingTextures),
        template_geometry_ring_textures: tables
            .remove(&CanonicalTable::TemplateGeometryRingTextures),
    })
}

fn required_table<S: std::hash::BuildHasher>(
    tables: &mut HashMap<CanonicalTable, RecordBatch, S>,
    table: CanonicalTable,
) -> Result<RecordBatch> {
    tables.remove(&table).ok_or_else(|| {
        Error::Unsupported(format!(
            "package manifest is missing required '{}' table",
            table.as_str()
        ))
    })
}

#[must_use]
pub fn schema_for_table(schemas: &CanonicalSchemaSet, table: CanonicalTable) -> &SchemaRef {
    match table {
        CanonicalTable::Metadata => &schemas.metadata,
        CanonicalTable::Transform => &schemas.transform,
        CanonicalTable::Extensions => &schemas.extensions,
        CanonicalTable::Vertices => &schemas.vertices,
        CanonicalTable::CityObjects => &schemas.cityobjects,
        CanonicalTable::CityObjectChildren => &schemas.cityobject_children,
        CanonicalTable::Geometries => &schemas.geometries,
        CanonicalTable::GeometryBoundaries => &schemas.geometry_boundaries,
        CanonicalTable::GeometryInstances => &schemas.geometry_instances,
        CanonicalTable::TemplateVertices => &schemas.template_vertices,
        CanonicalTable::TemplateGeometries => &schemas.template_geometries,
        CanonicalTable::TemplateGeometryBoundaries => &schemas.template_geometry_boundaries,
        CanonicalTable::Semantics => &schemas.semantics,
        CanonicalTable::SemanticChildren => &schemas.semantic_children,
        CanonicalTable::GeometrySurfaceSemantics => &schemas.geometry_surface_semantics,
        CanonicalTable::GeometryPointSemantics => &schemas.geometry_point_semantics,
        CanonicalTable::GeometryLinestringSemantics => &schemas.geometry_linestring_semantics,
        CanonicalTable::TemplateGeometrySemantics => &schemas.template_geometry_semantics,
        CanonicalTable::Materials => &schemas.materials,
        CanonicalTable::GeometrySurfaceMaterials => &schemas.geometry_surface_materials,
        CanonicalTable::TemplateGeometryMaterials => &schemas.template_geometry_materials,
        CanonicalTable::Textures => &schemas.textures,
        CanonicalTable::TextureVertices => &schemas.texture_vertices,
        CanonicalTable::GeometryRingTextures => &schemas.geometry_ring_textures,
        CanonicalTable::TemplateGeometryRingTextures => &schemas.template_geometry_ring_textures,
    }
}

/// Concatenates a sequence of record batches that share one schema.
///
/// # Errors
///
/// Returns an error when Arrow cannot concatenate the provided batches.
pub fn concat_record_batches(schema: &SchemaRef, batches: &[RecordBatch]) -> Result<RecordBatch> {
    if batches.is_empty() {
        return Ok(RecordBatch::new_empty(schema.clone()));
    }
    let batch_refs: Vec<&RecordBatch> = batches.iter().collect();
    arrow_select::concat::concat_batches(schema, batch_refs).map_err(Error::from)
}

/// Validates a batch schema against the canonical schema for one table.
///
/// # Errors
///
/// Returns an error when the actual schema differs from the expected schema.
pub fn validate_schema(
    expected: impl AsRef<arrow::datatypes::Schema>,
    actual: impl AsRef<arrow::datatypes::Schema>,
    table: CanonicalTable,
) -> Result<()> {
    let expected = expected.as_ref();
    let actual = actual.as_ref();

    if expected != actual {
        return Err(Error::SchemaMismatch {
            expected: format!("{}: {:?}", table.as_str(), expected),
            found: format!("{}: {:?}", table.as_str(), actual),
        });
    }
    Ok(())
}
