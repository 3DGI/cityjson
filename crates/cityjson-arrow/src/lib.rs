//! Arrow and Parquet transport for `cityjson-rs`.
//!
//! The semantic unit remains `cityjson::v2_0::OwnedCityModel`.
//! `CityModelArrowParts` is the canonical transport decomposition used by the
//! implemented Parquet package reader and writer.

pub mod convert;
pub mod error;
pub mod package;
pub mod schema;

pub use convert::{from_parts, to_parts};
pub use package::{read_package, read_package_dir, write_package, write_package_dir};
pub use schema::{
    CityArrowHeader, CityArrowPackageVersion, CityModelArrowParts, PackageManifest, PackageTables,
    ProjectedFieldSpec, ProjectedValueType, ProjectionLayout, canonical_schema_set,
};
