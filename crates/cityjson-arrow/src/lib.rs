//! cityarrow rewrite scaffold.
//!
//! The old prototype conversion tree is intentionally not part of the default
//! build. The acceptance harness is wired separately in `tests/` while the new
//! Arrow/Parquet architecture is rebuilt.

pub mod error;
pub mod schema;

pub use schema::{
    CityArrowHeader, CityArrowPackageVersion, CityModelArrowParts, PackageManifest, PackageTables,
    ProjectedFieldSpec, ProjectedValueType, ProjectionLayout, canonical_schema_set,
};
