#![allow(dead_code)]
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use cjval::CJValidator;
use once_cell::sync::Lazy;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

pub static DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    cargo_workspace_directory()
        .unwrap()
        .join("tests")
        .join("data")
        .join("v1_1")
});

pub static OUTPUT_DIR: Lazy<PathBuf> = Lazy::new(|| {
    cargo_workspace_directory()
        .unwrap()
        .join("tests")
        .join("output")
});

pub static INVALIDS_DIR: Lazy<PathBuf> = Lazy::new(|| {
    cargo_workspace_directory()
        .unwrap()
        .join("tests")
        .join("invalids")
});

/// Count the number of files in the invalids directory
pub fn count_invalids(invalids_dir: &PathBuf) -> usize {
    let mut count: usize = 0;
    if let Ok(read_dir) = std::fs::read_dir(invalids_dir) {
        count = read_dir.into_iter().filter(|e| e.is_ok()).count();
    }
    count
}

/// Validate a CityJSON str with [cjval]. If the CityJSON is invalid, serialize it for
/// later analysis.
pub fn validate(cityjson_str: &str, test_name: &str) {
    let val = CJValidator::from_str(&cityjson_str);
    // assert!(val.validate().iter().all(|(c, s)| s.is_valid()));
    let invalids: Vec<(String, String)> = val
        .validate()
        .into_iter()
        .filter(|(_, summary)| !summary.is_valid())
        .map(|(criterion, summary)| (criterion, summary.to_string()))
        .collect();
    if invalids.len() > 0 {
        // Serialize invalid citymodels for later analysis
        let idir = INVALIDS_DIR.clone();
        let invalids_count = count_invalids(&idir);
        let current_invalid_nr = invalids_count + 1;
        let fname = format!("{}_{}.city.json", test_name, current_invalid_nr);
        std::fs::write(idir.join(&fname), cityjson_str).unwrap();
        println!("Serialized invalid CityJSON to {}", &fname);
    }
    for (criterion, summary) in val.validate().iter() {
        assert!(
            summary.is_valid(),
            "{} is not valid with {}",
            criterion,
            summary
        )
    }
}

/// Assert that the data retains the same content after a deserialize-serialize roundtrip.
/// Assert that result == expected.
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
