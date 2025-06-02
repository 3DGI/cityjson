use clap::Parser;
use cityjson::v2_0::CityModel;
use cityjson::prelude::*;

use cjfake::{CJFakeConfig, CityModelBuilder};

fn main() {
    let config = CJFakeConfig::parse();
    // Use u32 vertex refs, ResourceId32, and OwnedStringStorage as defaults
    let cm: CityModel<u32, ResourceId32, OwnedStringStorage> = CityModelBuilder::new(config, None)
        .metadata(None)
        .vertices()
        .materials(None)
        .textures(None)
        .attributes()
        .cityobjects()
        .build();
    let cj_str = serde_json::to_string(&cm).unwrap();
    println!("{}", cj_str);
}
