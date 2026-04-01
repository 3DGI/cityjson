use super::{
    CanonicalTable, expected_schema_set, infer_cityobject_projections, infer_material_projection,
    infer_semantic_projection, infer_tail_projection, infer_texture_projection,
    package_manifest_path, validate_schema,
};
use crate::error::{Error, Result};
use crate::schema::{
    CanonicalSchemaSet, CityModelArrowParts, PackageManifest, PackageTableEncoding, PackageTables,
    ProjectedFieldSpec, ProjectionLayout,
};
use arrow::array::{Array, UInt64Array};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

const ALL_TABLES: [CanonicalTable; 25] = [
    CanonicalTable::Metadata,
    CanonicalTable::Transform,
    CanonicalTable::Extensions,
    CanonicalTable::Vertices,
    CanonicalTable::CityObjects,
    CanonicalTable::CityObjectChildren,
    CanonicalTable::Geometries,
    CanonicalTable::GeometryBoundaries,
    CanonicalTable::GeometryInstances,
    CanonicalTable::TemplateVertices,
    CanonicalTable::TemplateGeometries,
    CanonicalTable::TemplateGeometryBoundaries,
    CanonicalTable::Semantics,
    CanonicalTable::SemanticChildren,
    CanonicalTable::GeometrySurfaceSemantics,
    CanonicalTable::GeometryPointSemantics,
    CanonicalTable::GeometryLinestringSemantics,
    CanonicalTable::TemplateGeometrySemantics,
    CanonicalTable::Materials,
    CanonicalTable::GeometrySurfaceMaterials,
    CanonicalTable::TemplateGeometryMaterials,
    CanonicalTable::Textures,
    CanonicalTable::TextureVertices,
    CanonicalTable::GeometryRingTextures,
    CanonicalTable::TemplateGeometryRingTextures,
];

#[doc(hidden)]
pub fn write_package_dir_with_writer<F>(
    dir: impl AsRef<Path>,
    parts: &CityModelArrowParts,
    encoding: PackageTableEncoding,
    mut write_table: F,
) -> Result<PackageManifest>
where
    F: FnMut(&Path, CanonicalTable, &RecordBatch, PackageTableEncoding) -> Result<()>,
{
    let dir = dir.as_ref();
    fs::create_dir_all(dir)?;

    let schemas = expected_schema_set(&parts.projection);
    let mut manifest = PackageManifest::new(
        parts.header.citymodel_id.clone(),
        parts.header.cityjson_version.clone(),
    );
    manifest.package_schema = parts.header.package_version;
    manifest.table_encoding = encoding;
    write_core_tables(
        dir,
        parts,
        &schemas,
        encoding,
        &mut manifest,
        &mut write_table,
    )?;
    write_geometry_tables(
        dir,
        parts,
        &schemas,
        encoding,
        &mut manifest,
        &mut write_table,
    )?;
    write_semantic_tables(
        dir,
        parts,
        &schemas,
        encoding,
        &mut manifest,
        &mut write_table,
    )?;
    write_appearance_tables(
        dir,
        parts,
        &schemas,
        encoding,
        &mut manifest,
        &mut write_table,
    )?;

    let file = File::create(package_manifest_path(dir))?;
    serde_json::to_writer_pretty(file, &manifest)?;

    Ok(manifest)
}

#[doc(hidden)]
pub fn read_package_dir_with_loader<F>(
    dir: impl AsRef<Path>,
    required_encoding: Option<PackageTableEncoding>,
    mut load_table: F,
) -> Result<CityModelArrowParts>
where
    F: FnMut(
        &Path,
        Option<&PathBuf>,
        PackageTableEncoding,
    ) -> Result<Option<(SchemaRef, RecordBatch)>>,
{
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

    let loaded = load_tables(dir, &manifest, &mut load_table)?;
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
        template_geometry_materials: appearance.template_geometry_materials,
        textures: appearance.textures,
        texture_vertices: appearance.texture_vertices,
        geometry_ring_textures: appearance.geometry_ring_textures,
        template_geometry_ring_textures: appearance.template_geometry_ring_textures,
    })
}

struct LoadedTable {
    schema: SchemaRef,
    batch: RecordBatch,
}

#[derive(Default)]
struct LoadedTables {
    tables: HashMap<CanonicalTable, LoadedTable>,
}

