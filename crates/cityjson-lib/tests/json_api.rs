//! Public API contract for the explicit `cjlib::json` boundary layer.

use std::io::Cursor;

use cjlib::cityjson::v2_0::{BBox, Transform};
use serde_json::value::RawValue;

use cjlib::{CityJSONVersion, json};

#[test]
fn explicit_json_module_supports_document_and_stream_loading() -> cjlib::Result<()> {
    let document = br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#;
    let stream = br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}
{"type":"CityJSONFeature","id":"feature-1","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}
{"type":"CityJSONFeature","id":"feature-2","CityObjects":{"feature-2":{"type":"BuildingPart"}},"vertices":[]}
"#;
    let feature = br#"{"type":"CityJSONFeature","id":"feature-1","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}"#;

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
fn explicit_json_module_can_write_strict_cityjsonseq_with_explicit_transform() -> cjlib::Result<()>
{
    let base_root = json::from_slice(
        br#"{
            "type":"CityJSON",
            "version":"2.0",
            "metadata":{"title":"base-root"},
            "CityObjects":{},
            "vertices":[]
        }"#,
    )?;
    let feature = json::from_feature_slice(
        br#"{
            "type":"CityJSONFeature",
            "id":"feature-1",
            "metadata":{"title":"base-root"},
            "CityObjects":{
                "feature-1":{
                    "type":"Building",
                    "geometry":[{"type":"MultiPoint","boundaries":[0,1]}]
                }
            },
            "vertices":[[10,20,30],[12,22,31]]
        }"#,
    )?;

    let mut transform = Transform::new();
    transform.set_scale([0.5, 0.5, 1.0]);
    transform.set_translate([10.0, 20.0, 30.0]);

    let mut output = Vec::new();
    let report = json::write_cityjsonseq_refs(
        &mut output,
        &base_root,
        [&feature],
        &transform,
        json::CityJSONSeqWriteOptions::default(),
    )?;

    assert_eq!(report.feature_count, 1);
    assert_eq!(report.cityobject_count, 1);
    assert_eq!(
        report.geographical_extent,
        Some(BBox::new(10.0, 20.0, 30.0, 12.0, 22.0, 31.0))
    );

    let items = serde_json::Deserializer::from_slice(&output)
        .into_iter::<serde_json::Value>()
        .collect::<serde_json::Result<Vec<_>>>()
        .expect("strict CityJSONSeq output should parse");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["type"], "CityJSON");
    assert_eq!(items[0]["metadata"]["title"], "base-root");
    assert_eq!(
        items[0]["metadata"]["geographicalExtent"],
        serde_json::json!([10.0, 20.0, 30.0, 12.0, 22.0, 31.0])
    );
    assert_eq!(items[1]["type"], "CityJSONFeature");
    assert!(items[1].get("transform").is_none());
    assert_eq!(
        items[1]["vertices"],
        serde_json::json!([[0, 0, 0], [4, 4, 1]])
    );

    let roundtrip =
        json::read_cityjsonseq(Cursor::new(output))?.collect::<cjlib::Result<Vec<_>>>()?;
    assert_eq!(roundtrip.len(), 1);
    assert_eq!(
        roundtrip[0]
            .as_inner()
            .metadata()
            .and_then(|metadata| metadata.title()),
        Some("base-root")
    );

    Ok(())
}

