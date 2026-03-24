use std::io::Cursor;
use std::path::PathBuf;

use cjlib::{CityJSONVersion, CityModel, CityModelType};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("v2_0")
        .join(name)
}

fn cityobject_handle_by_id(model: &CityModel, id: &str) -> cjlib::prelude::CityObjectHandle {
    model
        .cityobjects()
        .iter()
        .find_map(|(handle, cityobject)| (cityobject.id() == id).then_some(handle))
        .unwrap_or_else(|| panic!("missing city object {id}"))
}

#[test]
fn new_citymodel_is_a_thin_v2_facade() {
    let model = CityModel::new(CityModelType::CityJSON);
    assert_eq!(model.type_citymodel(), CityModelType::CityJSON);
    assert_eq!(
        model.version(),
        Some(cjlib::cityjson::CityJSONVersion::V2_0)
    );
}

#[test]
fn version_aliases_are_preserved() {
    assert_eq!(
        CityJSONVersion::try_from("1.0.3").unwrap(),
        CityJSONVersion::V1_0
    );
    assert_eq!(
        CityJSONVersion::try_from("1.1.2").unwrap(),
        CityJSONVersion::V1_1
    );
    assert_eq!(
        CityJSONVersion::try_from("2.0.1").unwrap(),
        CityJSONVersion::V2_0
    );
}

#[test]
fn from_slice_imports_a_v2_document() {
    let bytes = std::fs::read(fixture_path("minimal.city.json")).unwrap();
    let model = CityModel::from_slice(&bytes).unwrap();

    assert_eq!(model.cityobjects().len(), 2);
    assert_eq!(model.material_count(), 1);
    assert_eq!(model.texture_count(), 1);
    assert_eq!(model.semantic_count(), 1);
    assert_eq!(model.transform().unwrap().scale(), [0.5, 0.5, 1.0]);
    assert_eq!(model.metadata().unwrap().title(), Some("Facade Fixture"));
    assert!(model.extensions().unwrap().get("Noise").is_some());
    assert!(model.extra().unwrap().contains_key("custom-root"));

    let building = model
        .cityobjects()
        .get(cityobject_handle_by_id(&model, "building-1"))
        .unwrap();
    assert_eq!(building.type_cityobject().to_string(), "Building");
    assert_eq!(
        building
            .attributes()
            .unwrap()
            .get("name")
            .unwrap()
            .to_string(),
        "\"Main\""
    );
    assert_eq!(building.children().unwrap().len(), 1);

    let geometry_handle = building.geometry().unwrap()[0];
    let geometry = model.get_geometry(geometry_handle).unwrap();
    assert_eq!(geometry.type_geometry().to_string(), "MultiSurface");
    assert_eq!(geometry.lod().unwrap().to_string(), "2.2");
    assert!(geometry.semantics().is_some());
    assert!(geometry.materials().is_some());
    assert!(geometry.textures().is_some());

    let part = model
        .cityobjects()
        .get(cityobject_handle_by_id(&model, "building-part-1"))
        .unwrap();
    assert_eq!(part.parents().unwrap().len(), 1);

    let first_vertex = model.get_vertex(cjlib::v2_0::VertexIndex::new(0)).unwrap();
    assert_eq!(first_vertex.x(), 10.0);
    assert_eq!(first_vertex.y(), 20.0);
}

#[test]
fn from_file_dispatches_document_and_jsonl_paths() {
    let document = CityModel::from_file(fixture_path("minimal.city.json")).unwrap();
    let stream = CityModel::from_file(fixture_path("stream.city.jsonl")).unwrap();

    assert_eq!(document.cityobjects().len(), 2);
    assert_eq!(stream.cityobjects().len(), 2);
}

#[test]
fn from_stream_merges_features_strictly() {
    let stream = std::fs::read_to_string(fixture_path("stream.city.jsonl")).unwrap();
    let model = CityModel::from_stream(Cursor::new(stream)).unwrap();

    assert_eq!(model.cityobjects().len(), 2);
    assert_eq!(model.geometry_count(), 2);

    let feature_2 = model
        .cityobjects()
        .get(cityobject_handle_by_id(&model, "feature-2"))
        .unwrap();
    assert_eq!(feature_2.parents().unwrap().len(), 1);
}

#[test]
fn from_slice_rejects_cityjsonfeature() {
    let error =
        CityModel::from_slice(br#"{"type":"CityJSONFeature","CityObjects":{},"vertices":[]}"#)
            .unwrap_err();
    assert_eq!(
        error.to_string(),
        "expected a CityJSON object, found CityJSONFeature"
    );
}

#[test]
fn from_slice_requires_version() {
    let error = CityModel::from_slice(br#"{"type":"CityJSON","CityObjects":{},"vertices":[]}"#)
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "CityJSON object must contain a version member"
    );
}

#[test]
fn from_stream_rejects_duplicate_cityobject_ids() {
    let stream = r#"
{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}
{"type":"CityJSONFeature","CityObjects":{"dup":{"type":"Building"}},"vertices":[]}
{"type":"CityJSONFeature","CityObjects":{"dup":{"type":"Building"}},"vertices":[]}
"#;

    let error = CityModel::from_stream(Cursor::new(stream)).unwrap_err();
    assert!(error.to_string().contains("duplicate city object id"));
}

#[test]
fn from_stream_rejects_mixed_versions() {
    let stream = r#"
{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}
{"type":"CityJSONFeature","version":"1.1","CityObjects":{"a":{"type":"Building"}},"vertices":[]}
"#;

    let error = CityModel::from_stream(Cursor::new(stream)).unwrap_err();
    assert!(error.to_string().contains("mixed CityJSON versions"));
}

#[test]
fn legacy_version_dispatch_is_still_todo() {
    let payload = br#"{"type":"CityJSON","version":"1.1","CityObjects":{},"vertices":[]}"#;
    let result = std::panic::catch_unwind(|| {
        let _ = CityModel::from_slice(payload);
    });
    assert!(result.is_err());
}
