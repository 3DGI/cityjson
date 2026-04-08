//! Persistent package IO for `cityarrow`.
//!
//! `cityparquet` owns the durable package/container boundary in the ADR 3
//! architecture. The current package format is a seekable single-file
//! container backed by Arrow canonical tables.

mod package;
pub mod spatial;

pub use cityarrow::schema::{
    CityArrowHeader, CityArrowPackageVersion, PackageManifest, PackageTableRef, ProjectedFieldSpec,
    ProjectedStructSpec, ProjectedValueSpec, ProjectionLayout, canonical_schema_set,
};
pub use package::{PackageReader, PackageWriter};
#[doc(hidden)]
pub use package::{
    read_package_model_file, read_package_parts_file, write_package_model_file,
    write_package_parts_file,
};
