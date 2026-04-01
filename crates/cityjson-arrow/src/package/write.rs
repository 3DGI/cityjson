use super::{
    CanonicalTable, expected_schema_set, package_manifest_path, package_table_path_for_encoding,
    validate_schema,
};
use crate::error::{Error, Result};
use crate::schema::{
    CanonicalSchemaSet, CityModelArrowParts, PackageManifest, PackageTableEncoding,
};
use arrow::array::{Array, UInt64Array};
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;
use std::fs::{self, File};
use std::path::Path;

/// Writes a package directory whose tables are stored as Arrow IPC files.
///
/// # Errors
///
/// Returns an error when schemas are invalid, tables are inconsistent, or the
/// package files cannot be written.
pub fn write_package_ipc(
    dir: impl AsRef<Path>,
    parts: &CityModelArrowParts,
) -> Result<PackageManifest> {
    write_package_ipc_dir(dir, parts)
}

/// Writes a package directory whose tables are stored as Arrow IPC files.
///
/// # Errors
///
/// Returns an error when schemas are invalid, tables are inconsistent, or the
/// package files cannot be written.
pub fn write_package_ipc_dir(
    dir: impl AsRef<Path>,
    parts: &CityModelArrowParts,
) -> Result<PackageManifest> {
    write_package_dir_with_encoding(dir, parts, PackageTableEncoding::ArrowIpcFile)
}

fn write_package_dir_with_encoding(
    dir: impl AsRef<Path>,
    parts: &CityModelArrowParts,
    encoding: PackageTableEncoding,
) -> Result<PackageManifest> {
    let dir = dir.as_ref();
    fs::create_dir_all(dir)?;

    let schemas = expected_schema_set(&parts.projection);
    let mut manifest = PackageManifest::new(
        parts.header.citymodel_id.clone(),
        parts.header.cityjson_version.clone(),
    );
    manifest.package_schema = parts.header.package_version;
    manifest.table_encoding = encoding;
    write_core_tables(dir, parts, &schemas, encoding, &mut manifest)?;
    write_geometry_tables(dir, parts, &schemas, encoding, &mut manifest)?;
    write_semantic_tables(dir, parts, &schemas, encoding, &mut manifest)?;
    write_appearance_tables(dir, parts, &schemas, encoding, &mut manifest)?;

    let manifest_path = package_manifest_path(dir);
    let file = File::create(&manifest_path)?;
    serde_json::to_writer_pretty(file, &manifest)?;

    Ok(manifest)
}

fn write_core_tables(
    dir: &Path,
    parts: &CityModelArrowParts,
    schemas: &CanonicalSchemaSet,
    encoding: PackageTableEncoding,
    manifest: &mut PackageManifest,
) -> Result<()> {
    validate_schema(
        &schemas.metadata,
        parts.metadata.schema(),
        CanonicalTable::Metadata,
    )?;
    ensure_single_row(&parts.metadata, CanonicalTable::Metadata)?;
    write_batch(dir, CanonicalTable::Metadata, &parts.metadata, encoding)?;
    manifest.tables.metadata = Some(CanonicalTable::Metadata.file_name_for(encoding).into());

    if let Some(transform) = &parts.transform {
        validate_schema(
            &schemas.transform,
            transform.schema(),
            CanonicalTable::Transform,
        )?;
        ensure_max_one_row(transform, CanonicalTable::Transform)?;
        write_batch(dir, CanonicalTable::Transform, transform, encoding)?;
        manifest.tables.transform = Some(CanonicalTable::Transform.file_name_for(encoding).into());
    }

    if let Some(extensions) = &parts.extensions {
        validate_schema(
            &schemas.extensions,
            extensions.schema(),
            CanonicalTable::Extensions,
        )?;
        write_batch(dir, CanonicalTable::Extensions, extensions, encoding)?;
        manifest.tables.extensions =
            Some(CanonicalTable::Extensions.file_name_for(encoding).into());
    }

    validate_schema(
        &schemas.vertices,
        parts.vertices.schema(),
        CanonicalTable::Vertices,
    )?;
    write_batch(dir, CanonicalTable::Vertices, &parts.vertices, encoding)?;
    manifest.tables.vertices = Some(CanonicalTable::Vertices.file_name_for(encoding).into());

    validate_schema(
        &schemas.cityobjects,
        parts.cityobjects.schema(),
        CanonicalTable::CityObjects,
    )?;
    write_batch(
        dir,
        CanonicalTable::CityObjects,
        &parts.cityobjects,
        encoding,
    )?;
    manifest.tables.cityobjects = Some(CanonicalTable::CityObjects.file_name_for(encoding).into());

    if let Some(children) = &parts.cityobject_children {
        validate_schema(
            &schemas.cityobject_children,
            children.schema(),
            CanonicalTable::CityObjectChildren,
        )?;
        write_batch(dir, CanonicalTable::CityObjectChildren, children, encoding)?;
        manifest.tables.cityobject_children = Some(
            CanonicalTable::CityObjectChildren
                .file_name_for(encoding)
                .into(),
        );
    }

    Ok(())
}

