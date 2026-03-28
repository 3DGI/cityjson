mod common;

use std::fs;

use common::ndjson_root;
use serde_json::Value;

#[test]
fn ndjson_tiles_start_with_metadata_and_then_features() {
    let root = ndjson_root();
    let sample = find_first(&root, "city.jsonl");
    let contents = fs::read_to_string(&sample).expect("ndjson tile must be readable");
    let mut lines = contents.lines();

    let metadata: Value = serde_json::from_str(lines.next().expect("first line"))
        .expect("first line must be valid JSON");
    let feature: Value = serde_json::from_str(lines.next().expect("second line"))
        .expect("second line must be valid JSON");

    assert_eq!(metadata["type"], "CityJSON");
    assert!(
        metadata.get("transform").is_some(),
        "metadata line must carry transform"
    );
    assert_eq!(feature["type"], "CityJSONFeature");
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
