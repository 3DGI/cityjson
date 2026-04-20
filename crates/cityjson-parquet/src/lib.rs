//! Persistent CityJSON package and native Parquet dataset IO.
//!
//! `cityjson-parquet` exposes two durable representations of the same canonical
//! CityJSON Arrow tables:
//!
//! - [`PackageWriter`] / [`PackageReader`] read and write the `.cityjson-parquet`
//!   single-file package. This is a seekable container backed by Arrow IPC table
//!   payloads.
//! - [`ParquetDatasetWriter`] / [`ParquetDatasetReader`] read and write a native
//!   Parquet dataset directory with one Parquet file per canonical table. This
//!   is the interoperability target for PyArrow, DuckDB, Polars, and similar
//!   Parquet-native tools.

mod dataset;
mod package;
pub mod spatial;

pub use cityjson_arrow::schema::{
    CityArrowHeader, CityArrowPackageVersion, PackageManifest, PackageTableRef, ProjectedFieldSpec,
    ProjectedStructSpec, ProjectedValueSpec, ProjectionLayout, canonical_schema_set,
};
pub use dataset::{
    ParquetDatasetManifest, ParquetDatasetReader, ParquetDatasetTableRef, ParquetDatasetWriter,
};
#[doc(hidden)]
pub use dataset::{
    read_parquet_dataset_manifest, read_parquet_dataset_model_dir, write_parquet_dataset_model_dir,
};
pub use package::{PackageReader, PackageWriter};
#[doc(hidden)]
pub use package::{
    read_package_model_file, read_package_parts_file, write_package_model_file,
    write_package_parts_file,
};
