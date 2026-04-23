use crate::convert;
use crate::error::Result;
use crate::schema::CityModelArrowParts;
use crate::schema::{CityArrowHeader, ProjectionLayout};
use arrow::record_batch::RecordBatch;
use cityjson::relational::RelationalAccess;
use cityjson::v2_0::OwnedCityModel;

pub use crate::transport::{
    CanonicalTable, CanonicalTableSink, canonical_table_order, canonical_table_position,
    collect_tables, concat_record_batches, schema_for_table, single_or_concat_batches,
    validate_schema,
};

/// Internal bridge for sibling crates that still need canonical transport
/// parts.
///
/// This is not part of the supported end-user API.
///
/// # Errors
///
/// Returns an error when canonical transport encoding fails.
pub fn encode_parts(model: &OwnedCityModel) -> Result<CityModelArrowParts> {
    convert::encode_parts(&model.relational())
}

/// Emits canonical transport tables without materializing the public read/write
/// path around a full parts aggregate.
///
/// This is not part of the supported end-user API.
///
/// # Errors
///
/// Returns an error when conversion fails or the sink rejects a table batch.
pub fn emit_tables<S: CanonicalTableSink>(model: &OwnedCityModel, sink: &mut S) -> Result<()> {
    convert::emit_tables(&model.relational(), sink)
}

/// Emits canonical transport tables from an existing parts aggregate.
///
/// This is not part of the supported end-user API.
///
/// # Errors
///
/// Returns an error when the sink rejects a table batch.
pub fn emit_part_tables<S: CanonicalTableSink>(
    parts: &CityModelArrowParts,
    sink: &mut S,
) -> Result<()> {
    convert::emit_part_tables(parts, sink)
}

/// Internal bridge for sibling crates that still need canonical transport
/// parts.
///
/// This is not part of the supported end-user API.
///
/// # Errors
///
/// Returns an error when canonical transport decoding fails.
pub fn decode_parts(parts: &CityModelArrowParts) -> Result<OwnedCityModel> {
    convert::decode_parts(parts)
}

/// Rebuilds canonical parts from ordered table batches.
///
/// This is not part of the supported end-user API.
///
/// # Errors
///
/// Returns an error when the ordered batch set is incomplete or inconsistent.
pub fn build_parts_from_tables(
    header: &CityArrowHeader,
    projection: &ProjectionLayout,
    tables: Vec<(CanonicalTable, RecordBatch)>,
) -> Result<CityModelArrowParts> {
    convert::build_parts_from_tables(header, projection, tables)
}

/// Serializes canonical parts through the live stream transport boundary.
///
/// This is not part of the supported end-user API.
///
/// # Errors
///
/// Returns an error when stream serialization fails.
pub fn write_stream_parts<W: std::io::Write>(parts: &CityModelArrowParts, writer: W) -> Result<()> {
    crate::stream::write_parts_stream(parts, writer)
}

/// Reads canonical parts through the live stream transport boundary.
///
/// This is not part of the supported end-user API.
///
/// # Errors
///
/// Returns an error when stream decoding fails.
pub fn read_stream_parts<R: std::io::Read>(reader: R) -> Result<CityModelArrowParts> {
    crate::stream::read_parts_stream(reader)
}

/// Incremental model decoder over ordered canonical table batches.
///
/// This is not part of the supported end-user API.
pub struct IncrementalDecoder(convert::IncrementalDecoder);

impl IncrementalDecoder {
    /// # Errors
    ///
    /// Returns an error when the stream or package prelude is invalid.
    pub fn new(header: CityArrowHeader, projection: ProjectionLayout) -> Result<Self> {
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
