use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use pretty_assertions::assert_eq;

pub fn strip_whitespace<S: AsRef<str>>(string: S) -> String {
    string.as_ref().replace(" ", "").replace("\n", "")
}

pub fn read_to_string(path: PathBuf) -> String {
    let mut file = File::open(path).unwrap();
    let mut json_string = String::new();
    file.read_to_string(&mut json_string).unwrap();
    json_string
}

pub fn read_to_slice(path: PathBuf) -> Vec<u8> {
    let mut file = File::open(path).unwrap();
    let mut json_buf = Vec::new();
    file.read_to_end(&mut json_buf).unwrap();
    json_buf
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

/// Assert that the data retains the same content after a deserialize-serialize roundtrip.
pub fn assert_eq_roundtrip<'de, T>(json_input: &'de str)
where
    T: Deserialize<'de> + Serialize,
{
    let cm = serde_json::from_str::<T>(&json_input).unwrap();
    let json_cm = serde_json::to_string(&cm).unwrap();
    let res: Value = serde_json::from_str(&json_cm).unwrap();
    let expected: Value = serde_json::from_str(&json_input).unwrap();
    assert_eq!(res, expected);
}
