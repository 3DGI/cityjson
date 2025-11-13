mod common;

use cjfake::prelude::*;

/// Can we fake a valid CityJSON with the default parameters?
#[test]
fn default() {
    let cm: CityModel<u32, ResourceId32, OwnedStringStorage> = CityModelBuilder::default().build();
    assert_eq!(cm.cityobjects().len(), 1);
}

/// Can we fake a valid CityJSON with a seed?
#[test]
fn seed() {
    let cm: CityModel<u32, ResourceId32, OwnedStringStorage> =
        CityModelBuilder::new(CJFakeConfig::default(), Some(10))
            .metadata(None)
            .vertices()
            .materials(None)
            .textures(None)
            .attributes(None)
            .cityobjects()
            .build();
    assert_eq!(cm.cityobjects().len(), 1);
}

/// Can we fake a valid CityJSON with custom builders?
#[test]
fn custom_builders() {
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
        .metadata(None)
        .vertices()
        .materials(None)
        .textures(None)
        .attributes(None)
        .cityobjects()
        .build();

    // Additional specific assertions
    assert_eq!(cm.cityobjects().len(), 2);

    // Check materials were generated
    let materials: Vec<_> = cm.iter_materials().collect();
    assert_eq!(materials.len(), 2);
    // Verify material properties were set on at least one material
    let (_, first_material) = materials[0];
    assert!(!first_material.name().is_empty());
    // Some properties may be randomly set, so we just check that the material exists

    // Check textures were generated
    let textures: Vec<_> = cm.iter_textures().collect();
    assert_eq!(textures.len(), 1);
    // Verify texture properties were set
    let (_, first_texture) = textures[0];
    assert!(!first_texture.image().is_empty());

    // Check that themes were generated
    assert!(cm.default_theme_material().is_some());

    // TODO: Validate the generated model
    // The new cityjson-rs API doesn't have serde serialization yet
}
