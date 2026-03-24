//! Raw access to internal data structures for efficient serialization.
//!
//! This module exposes zero-copy views over core containers so downstream crates
//! can build custom serializers (for example Parquet/Arrow exporters) without
//! rebuilding intermediate nested structures.

pub mod accessors;
pub mod v2_0;
pub mod views;
pub use accessors::*;
pub use v2_0::*;
pub use views::*;
