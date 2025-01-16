use std::fmt::Debug;

mod boundary;
mod materials;
mod reference_mapping;
mod semantics;
mod textures;
pub mod vertex;

pub use boundary::Boundary;
pub use materials::MaterialReference;
pub use reference_mapping::{SemanticMaterialMapping, TextureMapping};
pub use semantics::SemanticReference;
pub use textures::TextureReference;
pub use vertex::{Coordinate, Index, Vertex, VertexCoordinate, VertexIndex};

#[derive(Clone)]
pub struct Geometry<V: Vertex, S: SemanticReference, M: MaterialReference, T: TextureReference> {
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary<V>>,
    semantics_surfaces: Option<Vec<S>>,
    semantics_values: Option<SemanticMaterialMapping>,
    material_surfaces: Option<Vec<M>>,
    material_values: Option<SemanticMaterialMapping>,
    texture_surfaces: Option<Vec<T>>,
    texture_values: Option<TextureMapping>,
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

    // Local semantic
    #[derive(Clone, Debug)]
    struct LocalSemantic {
        type_: String,
        attributes: HashMap<String, String>,
    }

    impl SemanticReference for LocalSemantic {
        fn index(&self) -> Option<u32> {
            None
        }
    }

    // Global semantic
    #[derive(Clone, Debug)]
    struct GlobalSemantic(u32);

    impl SemanticReference for GlobalSemantic {
        fn index(&self) -> Option<u32> {
            Some(self.0)
        }
    }

    // Local material
    #[derive(Clone, Debug)]
    struct LocalMaterial {
        name: String,
        ambient_intensity: Option<f32>,
        diffuse_color: Option<[f32; 3]>,
    }

    impl MaterialReference for LocalMaterial {
        fn index(&self) -> Option<u32> {
            None
        }
    }

    // Global material
    #[derive(Clone, Debug)]
    struct GlobalMaterial(u32);

    impl MaterialReference for GlobalMaterial {
        fn index(&self) -> Option<u32> {
            Some(self.0)
        }
    }

    // Local texture
    #[derive(Clone, Debug)]
    struct LocalTexture {
        image: String,
        wrap_mode: String,
    }

    impl TextureReference for LocalTexture {
        fn index(&self) -> Option<u32> {
            None
        }
    }

    // Global texture
    #[derive(Clone, Debug)]
    struct GlobalTexture(u32);

    impl TextureReference for GlobalTexture {
        fn index(&self) -> Option<u32> {
            Some(self.0)
        }
    }

    #[test]
    fn test_geometry_with_local_references() {
        let geom: Geometry<VertexCoordinate, LocalSemantic, LocalMaterial, LocalTexture> =
            Geometry {
                type_geometry: GeometryType::MultiSurface,
                lod: Some(LoD::LoD1),
                boundaries: None,
                // Semantics
                semantics_surfaces: Some(vec![LocalSemantic {
                    type_: "RoofSurface".to_string(),
                    attributes: HashMap::new(),
                }]),
                semantics_values: Some(SemanticMaterialMapping {
                    surfaces: vec![Some(0)],
                    ..Default::default()
                }),
                // Materials
                material_surfaces: Some(vec![LocalMaterial {
                    name: "red_tiles".to_string(),
                    ambient_intensity: Some(0.5),
                    diffuse_color: Some([1.0, 0.0, 0.0]),
                }]),
                material_values: Some(SemanticMaterialMapping {
                    surfaces: vec![Some(0)],
                    ..Default::default()
                }),
                // Textures
                texture_surfaces: Some(vec![LocalTexture {
                    image: "roof_texture.png".to_string(),
                    wrap_mode: "repeat".to_string(),
                }]),
                texture_values: Some(TextureMapping::default()),
            };

        // Verify local references
        assert!(geom.semantics_surfaces.unwrap()[0].index().is_none());
        assert!(geom.material_surfaces.unwrap()[0].index().is_none());
        assert!(geom.texture_surfaces.unwrap()[0].index().is_none());
    }

    // Resource pool example
    struct ResourcePool {
        semantics: Vec<LocalSemantic>,
        materials: Vec<LocalMaterial>,
        textures: Vec<LocalTexture>,
    }

    impl ResourcePool {
        fn new() -> Self {
            Self {
                semantics: Vec::new(),
                materials: Vec::new(),
                textures: Vec::new(),
            }
        }

        fn add_semantic(&mut self, semantic: LocalSemantic) -> GlobalSemantic {
            let index = self.semantics.len() as u32;
            self.semantics.push(semantic);
            GlobalSemantic(index)
        }

