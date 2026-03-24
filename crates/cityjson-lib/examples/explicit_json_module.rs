use cjlib::{CityJSONVersion, json};

fn main() -> cjlib::Result<()> {
    let bytes = std::fs::read("tests/data/v2_0/minimal.city.json")?;

    assert_eq!(json::detect_version(&bytes)?, CityJSONVersion::V2_0);

    let model = json::from_slice(&bytes)?;
    println!("loaded {} CityObjects", model.cityobjects().len());

    Ok(())
}
