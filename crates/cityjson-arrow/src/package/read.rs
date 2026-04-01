use super::{
    CanonicalTable, concat_record_batches, expected_schema_set, infer_cityobject_projections,
    infer_material_projection, infer_semantic_projection, infer_tail_projection,
    infer_texture_projection, package_manifest_path, table_path_from_manifest, validate_schema,
};
use crate::error::{Error, Result};
use crate::schema::{
    CanonicalSchemaSet, CityModelArrowParts, PackageManifest, PackageTableEncoding,
    ProjectionLayout,
};
use arrow::datatypes::SchemaRef;
use arrow::ipc::reader::FileReader;
use arrow::record_batch::RecordBatch;
use std::fs::File;
use std::path::Path;

/// Reads a package directory whose tables are stored as Arrow IPC files.
///
/// # Errors
///
/// Returns an error when the manifest, schemas, or table contents cannot be read
/// or do not match the canonical package layout.
pub fn read_package_ipc(dir: impl AsRef<Path>) -> Result<CityModelArrowParts> {
    read_package_ipc_dir(dir)
}

/// Reads a package directory whose tables are stored as Arrow IPC files.
///
/// # Errors
///
/// Returns an error when the manifest, schemas, or table contents cannot be read
/// or do not match the canonical package layout.
pub fn read_package_ipc_dir(dir: impl AsRef<Path>) -> Result<CityModelArrowParts> {
    read_package_dir_with_encoding(dir, Some(PackageTableEncoding::ArrowIpcFile))
}

fn read_package_dir_with_encoding(
    dir: impl AsRef<Path>,
    required_encoding: Option<PackageTableEncoding>,
) -> Result<CityModelArrowParts> {
    let dir = dir.as_ref();
    let manifest = read_manifest(dir)?;
    if let Some(required_encoding) = required_encoding
        && manifest.table_encoding != required_encoding
    {
        return Err(Error::Unsupported(format!(
            "package uses {:?} tables but {:?} was requested",
            manifest.table_encoding, required_encoding
        )));
    }
    let loaded = load_tables(dir, &manifest)?;

    let projection = infer_projection_layout(&loaded)?;
    let schemas = expected_schema_set(&projection);
    let core = read_core_tables(&loaded, &schemas)?;
    let geometry = read_geometry_tables(&loaded, &schemas)?;
    let semantics = read_semantic_tables(&loaded, &schemas)?;
    let appearance = read_appearance_tables(&loaded, &schemas)?;

    Ok(CityModelArrowParts {
        header: (&manifest).into(),
        projection,
        metadata: core.metadata,
        transform: core.transform,
        extensions: core.extensions,
        vertices: core.vertices,
        cityobjects: core.cityobjects,
        cityobject_children: core.cityobject_children,
        geometries: geometry.geometries,
        geometry_boundaries: geometry.geometry_boundaries,
        geometry_instances: geometry.geometry_instances,
        template_vertices: geometry.template_vertices,
        template_geometries: geometry.template_geometries,
        template_geometry_boundaries: geometry.template_geometry_boundaries,
        semantics: semantics.semantics,
        semantic_children: semantics.semantic_children,
        geometry_surface_semantics: semantics.geometry_surface_semantics,
        geometry_point_semantics: semantics.geometry_point_semantics,
        geometry_linestring_semantics: semantics.geometry_linestring_semantics,
        template_geometry_semantics: semantics.template_geometry_semantics,
        materials: appearance.materials,
        geometry_surface_materials: appearance.geometry_surface_materials,
        geometry_point_materials: appearance.geometry_point_materials,
        geometry_linestring_materials: appearance.geometry_linestring_materials,
        template_geometry_materials: appearance.template_geometry_materials,
        textures: appearance.textures,
        texture_vertices: appearance.texture_vertices,
        geometry_ring_textures: appearance.geometry_ring_textures,
        template_geometry_ring_textures: appearance.template_geometry_ring_textures,
    })
}

fn read_manifest(dir: &Path) -> Result<PackageManifest> {
    let file = File::open(package_manifest_path(dir))?;
    Ok(serde_json::from_reader(file)?)
}