        fn add_material(&mut self, material: LocalMaterial) -> GlobalMaterial {
            let index = self.materials.len() as u32;
            self.materials.push(material);
            GlobalMaterial(index)
        }

        fn add_texture(&mut self, texture: LocalTexture) -> GlobalTexture {
            let index = self.textures.len() as u32;
            self.textures.push(texture);
            GlobalTexture(index)
        }
    }

    #[test]
    fn test_geometry_with_resource_pool() {
        let mut pool = ResourcePool::new();

        // Add resources to pool
        let semantic_ref = pool.add_semantic(LocalSemantic {
            type_: "RoofSurface".to_string(),
            attributes: HashMap::new(),
        });

        let material_ref = pool.add_material(LocalMaterial {
            name: "red_tiles".to_string(),
            ambient_intensity: Some(0.5),
            diffuse_color: Some([1.0, 0.0, 0.0]),
        });

        let texture_ref = pool.add_texture(LocalTexture {
            image: "roof_texture.png".to_string(),
            wrap_mode: "repeat".to_string(),
        });

        // Create geometry with global references
        let geom: Geometry<VertexCoordinate, GlobalSemantic, GlobalMaterial, GlobalTexture> =
            Geometry {
                type_geometry: GeometryType::MultiSurface,
                lod: Some(LoD::LoD1),
                boundaries: None,
                semantics_surfaces: Some(vec![semantic_ref]),
                semantics_values: Some(SemanticMaterialMapping {
                    surfaces: vec![Some(0)],
                    ..Default::default()
                }),
                material_surfaces: Some(vec![material_ref]),
                material_values: Some(SemanticMaterialMapping {
                    surfaces: vec![Some(0)],
                    ..Default::default()
                }),
                texture_surfaces: Some(vec![texture_ref]),
                texture_values: Some(TextureMapping::default()),
            };

        // Verify global references
        assert_eq!(geom.semantics_surfaces.unwrap()[0].index(), Some(0));
        assert_eq!(geom.material_surfaces.unwrap()[0].index(), Some(0));
        assert_eq!(geom.texture_surfaces.unwrap()[0].index(), Some(0));

        // Verify pool contents
        assert_eq!(pool.semantics[0].type_, "RoofSurface");
        assert_eq!(pool.materials[0].name, "red_tiles");
        assert_eq!(pool.textures[0].image, "roof_texture.png");
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

        let geom: Geometry<VertexCoordinate, LocalSemantic, LocalMaterial, LocalTexture> =
            Geometry {
                type_geometry: GeometryType::MultiSurface,
                lod: Some(LoD::LoD1),
                boundaries: None,
                semantics_surfaces: Some(vec![local_semantic]),
                semantics_values: Some(SemanticMaterialMapping {
                    surfaces: vec![Some(0)],
                    ..Default::default()
                }),
                material_surfaces: None,
                material_values: None,
                texture_surfaces: None,
                texture_values: None,
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

        let geom: Geometry<VertexCoordinate, GlobalSemantic, GlobalMaterial, GlobalTexture> =
            Geometry {
                type_geometry: GeometryType::MultiSurface,
                lod: Some(LoD::LoD1),
                boundaries: None,
                semantics_surfaces: Some(vec![GlobalSemantic(0)]),
                semantics_values: Some(SemanticMaterialMapping {
                    surfaces: vec![Some(0)],
                    ..Default::default()
                }),
                material_surfaces: None,
                material_values: None,
                texture_surfaces: None,
                texture_values: None,
            };

        // Verify semantic reference points to global pool
        assert_eq!(geom.semantics_surfaces.unwrap()[0].index(), Some(0));
    }

    // Example showing how a library might use global semantics
    struct CityModel {
        semantic_pool: Vec<LocalSemantic>,
        geometries: Vec<Geometry<VertexCoordinate, GlobalSemantic, GlobalMaterial, GlobalTexture>>,
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
            material_surfaces: None,
            material_values: None,
            texture_surfaces: None,
            texture_values: None,
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

        let geom: Geometry<VertexCoordinate, LocalSemantic, LocalMaterial, LocalTexture> =
            Geometry {
                type_geometry: GeometryType::MultiSurface,
                lod: Some(LoD::LoD1),
                boundaries: Some(boundary),
                semantics_values: None,
                material_surfaces: None,
                material_values: None,
                texture_surfaces: None,
                semantics_surfaces: None,
                texture_values: None,
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
