mod common;

use std::fs;

use cityjson_index::{CityIndex, StorageLayout};
use common::{
    bbox_for_model, feature_files_root, find_first, materialize_subset, model_contains_id,
    temp_index_path,
};
use serde_json::Value;

#[test]
fn feature_files_cityindex_supports_end_to_end_queries() {
    let source_root = feature_files_root();
    let sample = find_first(&source_root.join("features"), "city.jsonl", true);
    let root = materialize_subset(
        "feature-files-data",
        &source_root,
        &[source_root.join("metadata.json"), sample.clone()],
    );
    let bytes = fs::read(&sample).expect("sample feature file must be readable");
    let value: Value = serde_json::from_slice(&bytes).expect("valid JSON feature");
    let feature_id = value
        .get("id")
        .and_then(Value::as_str)
        .expect("feature file must carry a top-level id")
        .to_owned();

    let index_path = temp_index_path("feature-files");
    let mut index = CityIndex::open(
        StorageLayout::FeatureFiles {
            root: root.clone(),
            metadata_glob: "**/metadata.json".to_owned(),
            feature_glob: "**/*.city.jsonl".to_owned(),
        },
        &index_path,
    )
    .expect("feature-files index should open");

    index
        .reindex()
        .expect("feature-files reindex should succeed");

    let model = index
        .get(&feature_id)
        .expect("feature-files get should succeed")
        .expect("feature id should be indexed");
    assert!(model_contains_id(&model, &feature_id));

    let bbox = bbox_for_model(&model).expect("bbox should be computable from indexed model");
    let query_hits = index
        .query(&bbox)
        .expect("feature-files query should succeed");
    assert!(
        query_hits
            .iter()
            .any(|candidate| model_contains_id(candidate, &feature_id)),
        "query should return the selected feature"
    );

    let iter_hits = index
        .query_iter(&bbox)
        .expect("feature-files query_iter should succeed")
        .collect::<cityjson_lib::Result<Vec<_>>>()
        .expect("feature-files query_iter items should succeed");
    assert!(
        iter_hits
            .iter()
            .any(|candidate| model_contains_id(candidate, &feature_id)),
        "query_iter should return the selected feature"
    );

    let metadata = index
        .metadata()
        .expect("feature-files metadata should succeed");
    assert!(
        metadata
            .iter()
            .any(|entry| entry.get("transform").is_some()),
        "feature-files metadata should include at least one transform"
    );
}
