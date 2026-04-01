use crate::error::{Error, Result};
use crate::schema::{CityModelArrowParts, PackageManifest, PackageTableRef, canonical_schema_set};
use crate::transport::{
    CanonicalTable, build_parts, collect_tables, concat_record_batches, schema_for_table,
    validate_schema,
};
use arrow::ipc::reader::StreamReader;
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};

const STREAM_MAGIC: &[u8] = b"CITYARROW_STREAM_V2\0";

pub(crate) fn write_model_stream<W: Write>(
    parts: &CityModelArrowParts,
    mut writer: W,
) -> Result<()> {
    let tables = collect_tables(parts);
    let mut payloads = Vec::with_capacity(tables.len());
    let mut manifest = PackageManifest::new(
        parts.header.citymodel_id.clone(),
        parts.header.cityjson_version.clone(),
        parts.projection.clone(),
    );

    for (table, batch) in tables {
        let payload = serialize_stream_batch(&batch)?;
        manifest.tables.push(PackageTableRef {
            name: table.as_str().to_string(),
            offset: 0,
            length: u64::try_from(payload.len()).map_err(|_| {
                Error::Conversion(format!("{} payload length overflow", table.as_str()))
            })?,
            rows: batch.num_rows(),
        });
        payloads.push(payload);
    }

    let manifest_bytes = serde_json::to_vec(&manifest)?;
    let manifest_len = u64::try_from(manifest_bytes.len())
        .map_err(|_| Error::Conversion("stream manifest length overflow".to_string()))?;

    writer.write_all(STREAM_MAGIC)?;
    writer.write_all(&manifest_len.to_le_bytes())?;
    writer.write_all(&manifest_bytes)?;
    for payload in payloads {
        writer.write_all(&payload)?;
    }
    Ok(())
}

pub(crate) fn read_model_stream<R: Read>(mut reader: R) -> Result<CityModelArrowParts> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    if bytes.len() < STREAM_MAGIC.len() + 8 {
        return Err(Error::Unsupported(
            "stream is too short to be a cityarrow stream".to_string(),
        ));
    }
    if &bytes[..STREAM_MAGIC.len()] != STREAM_MAGIC {
        return Err(Error::Unsupported(
            "stream header magic is invalid".to_string(),
        ));
    }

    let mut manifest_len_bytes = [0_u8; 8];
    manifest_len_bytes.copy_from_slice(&bytes[STREAM_MAGIC.len()..STREAM_MAGIC.len() + 8]);
    let manifest_len = usize::try_from(u64::from_le_bytes(manifest_len_bytes)).map_err(|_| {
        Error::Conversion("stream manifest length does not fit in memory".to_string())
    })?;
    let manifest_start = STREAM_MAGIC.len() + 8;
    let manifest_end = manifest_start
        .checked_add(manifest_len)
        .ok_or_else(|| Error::Conversion("stream manifest range overflow".to_string()))?;
    let manifest: PackageManifest = serde_json::from_slice(
        bytes
            .get(manifest_start..manifest_end)
            .ok_or_else(|| Error::Unsupported("stream manifest range is invalid".to_string()))?,
    )?;

    let mut cursor = manifest_end;
    let schemas = canonical_schema_set(&manifest.projection);
    let mut tables = HashMap::new();
    for entry in &manifest.tables {
        let table = CanonicalTable::parse(&entry.name)?;
        let payload_len = usize::try_from(entry.length).map_err(|_| {
            Error::Conversion(format!(
                "{} payload length does not fit in memory",
                table.as_str()
            ))
        })?;
        let payload_end = cursor.checked_add(payload_len).ok_or_else(|| {
            Error::Conversion(format!("{} payload range overflow", table.as_str()))
        })?;
        let payload = bytes.get(cursor..payload_end).ok_or_else(|| {
            Error::Unsupported(format!("{} payload range is invalid", table.as_str()))
        })?;
        let batch = deserialize_stream_batch(payload, schema_for_table(&schemas, table), table)?;
        tables.insert(table, batch);
        cursor = payload_end;
    }

    build_parts(&manifest, tables)
}

fn serialize_stream_batch(batch: &RecordBatch) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = StreamWriter::try_new(&mut cursor, &batch.schema())?;
        writer.write(batch)?;
        writer.finish()?;
    }
    Ok(cursor.into_inner())
}

fn deserialize_stream_batch(
    payload: &[u8],
    expected_schema: &arrow::datatypes::SchemaRef,
    table: CanonicalTable,
) -> Result<RecordBatch> {
    let reader = StreamReader::try_new(Cursor::new(payload), None)?;
    let schema = reader.schema();
    validate_schema(expected_schema, &schema, table)?;
    let batches = reader.collect::<std::result::Result<Vec<_>, _>>()?;
    concat_record_batches(expected_schema, &batches)
}
