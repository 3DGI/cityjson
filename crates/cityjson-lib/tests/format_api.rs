//! Public API contract for the core JSON boundary.

use cityjson_lib::json;
use serde_json::Value;

#[test]
fn json_boundary_roundtrips_through_the_core_module() {
    let model = json::from_file("tests/data/v2_0/minimal.city.json")
        .expect("fixture should parse through the core JSON boundary");

    let bytes = json::to_vec(&model).expect("model should serialize");
    let roundtrip = json::from_slice(&bytes).expect("serialized bytes should parse");
    let model_value: Value =
        serde_json::from_slice(&bytes).expect("serialized bytes should be JSON");
    let roundtrip_bytes = json::to_vec(&roundtrip).expect("roundtrip model should serialize");
    let roundtrip_value: Value =
        serde_json::from_slice(&roundtrip_bytes).expect("roundtrip bytes should be JSON");

    assert!(!bytes.is_empty());
    assert_eq!(model_value, roundtrip_value);
}
