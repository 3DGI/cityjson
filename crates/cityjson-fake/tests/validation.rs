#[path = "common_lib/mod.rs"]
mod common_lib;

use cjfake::prelude::*;

fn build_and_validate(config: CJFakeConfig, test_name: &str) {
    let json = CityModelBuilder::<u32, OwnedStringStorage>::new(config, Some(42))
        .metadata(None)
        .vertices()
        .materials(None)
        .textures(None)
        .attributes(None)
        .cityobjects()
        .build_string()
        .expect("serialization failed");
    crate::common_lib::validate(&json, test_name);
}

fn geom_config(gt: GeometryType) -> CJFakeConfig {
    CJFakeConfig {
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![gt]),
            ..Default::default()
        },
        cityobjects: CityObjectConfig {
            min_cityobjects: 2,
            max_cityobjects: 2,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn validate_default() {
    build_and_validate(CJFakeConfig::default(), "validate_default");
}

#[test]
fn validate_multipoint() {
    build_and_validate(geom_config(GeometryType::MultiPoint), "validate_multipoint");
}

#[test]
fn validate_multilinestring() {
    build_and_validate(
        geom_config(GeometryType::MultiLineString),
        "validate_multilinestring",
    );
}

#[test]
fn validate_multisurface() {
    build_and_validate(
        geom_config(GeometryType::MultiSurface),
        "validate_multisurface",
    );
}

#[test]
fn validate_solid() {
    build_and_validate(geom_config(GeometryType::Solid), "validate_solid");
}

#[test]
fn validate_multisolid() {
    build_and_validate(geom_config(GeometryType::MultiSolid), "validate_multisolid");
}

#[test]
fn validate_compositesurface() {
    build_and_validate(
        geom_config(GeometryType::CompositeSurface),
        "validate_compositesurface",
    );
}

#[test]
fn validate_compositesolid() {
    build_and_validate(
        geom_config(GeometryType::CompositeSolid),
        "validate_compositesolid",
    );
}

#[test]
fn validate_hierarchy() {
    let config = CJFakeConfig {
        cityobjects: CityObjectConfig {
            min_cityobjects: 4,
            max_cityobjects: 4,
            cityobject_hierarchy: true,
            allowed_types_cityobject: Some(vec![CityObjectType::Building]),
            ..Default::default()
        },
        ..Default::default()
    };
    build_and_validate(config, "validate_hierarchy");
}

#[test]
fn validate_templates() {
    let config = CJFakeConfig {
        cityobjects: CityObjectConfig {
            min_cityobjects: 2,
            max_cityobjects: 2,
            ..Default::default()
        },
        templates: TemplateConfig {
            use_templates: true,
            ..Default::default()
        },
        ..Default::default()
    };
    build_and_validate(config, "validate_templates");
}

#[test]
fn validate_all_features() {
    let config = CJFakeConfig {
        cityobjects: CityObjectConfig {
            min_cityobjects: 3,
            max_cityobjects: 3,
            cityobject_hierarchy: true,
            allowed_types_cityobject: Some(vec![CityObjectType::Building]),
            ..Default::default()
        },
        materials: MaterialConfig {
            min_materials: 2,
            max_materials: 3,
            nr_themes_materials: 2,
            ..Default::default()
        },
        textures: TextureConfig {
            min_textures: 1,
            max_textures: 2,
            nr_themes_textures: 2,
            ..Default::default()
        },
        ..Default::default()
    };
    build_and_validate(config, "validate_all_features");
}

#[test]
fn deterministic_output() {
    let config = CJFakeConfig {
        cityobjects: CityObjectConfig {
            min_cityobjects: 2,
            max_cityobjects: 2,
            ..Default::default()
        },
        ..Default::default()
    };
    let json1 = CityModelBuilder::<u32, OwnedStringStorage>::new(config.clone(), Some(42))
        .metadata(None)
        .vertices()
        .materials(None)
        .textures(None)
        .attributes(None)
        .cityobjects()
        .build_string()
        .unwrap();
    let json2 = CityModelBuilder::<u32, OwnedStringStorage>::new(config, Some(42))
        .metadata(None)
        .vertices()
        .materials(None)
        .textures(None)
        .attributes(None)
        .cityobjects()
        .build_string()
        .unwrap();
    assert_eq!(json1, json2);
}