fn write_geometry_tables(
    dir: &Path,
    parts: &CityModelArrowParts,
    schemas: &CanonicalSchemaSet,
    encoding: PackageTableEncoding,
    manifest: &mut PackageManifest,
) -> Result<()> {
    validate_schema(
        &schemas.geometries,
        parts.geometries.schema(),
        CanonicalTable::Geometries,
    )?;
    validate_schema(
        &schemas.geometry_boundaries,
        parts.geometry_boundaries.schema(),
        CanonicalTable::GeometryBoundaries,
    )?;
    ensure_paired_geometry_tables(
        &parts.geometries,
        &parts.geometry_boundaries,
        "geometry_id",
        "geometries",
        "geometry_boundaries",
    )?;
    write_batch(dir, CanonicalTable::Geometries, &parts.geometries, encoding)?;
    write_batch(
        dir,
        CanonicalTable::GeometryBoundaries,
        &parts.geometry_boundaries,
        encoding,
    )?;
    manifest.tables.geometries = Some(CanonicalTable::Geometries.file_name_for(encoding).into());
    manifest.tables.geometry_boundaries = Some(
        CanonicalTable::GeometryBoundaries
            .file_name_for(encoding)
            .into(),
    );

    if let Some(geometry_instances) = &parts.geometry_instances {
        validate_schema(
            &schemas.geometry_instances,
            geometry_instances.schema(),
            CanonicalTable::GeometryInstances,
        )?;
        write_batch(
            dir,
            CanonicalTable::GeometryInstances,
            geometry_instances,
            encoding,
        )?;
        manifest.tables.geometry_instances = Some(
            CanonicalTable::GeometryInstances
                .file_name_for(encoding)
                .into(),
        );
    }

    if let Some(template_vertices) = &parts.template_vertices {
        validate_schema(
            &schemas.template_vertices,
            template_vertices.schema(),
            CanonicalTable::TemplateVertices,
        )?;
        write_batch(
            dir,
            CanonicalTable::TemplateVertices,
            template_vertices,
            encoding,
        )?;
        manifest.tables.template_vertices = Some(
            CanonicalTable::TemplateVertices
                .file_name_for(encoding)
                .into(),
        );
    }

    write_template_geometry_tables(dir, parts, schemas, encoding, manifest)?;

    Ok(())
}

