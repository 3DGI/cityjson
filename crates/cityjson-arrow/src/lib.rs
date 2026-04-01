//! Arrow and Arrow IPC transport for `cityjson-rs`.
//!
//! The semantic unit remains `cityjson::v2_0::OwnedCityModel`.
//! `CityModelArrowParts` is the canonical transport decomposition used by the
//! package readers and writers; it is not a second semantic model.

pub mod convert;
pub mod error;
pub mod package;
pub mod schema;

pub use convert::{from_parts, to_parts};
pub use package::{
    read_package_ipc, read_package_ipc_dir, write_package_ipc, write_package_ipc_dir,
};
pub use schema::{
    CityArrowHeader, CityArrowPackageVersion, CityModelArrowParts, PackageManifest,
    PackageTableEncoding, PackageTables, ProjectedFieldSpec, ProjectedValueType, ProjectionLayout,
    canonical_schema_set,
};
