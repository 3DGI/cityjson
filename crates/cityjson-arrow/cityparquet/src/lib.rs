//! Persistent package IO for `cityarrow`.
//!
//! `cityparquet` owns the durable package/container boundary in the ADR 3
//! architecture. The current package format is a seekable single-file
//! container backed by Arrow canonical tables.

mod package;

pub use cityarrow::schema::{
    CityArrowHeader, CityArrowPackageVersion, PackageManifest, PackageTableRef, ProjectedFieldSpec,
    ProjectedValueType, ProjectionLayout, canonical_schema_set,
};
pub use package::{PackageReader, PackageWriter};
