use cityjson_lib::json;

fn main() -> cityjson_lib::Result<()> {
    let model = json::from_file("tests/data/v2_0/minimal.city.json")?;

    let bytes = json::to_vec(&model)?;
    let text = json::to_string(&model)?;

    let mut writer = Vec::new();
    json::to_writer(&mut writer, &model)?;

    let reparsed = json::from_slice(&bytes)?;
    println!(
        "round-tripped {} CityObjects from {} bytes and {} chars",
        reparsed.cityobjects().len(),
        writer.len(),
        text.len()
    );

    Ok(())
}
