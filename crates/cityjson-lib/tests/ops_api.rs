//! Public API contract for higher-level `cjlib::ops` workflows.
//! These tests pin down the intended shape before the implementation exists.

use cjlib::{CityModel, ops};

#[test]
fn higher_level_model_workflows_live_under_ops() -> cjlib::Result<()> {
    let model = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;

    let selection = ops::Selection::from_ids(["feature-1"]);
    let subset = ops::subset(&model, selection)?;
    let merged = ops::merge([model, subset])?;

    let _surface_area = ops::geometry::surface_area(&merged, "feature-1")?;
    let _volume = ops::geometry::volume(&merged, "feature-1")?;

    Ok(())
}

#[test]
fn vertex_cleanup_returns_a_small_structured_report() -> cjlib::Result<()> {
    let mut model = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;
    let report = ops::vertices::clean(&mut model)?;

    let _duplicates_removed: usize = report.duplicates_removed;
    let _orphans_removed: usize = report.orphans_removed;

    Ok(())
}

#[test]
fn upgrade_and_lod_filter_are_part_of_the_ops_surface() -> cjlib::Result<()> {
    let model = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;

    let _ = ops::lod::filter(&model, "2")?;
    let _ = ops::upgrade(model)?;

    Ok(())
}
