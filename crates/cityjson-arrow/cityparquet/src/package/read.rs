use super::{
    CityModelArrowParts, PackageTableEncoding, concat_record_batches, read_package_dir_with_loader,
    table_path_from_manifest,
};
use arrow::array::RecordBatchReader;
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use cityarrow::error::{Error, Result};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Reads a package directory whose tables are stored as Parquet files.
///
/// # Errors
///
/// Returns an error when the manifest, schemas, or table contents cannot be read
/// or do not match the canonical package layout.
pub fn read_package(dir: impl AsRef<Path>) -> Result<CityModelArrowParts> {
    read_package_dir(dir)
}

/// Reads a package directory whose tables are stored as Parquet files.
///
/// # Errors
///
/// Returns an error when the manifest, schemas, or table contents cannot be read
/// or do not match the canonical package layout.
pub fn read_package_dir(dir: impl AsRef<Path>) -> Result<CityModelArrowParts> {
    read_package_dir_with_loader(dir, Some(PackageTableEncoding::Parquet), load_table)
}

fn load_table(
    dir: &Path,
    path: Option<&PathBuf>,
    encoding: PackageTableEncoding,
) -> Result<Option<(SchemaRef, RecordBatch)>> {
    let Some(path) = path else {
        return Ok(None);
    };

    let path = table_path_from_manifest(dir, path);
    let file = File::open(&path)?;

    let (schema, batches) = match encoding {
        PackageTableEncoding::Parquet => {
            let reader = ParquetRecordBatchReaderBuilder::try_new(file)?
                .with_batch_size(1024)
                .build()?;
            let schema = reader.schema().clone();
            let mut batches = Vec::new();
            for batch in reader {
                batches.push(batch?);
            }
            (schema, batches)
        }
        PackageTableEncoding::ArrowIpcFile => {
            return Err(Error::Unsupported(
                "cityparquet only supports Parquet package tables".to_string(),
            ));
        }
    };
    let batch = if batches.is_empty() {
        RecordBatch::new_empty(schema.clone())
    } else {
        concat_record_batches(&schema, &batches)?
    };
    Ok(Some((schema, batch)))
}
