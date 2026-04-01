use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use pretty_assertions::assert_eq;
use serde::Deserialize;
use serde_json::Value;

use serde_cityjson::{from_str_borrowed, from_str_owned, to_string};

/// # Panics
///
/// Panics if the file cannot be opened or read.
#[must_use]
pub fn read_to_string(path: PathBuf) -> String {
    let mut file = File::open(path).unwrap();
    let mut json_string = String::new();
    file.read_to_string(&mut json_string).unwrap();
    json_string
}

pub fn cargo_workspace_directory() -> Option<PathBuf> {
    #[derive(Deserialize)]
    struct Metadata {
        workspace_root: PathBuf,
    }

    env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            let output = Command::new(env::var_os("CARGO")?)
                .args(["metadata", "--format-version", "1"])
                .output()
                .ok()?;
            let metadata: Metadata = serde_json::from_slice(&output.stdout).ok()?;
            Some(metadata.workspace_root)
        })
}

/// # Panics
///
/// Panics if serialization or deserialization fails.
#[must_use]
pub fn roundtrip_value(input: &Value) -> Value {
    let model = from_str_owned(&serde_json::to_string(input).unwrap()).unwrap();
    serde_json::from_str(&to_string(&model).unwrap()).unwrap()
}

/// # Panics
///
/// Panics if serialization or deserialization fails.
#[must_use]
pub fn roundtrip_value_borrowed(input: &Value) -> Value {
    let s = serde_json::to_string(input).unwrap();
    let model = from_str_borrowed(&s).unwrap();
    serde_json::from_str(&to_string(&model).unwrap()).unwrap()
}

/// Assert that the data retains the same content after an adapter deserialize-serialize roundtrip.
///
/// # Panics
///
/// Panics if the roundtrip fails or the result does not match the input.
pub fn assert_eq_roundtrip(json_input: &str) {
    let expected: Value = serde_json::from_str(json_input).unwrap();
    let result = roundtrip_value(&expected);
    assert_eq!(result, expected);
}

/// Assert that the data retains the same content after a borrowed-mode roundtrip.
///
/// # Panics
///
/// Panics if the roundtrip fails or the result does not match the input.
pub fn assert_eq_roundtrip_borrowed(json_input: &str) {
    let expected: Value = serde_json::from_str(json_input).unwrap();
    let result = roundtrip_value_borrowed(&expected);
    assert_eq!(result, expected);
}

/// Assert that owned and borrowed modes both roundtrip the input correctly and produce
/// identical output.
///
/// # Panics
///
/// Panics if the roundtrip fails or the results do not match.
pub fn assert_eq_roundtrip_parity(json_input: &str) {
    let expected: Value = serde_json::from_str(json_input).unwrap();
    let owned = roundtrip_value(&expected);
    let borrowed = roundtrip_value_borrowed(&expected);
    assert_eq!(owned, expected, "owned roundtrip mismatch");
    assert_eq!(borrowed, expected, "borrowed roundtrip mismatch");
    assert_eq!(owned, borrowed, "owned/borrowed output mismatch");
}
