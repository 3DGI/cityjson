use arrow_array::RecordBatch;
use arrow_ipc::reader::FileReader;
use arrow_ipc::writer::FileWriter;
use arrow_schema::SchemaRef;
use cityjson::v2_0::OwnedCityModel;
use cityjson_arrow::error::{Error, Result};
use cityjson_arrow::internal::{
    CanonicalTable, CanonicalTableSink, IncrementalDecoder, build_parts_from_tables,
    emit_part_tables, emit_tables, schema_for_table, single_or_concat_batches, validate_schema,
};
use cityjson_arrow::schema::{
    CityArrowHeader, CityModelArrowParts, PackageManifest, PackageTableRef, ProjectionLayout,
    canonical_schema_set,
};
use memmap2::Mmap;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

const PACKAGE_MAGIC: &[u8] = b"CITYJSON_ARROW_PKG_V3\0";
const PACKAGE_FOOTER_MAGIC: &[u8] = b"CITYJSON_ARROW_PKG_V3IDX\0";
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
        write_package_model_file(path, model)
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
        read_package_model_file(path)
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

struct PackageSink {
    file: File,
    manifest: Option<PackageManifest>,
}

impl PackageSink {
    fn new(file: File) -> Self {
        Self {
            file,
            manifest: None,
        }
    }

    fn finish(mut self) -> Result<PackageManifest> {
        let manifest = self
            .manifest
            .take()
            .ok_or_else(|| Error::Conversion("package manifest was not initialized".to_string()))?;
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
        let manifest_offset = self.file.stream_position()?;
        let manifest_length = u64::try_from(manifest_bytes.len())
            .map_err(|_| Error::Conversion("manifest length overflow".to_string()))?;
        self.file.write_all(&manifest_bytes)?;
        self.file.write_all(&manifest_offset.to_le_bytes())?;
        self.file.write_all(&manifest_length.to_le_bytes())?;
        self.file.write_all(PACKAGE_FOOTER_MAGIC)?;
        self.file.flush()?;
        Ok(manifest)
    }
}

impl CanonicalTableSink for PackageSink {
    fn start(&mut self, header: &CityArrowHeader, projection: &ProjectionLayout) -> Result<()> {
        self.file.write_all(PACKAGE_MAGIC)?;
        self.manifest = Some(PackageManifest::new(
            header.citymodel_id.clone(),
            header.cityjson_version.clone(),
            projection.clone(),
        ));
        Ok(())
    }

    fn push_batch(&mut self, table: CanonicalTable, batch: RecordBatch) -> Result<()> {
        let offset = self.file.stream_position()?;
        write_file_batch(&mut self.file, &batch)?;
        let end = self.file.stream_position()?;
        let length = end.checked_sub(offset).ok_or_else(|| {
            Error::Conversion(format!("{} payload range underflow", table.as_str()))
        })?;
        self.manifest
            .as_mut()
            .ok_or_else(|| Error::Conversion("package manifest was not initialized".to_string()))?
            .tables
            .push(PackageTableRef {
                name: table.as_str().to_string(),
                offset,
                length,
                rows: batch.num_rows(),
            });
        Ok(())
    }
}

fn read_package_file(path: impl AsRef<Path>) -> Result<OwnedCityModel> {
    let path = path.as_ref();
    let mut file = File::open(path)?;
    let manifest = read_manifest_from_file(&mut file)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let schemas = canonical_schema_set(&manifest.projection);
    let mut decoder = IncrementalDecoder::new(
        CityArrowHeader::from(&manifest),
        manifest.projection.clone(),
    )?;

    for table in &manifest.tables {
        let kind = CanonicalTable::parse(&table.name)?;
        let batch = deserialize_file_batch(
            &mmap,
            table.offset,
            table.length,
            schema_for_table(&schemas, kind),
            kind,
            table.rows,
        )?;
        decoder.push_batch(kind, &batch)?;
    }

    decoder.finish()
}

#[doc(hidden)]
pub fn write_package_model_file(
    path: impl AsRef<Path>,
    model: &OwnedCityModel,
) -> Result<PackageManifest> {
    let file = File::create(path)?;
    let mut sink = PackageSink::new(file);
    emit_tables(model, &mut sink)?;
    sink.finish()
}

#[doc(hidden)]
pub fn write_package_parts_file(
    path: impl AsRef<Path>,
    parts: &CityModelArrowParts,
) -> Result<PackageManifest> {
    let file = File::create(path)?;
    let mut sink = PackageSink::new(file);
    emit_part_tables(parts, &mut sink)?;
    sink.finish()
}

#[doc(hidden)]
pub fn read_package_model_file(path: impl AsRef<Path>) -> Result<OwnedCityModel> {
    read_package_file(path)
}

