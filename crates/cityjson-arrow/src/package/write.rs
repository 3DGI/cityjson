use super::{CanonicalTable, package_table_path_for_encoding, write_package_dir_with_writer};
use crate::error::{Error, Result};
use crate::schema::{CityModelArrowParts, PackageManifest, PackageTableEncoding};
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;
use std::fs::File;
use std::path::Path;

/// Writes a package directory whose tables are stored as Arrow IPC files.
///
/// # Errors
///
/// Returns an error when schemas are invalid, tables are inconsistent, or the
/// package files cannot be written.
pub fn write_package_ipc(
    dir: impl AsRef<Path>,
    parts: &CityModelArrowParts,
) -> Result<PackageManifest> {
    write_package_ipc_dir(dir, parts)
}

/// Writes a package directory whose tables are stored as Arrow IPC files.
///
/// # Errors
///
/// Returns an error when schemas are invalid, tables are inconsistent, or the
/// package files cannot be written.
pub fn write_package_ipc_dir(
    dir: impl AsRef<Path>,
    parts: &CityModelArrowParts,
) -> Result<PackageManifest> {
    write_package_dir_with_writer(dir, parts, PackageTableEncoding::ArrowIpcFile, write_batch)
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
        PackageTableEncoding::ArrowIpcFile => {
            let mut writer = FileWriter::try_new(file, &batch.schema())?;
            writer.write(batch)?;
            writer.finish()?;
        }
        PackageTableEncoding::Parquet => {
            return Err(Error::Unsupported(
                "cityarrow only supports Arrow IPC package tables".to_string(),
            ));
        }
    }
    Ok(())
}
