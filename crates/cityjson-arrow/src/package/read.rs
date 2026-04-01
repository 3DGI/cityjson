use super::{concat_record_batches, read_package_dir_with_loader, table_path_from_manifest};
use crate::error::{Error, Result};
use crate::schema::{CityModelArrowParts, PackageTableEncoding};
use arrow::datatypes::SchemaRef;
use arrow::ipc::reader::FileReader;
use arrow::record_batch::RecordBatch;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Reads a package directory whose tables are stored as Arrow IPC files.
///
/// # Errors
///
/// Returns an error when the manifest, schemas, or table contents cannot be read
/// or do not match the canonical package layout.
pub fn read_package_ipc(dir: impl AsRef<Path>) -> Result<CityModelArrowParts> {
    read_package_ipc_dir(dir)
}

/// Reads a package directory whose tables are stored as Arrow IPC files.
///
/// # Errors
///
/// Returns an error when the manifest, schemas, or table contents cannot be read
/// or do not match the canonical package layout.
pub fn read_package_ipc_dir(dir: impl AsRef<Path>) -> Result<CityModelArrowParts> {
    read_package_dir_with_loader(dir, Some(PackageTableEncoding::ArrowIpcFile), load_table)
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
        PackageTableEncoding::ArrowIpcFile => {
            let reader = FileReader::try_new(file, None)?;
            let schema = reader.schema();
            let batches = reader.collect::<std::result::Result<Vec<_>, _>>()?;
            (schema, batches)
        }
        PackageTableEncoding::Parquet => {
            return Err(Error::Unsupported(
                "cityarrow only supports Arrow IPC package tables".to_string(),
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
