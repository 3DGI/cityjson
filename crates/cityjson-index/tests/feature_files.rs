mod common;

use std::fs;

use common::feature_files_root;
use serde_json::Value;

#[test]
fn feature_files_layout_exists_and_contains_cityjsonfeatures() {
    let root = feature_files_root();
    assert!(root.exists(), "feature-files fixture set must exist");
    assert!(
        root.join("metadata.json").exists(),
        "missing root metadata.json"
    );

    let sample = find_first_nonempty(&root.join("features"), "city.jsonl");
    let bytes = fs::read(&sample).expect("sample feature file must be readable");
    let value: Value = serde_json::from_slice(&bytes).expect("valid JSON feature");

    assert_eq!(value["type"], "CityJSONFeature");
    assert!(
        value
            .get("CityObjects")
            .and_then(Value::as_object)
            .is_some()
    );
}

fn find_first_nonempty(root: &std::path::Path, suffix: &str) -> std::path::PathBuf {
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry.expect("directory entry");
        if entry.file_type().is_file()
            && entry.path().to_string_lossy().ends_with(suffix)
            && entry.metadata().map(|meta| meta.len() > 0).unwrap_or(false)
        {
            return entry.path().to_path_buf();
        }
    }
    panic!("no {suffix} file found in {}", root.display());
}
