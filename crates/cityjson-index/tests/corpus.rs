mod common;

use std::fs;

use common::basisvoorziening_artifact;
use serde_json::Value;

#[test]
fn basisvoorziening_artifact_is_parseable_cityjson() {
    let Some(artifact) = basisvoorziening_artifact() else {
        return;
    };

    assert!(
        artifact.exists(),
        "pinned Basisvoorziening artifact should exist at {}",
        artifact.display()
    );

    let bytes = fs::read(&artifact).expect("artifact should be readable");
    let document: Value = serde_json::from_slice(&bytes).expect("artifact should be valid JSON");
    assert_eq!(document["type"], "CityJSON");
    assert!(
        document["CityObjects"]
            .as_object()
            .is_some_and(|objects| !objects.is_empty()),
        "artifact should contain CityObjects"
    );
    assert!(
        document.get("vertices").is_some(),
        "artifact should contain vertices"
    );
}