fn load_table(
    dir: &Path,
    path: Option<&std::path::PathBuf>,
    encoding: PackageTableEncoding,
) -> Result<Option<LoadedTable>> {
    let Some(path) = path else {
        return Ok(None);
    };

    let path = table_path_from_manifest(dir, path);
    let file = File::open(&path)?;

    let (schema, batches) = match encoding {
        PackageTableEncoding::ArrowIpcFile => {
            let reader = FileReader::try_new(file, None)?;
            let schema = reader.schema();
            let batches = reader.collect::<std::result::Result<Vec<_>, _>>()?;
            (schema, batches)
        }
        PackageTableEncoding::Parquet => {
            return Err(Error::Unsupported(
                "cityarrow only supports Arrow IPC package tables".to_string(),
            ));
        }
    };
    let batch = if batches.is_empty() {
        RecordBatch::new_empty(schema.clone())
    } else {
        concat_record_batches(&schema, &batches)?
    };
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

struct CoreTables {
    metadata: RecordBatch,
    transform: Option<RecordBatch>,
    extensions: Option<RecordBatch>,
    vertices: RecordBatch,
    cityobjects: RecordBatch,
    cityobject_children: Option<RecordBatch>,
}

struct GeometryTables {
    geometries: RecordBatch,
    geometry_boundaries: RecordBatch,
    geometry_instances: Option<RecordBatch>,
    template_vertices: Option<RecordBatch>,
    template_geometries: Option<RecordBatch>,
    template_geometry_boundaries: Option<RecordBatch>,
}

struct SemanticTables {
    semantics: Option<RecordBatch>,
    semantic_children: Option<RecordBatch>,
    geometry_surface_semantics: Option<RecordBatch>,
    geometry_point_semantics: Option<RecordBatch>,
    geometry_linestring_semantics: Option<RecordBatch>,
    template_geometry_semantics: Option<RecordBatch>,
}

struct AppearanceTables {
    materials: Option<RecordBatch>,
    geometry_surface_materials: Option<RecordBatch>,
    geometry_point_materials: Option<RecordBatch>,
    geometry_linestring_materials: Option<RecordBatch>,
    template_geometry_materials: Option<RecordBatch>,
    textures: Option<RecordBatch>,
    texture_vertices: Option<RecordBatch>,
    geometry_ring_textures: Option<RecordBatch>,
    template_geometry_ring_textures: Option<RecordBatch>,
}

fn load_tables(dir: &Path, manifest: &PackageManifest) -> Result<LoadedTables> {
    let encoding = manifest.table_encoding;
    Ok(LoadedTables {
        metadata: load_table(dir, manifest.tables.metadata.as_ref(), encoding)?,
        transform: load_table(dir, manifest.tables.transform.as_ref(), encoding)?,
        extensions: load_table(dir, manifest.tables.extensions.as_ref(), encoding)?,
        vertices: load_table(dir, manifest.tables.vertices.as_ref(), encoding)?,
        cityobjects: load_table(dir, manifest.tables.cityobjects.as_ref(), encoding)?,
        cityobject_children: load_table(
            dir,
            manifest.tables.cityobject_children.as_ref(),
            encoding,
        )?,
        geometries: load_table(dir, manifest.tables.geometries.as_ref(), encoding)?,
        geometry_boundaries: load_table(
            dir,
            manifest.tables.geometry_boundaries.as_ref(),
            encoding,
        )?,
        geometry_instances: load_table(dir, manifest.tables.geometry_instances.as_ref(), encoding)?,
        template_vertices: load_table(dir, manifest.tables.template_vertices.as_ref(), encoding)?,
        template_geometries: load_table(
            dir,
            manifest.tables.template_geometries.as_ref(),
            encoding,
        )?,
        template_geometry_boundaries: load_table(
            dir,
            manifest.tables.template_geometry_boundaries.as_ref(),
            encoding,
        )?,
        semantics: load_table(dir, manifest.tables.semantics.as_ref(), encoding)?,
        semantic_children: load_table(dir, manifest.tables.semantic_children.as_ref(), encoding)?,
        geometry_surface_semantics: load_table(
            dir,
            manifest.tables.geometry_surface_semantics.as_ref(),
            encoding,
        )?,
        geometry_point_semantics: load_table(
            dir,
            manifest.tables.geometry_point_semantics.as_ref(),
            encoding,
        )?,
        geometry_linestring_semantics: load_table(
            dir,
            manifest.tables.geometry_linestring_semantics.as_ref(),
            encoding,
        )?,
        template_geometry_semantics: load_table(
            dir,
            manifest.tables.template_geometry_semantics.as_ref(),
            encoding,
        )?,
        materials: load_table(dir, manifest.tables.materials.as_ref(), encoding)?,
        geometry_surface_materials: load_table(
            dir,
            manifest.tables.geometry_surface_materials.as_ref(),
            encoding,
        )?,
        geometry_point_materials: load_table(
            dir,
            manifest.tables.geometry_point_materials.as_ref(),
            encoding,
        )?,
        geometry_linestring_materials: load_table(
            dir,
            manifest.tables.geometry_linestring_materials.as_ref(),
            encoding,
        )?,
        template_geometry_materials: load_table(
            dir,
            manifest.tables.template_geometry_materials.as_ref(),
            encoding,
        )?,
        textures: load_table(dir, manifest.tables.textures.as_ref(), encoding)?,
        texture_vertices: load_table(dir, manifest.tables.texture_vertices.as_ref(), encoding)?,
        geometry_ring_textures: load_table(
            dir,
            manifest.tables.geometry_ring_textures.as_ref(),
            encoding,
        )?,
        template_geometry_ring_textures: load_table(
            dir,
            manifest.tables.template_geometry_ring_textures.as_ref(),
            encoding,
        )?,
    })
}

fn read_core_tables(loaded: &LoadedTables, schemas: &CanonicalSchemaSet) -> Result<CoreTables> {
    let metadata = required_table(
        loaded.metadata.as_ref(),
        CanonicalTable::Metadata,
        &schemas.metadata,
    )?;
    ensure_exact_row_count(&metadata, 1, CanonicalTable::Metadata)?;

    let transform = optional_table(
        loaded.transform.as_ref(),
        CanonicalTable::Transform,
        &schemas.transform,
    )?;
    if let Some(table) = &transform {
        ensure_max_row_count(table, 1, CanonicalTable::Transform)?;
    }

    Ok(CoreTables {
        metadata,
        transform,
        extensions: optional_table(
            loaded.extensions.as_ref(),
            CanonicalTable::Extensions,
            &schemas.extensions,
        )?,
        vertices: required_table(
            loaded.vertices.as_ref(),
            CanonicalTable::Vertices,
            &schemas.vertices,
        )?,
        cityobjects: required_table(
            loaded.cityobjects.as_ref(),
            CanonicalTable::CityObjects,
            &schemas.cityobjects,
        )?,
        cityobject_children: optional_table(
            loaded.cityobject_children.as_ref(),
            CanonicalTable::CityObjectChildren,
            &schemas.cityobject_children,
        )?,
    })
}

fn read_geometry_tables(
    loaded: &LoadedTables,
    schemas: &CanonicalSchemaSet,
) -> Result<GeometryTables> {
    let geometries = required_table(
        loaded.geometries.as_ref(),
        CanonicalTable::Geometries,
        &schemas.geometries,
    )?;
    let geometry_boundaries = required_table(
        loaded.geometry_boundaries.as_ref(),
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

    let template_geometries = optional_table(
        loaded.template_geometries.as_ref(),
        CanonicalTable::TemplateGeometries,
        &schemas.template_geometries,
    )?;
    let template_geometry_boundaries = optional_table(
        loaded.template_geometry_boundaries.as_ref(),
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

    Ok(GeometryTables {
        geometries,
        geometry_boundaries,
        geometry_instances: optional_table(
            loaded.geometry_instances.as_ref(),
            CanonicalTable::GeometryInstances,
            &schemas.geometry_instances,
        )?,
        template_vertices: optional_table(
            loaded.template_vertices.as_ref(),
            CanonicalTable::TemplateVertices,
            &schemas.template_vertices,
        )?,
        template_geometries,
        template_geometry_boundaries,
    })
}

fn read_semantic_tables(
    loaded: &LoadedTables,
    schemas: &CanonicalSchemaSet,
) -> Result<SemanticTables> {
    Ok(SemanticTables {
        semantics: optional_table(
            loaded.semantics.as_ref(),
            CanonicalTable::Semantics,
            &schemas.semantics,
        )?,
        semantic_children: optional_table(
            loaded.semantic_children.as_ref(),
            CanonicalTable::SemanticChildren,
            &schemas.semantic_children,
        )?,
        geometry_surface_semantics: optional_table(
            loaded.geometry_surface_semantics.as_ref(),
            CanonicalTable::GeometrySurfaceSemantics,
            &schemas.geometry_surface_semantics,
        )?,
        geometry_point_semantics: optional_table(
            loaded.geometry_point_semantics.as_ref(),
            CanonicalTable::GeometryPointSemantics,
            &schemas.geometry_point_semantics,
        )?,
        geometry_linestring_semantics: optional_table(
            loaded.geometry_linestring_semantics.as_ref(),
            CanonicalTable::GeometryLinestringSemantics,
            &schemas.geometry_linestring_semantics,
        )?,
        template_geometry_semantics: optional_table(
            loaded.template_geometry_semantics.as_ref(),
            CanonicalTable::TemplateGeometrySemantics,
            &schemas.template_geometry_semantics,
        )?,
    })
}

fn read_appearance_tables(
    loaded: &LoadedTables,
    schemas: &CanonicalSchemaSet,
) -> Result<AppearanceTables> {
    Ok(AppearanceTables {
        materials: optional_table(
            loaded.materials.as_ref(),
            CanonicalTable::Materials,
            &schemas.materials,
        )?,
        geometry_surface_materials: optional_table(
            loaded.geometry_surface_materials.as_ref(),
            CanonicalTable::GeometrySurfaceMaterials,
            &schemas.geometry_surface_materials,
        )?,
        geometry_point_materials: optional_table(
            loaded.geometry_point_materials.as_ref(),
            CanonicalTable::GeometryPointMaterials,
            &schemas.geometry_point_materials,
        )?,
        geometry_linestring_materials: optional_table(
            loaded.geometry_linestring_materials.as_ref(),
            CanonicalTable::GeometryLinestringMaterials,
            &schemas.geometry_linestring_materials,
        )?,
        template_geometry_materials: optional_table(
            loaded.template_geometry_materials.as_ref(),
            CanonicalTable::TemplateGeometryMaterials,
            &schemas.template_geometry_materials,
        )?,
        textures: optional_table(
            loaded.textures.as_ref(),
            CanonicalTable::Textures,
            &schemas.textures,
        )?,
        texture_vertices: optional_table(
            loaded.texture_vertices.as_ref(),
            CanonicalTable::TextureVertices,
            &schemas.texture_vertices,
        )?,
        geometry_ring_textures: optional_table(
            loaded.geometry_ring_textures.as_ref(),
            CanonicalTable::GeometryRingTextures,
            &schemas.geometry_ring_textures,
        )?,
        template_geometry_ring_textures: optional_table(
            loaded.template_geometry_ring_textures.as_ref(),
            CanonicalTable::TemplateGeometryRingTextures,
            &schemas.template_geometry_ring_textures,
        )?,
    })
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
    table: Option<&LoadedTable>,
    kind: CanonicalTable,
    expected: &SchemaRef,
) -> Result<RecordBatch> {
    let loaded = table.ok_or_else(|| Error::MissingField(kind.file_name().to_string()))?;
    validate_schema(expected, &loaded.schema, kind)?;
    Ok(loaded.batch.clone())
}

fn optional_table(
    table: Option<&LoadedTable>,
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
