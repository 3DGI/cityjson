use crate::error::{Error, Result};
use crate::schema::{CanonicalSchemaSet, CityArrowHeader, CityModelArrowParts, ProjectionLayout};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    #[must_use]
    pub const fn stream_tag(self) -> u8 {
        match self {
            Self::Metadata => 0,
            Self::Transform => 1,
            Self::Extensions => 2,
            Self::Vertices => 3,
            Self::TemplateVertices => 4,
            Self::TextureVertices => 5,
            Self::Semantics => 6,
            Self::SemanticChildren => 7,
            Self::Materials => 8,
            Self::Textures => 9,
            Self::TemplateGeometryBoundaries => 10,
            Self::TemplateGeometrySemantics => 11,
            Self::TemplateGeometryMaterials => 12,
            Self::TemplateGeometryRingTextures => 13,
            Self::TemplateGeometries => 14,
            Self::GeometryBoundaries => 15,
            Self::GeometrySurfaceSemantics => 16,
            Self::GeometryPointSemantics => 17,
            Self::GeometryLinestringSemantics => 18,
            Self::GeometrySurfaceMaterials => 19,
            Self::GeometryRingTextures => 20,
            Self::GeometryInstances => 21,
            Self::Geometries => 22,
            Self::CityObjects => 23,
            Self::CityObjectChildren => 24,
        }
    }

    /// # Errors
    ///
    /// Returns an error when `tag` does not identify a canonical table frame.
    pub fn from_stream_tag(tag: u8) -> Result<Self> {
        match tag {
            0 => Ok(Self::Metadata),
            1 => Ok(Self::Transform),
            2 => Ok(Self::Extensions),
            3 => Ok(Self::Vertices),
            4 => Ok(Self::TemplateVertices),
            5 => Ok(Self::TextureVertices),
            6 => Ok(Self::Semantics),
            7 => Ok(Self::SemanticChildren),
            8 => Ok(Self::Materials),
            9 => Ok(Self::Textures),
            10 => Ok(Self::TemplateGeometryBoundaries),
            11 => Ok(Self::TemplateGeometrySemantics),
            12 => Ok(Self::TemplateGeometryMaterials),
            13 => Ok(Self::TemplateGeometryRingTextures),
            14 => Ok(Self::TemplateGeometries),
            15 => Ok(Self::GeometryBoundaries),
            16 => Ok(Self::GeometrySurfaceSemantics),
            17 => Ok(Self::GeometryPointSemantics),
            18 => Ok(Self::GeometryLinestringSemantics),
            19 => Ok(Self::GeometrySurfaceMaterials),
            20 => Ok(Self::GeometryRingTextures),
            21 => Ok(Self::GeometryInstances),
            22 => Ok(Self::Geometries),
            23 => Ok(Self::CityObjects),
            24 => Ok(Self::CityObjectChildren),
            other => Err(Error::Unsupported(format!(
                "unknown canonical stream frame tag {other}"
            ))),
        }
    }

    #[must_use]
    pub const fn is_required(self) -> bool {
        matches!(
            self,
            Self::Metadata
                | Self::Vertices
                | Self::CityObjects
                | Self::Geometries
                | Self::GeometryBoundaries
        )
    }
}

#[doc(hidden)]
pub trait CanonicalTableSink {
    /// # Errors
    ///
    /// Returns an error when the sink cannot initialize the table stream.
    fn start(&mut self, header: &CityArrowHeader, projection: &ProjectionLayout) -> Result<()>;

    /// # Errors
    ///
    /// Returns an error when the sink cannot accept the canonical table batch.
    fn push_batch(&mut self, table: CanonicalTable, batch: RecordBatch) -> Result<()>;
}

#[must_use]
pub const fn canonical_table_order() -> &'static [CanonicalTable] {
    &[
        CanonicalTable::Metadata,
        CanonicalTable::Transform,
        CanonicalTable::Extensions,
        CanonicalTable::Vertices,
        CanonicalTable::TemplateVertices,
        CanonicalTable::TextureVertices,
        CanonicalTable::Semantics,
        CanonicalTable::SemanticChildren,
        CanonicalTable::Materials,
        CanonicalTable::Textures,
        CanonicalTable::TemplateGeometryBoundaries,
        CanonicalTable::TemplateGeometrySemantics,
        CanonicalTable::TemplateGeometryMaterials,
        CanonicalTable::TemplateGeometryRingTextures,
        CanonicalTable::TemplateGeometries,
        CanonicalTable::GeometryBoundaries,
        CanonicalTable::GeometrySurfaceSemantics,
        CanonicalTable::GeometryPointSemantics,
        CanonicalTable::GeometryLinestringSemantics,
        CanonicalTable::GeometrySurfaceMaterials,
        CanonicalTable::GeometryRingTextures,
        CanonicalTable::GeometryInstances,
        CanonicalTable::Geometries,
        CanonicalTable::CityObjects,
        CanonicalTable::CityObjectChildren,
    ]
}

#[must_use]
pub fn canonical_table_position(table: CanonicalTable) -> usize {
    canonical_table_order()
        .iter()
        .position(|candidate| *candidate == table)
        .expect("canonical table order must list every table exactly once")
}

#[must_use]
pub fn collect_tables(parts: &CityModelArrowParts) -> Vec<(CanonicalTable, RecordBatch)> {
    let mut tables = Vec::new();
    for (table, batch) in [
        (CanonicalTable::Metadata, Some(parts.metadata.clone())),
        (CanonicalTable::Transform, parts.transform.clone()),
        (CanonicalTable::Extensions, parts.extensions.clone()),
        (CanonicalTable::Vertices, Some(parts.vertices.clone())),
        (
            CanonicalTable::TemplateVertices,
            parts.template_vertices.clone(),
        ),
        (
            CanonicalTable::TextureVertices,
            parts.texture_vertices.clone(),
        ),
        (CanonicalTable::Semantics, parts.semantics.clone()),
        (
            CanonicalTable::SemanticChildren,
            parts.semantic_children.clone(),
        ),
        (CanonicalTable::Materials, parts.materials.clone()),
        (CanonicalTable::Textures, parts.textures.clone()),
        (
            CanonicalTable::TemplateGeometryBoundaries,
            parts.template_geometry_boundaries.clone(),
        ),
        (
            CanonicalTable::TemplateGeometrySemantics,
            parts.template_geometry_semantics.clone(),
        ),
        (
            CanonicalTable::TemplateGeometryMaterials,
            parts.template_geometry_materials.clone(),
        ),
        (
            CanonicalTable::TemplateGeometryRingTextures,
            parts.template_geometry_ring_textures.clone(),
        ),
        (
            CanonicalTable::TemplateGeometries,
            parts.template_geometries.clone(),
        ),
        (
            CanonicalTable::GeometryBoundaries,
            Some(parts.geometry_boundaries.clone()),
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
            CanonicalTable::GeometrySurfaceMaterials,
            parts.geometry_surface_materials.clone(),
        ),
        (
            CanonicalTable::GeometryRingTextures,
            parts.geometry_ring_textures.clone(),
        ),
        (
            CanonicalTable::GeometryInstances,
            parts.geometry_instances.clone(),
        ),
        (
            CanonicalTable::Geometries,
            Some(parts.geometries.clone()),
        ),
        (
            CanonicalTable::CityObjects,
            Some(parts.cityobjects.clone()),
        ),
        (
            CanonicalTable::CityObjectChildren,
            parts.cityobject_children.clone(),
        ),
    ] {
        push_optional(&mut tables, table, batch);
    }
    tables
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
