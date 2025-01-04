mod common;

use cjfake::{CJFakeConfig, CityModelBuilder, MaterialBuilder, MetadataBuilder, TextureBuilder};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;
use proptest::test_runner::FileFailurePersistence;
use serde_cityjson::v1_1::{CityModel, CityObjectType, GeometryType};

/// Can we fake a valid CityJSON with the default parameters?
#[test]
fn default() {
    let cm: CityModel = CityModelBuilder::default().build();
    assert_eq!(cm.cityobjects.len(), 1);
    let cj_str = serde_json::to_string::<CityModel>(&cm).unwrap();
    common::validate(&cj_str, "api_default");
}

/// Can we fake a valid CityJSON with a seed?
#[test]
fn seed() {
    let cm: CityModel = CityModelBuilder::new(CJFakeConfig::default(), Some(10))
        .metadata(None)
        .vertices()
        .materials(None)
        .textures(None)
        .attributes()
        .cityobjects()
        .build();
    assert_eq!(cm.cityobjects.len(), 1);
    let cj_str = serde_json::to_string::<CityModel>(&cm).unwrap();
    common::validate(&cj_str, "api_seed");
}

/// Can we fake a valid CityJSON with custom builders?
#[test]
fn custom_builders() {
    // Create custom metadata
    let metadata_builder = MetadataBuilder::new()
        .geographical_extent()
        .identifier()
        .point_of_contact()
        .reference_system();

    // Create custom material with specific properties
    let material_builder = MaterialBuilder::new()
        .name()
        .diffuse_color()
        .shininess()
        .transparency()
        .smooth();

    // Create custom texture with specific properties
    let texture_builder = TextureBuilder::new()
        .image_type()
        .image()
        .wrap_mode()
        .texture_type();

    // Configure model generation
    let config = CJFakeConfig {
        min_cityobjects: 2,
        max_cityobjects: 2,
        min_materials: 2,
        max_materials: 2,
        min_textures: 1,
        max_textures: 1,
        nr_themes_materials: 2,
        nr_themes_textures: 1,
        use_templates: true,
        ..Default::default()
    };

    // Build model with custom components
    let cm: CityModel = CityModelBuilder::new(config, None)
        .metadata(Some(metadata_builder))
        .vertices()
        .materials(Some(material_builder))
        .textures(Some(texture_builder))
        .attributes()
        .cityobjects()
        .build();

    // Additional specific assertions
    assert!(cm.metadata.is_some());

    assert_eq!(cm.cityobjects.len(), 2);

    if let Some(ref appearance) = cm.appearance {
        // Check materials were generated
        if let Some(ref materials) = appearance.materials {
            assert_eq!(materials.len(), 2);
            // Verify material properties were set
            assert!(materials[0].name.len() > 0);
            assert!(materials[0].diffuse_color.is_some());
            assert!(materials[0].shininess.is_some());
            assert!(materials[0].transparency.is_some());
            assert!(materials[0].is_smooth.is_some());
        }

        // Check textures were generated
        if let Some(ref textures) = appearance.textures {
            assert_eq!(textures.len(), 1);
            // Verify texture properties were set
            assert!(textures[0].image.len() > 0);
            assert!(textures[0].wrap_mode.is_some());
            assert!(textures[0].texture_type.is_some());
        }
    }

    // Check that themes were generated
    assert!(&cm
        .appearance
        .as_ref()
        .unwrap()
        .default_theme_material
        .is_some());

    // Validate the generated model
    let cj_str = serde_json::to_string::<CityModel>(&cm).unwrap();
    common::validate(&cj_str, "api_custom_builders");
}

fn city_object_type_strategy() -> impl Strategy<Value=Vec<CityObjectType>> {
    // Create all possible CityObjectTypes except Extension
    let types = vec![
        CityObjectType::Bridge,
        CityObjectType::BridgePart,
        CityObjectType::BridgeInstallation,
        CityObjectType::BridgeConstructiveElement,
        CityObjectType::BridgeRoom,
        CityObjectType::BridgeFurniture,
        CityObjectType::Building,
        CityObjectType::BuildingPart,
        CityObjectType::BuildingInstallation,
        CityObjectType::BuildingConstructiveElement,
        CityObjectType::BuildingFurniture,
        CityObjectType::BuildingStorey,
        CityObjectType::BuildingRoom,
        CityObjectType::BuildingUnit,
        CityObjectType::CityFurniture,
        CityObjectType::CityObjectGroup,
        CityObjectType::Default,
        CityObjectType::LandUse,
        CityObjectType::OtherConstruction,
        CityObjectType::PlantCover,
        CityObjectType::SolitaryVegetationObject,
        CityObjectType::TINRelief,
        CityObjectType::WaterBody,
        CityObjectType::Road,
        CityObjectType::Railway,
        CityObjectType::Waterway,
        CityObjectType::TransportSquare,
        CityObjectType::Tunnel,
        CityObjectType::TunnelPart,
        CityObjectType::TunnelInstallation,
        CityObjectType::TunnelConstructiveElement,
        CityObjectType::TunnelHollowSpace,
        CityObjectType::TunnelFurniture,
    ];
    let nr_types = types.len();
    vec(select(types), 1..nr_types)
}

