use cjlib::{CityModel, json};

fn main() -> cjlib::Result<()> {
    let model = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;

    let bytes = json::to_vec(&model)?;
    let text = json::to_string(&model)?;

    let mut writer = Vec::new();
    json::to_writer(&mut writer, &model)?;

    let reparsed = json::from_slice(&bytes)?;
    println!(
        "round-tripped {} CityObjects from {} bytes and {} chars",
        reparsed.as_inner().cityobjects().len(),
        writer.len(),
        text.len()
    );

    Ok(())
}
