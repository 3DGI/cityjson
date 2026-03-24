use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use pretty_assertions::assert_eq;
use serde::Deserialize;
use serde_json::Value;

use serde_cityjson::{from_str_owned, to_string};

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

pub fn roundtrip_value(input: &Value) -> Value {
    let model = from_str_owned(&serde_json::to_string(input).unwrap()).unwrap();
    serde_json::from_str(&to_string(&model).unwrap()).unwrap()
}

/// Assert that the data retains the same content after an adapter deserialize-serialize roundtrip.
pub fn assert_eq_roundtrip(json_input: &str) {
    let expected: Value = serde_json::from_str(json_input).unwrap();
    let result = roundtrip_value(&expected);
    assert_eq!(result, expected);
}

/// Assert that a JSON fragment retains the same content after being wrapped into a minimal
/// CityJSON document, passed through the adapter, and extracted again.
pub fn assert_eq_roundtrip_wrapped(
    json_input: &str,
    wrap: fn(Value) -> Value,
    extract: fn(&Value) -> Value,
) {
    let expected: Value = serde_json::from_str(json_input).unwrap();
    let wrapped = wrap(expected.clone());
    let result = roundtrip_value(&wrapped);
    assert_eq!(extract(&result), expected);
}
