//! Public API contract for the future `cityjson_lib::ops` boundary.

use std::collections::BTreeSet;

use cityjson_lib::cityjson::v2_0::CityObject;
use cityjson_lib::cityjson::{prelude::CityObjectHandle, resources::storage::OwnedStringStorage};
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
fn ops_filter_exact_id_keeps_only_match_and_strips_removed_references() {
    let model = fixture_subset_model().expect("filter fixture should parse");

    let filtered = ops::filter(&model, |ctx| ctx.id() == "building-part-1")
        .expect("ops::filter should accept an id predicate");

    assert_eq!(
        cityobject_ids(&filtered),
        BTreeSet::from([String::from("building-part-1")])
    );

    let (_, cityobject) = filtered
        .cityobjects()
        .iter()
        .next()
        .expect("filtered model should contain the selected CityObject");
    assert!(cityobject.parents().is_none_or(<[_]>::is_empty));
    assert!(cityobject.children().is_none_or(<[_]>::is_empty));
}

#[test]
fn ops_filter_with_relatives_from_middle_object_pulls_parent_child_closure() {
    let model = fixture_subset_model().expect("filter fixture should parse");

    let filtered = ops::filter_with_options(
        &model,
        ops::FilterOptions {
            include_relatives: true,
        },
        |ctx| ctx.id() == "building-part-1",
    )
    .expect("ops::filter_with_options should include relatives");

    assert_eq!(
        cityobject_ids(&filtered),
        BTreeSet::from([
            String::from("building-part-1"),
            String::from("building-part-2"),
            String::from("my-group"),
            String::from("root-building"),
        ])
    );
    assert_eq!(
        related_cityobject_ids(&filtered, "building-part-1", CityObject::parents),
        BTreeSet::from([String::from("my-group"), String::from("root-building")])
    );
    assert_eq!(
        related_cityobject_ids(&filtered, "building-part-1", CityObject::children),
        BTreeSet::from([String::from("building-part-2")])
    );
}

#[test]
fn ops_filter_with_relatives_from_group_includes_member_closure_and_connected_parents() {
    let model = fixture_subset_model().expect("filter fixture should parse");

    let filtered = ops::filter_with_options(
        &model,
        ops::FilterOptions {
            include_relatives: true,
        },
        |ctx| ctx.id() == "my-group",
    )
    .expect("ops::filter_with_options should include group relatives");

    assert_eq!(
        cityobject_ids(&filtered),
        BTreeSet::from([
            String::from("building-part-1"),
            String::from("building-part-2"),
            String::from("my-group"),
            String::from("root-building"),
        ])
    );
    assert_eq!(
        related_cityobject_ids(&filtered, "my-group", CityObject::children),
        BTreeSet::from([String::from("building-part-1")])
    );
    assert_eq!(
        related_cityobject_ids(&filtered, "building-part-1", CityObject::parents),
        BTreeSet::from([String::from("my-group"), String::from("root-building")])
    );
}

#[test]
fn ops_filter_empty_predicate_result_returns_empty_model() {
    let model = fixture_subset_model().expect("filter fixture should parse");

    let filtered =
        ops::filter(&model, |_| false).expect("ops::filter should allow an empty predicate result");

    assert!(filtered.cityobjects().is_empty());
}

#[test]
fn ops_filter_context_exposes_model_handle_cityobject_and_id() {
    let model = fixture_subset_model().expect("filter fixture should parse");
    let mut saw_middle_object = false;

    let filtered = ops::filter(&model, |ctx| {
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
    .expect("ops::filter should pass context into the predicate");

    assert!(saw_middle_object);
    assert_eq!(
        cityobject_ids(&filtered),
        BTreeSet::from([String::from("building-part-1")])
    );
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
