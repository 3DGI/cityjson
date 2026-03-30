use super::{
    CanonicalTable, expected_schema_set, package_manifest_path, package_table_path, validate_schema,
};
use crate::error::{Error, Result};
use crate::schema::{CityModelArrowParts, PackageManifest};
use arrow::array::{Array, UInt64Array};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use std::fs::{self, File};
use std::path::Path;

pub fn write_package(
    dir: impl AsRef<Path>,
    parts: &CityModelArrowParts,
) -> Result<PackageManifest> {
    write_package_dir(dir, parts)
}

pub fn write_package_dir(
    dir: impl AsRef<Path>,
    parts: &CityModelArrowParts,
) -> Result<PackageManifest> {
    let dir = dir.as_ref();
    fs::create_dir_all(dir)?;

    let schemas = expected_schema_set(&parts.projection);
    let mut manifest = PackageManifest::new(
        parts.header.citymodel_id.clone(),
        parts.header.cityjson_version.clone(),
    );
    manifest.package_schema = parts.header.package_version;

    validate_schema(
        &schemas.metadata,
        parts.metadata.schema(),
        CanonicalTable::Metadata,
    )?;
    ensure_single_row(&parts.metadata, CanonicalTable::Metadata)?;
    write_batch(dir, CanonicalTable::Metadata, &parts.metadata)?;
    manifest.tables.metadata = Some(CanonicalTable::Metadata.file_name().into());

    if let Some(transform) = &parts.transform {
        validate_schema(
            &schemas.transform,
            transform.schema(),
            CanonicalTable::Transform,
        )?;
        ensure_max_one_row(transform, CanonicalTable::Transform)?;
        write_batch(dir, CanonicalTable::Transform, transform)?;
        manifest.tables.transform = Some(CanonicalTable::Transform.file_name().into());
    }

    if let Some(extensions) = &parts.extensions {
        validate_schema(
            &schemas.extensions,
            extensions.schema(),
            CanonicalTable::Extensions,
        )?;
        write_batch(dir, CanonicalTable::Extensions, extensions)?;
        manifest.tables.extensions = Some(CanonicalTable::Extensions.file_name().into());
    }

    validate_schema(
        &schemas.vertices,
        parts.vertices.schema(),
        CanonicalTable::Vertices,
    )?;
    write_batch(dir, CanonicalTable::Vertices, &parts.vertices)?;
    manifest.tables.vertices = Some(CanonicalTable::Vertices.file_name().into());

    validate_schema(
        &schemas.cityobjects,
        parts.cityobjects.schema(),
        CanonicalTable::CityObjects,
    )?;
    write_batch(dir, CanonicalTable::CityObjects, &parts.cityobjects)?;
    manifest.tables.cityobjects = Some(CanonicalTable::CityObjects.file_name().into());

    if let Some(children) = &parts.cityobject_children {
        validate_schema(
            &schemas.cityobject_children,
            children.schema(),
            CanonicalTable::CityObjectChildren,
        )?;
        write_batch(dir, CanonicalTable::CityObjectChildren, children)?;
        manifest.tables.cityobject_children =
            Some(CanonicalTable::CityObjectChildren.file_name().into());
    }

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
    write_batch(dir, CanonicalTable::Geometries, &parts.geometries)?;
    write_batch(
        dir,
        CanonicalTable::GeometryBoundaries,
        &parts.geometry_boundaries,
    )?;
    manifest.tables.geometries = Some(CanonicalTable::Geometries.file_name().into());
    manifest.tables.geometry_boundaries =
        Some(CanonicalTable::GeometryBoundaries.file_name().into());

    if let Some(geometry_instances) = &parts.geometry_instances {
        validate_schema(
            &schemas.geometry_instances,
            geometry_instances.schema(),
            CanonicalTable::GeometryInstances,
        )?;
        write_batch(dir, CanonicalTable::GeometryInstances, geometry_instances)?;
        manifest.tables.geometry_instances =
            Some(CanonicalTable::GeometryInstances.file_name().into());
    }

    if let Some(template_vertices) = &parts.template_vertices {
        validate_schema(
            &schemas.template_vertices,
            template_vertices.schema(),
            CanonicalTable::TemplateVertices,
        )?;
        write_batch(dir, CanonicalTable::TemplateVertices, template_vertices)?;
        manifest.tables.template_vertices =
            Some(CanonicalTable::TemplateVertices.file_name().into());
    }

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
            write_batch(dir, CanonicalTable::TemplateGeometries, template_geometries)?;
            write_batch(
                dir,
                CanonicalTable::TemplateGeometryBoundaries,
                template_geometry_boundaries,
            )?;
            manifest.tables.template_geometries =
                Some(CanonicalTable::TemplateGeometries.file_name().into());
            manifest.tables.template_geometry_boundaries = Some(
                CanonicalTable::TemplateGeometryBoundaries
                    .file_name()
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

    if let Some(semantics) = &parts.semantics {
        validate_schema(
            &schemas.semantics,
            semantics.schema(),
            CanonicalTable::Semantics,
        )?;
        write_batch(dir, CanonicalTable::Semantics, semantics)?;
        manifest.tables.semantics = Some(CanonicalTable::Semantics.file_name().into());
    }

    if let Some(semantic_children) = &parts.semantic_children {
        validate_schema(
            &schemas.semantic_children,
            semantic_children.schema(),
            CanonicalTable::SemanticChildren,
        )?;
        write_batch(dir, CanonicalTable::SemanticChildren, semantic_children)?;
        manifest.tables.semantic_children =
            Some(CanonicalTable::SemanticChildren.file_name().into());
    }

    if let Some(geometry_surface_semantics) = &parts.geometry_surface_semantics {
        validate_schema(
            &schemas.geometry_surface_semantics,
            geometry_surface_semantics.schema(),
            CanonicalTable::GeometrySurfaceSemantics,
        )?;
        write_batch(
            dir,
            CanonicalTable::GeometrySurfaceSemantics,
            geometry_surface_semantics,
        )?;
        manifest.tables.geometry_surface_semantics =
            Some(CanonicalTable::GeometrySurfaceSemantics.file_name().into());
    }

    if let Some(materials) = &parts.materials {
        validate_schema(
            &schemas.materials,
            materials.schema(),
            CanonicalTable::Materials,
        )?;
        write_batch(dir, CanonicalTable::Materials, materials)?;
        manifest.tables.materials = Some(CanonicalTable::Materials.file_name().into());
    }

    if let Some(geometry_surface_materials) = &parts.geometry_surface_materials {
        validate_schema(
            &schemas.geometry_surface_materials,
            geometry_surface_materials.schema(),
            CanonicalTable::GeometrySurfaceMaterials,
        )?;
        write_batch(
            dir,
            CanonicalTable::GeometrySurfaceMaterials,
            geometry_surface_materials,
        )?;
        manifest.tables.geometry_surface_materials =
            Some(CanonicalTable::GeometrySurfaceMaterials.file_name().into());
    }

    if let Some(textures) = &parts.textures {
        validate_schema(
            &schemas.textures,
            textures.schema(),
            CanonicalTable::Textures,
        )?;
        write_batch(dir, CanonicalTable::Textures, textures)?;
        manifest.tables.textures = Some(CanonicalTable::Textures.file_name().into());
    }

    if let Some(texture_vertices) = &parts.texture_vertices {
        validate_schema(
            &schemas.texture_vertices,
            texture_vertices.schema(),
            CanonicalTable::TextureVertices,
        )?;
        write_batch(dir, CanonicalTable::TextureVertices, texture_vertices)?;
        manifest.tables.texture_vertices = Some(CanonicalTable::TextureVertices.file_name().into());
    }

    if let Some(geometry_ring_textures) = &parts.geometry_ring_textures {
        validate_schema(
            &schemas.geometry_ring_textures,
            geometry_ring_textures.schema(),
            CanonicalTable::GeometryRingTextures,
        )?;
        write_batch(
            dir,
            CanonicalTable::GeometryRingTextures,
            geometry_ring_textures,
        )?;
        manifest.tables.geometry_ring_textures =
            Some(CanonicalTable::GeometryRingTextures.file_name().into());
    }

    let manifest_path = package_manifest_path(dir);
    let file = File::create(&manifest_path)?;
    serde_json::to_writer_pretty(file, &manifest)?;

    Ok(manifest)
}

fn write_batch(dir: &Path, table: CanonicalTable, batch: &RecordBatch) -> Result<()> {
    let path = package_table_path(dir, table);
    let file = File::create(path)?;
    let mut writer = ArrowWriter::try_new(file, batch.schema(), None)?;
    writer.write(batch)?;
    writer.close()?;
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
