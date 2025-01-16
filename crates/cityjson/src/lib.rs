use std::fmt::Debug;

mod boundary;
mod reference_mapping;
mod semantics;
pub mod vertex;

pub use boundary::Boundary;
pub use reference_mapping::SemanticMaterialMapping;
pub use semantics::SemanticReference;
pub use vertex::{Coordinate, Index, Vertex, VertexCoordinate, VertexIndex};

#[derive(Clone)]
pub struct Geometry<V: Vertex, S: SemanticReference> {
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary<V>>,
    semantics_surfaces: Option<Vec<S>>,
    semantics_values: Option<SemanticMaterialMapping>,
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
    use crate::vertex::VertexCoordinate;
    use std::collections::HashMap;
    use vertex::Index;

    // Local semantic - contains the actual semantic data
    #[derive(Clone, Debug)]
    struct LocalSemantic {
        type_: String,
        attributes: HashMap<String, String>,
    }

    impl SemanticReference for LocalSemantic {
        fn index(&self) -> Option<u32> {
            None // Local semantics don't have indices
        }
    }

    // Global semantic reference - contains just an index
    #[derive(Clone, Debug)]
    struct GlobalSemantic(u32);

    impl SemanticReference for GlobalSemantic {
        fn index(&self) -> Option<u32> {
            Some(self.0)
        }
    }

    #[test]
    fn test_geometry_with_local_semantics() {
        let mut local_semantic = LocalSemantic {
            type_: "RoofSurface".to_string(),
            attributes: HashMap::new(),
        };
        local_semantic
            .attributes
            .insert("material".to_string(), "tiles".to_string());

        let geom: Geometry<VertexCoordinate, LocalSemantic> = Geometry {
            type_geometry: GeometryType::MultiSurface,
            lod: Some(LoD::LoD1),
            boundaries: None,
            semantics_surfaces: Some(vec![local_semantic]),
            semantics_values: Some(SemanticMaterialMapping {
                surfaces: vec![Some(0)],
                ..Default::default()
            }),
        };

        // Verify semantic reference is stored locally
        assert!(geom.semantics_surfaces.unwrap()[0].index().is_none());
    }

    #[test]
    fn test_geometry_with_global_semantics() {
        // Simulate a global semantic pool
        let _semantic_pool = vec![LocalSemantic {
            type_: "RoofSurface".to_string(),
            attributes: HashMap::new(),
        }];

        let geom: Geometry<VertexCoordinate, GlobalSemantic> = Geometry {
            type_geometry: GeometryType::MultiSurface,
            lod: Some(LoD::LoD1),
            boundaries: None,
            semantics_surfaces: Some(vec![GlobalSemantic(0)]),
            semantics_values: Some(SemanticMaterialMapping {
                surfaces: vec![Some(0)],
                ..Default::default()
            }),
        };

        // Verify semantic reference points to global pool
        assert_eq!(geom.semantics_surfaces.unwrap()[0].index(), Some(0));
    }

    // Example showing how a library might use global semantics
    struct CityModel {
        semantic_pool: Vec<LocalSemantic>,
        geometries: Vec<Geometry<VertexCoordinate, GlobalSemantic>>,
    }

    impl CityModel {
        fn new() -> Self {
            Self {
                semantic_pool: Vec::new(),
                geometries: Vec::new(),
            }
        }

        fn add_semantic(&mut self, semantic: LocalSemantic) -> GlobalSemantic {
            let index = self.semantic_pool.len() as u32;
            self.semantic_pool.push(semantic);
            GlobalSemantic(index)
        }
    }

    #[test]
    fn test_citymodel_with_global_semantics() {
        let mut model = CityModel::new();

        // Add semantic to pool and get reference
        let semantic_ref = model.add_semantic(LocalSemantic {
            type_: "RoofSurface".to_string(),
            attributes: HashMap::new(),
        });

        // Create geometry using semantic reference
        let geom = Geometry {
            type_geometry: GeometryType::MultiSurface,
            lod: Some(LoD::LoD1),
            boundaries: None,
            semantics_surfaces: Some(vec![semantic_ref]),
            semantics_values: Some(SemanticMaterialMapping {
                surfaces: vec![Some(0)],
                ..Default::default()
            }),
        };

        model.geometries.push(geom);

        // Verify semantic reference
        assert_eq!(
            model.geometries[0].semantics_surfaces.as_ref().unwrap()[0].index(),
            Some(0)
        );
        assert_eq!(model.semantic_pool[0].type_, "RoofSurface");
    }

    #[test]
    fn test_geometry_creation() {
        let boundary: Boundary<VertexCoordinate> = Boundary {
            vertices: vec![],
            rings: vec![],
            surfaces: vec![],
            shells: vec![],
            solids: vec![],
        };

        let geom: Geometry<VertexCoordinate, LocalSemantic> = Geometry {
            type_geometry: GeometryType::MultiSurface,
            lod: Some(LoD::LoD1),
            boundaries: Some(boundary),
            semantics_values: None,
            semantics_surfaces: None,
        };

        assert!(matches!(geom.type_geometry, GeometryType::MultiSurface));
        assert!(matches!(geom.lod, Some(LoD::LoD1)));
    }

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
