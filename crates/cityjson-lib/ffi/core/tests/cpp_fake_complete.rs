use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn cpp_fake_complete_example_matches_fixture_structurally() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("ffi/core should have a parent")
        .parent()
        .expect("repo root should exist")
        .to_path_buf();
    let binary = repo_root.join("target/ffi-cpp-build/cityjson_lib_cpp_fake_complete");
    assert!(
        binary.exists(),
        "expected built example binary at {}",
        binary.display()
    );

    let output = Command::new(&binary)
        .output()
        .expect("fake-complete example should run");
    assert!(
        output.status.success(),
        "example failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let actual: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("example output should be valid json");
    let fixture_bytes = fs::read(repo_root.join("tests/data/v2_0/cityjson_fake_complete.city.json"))
        .expect("fixture should be readable");
    let expected: serde_json::Value =
        serde_json::from_slice(&fixture_bytes).expect("fixture should be valid json");

    assert_eq!(actual, expected);
}
