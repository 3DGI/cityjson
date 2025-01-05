use std::path::PathBuf;
use once_cell::sync::Lazy;

use common::*;
use serde_cityjson::{from_str, CityJSON};

mod common;

static DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    cargo_workspace_directory()
        .unwrap()
        .join("tests")
        .join("data")
});

static DATA_DIR_V1_1: Lazy<PathBuf> = Lazy::new(|| {
    cargo_workspace_directory()
        .unwrap()
        .join("tests")
        .join("data")
        .join("v1_1")
});


#[test]
fn from_str_cityjson_v1_1() {
    let json_input = read_to_string(DATA_DIR_V1_1.join("cityjson_dummy_complete.city.json"));
    let cityjson = from_str(json_input.as_str()).unwrap();
    match cityjson {
        CityJSON::V1_1(cm) => {
            assert_eq!(cm.version.unwrap(), serde_cityjson::v1_1::CityJSONVersion::V1_1);
        }
        CityJSON::V2_0(_) => {panic!("CityJSON should be v1.1");}
    }
}
