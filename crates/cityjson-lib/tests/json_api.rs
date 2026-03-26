//! Public API contract for the explicit `cjlib::json` boundary layer.
//! The module described here does not need to exist yet; this test file defines the target surface.

use std::io::Cursor;

use cjlib::{CityJSONVersion, json};

#[test]
fn explicit_json_module_supports_document_and_stream_loading() -> cjlib::Result<()> {
    let document = br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#;
    let stream = br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}
{"type":"CityJSONFeature","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}
"#;

    let probe = json::probe(document)?;
    assert_eq!(probe.kind(), json::RootKind::CityJSON);
    assert_eq!(probe.version(), Some(CityJSONVersion::V2_0));

    let _ = json::from_slice(document)?;
    let _ = json::from_file("tests/data/v2_0/minimal.city.json")?;
    let _ = json::from_stream(Cursor::new(stream))?;

    Ok(())
}

#[test]
fn citymodel_constructors_are_aliases_for_the_default_json_path() {
    let citymodel_from_slice: fn(&[u8]) -> cjlib::Result<cjlib::CityModel> =
        cjlib::CityModel::from_slice;
    let json_from_slice: fn(&[u8]) -> cjlib::Result<cjlib::CityModel> = json::from_slice;

    let _ = citymodel_from_slice;
    let _ = json_from_slice;
}

#[test]
fn explicit_json_module_owns_serialization() -> cjlib::Result<()> {
    let model = json::from_file("tests/data/v2_0/minimal.city.json")?;

    let bytes = json::to_vec(&model)?;
    let text = json::to_string(&model)?;

    let mut writer = Vec::new();
    json::to_writer(&mut writer, &model)?;

    assert!(!bytes.is_empty());
    assert!(!text.is_empty());
    assert!(!writer.is_empty());

    Ok(())
}
