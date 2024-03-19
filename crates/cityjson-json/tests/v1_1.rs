use std::fs::File;
use std::io::Read;
use serde_cityjson::v1_1::*;

#[test]
fn objects() -> Result<(), String> {
    let cityjson_path = "resources/data/downloaded/30gz1_04.json";
    let mut file = File::open(cityjson_path).map_err(|e| e.to_string())?;
    let mut cityjson_json = String::new();
    file.read_to_string(&mut cityjson_json)
        .map_err(|e| e.to_string())?;
    let cm: CityModel = serde_json::from_str(&cityjson_json).map_err(|e| e.to_string())?;
    println!("{:?}", &cm.version);
    Ok(())
}