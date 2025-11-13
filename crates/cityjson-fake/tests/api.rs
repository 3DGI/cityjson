mod common;

use cjfake::prelude::*;

/// Can we fake a valid CityJSON with the default parameters?
#[test]
#[ignore] // TODO: Re-enable once cityobjects generation is implemented
fn default() {
    let cm: CityModel<u32, ResourceId32, OwnedStringStorage> = CityModelBuilder::default().build();
    assert_eq!(cm.cityobjects().len(), 1);
}

/// Can we fake a valid CityJSON with a seed?
#[test]
#[ignore] // TODO: Re-enable once cityobjects generation is implemented
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
