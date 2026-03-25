use cjlib::CityModel;

fn main() -> cjlib::Result<()> {
    let model = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;
    println!(
        "loaded {} CityObjects",
        model.as_inner().cityobjects().len()
    );

    let bytes = std::fs::read("tests/data/v2_0/minimal.city.json")?;
    let model = CityModel::from_slice(&bytes)?;
    println!(
        "loaded {} CityObjects",
        model.as_inner().cityobjects().len()
    );

    Ok(())
}
