use arrow::datatypes::SchemaRef;
use arrow::ipc::reader::FileReader;
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;
use cityarrow::error::{Error, Result};
use cityarrow::internal::{
    CanonicalTable, build_parts, collect_tables, concat_record_batches, decode_parts, encode_parts,
    schema_for_table, validate_schema,
};
use cityarrow::schema::{PackageManifest, PackageTableRef, canonical_schema_set};
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

fn write_package_file(
    path: impl AsRef<Path>,
    parts: &cityarrow::schema::CityModelArrowParts,
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

fn read_package_file(path: impl AsRef<Path>) -> Result<cityarrow::schema::CityModelArrowParts> {
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
