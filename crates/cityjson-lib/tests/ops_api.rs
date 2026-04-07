//! Public API contract for the future `cjlib::ops` boundary.

use cjlib::{json, ops};

#[test]
fn ops_merge_combines_self_contained_models() {
    let first = json::from_feature_slice(
        br#"{"type":"CityJSONFeature","id":"feature-1","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}"#,
    )
    .expect("feature fixture should parse");
    let second = json::from_feature_slice(
        br#"{"type":"CityJSONFeature","id":"feature-2","CityObjects":{"feature-2":{"type":"BuildingPart"}},"vertices":[]}"#,
    )
    .expect("feature fixture should parse");

    let merged = ops::merge([first, second]).expect("ops::merge should combine feature models");
    assert_eq!(merged.as_inner().cityobjects().len(), 2);
}

#[test]
fn ops_extract_filters_cityobjects_and_relations() {
    let model = json::from_slice(include_bytes!("data/v2_0/minimal.city.json"))
        .expect("fixture should parse");

    let extracted = ops::extract(&model, ["building-part-1"]).expect("extract should succeed");
    let cityobjects = extracted.as_inner().cityobjects();

    assert_eq!(cityobjects.len(), 1);
    let (_, part) = cityobjects.first().expect("one cityobject should remain");
    assert_eq!(part.id(), "building-part-1");
    assert!(part.parents().is_none());
}

#[test]
fn ops_cleanup_roundtrips_valid_models() {
    let model = json::from_slice(include_bytes!("data/v2_0/minimal.city.json"))
        .expect("fixture should parse");
    let cleaned = ops::cleanup(&model).expect("cleanup should roundtrip");

    assert_eq!(cleaned.as_inner().cityobjects().len(), 2);
    assert_eq!(cleaned.as_inner().geometry_count(), 2);
}