fn write_template_geometry_tables(
    dir: &Path,
    parts: &CityModelArrowParts,
    schemas: &CanonicalSchemaSet,
    encoding: PackageTableEncoding,
    manifest: &mut PackageManifest,
) -> Result<()> {
    match (
        &parts.template_geometries,
        &parts.template_geometry_boundaries,
    ) {
        (Some(template_geometries), Some(template_geometry_boundaries)) => {
            validate_schema(
                &schemas.template_geometries,
                template_geometries.schema(),
                CanonicalTable::TemplateGeometries,
            )?;
            validate_schema(
                &schemas.template_geometry_boundaries,
                template_geometry_boundaries.schema(),
                CanonicalTable::TemplateGeometryBoundaries,
            )?;
            ensure_paired_geometry_tables(
                template_geometries,
                template_geometry_boundaries,
                "template_geometry_id",
                "template_geometries",
                "template_geometry_boundaries",
            )?;
            write_batch(
                dir,
                CanonicalTable::TemplateGeometries,
                template_geometries,
                encoding,
            )?;
            write_batch(
                dir,
                CanonicalTable::TemplateGeometryBoundaries,
                template_geometry_boundaries,
                encoding,
            )?;
            manifest.tables.template_geometries = Some(
                CanonicalTable::TemplateGeometries
                    .file_name_for(encoding)
                    .into(),
            );
            manifest.tables.template_geometry_boundaries = Some(
                CanonicalTable::TemplateGeometryBoundaries
                    .file_name_for(encoding)
                    .into(),
            );
        }
        (None, None) => {}
        (Some(_), None) | (None, Some(_)) => {
            return Err(Error::Unsupported(
                "template_geometries and template_geometry_boundaries must either both be present or both be absent".to_string(),
            ));
        }
    }

    Ok(())
}

fn write_semantic_tables(
    dir: &Path,
    parts: &CityModelArrowParts,
    schemas: &CanonicalSchemaSet,
    encoding: PackageTableEncoding,
    manifest: &mut PackageManifest,
) -> Result<()> {
    maybe_write_table(
        dir,
        parts.semantics.as_ref(),
        &schemas.semantics,
        CanonicalTable::Semantics,
        encoding,
        &mut manifest.tables.semantics,
    )?;
    maybe_write_table(
        dir,
        parts.semantic_children.as_ref(),
        &schemas.semantic_children,
        CanonicalTable::SemanticChildren,
        encoding,
        &mut manifest.tables.semantic_children,
    )?;
    maybe_write_table(
        dir,
        parts.geometry_surface_semantics.as_ref(),
        &schemas.geometry_surface_semantics,
        CanonicalTable::GeometrySurfaceSemantics,
        encoding,
        &mut manifest.tables.geometry_surface_semantics,
    )?;
    maybe_write_table(
        dir,
        parts.geometry_point_semantics.as_ref(),
        &schemas.geometry_point_semantics,
        CanonicalTable::GeometryPointSemantics,
        encoding,
        &mut manifest.tables.geometry_point_semantics,
    )?;
    maybe_write_table(
        dir,
        parts.geometry_linestring_semantics.as_ref(),
        &schemas.geometry_linestring_semantics,
        CanonicalTable::GeometryLinestringSemantics,
        encoding,
        &mut manifest.tables.geometry_linestring_semantics,
    )?;
    maybe_write_table(
        dir,
        parts.template_geometry_semantics.as_ref(),
        &schemas.template_geometry_semantics,
        CanonicalTable::TemplateGeometrySemantics,
        encoding,
        &mut manifest.tables.template_geometry_semantics,
    )?;
    Ok(())
}

