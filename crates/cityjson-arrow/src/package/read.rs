use super::{
    CanonicalTable, concat_record_batches, expected_schema_set, infer_cityobject_projections,
    infer_material_projection, infer_semantic_projection, infer_tail_projection,
    infer_texture_projection, package_manifest_path, table_path_from_manifest, validate_schema,
};
use crate::error::{Error, Result};
use crate::schema::{CityModelArrowParts, PackageManifest, ProjectionLayout};
use arrow::array::RecordBatchReader;
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use std::fs::File;
use std::path::Path;

pub fn read_package(dir: impl AsRef<Path>) -> Result<CityModelArrowParts> {
    read_package_dir(dir)
}

pub fn read_package_dir(dir: impl AsRef<Path>) -> Result<CityModelArrowParts> {
    let dir = dir.as_ref();
    let manifest = read_manifest(dir)?;

    let loaded = LoadedTables {
        metadata: load_table(dir, manifest.tables.metadata.as_ref())?,
        transform: load_table(dir, manifest.tables.transform.as_ref())?,
        extensions: load_table(dir, manifest.tables.extensions.as_ref())?,
        vertices: load_table(dir, manifest.tables.vertices.as_ref())?,
        cityobjects: load_table(dir, manifest.tables.cityobjects.as_ref())?,
        cityobject_children: load_table(dir, manifest.tables.cityobject_children.as_ref())?,
        geometries: load_table(dir, manifest.tables.geometries.as_ref())?,
        geometry_boundaries: load_table(dir, manifest.tables.geometry_boundaries.as_ref())?,
        geometry_instances: load_table(dir, manifest.tables.geometry_instances.as_ref())?,
        template_vertices: load_table(dir, manifest.tables.template_vertices.as_ref())?,
        template_geometries: load_table(dir, manifest.tables.template_geometries.as_ref())?,
        template_geometry_boundaries: load_table(
            dir,
            manifest.tables.template_geometry_boundaries.as_ref(),
        )?,
        semantics: load_table(dir, manifest.tables.semantics.as_ref())?,
        semantic_children: load_table(dir, manifest.tables.semantic_children.as_ref())?,
        geometry_surface_semantics: load_table(
            dir,
            manifest.tables.geometry_surface_semantics.as_ref(),
        )?,
        geometry_point_semantics: load_table(
            dir,
            manifest.tables.geometry_point_semantics.as_ref(),
        )?,
        geometry_linestring_semantics: load_table(
            dir,
            manifest.tables.geometry_linestring_semantics.as_ref(),
        )?,
        template_geometry_semantics: load_table(
            dir,
            manifest.tables.template_geometry_semantics.as_ref(),
        )?,
        materials: load_table(dir, manifest.tables.materials.as_ref())?,
        geometry_surface_materials: load_table(
            dir,
            manifest.tables.geometry_surface_materials.as_ref(),
        )?,
        geometry_point_materials: load_table(
            dir,
            manifest.tables.geometry_point_materials.as_ref(),
        )?,
        geometry_linestring_materials: load_table(
            dir,
            manifest.tables.geometry_linestring_materials.as_ref(),
        )?,
        template_geometry_materials: load_table(
            dir,
            manifest.tables.template_geometry_materials.as_ref(),
        )?,
        textures: load_table(dir, manifest.tables.textures.as_ref())?,
        texture_vertices: load_table(dir, manifest.tables.texture_vertices.as_ref())?,
        geometry_ring_textures: load_table(dir, manifest.tables.geometry_ring_textures.as_ref())?,
        template_geometry_ring_textures: load_table(
            dir,
            manifest.tables.template_geometry_ring_textures.as_ref(),
        )?,
    };

    let projection = infer_projection_layout(&loaded)?;
    let schemas = expected_schema_set(&projection);

    let metadata = required_table(
        &loaded.metadata,
        CanonicalTable::Metadata,
        &schemas.metadata,
    )?;
    ensure_exact_row_count(&metadata, 1, CanonicalTable::Metadata)?;

    let transform = optional_table(
        &loaded.transform,
        CanonicalTable::Transform,
        &schemas.transform,
    )?;
    if let Some(table) = &transform {
        ensure_max_row_count(table, 1, CanonicalTable::Transform)?;
    }

    let extensions = optional_table(
        &loaded.extensions,
        CanonicalTable::Extensions,
        &schemas.extensions,
    )?;

    let vertices = required_table(
        &loaded.vertices,
        CanonicalTable::Vertices,
        &schemas.vertices,
    )?;

    let cityobjects = required_table(
        &loaded.cityobjects,
        CanonicalTable::CityObjects,
        &schemas.cityobjects,
    )?;

    let cityobject_children = optional_table(
        &loaded.cityobject_children,
        CanonicalTable::CityObjectChildren,
        &schemas.cityobject_children,
    )?;

    let geometries = required_table(
        &loaded.geometries,
        CanonicalTable::Geometries,
        &schemas.geometries,
    )?;
    let geometry_boundaries = required_table(
        &loaded.geometry_boundaries,
        CanonicalTable::GeometryBoundaries,
        &schemas.geometry_boundaries,
    )?;
    ensure_paired_geometry_tables(
        &geometries,
        &geometry_boundaries,
        "geometry_id",
        "geometries",
        "geometry_boundaries",
    )?;

    let geometry_instances = optional_table(
        &loaded.geometry_instances,
        CanonicalTable::GeometryInstances,
        &schemas.geometry_instances,
    )?;

    let template_vertices = optional_table(
        &loaded.template_vertices,
        CanonicalTable::TemplateVertices,
        &schemas.template_vertices,
    )?;

    let template_geometries = optional_table(
        &loaded.template_geometries,
        CanonicalTable::TemplateGeometries,
        &schemas.template_geometries,
    )?;

    let template_geometry_boundaries = optional_table(
        &loaded.template_geometry_boundaries,
        CanonicalTable::TemplateGeometryBoundaries,
        &schemas.template_geometry_boundaries,
    )?;
    if let (Some(template_geometries), Some(template_geometry_boundaries)) =
        (&template_geometries, &template_geometry_boundaries)
    {
        ensure_paired_geometry_tables(
            template_geometries,
            template_geometry_boundaries,
            "template_geometry_id",
            "template_geometries",
            "template_geometry_boundaries",
        )?;
    }

    let semantics = optional_table(
        &loaded.semantics,
        CanonicalTable::Semantics,
        &schemas.semantics,
    )?;

    let semantic_children = optional_table(
        &loaded.semantic_children,
        CanonicalTable::SemanticChildren,
        &schemas.semantic_children,
    )?;

    let geometry_surface_semantics = optional_table(
        &loaded.geometry_surface_semantics,
        CanonicalTable::GeometrySurfaceSemantics,
        &schemas.geometry_surface_semantics,
    )?;

    let geometry_point_semantics = optional_table(
        &loaded.geometry_point_semantics,
        CanonicalTable::GeometryPointSemantics,
        &schemas.geometry_point_semantics,
    )?;

    let geometry_linestring_semantics = optional_table(
        &loaded.geometry_linestring_semantics,
        CanonicalTable::GeometryLinestringSemantics,
        &schemas.geometry_linestring_semantics,
    )?;

    let template_geometry_semantics = optional_table(
        &loaded.template_geometry_semantics,
        CanonicalTable::TemplateGeometrySemantics,
        &schemas.template_geometry_semantics,
    )?;

    let materials = optional_table(
        &loaded.materials,
        CanonicalTable::Materials,
        &schemas.materials,
    )?;

    let geometry_surface_materials = optional_table(
        &loaded.geometry_surface_materials,
        CanonicalTable::GeometrySurfaceMaterials,
        &schemas.geometry_surface_materials,
    )?;

    let geometry_point_materials = optional_table(
        &loaded.geometry_point_materials,
        CanonicalTable::GeometryPointMaterials,
        &schemas.geometry_point_materials,
    )?;

    let geometry_linestring_materials = optional_table(
        &loaded.geometry_linestring_materials,
        CanonicalTable::GeometryLinestringMaterials,
        &schemas.geometry_linestring_materials,
    )?;

    let template_geometry_materials = optional_table(
        &loaded.template_geometry_materials,
        CanonicalTable::TemplateGeometryMaterials,
        &schemas.template_geometry_materials,
    )?;

    let textures = optional_table(
        &loaded.textures,
        CanonicalTable::Textures,
        &schemas.textures,
    )?;

    let texture_vertices = optional_table(
        &loaded.texture_vertices,
        CanonicalTable::TextureVertices,
        &schemas.texture_vertices,
    )?;

    let geometry_ring_textures = optional_table(
        &loaded.geometry_ring_textures,
        CanonicalTable::GeometryRingTextures,
        &schemas.geometry_ring_textures,
    )?;

    let template_geometry_ring_textures = optional_table(
        &loaded.template_geometry_ring_textures,
        CanonicalTable::TemplateGeometryRingTextures,
        &schemas.template_geometry_ring_textures,
    )?;

    Ok(CityModelArrowParts {
        header: (&manifest).into(),
        projection,
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
        semantic_children,
        geometry_surface_semantics,
        geometry_point_semantics,
        geometry_linestring_semantics,
        template_geometry_semantics,
        materials,
        geometry_surface_materials,
        geometry_point_materials,
        geometry_linestring_materials,
        template_geometry_materials,
        textures,
        texture_vertices,
        geometry_ring_textures,
        template_geometry_ring_textures,
    })
}

