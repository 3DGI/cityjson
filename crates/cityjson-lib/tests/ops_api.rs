//! Public API contract for the `cityjson_lib::ops` boundary.

use std::collections::BTreeSet;

use cityjson_lib::cityjson_types::v2_0::CityObject;
use cityjson_lib::cityjson_types::{
    prelude::CityObjectHandle, resources::storage::OwnedStringStorage,
};
use cityjson_lib::{json, ops};
use serde_json::Value;

type TestCityObject = CityObject<OwnedStringStorage>;

fn cityobject_ids(model: &cityjson_lib::CityModel) -> BTreeSet<String> {
    model
        .cityobjects()
        .iter()
        .map(|(_, cityobject)| cityobject.id().to_owned())
        .collect()
}

fn feature_root_id(model: &cityjson_lib::CityModel) -> Option<String> {
    model.id().and_then(|handle| {
        model
            .cityobjects()
            .get(handle)
            .map(|cityobject| cityobject.id().to_owned())
    })
}

fn geometry_count(model: &cityjson_lib::CityModel, id: &str) -> usize {
    model
        .cityobjects()
        .iter()
        .find_map(|(_, cityobject)| {
            (cityobject.id() == id).then_some(match cityobject.geometry() {
                Some(geometry) => geometry.len(),
                None => 0,
            })
        })
        .expect("CityObject should exist")
}

fn related_cityobject_ids(
    model: &cityjson_lib::CityModel,
    id: &str,
    relation: fn(&TestCityObject) -> Option<&[CityObjectHandle]>,
) -> BTreeSet<String> {
    let cityobject = model
        .cityobjects()
        .iter()
        .find_map(|(_, cityobject)| (cityobject.id() == id).then_some(cityobject))
        .expect("CityObject should exist");

    relation(cityobject)
        .into_iter()
        .flatten()
        .map(|handle| {
            model
                .cityobjects()
                .get(*handle)
                .expect("related CityObject should exist")
                .id()
                .to_owned()
        })
        .collect()
}

fn fixture_subset_model() -> cityjson_lib::Result<cityjson_lib::CityModel> {
    json::from_slice(include_bytes!("data/v2_0/ops/subset_source.city.json"))
}

fn fixture_merge_left() -> cityjson_lib::Result<cityjson_lib::CityModel> {
    json::from_slice(include_bytes!("data/v2_0/ops/merge_left.city.json"))
}

fn fixture_merge_right() -> cityjson_lib::Result<cityjson_lib::CityModel> {
    json::from_slice(include_bytes!("data/v2_0/ops/merge_right.city.json"))
}

fn set_transform(model: &mut cityjson_lib::CityModel, scale: [f64; 3], translate: [f64; 3]) {
    model.transform_mut().set_scale(scale);
    model.transform_mut().set_translate(translate);
}

#[test]
fn ops_select_cityobjects_keeps_original_feature_root_when_it_survives() {
    let model = json::from_feature_slice(
        br#"{
            "type":"CityJSONFeature",
            "id":"root-building",
            "CityObjects":{
                "root-building":{"type":"Building","children":["building-part-1"]},
                "building-part-1":{"type":"BuildingPart","parents":["root-building"]},
                "other-building":{"type":"Building"}
            },
            "vertices":[]
        }"#,
    )
    .expect("feature fixture should parse");

    let selection = ops::select_cityobjects(&model, |ctx| ctx.id() == "root-building")
        .expect("selection should succeed");
    let extracted = ops::extract(&model, &selection).expect("extract should preserve the root");

    assert_eq!(
        feature_root_id(&extracted),
        Some(String::from("root-building"))
    );
    assert_eq!(
        cityobject_ids(&extracted),
        BTreeSet::from([String::from("root-building")])
    );
}

#[test]
fn ops_extract_reroots_to_surviving_parentless_object() {
    let model = json::from_feature_slice(
        br#"{
            "type":"CityJSONFeature",
            "id":"root-building",
            "CityObjects":{
                "root-building":{"type":"Building","children":["building-part-1"]},
                "building-part-1":{"type":"BuildingPart","parents":["root-building"]},
                "other-building":{"type":"Building"}
            },
            "vertices":[]
        }"#,
    )
    .expect("feature fixture should parse");

    let selection = ops::select_cityobjects(&model, |ctx| ctx.id() == "other-building")
        .expect("selection should succeed");
    let subset = ops::extract(&model, &selection)
        .expect("extract should reroot to the surviving parentless CityObject");

    assert_eq!(
        feature_root_id(&subset),
        Some(String::from("other-building"))
    );
    assert_eq!(
        cityobject_ids(&subset),
        BTreeSet::from([String::from("other-building")])
    );
}