fn write_appearance_tables(
    dir: &Path,
    parts: &CityModelArrowParts,
    schemas: &CanonicalSchemaSet,
    encoding: PackageTableEncoding,
    manifest: &mut PackageManifest,
) -> Result<()> {
    maybe_write_table(
        dir,
        parts.materials.as_ref(),
        &schemas.materials,
        CanonicalTable::Materials,
        encoding,
        &mut manifest.tables.materials,
    )?;
    maybe_write_table(
        dir,
        parts.geometry_surface_materials.as_ref(),
        &schemas.geometry_surface_materials,
        CanonicalTable::GeometrySurfaceMaterials,
        encoding,
        &mut manifest.tables.geometry_surface_materials,
    )?;
    maybe_write_table(
        dir,
        parts.geometry_point_materials.as_ref(),
        &schemas.geometry_point_materials,
        CanonicalTable::GeometryPointMaterials,
        encoding,
        &mut manifest.tables.geometry_point_materials,
    )?;
    maybe_write_table(
        dir,
        parts.geometry_linestring_materials.as_ref(),
        &schemas.geometry_linestring_materials,
        CanonicalTable::GeometryLinestringMaterials,
        encoding,
        &mut manifest.tables.geometry_linestring_materials,
    )?;
    maybe_write_table(
        dir,
        parts.template_geometry_materials.as_ref(),
        &schemas.template_geometry_materials,
        CanonicalTable::TemplateGeometryMaterials,
        encoding,
        &mut manifest.tables.template_geometry_materials,
    )?;
    maybe_write_table(
        dir,
        parts.textures.as_ref(),
        &schemas.textures,
        CanonicalTable::Textures,
        encoding,
        &mut manifest.tables.textures,
    )?;
    maybe_write_table(
        dir,
        parts.texture_vertices.as_ref(),
        &schemas.texture_vertices,
        CanonicalTable::TextureVertices,
        encoding,
        &mut manifest.tables.texture_vertices,
    )?;
    maybe_write_table(
        dir,
        parts.geometry_ring_textures.as_ref(),
        &schemas.geometry_ring_textures,
        CanonicalTable::GeometryRingTextures,
        encoding,
        &mut manifest.tables.geometry_ring_textures,
    )?;
    maybe_write_table(
        dir,
        parts.template_geometry_ring_textures.as_ref(),
        &schemas.template_geometry_ring_textures,
        CanonicalTable::TemplateGeometryRingTextures,
        encoding,
        &mut manifest.tables.template_geometry_ring_textures,
    )?;
    Ok(())
}

fn maybe_write_table(
    dir: &Path,
    batch: Option<&RecordBatch>,
    expected_schema: &std::sync::Arc<arrow::datatypes::Schema>,
    table: CanonicalTable,
    encoding: PackageTableEncoding,
    destination: &mut Option<std::path::PathBuf>,
) -> Result<()> {
    let Some(batch) = batch else {
        return Ok(());
    };
    validate_schema(expected_schema, batch.schema(), table)?;
    write_batch(dir, table, batch, encoding)?;
    *destination = Some(table.file_name_for(encoding).into());
    Ok(())
}

fn write_batch(
    dir: &Path,
    table: CanonicalTable,
    batch: &RecordBatch,
    encoding: PackageTableEncoding,
) -> Result<()> {
    let path = package_table_path_for_encoding(dir, table, encoding);
    let file = File::create(path)?;
    match encoding {
        PackageTableEncoding::ArrowIpcFile => {
            let mut writer = FileWriter::try_new(file, &batch.schema())?;
            writer.write(batch)?;
            writer.finish()?;
        }
        PackageTableEncoding::Parquet => {
            return Err(Error::Unsupported(
                "cityarrow only supports Arrow IPC package tables".to_string(),
            ));
        }
    }
    Ok(())
}

fn ensure_single_row(batch: &RecordBatch, table: CanonicalTable) -> Result<()> {
    if batch.num_rows() != 1 {
        return Err(Error::Unsupported(format!(
            "{} must contain exactly one row, found {}",
            table.file_name(),
            batch.num_rows()
        )));
    }
    Ok(())
}

fn ensure_max_one_row(batch: &RecordBatch, table: CanonicalTable) -> Result<()> {
    if batch.num_rows() > 1 {
        return Err(Error::Unsupported(format!(
            "{} must contain at most one row, found {}",
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
        .downcast_ref::<UInt64Array>()
        .ok_or_else(|| Error::Conversion(format!("failed to read {left_name}.{id_field}")))?;
    let right_ids = right_ids
        .as_any()
        .downcast_ref::<UInt64Array>()
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
