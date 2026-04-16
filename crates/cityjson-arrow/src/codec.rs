use crate::convert;
use crate::error::{Error, Result};
use crate::schema::{CityArrowHeader, CityArrowPackageVersion, ProjectionLayout};
use crate::stream;
use crate::transport::{CanonicalTable, CanonicalTableSink};
use arrow::record_batch::RecordBatch;
use cityjson::relational::RelationalAccess;
use cityjson::v2_0::OwnedCityModel;
use std::collections::VecDeque;
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaVersion {
    V3Alpha2,
}

impl SchemaVersion {
    #[must_use]
    pub const fn package_version(self) -> CityArrowPackageVersion {
        match self {
            Self::V3Alpha2 => CityArrowPackageVersion::V3Alpha2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportOptions {
    pub schema_version: SchemaVersion,
    pub batch_row_limit: usize,
    pub dictionary_encode_strings: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            schema_version: SchemaVersion::V3Alpha2,
            batch_row_limit: 65_536,
            dictionary_encode_strings: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportOptions {
    pub expected_schema_version: Option<SchemaVersion>,
    pub symbol_storage: cityjson::symbols::SymbolStorageOptions,
    pub validate_schema: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            expected_schema_version: Some(SchemaVersion::V3Alpha2),
            symbol_storage: cityjson::symbols::SymbolStorageOptions::default(),
            validate_schema: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WriteReport {
    pub batch_count: usize,
    pub row_count: usize,
    pub payload_bytes: u64,
}

pub struct ModelBatchReader {
    header: CityArrowHeader,
    projection: ProjectionLayout,
    batches: VecDeque<(CanonicalTable, RecordBatch)>,
}

impl ModelBatchReader {
    #[must_use]
    pub const fn header(&self) -> &CityArrowHeader {
        &self.header
    }

    #[must_use]
    pub const fn projection(&self) -> &ProjectionLayout {
        &self.projection
    }
}

impl Iterator for ModelBatchReader {
    type Item = (CanonicalTable, RecordBatch);

    fn next(&mut self) -> Option<Self::Item> {
        self.batches.pop_front()
    }
}

pub struct ModelBatchDecoder(convert::IncrementalDecoder);

impl ModelBatchDecoder {
    /// # Errors
    ///
    /// Returns an error when the schema version is not supported or the
    /// canonical projection is invalid.
    pub fn new(
        header: CityArrowHeader,
        projection: ProjectionLayout,
        options: &ImportOptions,
    ) -> Result<Self> {
        validate_expected_schema_version(options, header.package_version)?;
        convert::IncrementalDecoder::new(header, projection).map(Self)
    }

    /// # Errors
    ///
    /// Returns an error when the batch order, schema, or data is invalid.
    pub fn push_batch(&mut self, table: CanonicalTable, batch: &RecordBatch) -> Result<()> {
        self.0.push_batch(table, batch)
    }

    /// # Errors
    ///
    /// Returns an error when required tables are missing or reconstruction
    /// cannot finish successfully.
    pub fn finish(self) -> Result<OwnedCityModel> {
        self.0.finish()
    }
}

/// # Errors
///
/// Returns an error when export conversion fails.
pub fn export_reader(model: &OwnedCityModel, options: &ExportOptions) -> Result<ModelBatchReader> {
    validate_export_schema_version(options)?;
    let relational = model.relational();
    let mut sink = BatchReaderSink::default();
    convert::emit_tables(&relational, &mut sink)?;
    sink.finish()
}

/// # Errors
///
/// Returns an error when ordered batch import fails.
pub fn import_batches<I>(
    header: CityArrowHeader,
    projection: ProjectionLayout,
    batches: I,
    options: &ImportOptions,
) -> Result<OwnedCityModel>
where
    I: IntoIterator<Item = (CanonicalTable, RecordBatch)>,
{
    let mut decoder = ModelBatchDecoder::new(header, projection, options)?;
    for (table, batch) in batches {
        decoder.push_batch(table, &batch)?;
    }
    decoder.finish()
}

/// # Errors
///
/// Returns an error when conversion or stream serialization fails.
pub fn write_stream<W: Write>(
    writer: W,
    model: &OwnedCityModel,
    options: &ExportOptions,
) -> Result<WriteReport> {
    validate_export_schema_version(options)?;
    let relational = model.relational();
    stream::write_model_stream(&relational, writer)
}

/// # Errors
///
/// Returns an error when stream decoding or model reconstruction fails.
pub fn read_stream<R: Read>(reader: R, options: &ImportOptions) -> Result<OwnedCityModel> {
    let (header, projection, batches) = stream::read_stream_batches(reader)?;
    let ordered_batches = batches
        .into_iter()
        .map(|(table, expected_rows, batch)| {
            if batch.num_rows() == expected_rows {
                Ok((table, batch))
            } else {
                Err(Error::Conversion(format!(
                    "{} frame declared {expected_rows} rows but decoded {} rows",
                    table.as_str(),
                    batch.num_rows()
                )))
            }
        })
        .collect::<Result<Vec<_>>>()?;
    import_batches(header, projection, ordered_batches, options)
}

fn validate_export_schema_version(options: &ExportOptions) -> Result<()> {
    if options.schema_version == SchemaVersion::V3Alpha2 {
        Ok(())
    } else {
        Err(Error::Unsupported(format!(
            "schema version '{}' is not supported by this crate",
            options.schema_version.package_version().as_str()
        )))
    }
}

fn validate_expected_schema_version(
    options: &ImportOptions,
    actual: CityArrowPackageVersion,
) -> Result<()> {
    if let Some(expected) = options.expected_schema_version
        && expected.package_version() != actual
    {
        return Err(Error::Unsupported(format!(
            "stream uses '{}' but '{}' was requested",
            actual.as_str(),
            expected.package_version().as_str()
        )));
    }
    Ok(())
}

#[derive(Default)]
struct BatchReaderSink {
    header: Option<CityArrowHeader>,
    projection: Option<ProjectionLayout>,
    batches: VecDeque<(CanonicalTable, RecordBatch)>,
}

impl BatchReaderSink {
    fn finish(self) -> Result<ModelBatchReader> {
        Ok(ModelBatchReader {
            header: self
                .header
                .ok_or_else(|| Error::Conversion("missing canonical table header".to_string()))?,
            projection: self.projection.ok_or_else(|| {
                Error::Conversion("missing canonical table projection".to_string())
            })?,
            batches: self.batches,
        })
    }
}

impl CanonicalTableSink for BatchReaderSink {
    fn start(&mut self, header: &CityArrowHeader, projection: &ProjectionLayout) -> Result<()> {
        self.header = Some(header.clone());
        self.projection = Some(projection.clone());
        Ok(())
    }

    fn push_batch(&mut self, table: CanonicalTable, batch: RecordBatch) -> Result<()> {
        self.batches.push_back((table, batch));
        Ok(())
    }
}
