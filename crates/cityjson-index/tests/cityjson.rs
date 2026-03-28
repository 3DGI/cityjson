mod common;

use std::fs;

use common::cityjson_root;
use serde_json::Value;

#[test]
fn cityjson_tiles_are_valid_cityjson_documents() {
    let root = cityjson_root();
    let sample = find_first(&root, "city.json");
    let bytes = fs::read(&sample).expect("cityjson tile must be readable");
    let value: Value = serde_json::from_slice(&bytes).expect("valid JSON");

    assert_eq!(value["type"], "CityJSON");
    assert!(
        value
            .get("CityObjects")
            .and_then(Value::as_object)
            .is_some()
    );
    assert!(value.get("vertices").and_then(Value::as_array).is_some());
    assert!(
        value.get("transform").is_some(),
        "tile must carry a transform"
    );
}

fn find_first(root: &std::path::Path, suffix: &str) -> std::path::PathBuf {
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry.expect("directory entry");
        if entry.file_type().is_file() && entry.path().to_string_lossy().ends_with(suffix) {
            return entry.path().to_path_buf();
        }
    }
    panic!("no {suffix} file found in {}", root.display());
}