#[test]
fn explicit_json_module_can_write_strict_cityjsonseq_with_auto_transform() -> cjlib::Result<()> {
    let base_root = json::from_slice(
        br#"{
            "type":"CityJSON",
            "version":"2.0",
            "metadata":{"title":"base-root"},
            "CityObjects":{},
            "vertices":[]
        }"#,
    )?;
    let feature_a = json::from_feature_slice(
        br#"{
            "type":"CityJSONFeature",
            "id":"feature-a",
            "metadata":{"title":"base-root"},
            "CityObjects":{
                "feature-a":{
                    "type":"Building",
                    "geometry":[{"type":"MultiPoint","boundaries":[0,1]}]
                }
            },
            "vertices":[[10,20,30],[12,23,35]]
        }"#,
    )?;
    let feature_b = json::from_feature_slice(
        br#"{
            "type":"CityJSONFeature",
            "id":"feature-b",
            "metadata":{"title":"base-root"},
            "CityObjects":{
                "feature-b":{
                    "type":"BuildingPart",
                    "geometry":[{"type":"MultiPoint","boundaries":[0]}]
                }
            },
            "vertices":[[9,21,40]]
        }"#,
    )?;

    let mut output = Vec::new();
    let report = json::write_cityjsonseq_auto_transform_refs(
        &mut output,
        &base_root,
        [&feature_a, &feature_b],
        json::AutoTransformOptions {
            scale: [0.5, 1.0, 5.0],
            ..Default::default()
        },
    )?;

    assert_eq!(
        report.geographical_extent,
        Some(BBox::new(9.0, 20.0, 30.0, 12.0, 23.0, 40.0))
    );
    assert_eq!(report.transform.scale(), [0.5, 1.0, 5.0]);
    assert_eq!(report.transform.translate(), [9.0, 20.0, 30.0]);

    let items = serde_json::Deserializer::from_slice(&output)
        .into_iter::<serde_json::Value>()
        .collect::<serde_json::Result<Vec<_>>>()
        .expect("strict CityJSONSeq output should parse");
    assert_eq!(
        items[0]["transform"]["translate"],
        serde_json::json!([9.0, 20.0, 30.0])
    );

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
        "id":"feature-1",
        "CityObjects":{"feature-1":{"type":"Building","geometry":[{"type":"MultiSurface","boundaries":[[[0,1,2]]]}]}},
        "vertices":[[0,0,0],[2,0,0],[2,4,5]]
    }"#;

    let model = json::staged::from_feature_slice_with_base(feature, document)?;
    let vertices = model.as_inner().vertices();
    let v0 = vertices.as_slice()[0].to_array();
    let v2 = vertices.as_slice()[2].to_array();

    assert_eq!(v0, [10.0, 20.0, 30.0]);
    assert_eq!(v2, [11.0, 22.0, 35.0]);

    Ok(())
}

#[test]
fn explicit_json_module_can_materialize_feature_parts_with_a_base_document() -> cjlib::Result<()> {
    let document = br#"{
        "type":"CityJSON",
        "version":"2.0",
        "transform":{"scale":[0.5,0.5,1.0],"translate":[10.0,20.0,30.0]},
        "metadata":{"title":"base-root"},
        "CityObjects":{},
        "vertices":[]
    }"#;
    let object = RawValue::from_string(
        r#"{"type":"Building","geometry":[{"type":"MultiSurface","boundaries":[[[0,2,1]]]}]}"#
            .to_owned(),
    )
    .expect("raw feature object");
    let cityobjects = [json::staged::FeatureObjectFragment {
        id: "feature-1",
        object: object.as_ref(),
    }];
    let vertices = [[0, 0, 0], [2, 0, 0], [1, 0, 0]];
    let parts = json::staged::FeatureAssembly {
        id: "feature-1",
        cityobjects: &cityobjects,
        vertices: &vertices,
    };

    let model = json::staged::from_feature_assembly_with_base(parts, document)?;
    let vertices = model.as_inner().vertices();
    let text = json::to_string(&model)?;
    let output: serde_json::Value = serde_json::from_str(&text)?;

    assert_eq!(output["metadata"]["title"], "base-root");
    assert_eq!(
        output["vertices"],
        serde_json::json!([[0, 0, 0], [2, 0, 0], [1, 0, 0]])
    );
    assert_eq!(vertices.as_slice()[0].to_array(), [10.0, 20.0, 30.0]);
    assert_eq!(vertices.as_slice()[2].to_array(), [10.5, 20.0, 30.0]);

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
fn explicit_json_module_can_write_features_without_building_a_string() -> cjlib::Result<()> {
    let feature = br#"{"type":"CityJSONFeature","id":"feature-1","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}"#;
    let model = json::from_feature_slice(feature)?;

    let mut writer = Vec::new();
    json::to_feature_writer(&mut writer, &model)?;

    assert_eq!(
        String::from_utf8(writer).expect("feature writer output is valid UTF-8"),
        json::to_feature_string(&model)?,
    );

    Ok(())
}

#[test]
fn document_loading_rejects_jsonl_streams() {
    let error = json::from_file("tests/data/v1_1/fake.city.jsonl").unwrap_err();
    assert_eq!(error.kind(), cjlib::ErrorKind::Unsupported);
}
