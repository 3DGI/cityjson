use std::fs;
use std::io::Write;

#[path = "support/mod.rs"]
mod support;

use serde_cityjson::{from_str_owned, to_string_validated};
use tempfile::Builder;

#[test]
fn minimal_serde_cityjson_fixture_survives_the_cityarrow_bridge() {
    let input_path = support::sibling_serde_cityjson_root()
        .join("tests/data/v2_0/cityjson_minimal_complete.city.json");
    let input_json = fs::read_to_string(&input_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", input_path.display()));

    let model = from_str_owned(&input_json).unwrap_or_else(|error| {
        panic!(
            "serde_cityjson failed for {}: {error}",
            input_path.display()
        )
    });
    let model = support::roundtrip_via_cityarrow(model);

    let output_json = to_string_validated(&model).unwrap_or_else(|error| {
        panic!(
            "serde_cityjson validation failed for {}: {error}",
            input_path.display()
        )
    });

    let mut temp = Builder::new()
        .prefix("cityarrow-minimal-bridge")
        .suffix(".city.json")
        .tempfile()
        .expect("failed to create temp output");
    temp.write_all(output_json.as_bytes())
        .expect("failed to write temp output");
    temp.flush().expect("failed to flush temp output");

    support::cjval_validate(temp.path());
}