impl LoadedTables {
    fn insert(&mut self, table: CanonicalTable, schema: SchemaRef, batch: RecordBatch) {
        self.tables.insert(table, LoadedTable { schema, batch });
    }

    fn get(&self, table: CanonicalTable) -> Option<&LoadedTable> {
        self.tables.get(&table)
    }
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
    template_geometry_materials: Option<RecordBatch>,
    textures: Option<RecordBatch>,
    texture_vertices: Option<RecordBatch>,
    geometry_ring_textures: Option<RecordBatch>,
    template_geometry_ring_textures: Option<RecordBatch>,
}

fn write_core_tables<F>(
    dir: &Path,
    parts: &CityModelArrowParts,
    schemas: &CanonicalSchemaSet,
    encoding: PackageTableEncoding,
    manifest: &mut PackageManifest,
    write_table: &mut F,
) -> Result<()>
where
    F: FnMut(&Path, CanonicalTable, &RecordBatch, PackageTableEncoding) -> Result<()>,
{
    write_required_table(
        dir,
        &parts.metadata,
        schemas,
        CanonicalTable::Metadata,
        encoding,
        manifest,
        write_table,
    )?;
    ensure_exact_row_count(&parts.metadata, 1, CanonicalTable::Metadata)?;

    if let Some(transform) = &parts.transform {
        write_required_table(
            dir,
            transform,
            schemas,
            CanonicalTable::Transform,
            encoding,
            manifest,
            write_table,
        )?;
        ensure_max_row_count(transform, 1, CanonicalTable::Transform)?;
    }

    for (batch, table) in [
        (parts.extensions.as_ref(), CanonicalTable::Extensions),
        (Some(&parts.vertices), CanonicalTable::Vertices),
        (Some(&parts.cityobjects), CanonicalTable::CityObjects),
        (
            parts.cityobject_children.as_ref(),
            CanonicalTable::CityObjectChildren,
        ),
    ] {
        maybe_write_table(dir, batch, schemas, table, encoding, manifest, write_table)?;
    }

    Ok(())
}

