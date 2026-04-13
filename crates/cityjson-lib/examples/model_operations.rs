use cityjson_lib::{CityModel, ops};

fn main() -> cityjson_lib::Result<()> {
    let model = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;
    let _ = ops::merge([model]);

    Ok(())
}