#[test]
fn ops_extract_errors_when_feature_loses_everything() {
    let model = json::from_feature_slice(
        br#"{
            "type":"CityJSONFeature",
            "id":"root-building",
            "CityObjects":{
                "root-building":{"type":"Building","children":["building-part-1"]},
                "building-part-1":{"type":"BuildingPart","parents":["root-building"]},
                "other-building":{"type":"Building"}
            },
            "vertices":[]
        }"#,
    )
    .expect("feature fixture should parse");

    let selection = ops::select_cityobjects(&model, |_| false).expect("selection should succeed");
    let error = ops::extract(&model, &selection)
        .expect_err("extract should fail when a feature loses its root");

    assert_eq!(error.kind(), cityjson_lib::ErrorKind::Model);
}

#[test]
fn ops_subset_keeps_top_level_selection_as_is() {
    let model = fixture_subset_model().expect("subset fixture should parse");

    let subset = ops::subset(&model, ["other-building"], false)
        .expect("ops::subset should accept a single top-level id");

    assert_eq!(
        cityobject_ids(&subset),
        BTreeSet::from([String::from("other-building")])
    );
}

#[test]
fn ops_subset_includes_recursive_children_and_group_members() {
    let model = fixture_subset_model().expect("subset fixture should parse");

    let subset = ops::subset(&model, ["my-group"], false)
        .expect("ops::subset should include the selected group and its closure");

    assert_eq!(
        cityobject_ids(&subset),
        BTreeSet::from([
            String::from("building-part-1"),
            String::from("building-part-2"),
            String::from("my-group"),
        ])
    );
}

#[test]
fn ops_subset_can_exclude_the_selected_closure() {
    let model = fixture_subset_model().expect("subset fixture should parse");

    let subset = ops::subset(&model, ["root-building"], true)
        .expect("ops::subset should support exclude mode");

    assert_eq!(
        cityobject_ids(&subset),
        BTreeSet::from([String::from("my-group"), String::from("other-building")])
    );
}

#[test]
fn ops_select_cityobjects_exact_id_keeps_only_match_and_strips_removed_references() {
    let model = fixture_subset_model().expect("selection fixture should parse");

    let selection = ops::select_cityobjects(&model, |ctx| ctx.id() == "building-part-1")
        .expect("selection should succeed");
    let extracted = ops::extract(&model, &selection)
        .expect("select_cityobjects + extract should accept an id predicate");

    assert_eq!(
        cityobject_ids(&extracted),
        BTreeSet::from([String::from("building-part-1")])
    );

    let (_, cityobject) = extracted
        .cityobjects()
        .iter()
        .next()
        .expect("filtered model should contain the selected CityObject");
    assert!(cityobject.parents().is_none_or(<[_]>::is_empty));
    assert!(cityobject.children().is_none_or(<[_]>::is_empty));
}

#[test]
fn ops_select_cityobjects_with_relatives_from_middle_object_pulls_parent_child_closure() {
    let model = fixture_subset_model().expect("selection fixture should parse");

    let selection = ops::select_cityobjects(&model, |ctx| ctx.id() == "building-part-1")
        .expect("selection should succeed")
        .include_relatives(&model)
        .expect("include_relatives should include relatives");
    let extracted = ops::extract(&model, &selection).expect("extract should succeed");

    assert_eq!(
        cityobject_ids(&extracted),
        BTreeSet::from([
            String::from("building-part-1"),
            String::from("building-part-2"),
            String::from("my-group"),
            String::from("root-building"),
        ])
    );
    assert_eq!(
        related_cityobject_ids(&extracted, "building-part-1", CityObject::parents),
        BTreeSet::from([String::from("my-group"), String::from("root-building")])
    );
    assert_eq!(
        related_cityobject_ids(&extracted, "building-part-1", CityObject::children),
        BTreeSet::from([String::from("building-part-2")])
    );
}

