use clap::Parser;
use serde_cityjson::v1_1::CityModel;

use cjfake::{CJFakeConfig, CityModelBuilder};

fn main() {
    let config = CJFakeConfig::parse();
    let cm: CityModel = CityModelBuilder::new(config, None)
        .metadata(None)
        .vertices()
        .materials(None)
        .textures(None)
        .attributes()
        .cityobjects()
        .build();
    let cj_str = serde_json::to_string::<CityModel>(&cm).unwrap();
    println!("{}", cj_str);
}
