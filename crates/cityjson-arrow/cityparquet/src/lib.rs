//! Parquet transport for `cityjson-rs`.
//!
//! This crate keeps the canonical transport model and conversion logic from
//! `cityarrow`, but exposes Parquet package I/O as the public entry point.

pub mod package;

pub use cityarrow::convert::{from_parts, to_parts};
pub use cityarrow::error::{Error, Result};
pub use cityarrow::schema::{
    CityArrowHeader, CityArrowPackageVersion, CityModelArrowParts, PackageManifest,
    PackageTableEncoding, PackageTables, ProjectedFieldSpec, ProjectedValueType, ProjectionLayout,
    canonical_schema_set,
};
pub use package::{read_package, read_package_dir, write_package, write_package_dir};