fn read_manifest(dir: &Path) -> Result<PackageManifest> {
    let file = File::open(package_manifest_path(dir))?;
    Ok(serde_json::from_reader(file)?)
}

fn load_table(dir: &Path, path: Option<&std::path::PathBuf>) -> Result<Option<LoadedTable>> {
    let Some(path) = path else {
        return Ok(None);
    };

    let path = table_path_from_manifest(dir, path);
    let file = File::open(&path)?;
    let mut reader = ParquetRecordBatchReaderBuilder::try_new(file)?
        .with_batch_size(1024)
        .build()?;
    let schema = reader.schema().clone();
    let mut batches = Vec::new();
    while let Some(batch) = reader.next() {
        batches.push(batch?);
    }
    let batch = concat_record_batches(&schema, &batches)?;
    Ok(Some(LoadedTable { schema, batch }))
}

#[derive(Default)]
struct LoadedTables {
    metadata: Option<LoadedTable>,
    transform: Option<LoadedTable>,
    extensions: Option<LoadedTable>,
    vertices: Option<LoadedTable>,
    cityobjects: Option<LoadedTable>,
    cityobject_children: Option<LoadedTable>,
    geometries: Option<LoadedTable>,
    geometry_boundaries: Option<LoadedTable>,
    geometry_instances: Option<LoadedTable>,
    template_vertices: Option<LoadedTable>,
    template_geometries: Option<LoadedTable>,
    template_geometry_boundaries: Option<LoadedTable>,
    semantics: Option<LoadedTable>,
    semantic_children: Option<LoadedTable>,
    geometry_surface_semantics: Option<LoadedTable>,
    geometry_point_semantics: Option<LoadedTable>,
    geometry_linestring_semantics: Option<LoadedTable>,
    template_geometry_semantics: Option<LoadedTable>,
    materials: Option<LoadedTable>,
    geometry_surface_materials: Option<LoadedTable>,
    geometry_point_materials: Option<LoadedTable>,
    geometry_linestring_materials: Option<LoadedTable>,
    template_geometry_materials: Option<LoadedTable>,
    textures: Option<LoadedTable>,
    texture_vertices: Option<LoadedTable>,
    geometry_ring_textures: Option<LoadedTable>,
    template_geometry_ring_textures: Option<LoadedTable>,
}

