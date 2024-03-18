use std::fs::File;
use std::io::Read;
use serde_cityjson::v2_0::*;

#[test]
fn objects() -> Result<(), String> {
    let cityjson_path = "tests/data/objects.city.json";
    let mut file = File::open(cityjson_path).map_err(|e| e.to_string())?;
    let mut cityjson_json = String::new();
    file.read_to_string(&mut cityjson_json)
        .map_err(|e| e.to_string())?;
    let cm: CityModel = serde_json::from_str(&cityjson_json).map_err(|e| e.to_string())?;
    println!("{:?}", &cm.version);
    Ok(())
}

#[test]
fn geometries() -> Result<(), String> {
    let cityjson_path = "tests/data/geometries.city.json";
    let mut file = File::open(cityjson_path).map_err(|e| e.to_string())?;
    let mut cityjson_json = String::new();
    file.read_to_string(&mut cityjson_json)
        .map_err(|e| e.to_string())?;
    let cm: CityModel = serde_json::from_str(&cityjson_json).map_err(|e| e.to_string())?;
    println!("{:?}", &cm.version);
    Ok(())
}

#[test]
fn attributes() -> Result<(), String> {
    let cityjson_path = "tests/data/attributes.city.json";
    let mut file = File::open(cityjson_path).map_err(|e| e.to_string())?;
    let mut cityjson_json = Vec::new();
    file.read_to_end(&mut cityjson_json)
        .map_err(|e| e.to_string())?;
    let cm: CityModel = serde_json::from_slice(&cityjson_json).map_err(|e| e.to_string())?;
    println!("{:?}", &cm.version);
    Ok(())
}

#[test]
fn borrow_value() -> Result<(), String> {
    let cityjson_path = "tests/data/attributes.city.json";
    let mut file = File::open(cityjson_path).map_err(|e| e.to_string())?;
    let mut cityjson_json = Vec::new();
    file.read_to_end(&mut cityjson_json)
        .map_err(|e| e.to_string())?;
    let cm: serde_json_borrow::Value = serde_json::from_slice(&cityjson_json).map_err(|e| e.to_string())?;
    println!("{:?}", &cm.get("version"));
    Ok(())
}