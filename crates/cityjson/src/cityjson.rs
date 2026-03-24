//! # `CityJSON` types and traits
//!
//! Shared internal modules used to implement versioned `CityJSON` APIs.

pub mod core;

#[cfg(test)]
mod tests {
    use crate::cityjson::core::geometry::{GeometryType, LoD};

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
