use cjlib::{CityModel, ops};

fn main() -> cjlib::Result<()> {
    let model = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;

    let selection = ops::Selection::from_ids(["feature-1"]);
    let subset = ops::subset(&model, selection)?;
    let merged = ops::merge([model, subset])?;

    let _surface_area = ops::geometry::surface_area(&merged, "feature-1")?;
    let _volume = ops::geometry::volume(&merged, "feature-1")?;

    let mut model = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;
    let _report = ops::vertices::clean(&mut model)?;

    Ok(())
}
