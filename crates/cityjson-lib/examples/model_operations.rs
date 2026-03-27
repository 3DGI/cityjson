use cjlib::{CityModel, ops};

fn main() -> cjlib::Result<()> {
    let model = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;
    let _ = ops::merge([model]);

    Ok(())
}
