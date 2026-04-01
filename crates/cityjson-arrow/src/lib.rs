//! Arrow transport for `cityjson-rs`.
//!
//! The semantic boundary remains `cityjson::v2_0::OwnedCityModel`.
//! Canonical Arrow tables are now an internal detail used by the live stream
//! encoder/decoder and the single-file package reader/writer.

pub mod convert;
pub mod error;
#[doc(hidden)]
pub mod internal;
pub mod schema;
mod stream;
#[doc(hidden)]
pub mod transport;

pub use convert::{ModelDecoder, ModelEncoder};
pub use schema::{
    CityArrowHeader, CityArrowPackageVersion, PackageManifest, PackageTableRef, ProjectedFieldSpec,
    ProjectedValueType, ProjectionLayout, canonical_schema_set,
};
