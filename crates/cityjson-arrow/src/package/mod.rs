use crate::convert::{decode_parts, encode_parts};
use crate::error::{Error, Result};
use crate::schema::{
    CanonicalSchemaSet, CityModelArrowParts, PackageManifest, PackageTableRef, canonical_schema_set,
};
use arrow::datatypes::SchemaRef;
use arrow::ipc::reader::FileReader;
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;
use cityjson::v2_0::OwnedCityModel;
use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;

const PACKAGE_MAGIC: &[u8] = b"CITYARROW_PKG_V2\0";
const PACKAGE_FOOTER_MAGIC: &[u8] = b"CITYARROW_PKG_IDX\0";
const FOOTER_LEN: usize = 8 + 8 + PACKAGE_FOOTER_MAGIC.len();

#[derive(Debug, Default, Clone, Copy)]
pub struct PackageWriter;

impl PackageWriter {
    /// Writes a single-file `CityArrow` package.
    ///
    /// # Errors
    ///
    /// Returns an error when model conversion or package serialization fails.
    pub fn write_file(
        &self,
        path: impl AsRef<Path>,
        model: &OwnedCityModel,
    ) -> Result<PackageManifest> {
        let parts = encode_parts(model)?;
        write_package_file(path, &parts)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PackageReader;

impl PackageReader {
    /// Reads a single-file `CityArrow` package into an in-memory model.
    ///
    /// # Errors
    ///
    /// Returns an error when the package cannot be read or decoded.
    pub fn read_file(&self, path: impl AsRef<Path>) -> Result<OwnedCityModel> {
        let parts = read_package_file(path)?;
        decode_parts(&parts)
    }

    /// Reads only the package manifest from a single-file `CityArrow` package.
    ///
    /// # Errors
    ///
    /// Returns an error when the package footer or manifest cannot be read.
    pub fn read_manifest(&self, path: impl AsRef<Path>) -> Result<PackageManifest> {
        read_package_manifest(path)
    }
}

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

pub(crate) fn write_package_file(
    path: impl AsRef<Path>,
    parts: &CityModelArrowParts,
) -> Result<PackageManifest> {
    let path = path.as_ref();
    let mut payloads = Vec::new();
    let mut offset = u64::try_from(PACKAGE_MAGIC.len())
        .map_err(|_| Error::Conversion("package magic length overflow".to_string()))?;

    for (table, batch) in collect_tables(parts) {
        let bytes = serialize_file_batch(&batch)?;
        let length = u64::try_from(bytes.len()).map_err(|_| {
            Error::Conversion(format!("{} payload length overflow", table.as_str()))
        })?;
        payloads.push((
            PackageTableRef {
                name: table.as_str().to_string(),
                offset,
                length,
                rows: batch.num_rows(),
            },
            bytes,
        ));
        offset += length;
    }

    let mut manifest = PackageManifest::new(
        parts.header.citymodel_id.clone(),
        parts.header.cityjson_version.clone(),
        parts.projection.clone(),
    );
    manifest.tables = payloads.iter().map(|(entry, _)| entry.clone()).collect();
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    let manifest_offset = offset;
    let manifest_length = u64::try_from(manifest_bytes.len())
        .map_err(|_| Error::Conversion("manifest length overflow".to_string()))?;

    let mut file = File::create(path)?;
    file.write_all(PACKAGE_MAGIC)?;
    for (_, payload) in &payloads {
        file.write_all(payload)?;
    }
    file.write_all(&manifest_bytes)?;
    file.write_all(&manifest_offset.to_le_bytes())?;
    file.write_all(&manifest_length.to_le_bytes())?;
    file.write_all(PACKAGE_FOOTER_MAGIC)?;
    file.flush()?;

    Ok(manifest)
}

pub(crate) fn read_package_file(path: impl AsRef<Path>) -> Result<CityModelArrowParts> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let manifest = manifest_from_bytes(&mmap)?;
    let schemas = canonical_schema_set(&manifest.projection);
    let mut tables = HashMap::new();
    for table in &manifest.tables {
        let kind = CanonicalTable::parse(&table.name)?;
        let batch = deserialize_file_batch(
            &mmap,
            table.offset,
            table.length,
            schema_for_table(&schemas, kind),
            kind,
        )?;
        tables.insert(kind, batch);
    }
    build_parts(&manifest, tables)
}

/// Reads only the manifest from a single-file package.
///
/// # Errors
///
/// Returns an error when the package footer or manifest cannot be read.
pub fn read_package_manifest(path: impl AsRef<Path>) -> Result<PackageManifest> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    manifest_from_bytes(&bytes)
}

