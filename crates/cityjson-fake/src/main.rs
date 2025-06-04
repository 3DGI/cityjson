use clap::Parser;
use cjfake::prelude::*;

fn main() {
    let config = CJFakeConfig::parse();
    // Use u32 vertex refs, ResourceId32, and OwnedStringStorage as defaults
    let cm: CityModel<u32, ResourceId32, OwnedStringStorage> = CityModelBuilder::new(config, None)
        .metadata(None)
        .vertices(None)
        .materials(None)
        .textures(None)
        .attributes(None)
        .cityobjects()
        .build();
    // let cj_str = serde_json::to_string(&cm).unwrap();
    println!("serialization to string not implemented yet");
}