fn write_geometry_tables<F>(
    dir: &Path,
    parts: &CityModelArrowParts,
    schemas: &CanonicalSchemaSet,
    encoding: PackageTableEncoding,
    manifest: &mut PackageManifest,
    write_table: &mut F,
) -> Result<()>
where
    F: FnMut(&Path, CanonicalTable, &RecordBatch, PackageTableEncoding) -> Result<()>,
{
    write_required_table(
        dir,
        &parts.geometries,
        schemas,
        CanonicalTable::Geometries,
        encoding,
        manifest,
        write_table,
    )?;
    write_required_table(
        dir,
        &parts.geometry_boundaries,
        schemas,
        CanonicalTable::GeometryBoundaries,
        encoding,
        manifest,
        write_table,
    )?;
    ensure_paired_geometry_tables(
        &parts.geometries,
        &parts.geometry_boundaries,
        "geometry_id",
        "geometries",
        "geometry_boundaries",
    )?;

    for (batch, table) in [
        (
            parts.geometry_instances.as_ref(),
            CanonicalTable::GeometryInstances,
        ),
        (
            parts.template_vertices.as_ref(),
            CanonicalTable::TemplateVertices,
        ),
    ] {
        maybe_write_table(dir, batch, schemas, table, encoding, manifest, write_table)?;
    }

    match (
        &parts.template_geometries,
        &parts.template_geometry_boundaries,
    ) {
        (Some(template_geometries), Some(template_geometry_boundaries)) => {
            write_required_table(
                dir,
                template_geometries,
                schemas,
                CanonicalTable::TemplateGeometries,
                encoding,
                manifest,
                write_table,
            )?;
            write_required_table(
                dir,
                template_geometry_boundaries,
                schemas,
                CanonicalTable::TemplateGeometryBoundaries,
                encoding,
                manifest,
                write_table,
            )?;
            ensure_paired_geometry_tables(
                template_geometries,
                template_geometry_boundaries,
                "template_geometry_id",
                "template_geometries",
                "template_geometry_boundaries",
            )?;
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

fn write_semantic_tables<F>(
    dir: &Path,
    parts: &CityModelArrowParts,
    schemas: &CanonicalSchemaSet,
    encoding: PackageTableEncoding,
    manifest: &mut PackageManifest,
    write_table: &mut F,
) -> Result<()>
where
    F: FnMut(&Path, CanonicalTable, &RecordBatch, PackageTableEncoding) -> Result<()>,
{
    for (batch, table) in [
        (parts.semantics.as_ref(), CanonicalTable::Semantics),
        (
            parts.semantic_children.as_ref(),
            CanonicalTable::SemanticChildren,
        ),
        (
            parts.geometry_surface_semantics.as_ref(),
            CanonicalTable::GeometrySurfaceSemantics,
        ),
        (
            parts.geometry_point_semantics.as_ref(),
            CanonicalTable::GeometryPointSemantics,
        ),
        (
            parts.geometry_linestring_semantics.as_ref(),
            CanonicalTable::GeometryLinestringSemantics,
        ),
        (
            parts.template_geometry_semantics.as_ref(),
            CanonicalTable::TemplateGeometrySemantics,
        ),
    ] {
        maybe_write_table(dir, batch, schemas, table, encoding, manifest, write_table)?;
    }

    Ok(())
}

fn write_appearance_tables<F>(
    dir: &Path,
    parts: &CityModelArrowParts,
    schemas: &CanonicalSchemaSet,
    encoding: PackageTableEncoding,
    manifest: &mut PackageManifest,
    write_table: &mut F,
) -> Result<()>
where
    F: FnMut(&Path, CanonicalTable, &RecordBatch, PackageTableEncoding) -> Result<()>,
{
    for (batch, table) in [
        (parts.materials.as_ref(), CanonicalTable::Materials),
        (
            parts.geometry_surface_materials.as_ref(),
            CanonicalTable::GeometrySurfaceMaterials,
        ),
        (
            parts.template_geometry_materials.as_ref(),
            CanonicalTable::TemplateGeometryMaterials,
        ),
        (parts.textures.as_ref(), CanonicalTable::Textures),
        (
            parts.texture_vertices.as_ref(),
            CanonicalTable::TextureVertices,
        ),
        (
            parts.geometry_ring_textures.as_ref(),
            CanonicalTable::GeometryRingTextures,
        ),
        (
            parts.template_geometry_ring_textures.as_ref(),
            CanonicalTable::TemplateGeometryRingTextures,
        ),
    ] {
        maybe_write_table(dir, batch, schemas, table, encoding, manifest, write_table)?;
    }

    Ok(())
}

fn write_required_table<F>(
    dir: &Path,
    batch: &RecordBatch,
    schemas: &CanonicalSchemaSet,
    table: CanonicalTable,
    encoding: PackageTableEncoding,
    manifest: &mut PackageManifest,
    write_table: &mut F,
) -> Result<()>
where
    F: FnMut(&Path, CanonicalTable, &RecordBatch, PackageTableEncoding) -> Result<()>,
{
    validate_schema(schema_for_table(schemas, table), batch.schema(), table)?;
    write_table(dir, table, batch, encoding)?;
    record_manifest_table(manifest, table, encoding);
    Ok(())
}

fn maybe_write_table<F>(
    dir: &Path,
    batch: Option<&RecordBatch>,
    schemas: &CanonicalSchemaSet,
    table: CanonicalTable,
    encoding: PackageTableEncoding,
    manifest: &mut PackageManifest,
    write_table: &mut F,
) -> Result<()>
where
    F: FnMut(&Path, CanonicalTable, &RecordBatch, PackageTableEncoding) -> Result<()>,
{
    let Some(batch) = batch else {
        return Ok(());
    };
    write_required_table(dir, batch, schemas, table, encoding, manifest, write_table)
}

fn record_manifest_table(
    manifest: &mut PackageManifest,
    table: CanonicalTable,
    encoding: PackageTableEncoding,
) {
    *manifest_table_destination(&mut manifest.tables, table) =
        Some(table.file_name_for(encoding).into());
}

fn read_manifest(dir: &Path) -> Result<PackageManifest> {
    let file = File::open(package_manifest_path(dir))?;
    Ok(serde_json::from_reader(file)?)
}

fn load_tables<F>(
    dir: &Path,
    manifest: &PackageManifest,
    load_table: &mut F,
) -> Result<LoadedTables>
where
    F: FnMut(
        &Path,
        Option<&PathBuf>,
        PackageTableEncoding,
    ) -> Result<Option<(SchemaRef, RecordBatch)>>,
{
    let mut loaded = LoadedTables::default();
    for table in ALL_TABLES {
        if let Some((schema, batch)) = load_table(
            dir,
            manifest_table_path(&manifest.tables, table),
            manifest.table_encoding,
        )? {
            loaded.insert(table, schema, batch);
        }
    }
    Ok(loaded)
}

fn read_core_tables(loaded: &LoadedTables, schemas: &CanonicalSchemaSet) -> Result<CoreTables> {
    let metadata = required_table(
        loaded.get(CanonicalTable::Metadata),
        schemas,
        CanonicalTable::Metadata,
    )?;
    ensure_exact_row_count(&metadata, 1, CanonicalTable::Metadata)?;

    let transform = optional_table(
        loaded.get(CanonicalTable::Transform),
        schemas,
        CanonicalTable::Transform,
    )?;
    if let Some(table) = &transform {
        ensure_max_row_count(table, 1, CanonicalTable::Transform)?;
    }

    Ok(CoreTables {
        metadata,
        transform,
        extensions: optional_table(
            loaded.get(CanonicalTable::Extensions),
            schemas,
            CanonicalTable::Extensions,
        )?,
        vertices: required_table(
            loaded.get(CanonicalTable::Vertices),
            schemas,
            CanonicalTable::Vertices,
        )?,
        cityobjects: required_table(
            loaded.get(CanonicalTable::CityObjects),
            schemas,
            CanonicalTable::CityObjects,
        )?,
        cityobject_children: optional_table(
            loaded.get(CanonicalTable::CityObjectChildren),
            schemas,
            CanonicalTable::CityObjectChildren,
        )?,
    })
}

fn read_geometry_tables(
    loaded: &LoadedTables,
    schemas: &CanonicalSchemaSet,
) -> Result<GeometryTables> {
    let geometries = required_table(
        loaded.get(CanonicalTable::Geometries),
        schemas,
        CanonicalTable::Geometries,
    )?;
    let geometry_boundaries = required_table(
        loaded.get(CanonicalTable::GeometryBoundaries),
        schemas,
        CanonicalTable::GeometryBoundaries,
    )?;
    ensure_paired_geometry_tables(
        &geometries,
        &geometry_boundaries,
        "geometry_id",
        "geometries",
        "geometry_boundaries",
    )?;

    let template_geometries = optional_table(
        loaded.get(CanonicalTable::TemplateGeometries),
        schemas,
        CanonicalTable::TemplateGeometries,
    )?;
    let template_geometry_boundaries = optional_table(
        loaded.get(CanonicalTable::TemplateGeometryBoundaries),
        schemas,
        CanonicalTable::TemplateGeometryBoundaries,
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
            loaded.get(CanonicalTable::GeometryInstances),
            schemas,
            CanonicalTable::GeometryInstances,
        )?,
        template_vertices: optional_table(
            loaded.get(CanonicalTable::TemplateVertices),
            schemas,
            CanonicalTable::TemplateVertices,
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
            loaded.get(CanonicalTable::Semantics),
            schemas,
            CanonicalTable::Semantics,
        )?,
        semantic_children: optional_table(
            loaded.get(CanonicalTable::SemanticChildren),
            schemas,
            CanonicalTable::SemanticChildren,
        )?,
        geometry_surface_semantics: optional_table(
            loaded.get(CanonicalTable::GeometrySurfaceSemantics),
            schemas,
            CanonicalTable::GeometrySurfaceSemantics,
        )?,
        geometry_point_semantics: optional_table(
            loaded.get(CanonicalTable::GeometryPointSemantics),
            schemas,
            CanonicalTable::GeometryPointSemantics,
        )?,
        geometry_linestring_semantics: optional_table(
            loaded.get(CanonicalTable::GeometryLinestringSemantics),
            schemas,
            CanonicalTable::GeometryLinestringSemantics,
        )?,
        template_geometry_semantics: optional_table(
            loaded.get(CanonicalTable::TemplateGeometrySemantics),
            schemas,
            CanonicalTable::TemplateGeometrySemantics,
        )?,
    })
}

fn read_appearance_tables(
    loaded: &LoadedTables,
    schemas: &CanonicalSchemaSet,
) -> Result<AppearanceTables> {
    Ok(AppearanceTables {
        materials: optional_table(
            loaded.get(CanonicalTable::Materials),
            schemas,
            CanonicalTable::Materials,
        )?,
        geometry_surface_materials: optional_table(
            loaded.get(CanonicalTable::GeometrySurfaceMaterials),
            schemas,
            CanonicalTable::GeometrySurfaceMaterials,
        )?,
        template_geometry_materials: optional_table(
            loaded.get(CanonicalTable::TemplateGeometryMaterials),
            schemas,
            CanonicalTable::TemplateGeometryMaterials,
        )?,
        textures: optional_table(
            loaded.get(CanonicalTable::Textures),
            schemas,
            CanonicalTable::Textures,
        )?,
        texture_vertices: optional_table(
            loaded.get(CanonicalTable::TextureVertices),
            schemas,
            CanonicalTable::TextureVertices,
        )?,
        geometry_ring_textures: optional_table(
            loaded.get(CanonicalTable::GeometryRingTextures),
            schemas,
            CanonicalTable::GeometryRingTextures,
        )?,
        template_geometry_ring_textures: optional_table(
            loaded.get(CanonicalTable::TemplateGeometryRingTextures),
            schemas,
            CanonicalTable::TemplateGeometryRingTextures,
        )?,
    })
}

fn infer_projection_layout(loaded: &LoadedTables) -> Result<ProjectionLayout> {
    let mut layout = ProjectionLayout::default();

    if let Some(table) = loaded.get(CanonicalTable::Metadata) {
        layout.metadata_extra = infer_tail_projection(table.schema.as_ref(), 7)?;
    }

    if let Some(table) = loaded.get(CanonicalTable::CityObjects) {
        let (attributes, extra) = infer_cityobject_projections(table.schema.as_ref())?;
        layout.cityobject_attributes = attributes;
        layout.cityobject_extra = extra;
    }

    layout.geometry_extra = infer_consistent_geometry_projection(loaded)?;

    if let Some(table) = loaded.get(CanonicalTable::Semantics) {
        layout.semantic_attributes = infer_semantic_projection(table.schema.as_ref())?;
    }

    if let Some(table) = loaded.get(CanonicalTable::Materials) {
        layout.material_payload = infer_material_projection(table.schema.as_ref())?;
    }

    if let Some(table) = loaded.get(CanonicalTable::Textures) {
        layout.texture_payload = infer_texture_projection(table.schema.as_ref())?;
    }

    Ok(layout)
}

fn infer_consistent_geometry_projection(loaded: &LoadedTables) -> Result<Vec<ProjectedFieldSpec>> {
    let mut geometry_extra: Option<Vec<_>> = None;

    for (table, start_index) in [
        (CanonicalTable::Geometries, 6usize),
        (CanonicalTable::GeometryInstances, 8usize),
        (CanonicalTable::TemplateGeometries, 4usize),
    ] {
        let Some(table) = loaded.get(table) else {
            continue;
        };
        let candidate = infer_tail_projection(table.schema.as_ref(), start_index)?;
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
    schemas: &CanonicalSchemaSet,
    kind: CanonicalTable,
) -> Result<RecordBatch> {
    let loaded = table.ok_or_else(|| Error::MissingField(kind.file_name().to_string()))?;
    validate_schema(schema_for_table(schemas, kind), &loaded.schema, kind)?;
    Ok(loaded.batch.clone())
}

fn optional_table(
    table: Option<&LoadedTable>,
    schemas: &CanonicalSchemaSet,
    kind: CanonicalTable,
) -> Result<Option<RecordBatch>> {
    match table {
        Some(loaded) => {
            validate_schema(schema_for_table(schemas, kind), &loaded.schema, kind)?;
            Ok(Some(loaded.batch.clone()))
        }
        None => Ok(None),
    }
}

fn schema_for_table(schemas: &CanonicalSchemaSet, table: CanonicalTable) -> &SchemaRef {
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

fn manifest_table_path(tables: &PackageTables, table: CanonicalTable) -> Option<&PathBuf> {
    match table {
        CanonicalTable::Metadata => tables.metadata.as_ref(),
        CanonicalTable::Transform => tables.transform.as_ref(),
        CanonicalTable::Extensions => tables.extensions.as_ref(),
        CanonicalTable::Vertices => tables.vertices.as_ref(),
        CanonicalTable::CityObjects => tables.cityobjects.as_ref(),
        CanonicalTable::CityObjectChildren => tables.cityobject_children.as_ref(),
        CanonicalTable::Geometries => tables.geometries.as_ref(),
        CanonicalTable::GeometryBoundaries => tables.geometry_boundaries.as_ref(),
        CanonicalTable::GeometryInstances => tables.geometry_instances.as_ref(),
        CanonicalTable::TemplateVertices => tables.template_vertices.as_ref(),
        CanonicalTable::TemplateGeometries => tables.template_geometries.as_ref(),
        CanonicalTable::TemplateGeometryBoundaries => tables.template_geometry_boundaries.as_ref(),
        CanonicalTable::Semantics => tables.semantics.as_ref(),
        CanonicalTable::SemanticChildren => tables.semantic_children.as_ref(),
        CanonicalTable::GeometrySurfaceSemantics => tables.geometry_surface_semantics.as_ref(),
        CanonicalTable::GeometryPointSemantics => tables.geometry_point_semantics.as_ref(),
        CanonicalTable::GeometryLinestringSemantics => tables.geometry_linestring_semantics.as_ref(),
        CanonicalTable::TemplateGeometrySemantics => tables.template_geometry_semantics.as_ref(),
        CanonicalTable::Materials => tables.materials.as_ref(),
        CanonicalTable::GeometrySurfaceMaterials => tables.geometry_surface_materials.as_ref(),
        CanonicalTable::TemplateGeometryMaterials => tables.template_geometry_materials.as_ref(),
        CanonicalTable::Textures => tables.textures.as_ref(),
        CanonicalTable::TextureVertices => tables.texture_vertices.as_ref(),
        CanonicalTable::GeometryRingTextures => tables.geometry_ring_textures.as_ref(),
        CanonicalTable::TemplateGeometryRingTextures => tables.template_geometry_ring_textures.as_ref(),
    }
}

fn manifest_table_destination(
    tables: &mut PackageTables,
    table: CanonicalTable,
) -> &mut Option<PathBuf> {
    match table {
        CanonicalTable::Metadata => &mut tables.metadata,
        CanonicalTable::Transform => &mut tables.transform,
        CanonicalTable::Extensions => &mut tables.extensions,
        CanonicalTable::Vertices => &mut tables.vertices,
        CanonicalTable::CityObjects => &mut tables.cityobjects,
        CanonicalTable::CityObjectChildren => &mut tables.cityobject_children,
        CanonicalTable::Geometries => &mut tables.geometries,
        CanonicalTable::GeometryBoundaries => &mut tables.geometry_boundaries,
        CanonicalTable::GeometryInstances => &mut tables.geometry_instances,
        CanonicalTable::TemplateVertices => &mut tables.template_vertices,
        CanonicalTable::TemplateGeometries => &mut tables.template_geometries,
        CanonicalTable::TemplateGeometryBoundaries => &mut tables.template_geometry_boundaries,
        CanonicalTable::Semantics => &mut tables.semantics,
        CanonicalTable::SemanticChildren => &mut tables.semantic_children,
        CanonicalTable::GeometrySurfaceSemantics => &mut tables.geometry_surface_semantics,
        CanonicalTable::GeometryPointSemantics => &mut tables.geometry_point_semantics,
        CanonicalTable::GeometryLinestringSemantics => &mut tables.geometry_linestring_semantics,
        CanonicalTable::TemplateGeometrySemantics => &mut tables.template_geometry_semantics,
        CanonicalTable::Materials => &mut tables.materials,
        CanonicalTable::GeometrySurfaceMaterials => &mut tables.geometry_surface_materials,
        CanonicalTable::TemplateGeometryMaterials => &mut tables.template_geometry_materials,
        CanonicalTable::Textures => &mut tables.textures,
        CanonicalTable::TextureVertices => &mut tables.texture_vertices,
        CanonicalTable::GeometryRingTextures => &mut tables.geometry_ring_textures,
        CanonicalTable::TemplateGeometryRingTextures => &mut tables.template_geometry_ring_textures,
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
        .downcast_ref::<UInt64Array>()
        .ok_or_else(|| Error::Conversion(format!("failed to read {left_name}.{id_field}")))?;
    let right_ids = right_ids
        .as_any()
        .downcast_ref::<UInt64Array>()
        .ok_or_else(|| Error::Conversion(format!("failed to read {right_name}.{id_field}")))?;

    for index in 0..left_ids.len() {
        if left_ids.value(index) != right_ids.value(index) {
            return Err(Error::Unsupported(format!(
                "{left_name} and {right_name} must align on {id_field}"
            )));
        }
    }

    Ok(())
}