fn geometry_type_strategy() -> impl Strategy<Value=Vec<GeometryType>> {
    let types = vec![
        GeometryType::MultiPoint,
        GeometryType::MultiLineString,
        GeometryType::MultiSurface,
        GeometryType::CompositeSurface,
        GeometryType::Solid,
        GeometryType::MultiSolid,
        GeometryType::CompositeSolid,
        GeometryType::GeometryInstance,
    ];
    let nr_types = types.len();
    vec(select(types), 1..nr_types)
}

proptest! {
    #![proptest_config(ProptestConfig{
        cases: 100,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("proptest-regressions"))),
        ..Default::default()
    })]

    /// Can we fake valid CityJSON with various parameters?
    #[test]
    fn fuzz_config(
        allowed_types_cityobject in city_object_type_strategy(),
        allowed_types_geometry in geometry_type_strategy(),
        cityobject_hierarchy in any::<bool>(),
        min_coordinate in i64::MIN..=0i64,
        max_coordinate in 0i64..=i64::MAX,
        nr_themes_materials in 1u32..3,
        nr_themes_textures in 1u32..3,
        max_vertices_texture in 4u32..10,
        use_templates in any::<bool>(),
    ) {
        let min_cityobjects = 2u32;
        let max_cityobjects = 2u32;
        let config = CJFakeConfig {
            allowed_types_cityobject: Some(allowed_types_cityobject.clone()),
            allowed_types_geometry: Some(allowed_types_geometry.clone()),
            cityobject_hierarchy,
            min_coordinate,
            max_coordinate,
            nr_themes_materials,
            nr_themes_textures,
            max_vertices_texture,
            use_templates,
            min_cityobjects,
            max_cityobjects,
            ..Default::default()
        };

        let cm: CityModel = CityModelBuilder::new(config, None).metadata(None)
            .vertices()
            .materials(None)
            .textures(None)
            .attributes()
            .cityobjects().build();
        let cj_str = serde_json::to_string::<CityModel>(&cm).unwrap();
        common::validate(&cj_str, "api_fuzz");

        // ----- Assertions -----
        // 1. Vertex coordinate range check
        for vertex in &cm.vertices {
            assert_eq!(vertex.len(), 3);
            for coord in vertex.iter() {
                let coord = *coord as i64; // Ensure comparison works with i64 bounds
                assert!(coord >= min_coordinate && coord <= max_coordinate, "Vertex coordinate out of configured range");
            }
        }

        // 2. CityObject type is allowed
        for (id, cityobject) in cm.cityobjects.iter() {
            assert!(
                allowed_types_cityobject.contains(&cityobject.type_co),
                "CityObject (ID: {}) type {:?} is not allowed",
                id,
                cityobject.type_co
            );

            if let Some(ref geometry) = cityobject.geometry {
                for geom in geometry {
                    assert!(
                    allowed_types_geometry.contains(&geom.type_),
                    "Geometry type {:?} is not allowed",
                    geom.type_
                );
                }
            }
        }

        // 4. Verify themes (materials and textures)
        if let Some(appearance) = cm.appearance {
            // Validate the number of materials themes
            if let Some(materials) = appearance.materials {
                assert!(materials.len() <= nr_themes_materials as usize);
            }

            // Validate the number of texture themes
            if let Some(textures) = appearance.textures {
                assert!(textures.len() <= nr_themes_textures as usize);
            }
        }

        // Ensure CityModel contains at least the configured number of city objects
        assert!(cm.cityobjects.len() >= min_cityobjects as usize && cm.cityobjects.len() <= max_cityobjects as usize, "CityObjects count is out of configured range");
    }
}
