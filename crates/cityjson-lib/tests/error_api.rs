//! Public API contract for structured error handling.
//! The goal is a stable category surface, not string matching.

use cjlib::{ErrorKind, json};

#[test]
fn missing_version_is_a_structured_version_error() {
    let error = json::from_slice(br#"{"type":"CityJSON","CityObjects":{},"vertices":[]}"#).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Version);
}

#[test]
fn wrong_root_type_is_a_structured_shape_error() {
    let error =
        json::from_slice(br#"{"type":"CityJSONFeature","CityObjects":{},"vertices":[]}"#).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Shape);
}
