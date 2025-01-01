mod common;

use cjfake::{CJFakeConfig, CityModelBuilder, MaterialBuilder, MetadataBuilder, TextureBuilder};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;
use serde_cityjson::v1_1::{CityModel, CityObjectType, GeometryType};

/// Can we fake a valid CityJSON with the default parameters?
#[test]
fn default() {
    let cm: CityModel = CityModelBuilder::default().build();
    let cj_str = serde_json::to_string::<CityModel>(&cm).unwrap();
    common::validate(&cj_str, "api_default");
}

/// Can we fake a valid CityJSON with a seed?
#[test]
fn seed() {
    let cm: CityModel = CityModelBuilder::new(CJFakeConfig::default(), Some(10)).build();
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
        max_cityobjects: 3,
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

    // Validate the generated model
    let cj_str = serde_json::to_string::<CityModel>(&cm).unwrap();
    common::validate(&cj_str, "api_custom_builders");

    // Additional specific assertions
    assert!(cm.metadata.is_some());

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
    assert!(cm.appearance.unwrap().default_theme_material.is_some());
}

fn city_object_type_strategy() -> impl Strategy<Value = Vec<CityObjectType>> {
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

fn geometry_type_strategy() -> impl Strategy<Value = Vec<GeometryType>> {
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
    #![proptest_config(ProptestConfig::with_cases(100))]

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
        let config = CJFakeConfig {
            allowed_types_cityobject: Some(allowed_types_cityobject),
            allowed_types_geometry: Some(allowed_types_geometry),
            cityobject_hierarchy,
            min_coordinate,
            max_coordinate,
            nr_themes_materials,
            nr_themes_textures,
            max_vertices_texture,
            use_templates,
            min_cityobjects: 2,
            max_cityobjects: 2,
            ..Default::default()
        };

        let cm: CityModel = CityModelBuilder::new(config, None).build();
        let cj_str = serde_json::to_string::<CityModel>(&cm).unwrap();
        common::validate(&cj_str, "api_fuzz");
    }
}
