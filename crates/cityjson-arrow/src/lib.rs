//! cityarrow rewrite scaffold.
//!
//! The old prototype conversion tree is intentionally not part of the default
//! build. The acceptance harness is wired separately in `tests/` while the new
//! Arrow/Parquet architecture is rebuilt.

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
