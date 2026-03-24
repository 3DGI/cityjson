//! Public API contract for structured error handling.
//! The goal is a stable category surface, not string matching.

use cjlib::{ErrorKind, json};

#[test]
fn invalid_json_is_a_structured_syntax_error() {
    let error = json::from_slice(br#"{"type":"CityJSON""#).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax);
}

#[test]
fn missing_file_is_a_structured_io_error() {
    let error = json::from_file("tests/data/does-not-exist.city.json").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Io);
}

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

#[test]
fn unsupported_version_is_a_structured_unsupported_error() {
    let error =
        json::from_slice(br#"{"type":"CityJSON","version":"9.9","CityObjects":{},"vertices":[]}"#)
            .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Unsupported);
}
