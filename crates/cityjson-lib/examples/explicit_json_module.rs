use cityjson_lib::{CityJSONVersion, json};

fn main() -> cityjson_lib::Result<()> {
    let bytes = std::fs::read("tests/data/v2_0/minimal.city.json")?;

    let probe = json::probe(&bytes)?;
    assert_eq!(probe.version(), Some(CityJSONVersion::V2_0));

    let model = json::from_slice(&bytes)?;
    println!("loaded {} CityObjects", model.cityobjects().len());

    Ok(())
}
