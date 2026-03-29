mod common;

use std::fs;
use std::path::Path;

use cjindex::{CityIndex, StorageLayout};
use common::{bbox_for_model, find_first, model_contains_id, ndjson_root, temp_index_path};

#[test]
fn ndjson_cityindex_supports_end_to_end_queries() {
    let source_root = ndjson_root();
    let sample = find_first(&source_root, "city.jsonl", true);
    let sample_fixture = derive_small_ndjson_fixture(&sample);
    let feature_id = "ndjson-test-feature".to_owned();

    let index_path = temp_index_path("ndjson");
    let mut index = CityIndex::open(
        StorageLayout::Ndjson {
            paths: vec![sample_fixture.clone()],
        },
        &index_path,
    )
    .expect("ndjson index should open");

    index.reindex().expect("ndjson reindex should succeed");

    let model = index
        .get(&feature_id)
        .expect("ndjson get should succeed")
        .expect("feature id should be indexed");
    assert!(model_contains_id(&model, &feature_id));

    let bbox = bbox_for_model(&model).expect("bbox should be computable from indexed model");
    let query_hits = index.query(&bbox).expect("ndjson query should succeed");
    assert!(
        query_hits
            .iter()
            .any(|candidate| model_contains_id(candidate, &feature_id)),
        "query should return the selected feature"
    );

    let iter_hits = index
        .query_iter(&bbox)
        .expect("ndjson query_iter should succeed")
        .collect::<cjlib::Result<Vec<_>>>()
        .expect("ndjson query_iter items should succeed");
    assert!(
        iter_hits
            .iter()
            .any(|candidate| model_contains_id(candidate, &feature_id)),
        "query_iter should return the selected feature"
    );

    let metadata = index.metadata().expect("ndjson metadata should succeed");
    assert!(
        metadata
            .iter()
            .any(|entry| entry.get("transform").is_some()),
        "ndjson metadata should include at least one transform"
    );
}

fn derive_small_ndjson_fixture(source: &Path) -> std::path::PathBuf {
    let contents = fs::read_to_string(source).expect("sample ndjson tile must be readable");
    let mut lines = contents.lines();
    let metadata = lines.next().expect("sample tile must contain metadata");
    let path = std::env::temp_dir().join(format!(
        "cjindex-ndjson-sample-{}.jsonl",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time must be after the unix epoch")
            .as_nanos()
    ));
    let feature = serde_json::json!({
        "type": "CityJSONFeature",
        "id": "ndjson-test-feature",
        "CityObjects": {
            "ndjson-test-feature": {
                "type": "Building",
                "geometry": [{
                    "type": "MultiSurface",
                    "lod": "1.0",
                    "boundaries": [[[0, 1, 2]]]
                }]
            }
        },
        "vertices": [
            [0, 0, 0],
            [1, 0, 0],
            [0, 1, 0]
        ]
    });

    fs::write(
        &path,
        format!(
            "{metadata}\n{}\n",
            serde_json::to_string(&feature).expect("feature JSON")
        ),
    )
    .expect("derived NDJSON fixture must be writable");
    path
}
