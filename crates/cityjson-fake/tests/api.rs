mod common;

use cjfake::{CJFakeConfig, CityModelBuilder};
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
