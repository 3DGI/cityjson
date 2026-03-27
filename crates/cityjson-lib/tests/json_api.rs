//! Public API contract for the explicit `cjlib::json` boundary layer.

use std::io::Cursor;

use cjlib::{CityJSONVersion, json};

#[test]
fn explicit_json_module_supports_document_and_stream_loading() -> cjlib::Result<()> {
    let document = br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#;
    let stream = br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}
{"type":"CityJSONFeature","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}
{"type":"CityJSONFeature","CityObjects":{"feature-2":{"type":"BuildingPart"}},"vertices":[]}
"#;
    let feature = br#"{"type":"CityJSONFeature","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}"#;

    let probe = json::probe(document)?;
    assert_eq!(probe.kind(), json::RootKind::CityJSON);
    assert_eq!(probe.version(), Some(CityJSONVersion::V2_0));

    let _ = json::from_slice(document)?;
    let _ = json::from_feature_slice(feature)?;
    let _ = json::from_file("tests/data/v2_0/minimal.city.json")?;
    let _ = json::from_feature_file("tests/data/v2_0/minimal.city.jsonl")?;
    let models =
        json::read_feature_stream(Cursor::new(stream))?.collect::<cjlib::Result<Vec<_>>>()?;
    assert_eq!(models.len(), 2);

    let mut writer = Vec::new();
    json::write_feature_stream(&mut writer, models.clone())?;

    let output = String::from_utf8(writer).expect("feature stream output is valid UTF-8");
    let expected = models
        .iter()
        .map(json::to_feature_string)
        .collect::<cjlib::Result<Vec<_>>>()?
        .join("\n")
        + "\n";
    assert_eq!(output, expected);

    Ok(())
}

#[test]
fn explicit_json_module_can_materialize_standalone_features_with_a_base_document()
-> cjlib::Result<()> {
    let document = br#"{
        "type":"CityJSON",
        "version":"2.0",
        "transform":{"scale":[0.5,0.5,1.0],"translate":[10.0,20.0,30.0]},
        "CityObjects":{},
        "vertices":[]
    }"#;
    let feature = br#"{
        "type":"CityJSONFeature",
        "CityObjects":{"feature-1":{"type":"Building","geometry":[{"type":"MultiSurface","boundaries":[[[0,1,2]]]}]}},
        "vertices":[[0,0,0],[2,0,0],[2,4,5]]
    }"#;

    let model = json::from_feature_slice_with_base(feature, document)?;
    let vertices = model.as_inner().vertices();
    let v0 = vertices.as_slice()[0].to_array();
    let v2 = vertices.as_slice()[2].to_array();

    assert_eq!(v0, [10.0, 20.0, 30.0]);
    assert_eq!(v2, [11.0, 22.0, 35.0]);

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

#[test]
fn document_loading_rejects_jsonl_streams() {
    let error = json::from_file("tests/data/v1_1/fake.city.jsonl").unwrap_err();
    assert_eq!(error.kind(), cjlib::ErrorKind::Unsupported);
}