#[test]
fn ops_select_cityobjects_with_relatives_from_group_includes_member_closure_and_connected_parents()
{
    let model = fixture_subset_model().expect("selection fixture should parse");

    let selection = ops::select_cityobjects(&model, |ctx| ctx.id() == "my-group")
        .expect("selection should succeed")
        .include_relatives(&model)
        .expect("include_relatives should include group relatives");
    let extracted = ops::extract(&model, &selection).expect("extract should succeed");

    assert_eq!(
        cityobject_ids(&extracted),
        BTreeSet::from([
            String::from("building-part-1"),
            String::from("building-part-2"),
            String::from("my-group"),
            String::from("root-building"),
        ])
    );
    assert_eq!(
        related_cityobject_ids(&extracted, "my-group", CityObject::children),
        BTreeSet::from([String::from("building-part-1")])
    );
    assert_eq!(
        related_cityobject_ids(&extracted, "building-part-1", CityObject::parents),
        BTreeSet::from([String::from("my-group"), String::from("root-building")])
    );
}

#[test]
fn ops_select_cityobjects_empty_predicate_result_returns_empty_model() {
    let model = fixture_subset_model().expect("selection fixture should parse");

    let selection = ops::select_cityobjects(&model, |_| false).expect("selection should succeed");
    let extracted = ops::extract(&model, &selection).expect("extract should allow an empty result");

    assert!(extracted.cityobjects().is_empty());
}

#[test]
fn ops_select_cityobjects_context_exposes_model_handle_cityobject_and_id() {
    let model = fixture_subset_model().expect("selection fixture should parse");
    let mut saw_middle_object = false;

    let selection = ops::select_cityobjects(&model, |ctx| {
        assert!(std::ptr::eq(ctx.model(), &raw const model));
        assert_eq!(ctx.cityobject().id(), ctx.id());
        assert_eq!(
            ctx.model()
                .cityobjects()
                .get(ctx.handle())
                .expect("context handle should resolve")
                .id(),
            ctx.id()
        );

        let matches = ctx.id() == "building-part-1";
        saw_middle_object |= matches;
        matches
    })
    .expect("selection should pass context into the predicate");

    let extracted = ops::extract(&model, &selection).expect("extract should succeed");

    assert!(saw_middle_object);
    assert_eq!(
        cityobject_ids(&extracted),
        BTreeSet::from([String::from("building-part-1")])
    );
}

#[test]
fn ops_select_geometries_keeps_only_matching_geometry_handles() {
    let model = fixture_merge_left().expect("geometry fixture should parse");

    let selection = ops::select_geometries(&model, |ctx| {
        ctx.cityobject_id() == "shared-furniture" && ctx.geometry_index() == 0
    })
    .expect("selection should succeed");
    let extracted = ops::extract(&model, &selection).expect("extract should succeed");

    assert_eq!(
        cityobject_ids(&extracted),
        BTreeSet::from([String::from("shared-furniture")])
    );
    assert_eq!(geometry_count(&extracted, "shared-furniture"), 1);
}

#[test]
fn ops_model_selection_union_and_intersection_handle_whole_and_partial_selections() {
    let model = fixture_merge_left().expect("geometry fixture should parse");

    let whole = ops::select_cityobjects(&model, |ctx| ctx.id() == "shared-furniture")
        .expect("whole selection should succeed");
    let first_geometry = ops::select_geometries(&model, |ctx| {
        ctx.cityobject_id() == "shared-furniture" && ctx.geometry_index() == 0
    })
    .expect("first partial selection should succeed");
    let second_geometry = ops::select_geometries(&model, |ctx| {
        ctx.cityobject_id() == "shared-furniture" && ctx.geometry_index() == 1
    })
    .expect("second partial selection should succeed");

    let union = whole.union(&first_geometry);
    let union_extract = ops::extract(&model, &union).expect("union extract should succeed");
    assert_eq!(geometry_count(&union_extract, "shared-furniture"), 2);

    let whole_intersection = whole.intersection(&first_geometry);
    let whole_intersection_extract = ops::extract(&model, &whole_intersection)
        .expect("whole/partial intersection should succeed");
    assert_eq!(
        geometry_count(&whole_intersection_extract, "shared-furniture"),
        1
    );

    let disjoint = first_geometry.intersection(&second_geometry);
    assert!(disjoint.is_empty());
    let disjoint_extract = ops::extract(&model, &disjoint)
        .expect("empty intersection should extract to an empty model");
    assert!(disjoint_extract.cityobjects().is_empty());
}