#[doc(hidden)]
pub fn read_package_parts_file(path: impl AsRef<Path>) -> Result<CityModelArrowParts> {
    let path = path.as_ref();
    let mut file = File::open(path)?;
    let manifest = read_manifest_from_file(&mut file)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let schemas = canonical_schema_set(&manifest.projection);
    let mut tables = Vec::with_capacity(manifest.tables.len());

    for table in &manifest.tables {
        let kind = CanonicalTable::parse(&table.name)?;
        let batch = deserialize_file_batch(
            &mmap,
            table.offset,
            table.length,
            schema_for_table(&schemas, kind),
            kind,
            table.rows,
        )?;
        tables.push((kind, batch));
    }

    let header = CityArrowHeader::from(&manifest);
    build_parts_from_tables(&header, &manifest.projection, tables)
}

/// Reads only the manifest from a single-file package.
///
/// # Errors
///
/// Returns an error when the package footer or manifest cannot be read.
pub fn read_package_manifest(path: impl AsRef<Path>) -> Result<PackageManifest> {
    let mut file = File::open(path)?;
    read_manifest_from_file(&mut file)
}

fn read_manifest_from_file(file: &mut File) -> Result<PackageManifest> {
    let file_len = usize::try_from(file.metadata()?.len())
        .map_err(|_| Error::Conversion("package file length does not fit in memory".to_string()))?;
    if file_len < PACKAGE_MAGIC.len() + FOOTER_LEN {
        return Err(Error::Unsupported(
            "package is too small to be a cityjson-arrow package".to_string(),
        ));
    }

    let mut magic = vec![0_u8; PACKAGE_MAGIC.len()];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut magic)?;
    if magic != PACKAGE_MAGIC {
        return Err(Error::Unsupported(
            "package header magic is invalid".to_string(),
        ));
    }

    let footer_start = u64::try_from(file_len - FOOTER_LEN)
        .map_err(|_| Error::Conversion("package footer offset overflow".to_string()))?;
    file.seek(SeekFrom::Start(footer_start))?;
    let mut footer = [0_u8; FOOTER_LEN];
    file.read_exact(&mut footer)?;
    if &footer[16..] != PACKAGE_FOOTER_MAGIC {
        return Err(Error::Unsupported(
            "package footer magic is invalid".to_string(),
        ));
    }

    let manifest_offset = usize::try_from(u64::from_le_bytes(
        footer[..8].try_into().expect("footer slice"),
    ))
    .map_err(|_| Error::Conversion("manifest offset does not fit in memory".to_string()))?;
    let manifest_length = usize::try_from(u64::from_le_bytes(
        footer[8..16].try_into().expect("footer slice"),
    ))
    .map_err(|_| Error::Conversion("manifest length does not fit in memory".to_string()))?;
    let footer_start_usize = usize::try_from(footer_start)
        .map_err(|_| Error::Conversion("footer offset does not fit in memory".to_string()))?;
    let manifest_end = manifest_offset
        .checked_add(manifest_length)
        .ok_or_else(|| Error::Conversion("manifest range overflow".to_string()))?;
    if manifest_end > footer_start_usize || manifest_offset < PACKAGE_MAGIC.len() {
        return Err(Error::Unsupported(
            "package manifest range is invalid".to_string(),
        ));
    }

    let mut manifest_bytes = vec![0_u8; manifest_length];
    file.seek(SeekFrom::Start(u64::try_from(manifest_offset).map_err(
        |_| Error::Conversion("manifest offset overflow".to_string()),
    )?))?;
    file.read_exact(&mut manifest_bytes)?;
    serde_json::from_slice(&manifest_bytes).map_err(Error::from)
}

fn write_file_batch<W: Write>(writer: &mut W, batch: &RecordBatch) -> Result<()> {
    let mut file = FileWriter::try_new(writer, &batch.schema())?;
    file.write(batch)?;
    file.finish()?;
    Ok(())
}

fn deserialize_file_batch(
    bytes: &[u8],
    offset: u64,
    length: u64,
    expected_schema: &SchemaRef,
    table: CanonicalTable,
    expected_rows: usize,
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
    let mut reader = FileReader::try_new(Cursor::new(slice), None)?;
    let schema = reader.schema();
    validate_schema(expected_schema, &schema, table)?;
    let batch = single_or_concat_batches(expected_schema, &mut reader)?;
    if batch.num_rows() != expected_rows {
        return Err(Error::Conversion(format!(
            "{} table declared {expected_rows} rows but decoded {} rows",
            table.as_str(),
            batch.num_rows()
        )));
    }
    Ok(batch)
}
