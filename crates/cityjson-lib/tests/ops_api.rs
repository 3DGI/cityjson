//! Public API contract for the future `cjlib::ops` boundary.

use cjlib::{json, ops};

#[test]
#[should_panic(expected = "implement model-authoritative merge delegation through cityjson-rs")]
fn ops_merge_combines_self_contained_models() {
    let first = json::from_feature_slice(
        br#"{"type":"CityJSONFeature","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}"#,
    )
    .expect("feature fixture should parse");
    let second = json::from_feature_slice(
        br#"{"type":"CityJSONFeature","CityObjects":{"feature-2":{"type":"BuildingPart"}},"vertices":[]}"#,
    )
    .expect("feature fixture should parse");

    let merged = ops::merge([first, second]).expect("ops::merge is intentionally unimplemented");
    assert_eq!(merged.as_inner().cityobjects().len(), 2);
}