#[test]
fn ops_merge_coalesces_overlapping_models_and_remaps_templates() {
    let left = fixture_merge_left().expect("left merge fixture should parse");
    let right = fixture_merge_right().expect("right merge fixture should parse");

    let merged = ops::merge([left, right]).expect("ops::merge should accept overlapping models");

    assert_eq!(merged.cityobjects().len(), 3);
    assert_eq!(merged.geometry_count(), 8);
    assert_eq!(merged.geometry_template_count(), 2);
    assert_eq!(merged.material_count(), 5);
    assert_eq!(merged.texture_count(), 4);
    assert_eq!(merged.vertices().len(), 8);

    let merged_json: Value = serde_json::from_str(
        &json::to_string(&merged).expect("merged model should serialize to JSON"),
    )
    .expect("merged JSON should parse");

    assert_eq!(
        cityobject_ids(&merged),
        BTreeSet::from([
            String::from("left-only"),
            String::from("right-only"),
            String::from("shared-furniture"),
        ])
    );

    let shared = &merged_json["CityObjects"]["shared-furniture"];
    assert_eq!(shared["attributes"]["ducimus"], serde_json::json!(true));
    assert!(shared["attributes"].get("optio").is_some());

    let geometries = shared["geometry"]
        .as_array()
        .expect("shared-furniture should keep its geometries");
    assert_eq!(geometries.len(), 4);
    assert_eq!(
        geometries
            .iter()
            .filter(|geometry| geometry["template"].as_u64() == Some(0))
            .count(),
        2
    );
    assert_eq!(
        geometries
            .iter()
            .filter(|geometry| geometry["template"].as_u64() == Some(1))
            .count(),
        2
    );

    assert_eq!(
        merged_json["geometry-templates"]["vertices-templates"]
            .as_array()
            .expect("merged templates should keep their template vertices")
            .len(),
        36
    );
}

#[test]
fn ops_append_accepts_mismatched_transforms_and_clears_the_result() {
    let mut left = fixture_merge_left().expect("left merge fixture should parse");
    let mut right = fixture_merge_right().expect("right merge fixture should parse");
    set_transform(&mut left, [1.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
    set_transform(&mut right, [2.0, 2.0, 2.0], [10.0, 0.0, 0.0]);

    ops::append(&mut left, &right).expect("ops::append should merge mismatched transforms");

    assert!(left.transform().is_none());
}

#[test]
fn ops_merge_preserves_identical_transforms() {
    let left = fixture_merge_left().expect("left merge fixture should parse");
    let right = fixture_merge_right().expect("right merge fixture should parse");

    let merged = ops::merge([left, right]).expect("ops::merge should accept identical transforms");

    assert!(merged.transform().is_some());
}

#[test]
fn ops_merge_clears_mixed_transforms() {
    let mut left = fixture_merge_left().expect("left merge fixture should parse");
    let mut right = fixture_merge_right().expect("right merge fixture should parse");
    set_transform(&mut left, [1.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
    set_transform(&mut right, [2.0, 2.0, 2.0], [10.0, 0.0, 0.0]);

    let merged = ops::merge([left, right]).expect("ops::merge should accept mixed transforms");

    assert!(merged.transform().is_none());
}

#[test]
fn ops_merge_preserves_a_single_shared_transform_when_the_first_model_has_none() {
    let left = json::from_feature_slice(
        br#"{
            "type":"CityJSONFeature",
            "id":"feature-a",
            "CityObjects":{
                "feature-a":{"type":"Building"}
            },
            "vertices":[]
        }"#,
    )
    .expect("feature without transform should parse");
    let right = json::from_feature_slice(
        br#"{
            "type":"CityJSONFeature",
            "id":"feature-b",
            "transform":{"scale":[1.0,1.0,1.0],"translate":[0.0,0.0,0.0]},
            "CityObjects":{
                "feature-b":{"type":"Building"}
            },
            "vertices":[]
        }"#,
    )
    .expect("feature with transform should parse");

    let merged = ops::merge([left, right]).expect("ops::merge should preserve a single transform");

    assert!(merged.transform().is_some());
}

#[test]
fn ops_merge_without_transform_serializes_without_transform_and_roundtrips() {
    let mut left = fixture_merge_left().expect("left merge fixture should parse");
    let mut right = fixture_merge_right().expect("right merge fixture should parse");
    set_transform(&mut left, [1.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
    set_transform(&mut right, [2.0, 2.0, 2.0], [10.0, 0.0, 0.0]);

    let merged = ops::merge([left, right]).expect("ops::merge should accept mixed transforms");
    assert!(merged.transform().is_none());

    let merged_json = json::to_string(&merged).expect("merged model should serialize");
    let written: Value = serde_json::from_str(&merged_json).expect("merged JSON should parse");
    assert!(written.get("transform").is_none());

    let reparsed = json::from_slice(merged_json.as_bytes()).expect("merged model should roundtrip");
    assert!(reparsed.transform().is_none());
    assert_eq!(cityobject_ids(&reparsed), cityobject_ids(&merged));
}