struct LoadedTable {
    schema: SchemaRef,
    batch: RecordBatch,
}

fn infer_projection_layout(loaded: &LoadedTables) -> Result<ProjectionLayout> {
    let mut layout = ProjectionLayout::default();

    if let Some(table) = &loaded.metadata {
        layout.metadata_extra = infer_tail_projection(table.schema.as_ref(), 7)?;
    }

    if let Some(table) = &loaded.cityobjects {
        let (attributes, extra) = infer_cityobject_projections(table.schema.as_ref())?;
        layout.cityobject_attributes = attributes;
        layout.cityobject_extra = extra;
    }

    layout.geometry_extra = infer_consistent_geometry_projection(loaded)?;

    if let Some(table) = &loaded.semantics {
        layout.semantic_attributes = infer_semantic_projection(table.schema.as_ref())?;
    }

    if let Some(table) = &loaded.materials {
        layout.material_payload = infer_material_projection(table.schema.as_ref())?;
    }

    if let Some(table) = &loaded.textures {
        layout.texture_payload = infer_texture_projection(table.schema.as_ref())?;
    }

    Ok(layout)
}

fn infer_consistent_geometry_projection(
    loaded: &LoadedTables,
) -> Result<Vec<crate::schema::ProjectedFieldSpec>> {
    let mut geometry_extra: Option<Vec<_>> = None;

    for (schema, start_index) in [
        loaded
            .geometries
            .as_ref()
            .map(|table| (table.schema.as_ref(), 6usize)),
        loaded
            .geometry_instances
            .as_ref()
            .map(|table| (table.schema.as_ref(), 8usize)),
        loaded
            .template_geometries
            .as_ref()
            .map(|table| (table.schema.as_ref(), 4usize)),
    ]
    .into_iter()
    .flatten()
    {
        let candidate = infer_tail_projection(schema, start_index)?;
        match &geometry_extra {
            Some(existing) if *existing != candidate => {
                return Err(Error::Unsupported(
                    "geometry-related tables disagree on projected fields".to_string(),
                ));
            }
            None => geometry_extra = Some(candidate),
            _ => {}
        }
    }

    Ok(geometry_extra.unwrap_or_default())
}