fn manifest_from_bytes(bytes: &[u8]) -> Result<PackageManifest> {
    if bytes.len() < PACKAGE_MAGIC.len() + FOOTER_LEN {
        return Err(Error::Unsupported(
            "package is too small to be a cityarrow package".to_string(),
        ));
    }
    if &bytes[..PACKAGE_MAGIC.len()] != PACKAGE_MAGIC {
        return Err(Error::Unsupported(
            "package header magic is invalid".to_string(),
        ));
    }

    let footer_start = bytes.len() - FOOTER_LEN;
    if &bytes[footer_start + 16..] != PACKAGE_FOOTER_MAGIC {
        return Err(Error::Unsupported(
            "package footer magic is invalid".to_string(),
        ));
    }

    let mut offset_bytes = [0_u8; 8];
    offset_bytes.copy_from_slice(&bytes[footer_start..footer_start + 8]);
    let manifest_offset = usize::try_from(u64::from_le_bytes(offset_bytes))
        .map_err(|_| Error::Conversion("manifest offset does not fit in memory".to_string()))?;

    let mut length_bytes = [0_u8; 8];
    length_bytes.copy_from_slice(&bytes[footer_start + 8..footer_start + 16]);
    let manifest_length = usize::try_from(u64::from_le_bytes(length_bytes))
        .map_err(|_| Error::Conversion("manifest length does not fit in memory".to_string()))?;

    let manifest_end = manifest_offset
        .checked_add(manifest_length)
        .ok_or_else(|| Error::Conversion("manifest range overflow".to_string()))?;
    if manifest_end > footer_start {
        return Err(Error::Unsupported(
            "package manifest range is invalid".to_string(),
        ));
    }

    Ok(serde_json::from_slice(
        &bytes[manifest_offset..manifest_end],
    )?)
}

fn serialize_file_batch(batch: &RecordBatch) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = FileWriter::try_new(&mut cursor, &batch.schema())?;
        writer.write(batch)?;
        writer.finish()?;
    }
    Ok(cursor.into_inner())
}

fn deserialize_file_batch(
    bytes: &[u8],
    offset: u64,
    length: u64,
    expected_schema: &SchemaRef,
    table: CanonicalTable,
) -> Result<RecordBatch> {
    let start = usize::try_from(offset).map_err(|_| {
        Error::Conversion(format!("{} offset does not fit in memory", table.as_str()))
    })?;
    let payload_len = usize::try_from(length).map_err(|_| {
        Error::Conversion(format!("{} length does not fit in memory", table.as_str()))
    })?;
    let end = start
        .checked_add(payload_len)
        .ok_or_else(|| Error::Conversion(format!("{} byte range overflow", table.as_str())))?;
    let slice = bytes.get(start..end).ok_or_else(|| {
        Error::Unsupported(format!("{} payload range is out of bounds", table.as_str()))
    })?;
    let reader = FileReader::try_new(Cursor::new(slice), None)?;
    let schema = reader.schema();
    validate_schema(expected_schema, &schema, table)?;
    let batches = reader.collect::<std::result::Result<Vec<_>, _>>()?;
    concat_record_batches(expected_schema, &batches)
}

pub(crate) fn collect_tables(parts: &CityModelArrowParts) -> Vec<(CanonicalTable, RecordBatch)> {
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

pub(crate) fn build_parts(
    manifest: &PackageManifest,
    mut tables: HashMap<CanonicalTable, RecordBatch>,
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

fn required_table(
    tables: &mut HashMap<CanonicalTable, RecordBatch>,
    table: CanonicalTable,
) -> Result<RecordBatch> {
    tables.remove(&table).ok_or_else(|| {
        Error::Unsupported(format!(
            "package manifest is missing required '{}' table",
            table.as_str()
        ))
    })
}

pub(crate) fn schema_for_table(schemas: &CanonicalSchemaSet, table: CanonicalTable) -> &SchemaRef {
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
