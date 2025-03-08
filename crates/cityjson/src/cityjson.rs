//! # CityJSON types and traits
//!
//! These types are version-agnostic, as they are not expected to change across versions.
pub mod appearance;
pub mod attributes;
pub mod citymodel;
pub mod cityobject;
pub mod coordinate;
pub mod extension;
pub mod geometry;
pub mod metadata;
pub mod transform;
pub mod vertex;
pub mod geometry_refactor;

#[cfg(test)]
mod tests {
    use crate::cityjson::geometry::{GeometryType, LoD};

    #[test]
    fn test_geometry_type_equality() {
        assert_eq!(GeometryType::MultiPoint, GeometryType::MultiPoint);
        assert_ne!(GeometryType::MultiPoint, GeometryType::MultiSurface);
    }

    #[test]
    fn test_lod_ordering() {
        assert!(LoD::LoD0 < LoD::LoD1);
        assert!(LoD::LoD1 < LoD::LoD2);
        assert!(LoD::LoD2 < LoD::LoD3);
        assert!(LoD::LoD0_1 > LoD::LoD0);
        assert!(LoD::LoD1_2 > LoD::LoD1);
    }
}
