mod common;

use cjfake::prelude::*;
use proptest::prelude::*;

/// Can we fake a valid CityJSON with the default parameters?
#[test]
fn default() {
    let cm: CityModel<u32, ResourceId32, OwnedStringStorage> = CityModelBuilder::default().build();
    assert_eq!(cm.cityobjects().len(), 1);
}

/// Can we fake a valid CityJSON with a seed?
#[test]
fn seed() {
    let cm: CityModel<u32, ResourceId32, OwnedStringStorage> = CityModelBuilder::new(CJFakeConfig::default(), Some(10))
        .metadata(None)
        .vertices(None)
        .materials(None)
        .textures(None)
        .attributes(None)
        .cityobjects()
        .build();
    assert_eq!(cm.cityobjects().len(), 1);
}
