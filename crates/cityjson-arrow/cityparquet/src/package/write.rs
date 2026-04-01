use super::{
    CanonicalTable, CityModelArrowParts, PackageManifest, PackageTableEncoding,
    package_table_path_for_encoding, write_package_dir_with_writer,
};
use arrow::record_batch::RecordBatch;
use cityarrow::error::{Error, Result};
use parquet::arrow::ArrowWriter;
use std::fs::File;
use std::path::Path;

/// Writes a package directory whose tables are stored as Parquet files.
///
/// # Errors
///
/// Returns an error when schemas are invalid, tables are inconsistent, or the
/// package files cannot be written.
pub fn write_package(
    dir: impl AsRef<Path>,
    parts: &CityModelArrowParts,
) -> Result<PackageManifest> {
    write_package_dir(dir, parts)
}

/// Writes a package directory whose tables are stored as Parquet files.
///
/// # Errors
///
/// Returns an error when schemas are invalid, tables are inconsistent, or the
/// package files cannot be written.
pub fn write_package_dir(
    dir: impl AsRef<Path>,
    parts: &CityModelArrowParts,
) -> Result<PackageManifest> {
    write_package_dir_with_writer(dir, parts, PackageTableEncoding::Parquet, write_batch)
}

fn write_batch(
    dir: &Path,
    table: CanonicalTable,
    batch: &RecordBatch,
    encoding: PackageTableEncoding,
) -> Result<()> {
    let path = package_table_path_for_encoding(dir, table, encoding);
    let file = File::create(path)?;
    match encoding {
        PackageTableEncoding::Parquet => {
            let mut writer = ArrowWriter::try_new(file, batch.schema(), None)?;
            writer.write(batch)?;
            writer.close()?;
        }
        PackageTableEncoding::ArrowIpcFile => {
            return Err(Error::Unsupported(
                "cityparquet only supports Parquet package tables".to_string(),
            ));
        }
    }
    Ok(())
}
