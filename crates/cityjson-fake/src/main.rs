mod cli;

use cli::CJFakeConfig;
use clap::Parser;
use serde_cityjson::v1_1::CityModel;

use cjfake::CityModelBuilder;

fn main() {
    let config = CJFakeConfig::parse();
    let cm: CityModel = CityModelBuilder::default().build();
    let cj_str = serde_json::to_string::<CityModel>(&cm).unwrap();
    println!("{}", cj_str);
}
