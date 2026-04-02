use crate::convert::{IncrementalDecoder, emit_tables};
use crate::error::{Error, Result};
use crate::internal::{build_parts_from_tables, emit_part_tables};
use crate::schema::{CityArrowHeader, CityModelArrowParts, ProjectionLayout, canonical_schema_set};
use crate::transport::{
    CanonicalTable, CanonicalTableSink, concat_record_batches, schema_for_table, validate_schema,
};
use arrow::ipc::reader::StreamReader;
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use cityjson::v2_0::OwnedCityModel;
use serde::{Deserialize, Serialize};
use std::io::{ErrorKind, Read, Write};

const STREAM_MAGIC: &[u8] = b"CITYARROW_STREAM_V3\0";
const STREAM_END_TAG: u8 = u8::MAX;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StreamPrelude {
    header: CityArrowHeader,
    projection: ProjectionLayout,
}

type StreamFrames = Vec<(CanonicalTable, usize, RecordBatch)>;

pub(crate) fn write_model_stream<W: Write>(model: &OwnedCityModel, writer: W) -> Result<()> {
    let mut sink = StreamSink::new(writer);
    emit_tables(model, &mut sink)?;
    sink.finish()
}

pub(crate) fn read_model_stream<R: Read>(mut reader: R) -> Result<OwnedCityModel> {
    let (prelude, tables) = read_stream_frames(&mut reader)?;
    let schemas = canonical_schema_set(&prelude.projection);
    let mut decoder = IncrementalDecoder::new(prelude.header, prelude.projection)?;

    for (table, expected_rows, batch) in tables {
        if batch.num_rows() != expected_rows {
            return Err(Error::Conversion(format!(
                "{} frame declared {expected_rows} rows but decoded {} rows",
                table.as_str(),
                batch.num_rows()
            )));
        }
        validate_schema(schema_for_table(&schemas, table), batch.schema(), table)?;
        decoder.push_batch(table, &batch)?;
    }

    decoder.finish()
}

pub(crate) fn write_parts_stream<W: Write>(parts: &CityModelArrowParts, writer: W) -> Result<()> {
    let mut sink = StreamSink::new(writer);
    emit_part_tables(parts, &mut sink)?;
    sink.finish()
}

pub(crate) fn read_parts_stream<R: Read>(mut reader: R) -> Result<CityModelArrowParts> {
    let (prelude, tables) = read_stream_frames(&mut reader)?;
    let ordered_tables = tables
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
    build_parts_from_tables(&prelude.header, &prelude.projection, ordered_tables)
}

struct StreamSink<W> {
    writer: W,
    started: bool,
}

impl<W> StreamSink<W> {
    const fn new(writer: W) -> Self {
        Self {
            writer,
            started: false,
        }
    }
}

impl<W: Write> StreamSink<W> {
    fn finish(&mut self) -> Result<()> {
        if self.started {
            self.writer.write_all(&[STREAM_END_TAG])?;
        }
        Ok(())
    }
}

impl<W: Write> CanonicalTableSink for StreamSink<W> {
    fn start(&mut self, header: &CityArrowHeader, projection: &ProjectionLayout) -> Result<()> {
        let prelude_bytes = serde_json::to_vec(&StreamPrelude {
            header: header.clone(),
            projection: projection.clone(),
        })?;
        let prelude_len = u64::try_from(prelude_bytes.len())
            .map_err(|_| Error::Conversion("stream prelude length overflow".to_string()))?;

        self.writer.write_all(STREAM_MAGIC)?;
        self.writer.write_all(&prelude_len.to_le_bytes())?;
        self.writer.write_all(&prelude_bytes)?;
        self.started = true;
        Ok(())
    }

    fn push_batch(&mut self, table: CanonicalTable, batch: RecordBatch) -> Result<()> {
        if !self.started {
            return Err(Error::Unsupported(
                "stream sink must be started before writing table batches".to_string(),
            ));
        }
        self.writer.write_all(&[table.stream_tag()])?;
        self.writer.write_all(
            &u64::try_from(batch.num_rows())
                .map_err(|_| Error::Conversion("stream row count overflow".to_string()))?
                .to_le_bytes(),
        )?;
        write_stream_batch(&mut self.writer, &batch)?;
        Ok(())
    }
}

fn read_stream_prelude<R: Read>(reader: &mut R) -> Result<StreamPrelude> {
    let mut magic = vec![0_u8; STREAM_MAGIC.len()];
    reader.read_exact(&mut magic)?;
    if magic != STREAM_MAGIC {
        return Err(Error::Unsupported(
            "stream header magic is invalid".to_string(),
        ));
    }

    let prelude_len = usize::try_from(read_u64(reader)?)
        .map_err(|_| Error::Conversion("stream prelude length does not fit in memory".to_string()))?;
    let mut prelude_bytes = vec![0_u8; prelude_len];
    reader.read_exact(&mut prelude_bytes)?;
    serde_json::from_slice(&prelude_bytes).map_err(Error::from)
}

fn read_stream_frames<R: Read>(reader: &mut R) -> Result<(StreamPrelude, StreamFrames)> {
    let prelude = read_stream_prelude(reader)?;
    let schemas = canonical_schema_set(&prelude.projection);
    let mut tables = Vec::new();
    loop {
        let tag = read_u8(reader)?;
        if tag == STREAM_END_TAG {
            break;
        }
        let table = CanonicalTable::from_stream_tag(tag)?;
        let expected_rows = usize::try_from(read_u64(reader)?)
            .map_err(|_| Error::Conversion("stream row count does not fit in memory".to_string()))?;
        let batch = deserialize_stream_batch(
            reader,
            schema_for_table(&schemas, table),
            table,
            expected_rows,
        )?;
        tables.push((table, expected_rows, batch));
    }
    Ok((prelude, tables))
}

fn read_u8<R: Read>(reader: &mut R) -> Result<u8> {
    let mut byte = [0_u8; 1];
    reader.read_exact(&mut byte).map_err(|error| {
        if error.kind() == ErrorKind::UnexpectedEof {
            Error::Unsupported("stream ended before the final frame marker".to_string())
        } else {
            Error::from(error)
        }
    })?;
    Ok(byte[0])
}

fn read_u64<R: Read>(reader: &mut R) -> Result<u64> {
    let mut bytes = [0_u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

fn write_stream_batch<W: Write>(writer: &mut W, batch: &RecordBatch) -> Result<()> {
    let mut stream = StreamWriter::try_new(writer, &batch.schema())?;
    stream.write(batch)?;
    stream.finish()?;
    Ok(())
}

fn deserialize_stream_batch<R: Read>(
    reader: &mut R,
    expected_schema: &arrow::datatypes::SchemaRef,
    table: CanonicalTable,
    expected_rows: usize,
) -> Result<RecordBatch> {
    let stream = StreamReader::try_new(reader.by_ref(), None)?;
    let schema = stream.schema();
    validate_schema(expected_schema, &schema, table)?;
    let batches = stream.collect::<std::result::Result<Vec<_>, _>>()?;
    let batch = concat_record_batches(expected_schema, &batches)?;
    let _ = expected_rows;
    Ok(batch)
}
