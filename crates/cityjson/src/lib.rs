use std::fmt::Debug;

pub mod boundary;
pub mod boundary_nested;
pub mod coordinate;
pub mod errors;
mod resources_semantics_materials;
mod resources_textures;
pub mod vertex;

use crate::vertex::VertexInteger;
pub use boundary::Boundary;
pub use coordinate::VertexCoordinate;
pub use resources_semantics_materials::SemanticMaterialMap;
pub use resources_textures::TextureMap;
pub use vertex::VertexIndex;

#[derive(Clone)]
#[allow(unused)]
pub struct Geometry<T: VertexInteger> {
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary<T>>,
    semantics: Option<SemanticMaterialMap<T>>,
    template_boundaries: Option<usize>,
    template_transformation_matrix: Option<[f64; 16]>,
}

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum GeometryType {
    MultiPoint,
    MultiLineString,
    MultiSurface,
    CompositeSurface,
    Solid,
    MultiSolid,
    CompositeSolid,
    GeometryInstance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LoD {
    LoD0,
    LoD0_0,
    LoD0_1,
    LoD0_2,
    LoD0_3,
    LoD1,
    LoD1_0,
    LoD1_1,
    LoD1_2,
    LoD1_3,
    LoD2,
    LoD2_0,
    LoD2_1,
    LoD2_2,
    LoD2_3,
    LoD3,
    LoD3_0,
    LoD3_1,
    LoD3_2,
    LoD3_3,
}

#[cfg(test)]
mod tests {
    use super::*;

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
