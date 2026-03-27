//! Public API contract for the future `cjlib::ops` boundary.

use cjlib::{json, ops};

#[test]
fn ops_merge_combines_self_contained_models() -> cjlib::Result<()> {
    let first = json::from_feature_slice(
        br#"{"type":"CityJSONFeature","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}"#,
    )?;
    let second = json::from_feature_slice(
        br#"{"type":"CityJSONFeature","CityObjects":{"feature-2":{"type":"BuildingPart"}},"vertices":[]}"#,
    )?;

    let merged = ops::merge([first, second])?;
    assert_eq!(merged.as_inner().cityobjects().len(), 2);

    Ok(())
}
