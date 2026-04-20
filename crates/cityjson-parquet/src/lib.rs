//! Persistent package IO for `cityjson-arrow`.
//!
//! `cityjson-parquet` owns the durable package/container boundary in the ADR 3
//! architecture. The current package format is a seekable single-file
//! container backed by Arrow canonical tables.

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
