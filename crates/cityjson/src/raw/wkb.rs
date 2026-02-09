//! WKB export hooks for `GeoParquet` interoperability.
//!
//! Full WKB geometry conversion is intentionally left to higher-level crates.

/// Trait for exporting a geometry-like type to WKB bytes.
pub trait ToWkb {
    /// Export into WKB.
    fn to_wkb(&self) -> Vec<u8>;

    /// Export into EWKB with an SRID.
    fn to_ewkb(&self, srid: i32) -> Vec<u8>;
}
