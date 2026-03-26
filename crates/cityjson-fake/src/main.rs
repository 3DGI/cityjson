use cjfake::prelude::*;
use clap::Parser;

fn main() {
    let config = CJFakeConfig::parse();
    let json = CityModelBuilder::<u32, OwnedStringStorage>::new(config, None)
        .metadata(None)
        .vertices()
        .materials(None)
        .textures(None)
        .attributes(None)
        .cityobjects()
        .build_string()
        .expect("serialization failed");
    println!("{json}");
}