fn required_table(
    table: &Option<LoadedTable>,
    kind: CanonicalTable,
    expected: &SchemaRef,
) -> Result<RecordBatch> {
    let loaded = table
        .as_ref()
        .ok_or_else(|| Error::MissingField(kind.file_name().to_string()))?;
    validate_schema(expected, &loaded.schema, kind)?;
    Ok(loaded.batch.clone())
}

fn optional_table(
    table: &Option<LoadedTable>,
    kind: CanonicalTable,
    expected: &SchemaRef,
) -> Result<Option<RecordBatch>> {
    match table {
        Some(loaded) => {
            validate_schema(expected, &loaded.schema, kind)?;
            Ok(Some(loaded.batch.clone()))
        }
        None => Ok(None),
    }
}

fn ensure_exact_row_count(
    batch: &RecordBatch,
    expected: usize,
    table: CanonicalTable,
) -> Result<()> {
    if batch.num_rows() != expected {
        return Err(Error::Unsupported(format!(
            "{} must contain exactly {expected} rows, found {}",
            table.file_name(),
            batch.num_rows()
        )));
    }
    Ok(())
}

fn ensure_max_row_count(batch: &RecordBatch, max: usize, table: CanonicalTable) -> Result<()> {
    if batch.num_rows() > max {
        return Err(Error::Unsupported(format!(
            "{} must contain at most {max} rows, found {}",
            table.file_name(),
            batch.num_rows()
        )));
    }
    Ok(())
}

fn ensure_paired_geometry_tables(
    geometries: &RecordBatch,
    boundaries: &RecordBatch,
    id_field: &str,
    left_name: &str,
    right_name: &str,
) -> Result<()> {
    if geometries.num_rows() != boundaries.num_rows() {
        return Err(Error::Unsupported(format!(
            "{left_name} and {right_name} must have the same row count"
        )));
    }

    let left_ids = geometries
        .column_by_name(id_field)
        .ok_or_else(|| Error::MissingField(id_field.to_string()))?;
    let right_ids = boundaries
        .column_by_name(id_field)
        .ok_or_else(|| Error::MissingField(id_field.to_string()))?;

    let left_ids = left_ids
        .as_any()
        .downcast_ref::<arrow::array::UInt64Array>()
        .ok_or_else(|| Error::Conversion(format!("failed to read {left_name}.{id_field}")))?;
    let right_ids = right_ids
        .as_any()
        .downcast_ref::<arrow::array::UInt64Array>()
        .ok_or_else(|| Error::Conversion(format!("failed to read {right_name}.{id_field}")))?;

    for index in 0..left_ids.len() {
        if left_ids.value(index) != right_ids.value(index) {
            return Err(Error::Unsupported(format!(
                "{left_name} and {right_name} must be aligned by {id_field}"
            )));
        }
    }

    Ok(())
}
