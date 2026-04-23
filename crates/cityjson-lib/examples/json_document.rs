use cityjson_lib::json;

fn main() -> cityjson_lib::Result<()> {
    let model = json::from_file("tests/data/v2_0/minimal.city.json")?;
    println!("loaded {} CityObjects", model.cityobjects().len());

    let bytes = std::fs::read("tests/data/v2_0/minimal.city.json")?;
    let model = json::from_slice(&bytes)?;
    println!("loaded {} CityObjects", model.cityobjects().len());

    Ok(())
}
