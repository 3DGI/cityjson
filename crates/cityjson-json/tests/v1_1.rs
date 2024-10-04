use std::path::PathBuf;

use once_cell::sync::Lazy;

use common::*;
use serde_cityjson::v1_1::*;

mod common;

static DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    cargo_workspace_directory()
        .unwrap()
        .join("tests")
        .join("data")
        .join("v1_1")
});

#[test]
fn citymodel_dummy_complete() {
    let json_input = read_to_string(DATA_DIR.join("cityjson_dummy_complete.city.json"));
    assert_eq_roundtrip::<CityModel>(&json_input);
}

// Can we deserialize all objects as they should be?
#[test]
fn citymodel_dummy_complete_deserialize() {
    let json_input = read_to_string(DATA_DIR.join("cityjson_dummy_complete.city.json"));
    let cm: CityModel = serde_json::from_str(&json_input).unwrap();
    assert!(cm.vertices.len() > 0);
    assert!(cm.extensions.is_some());
    assert!(cm.metadata.is_some());
    assert!(cm.cityobjects.len() > 0);
    assert!(cm.appearance.is_some());
    assert!(cm.geometry_templates.is_some());
    // todo: I think this should be None if there are no extra root properties, but that might not be possible with serde flatten etc.
    // assert!(cm.extra.is_none());
}

#[test]
fn citymodel_minimal_complete() {
    let json_input = read_to_string(DATA_DIR.join("cityjson_minimal_complete.city.json"));
    assert_eq_roundtrip::<CityModel>(&json_input);
}

#[test]
fn cityjsonfeature_minimal_complete() {
    let json_input = read_to_string(DATA_DIR.join("cityjsonfeature_minimal_complete.city.jsonl"));
    assert_eq_roundtrip::<CityModel>(&json_input);
}

#[test]
fn transform() {
    let json_input = read_to_string(DATA_DIR.join("transform.city.json"));
    assert_eq_roundtrip::<Transform>(&json_input);
}

#[test]
fn cityobject_complete() {
    let json_input = read_to_string(DATA_DIR.join("cityobject_complete.city.json"));
    assert_eq_roundtrip::<CityObject>(&json_input);
}

#[test]
fn cityobject_extended() {
    let json_input = read_to_string(DATA_DIR.join("cityobject_extended.city.json"));
    assert_eq_roundtrip::<CityObject>(&json_input);
}

#[test]
fn geometry_instance() {
    let json_input = read_to_string(DATA_DIR.join("geometry_instance.city.json"));
    assert_eq_roundtrip::<Geometry>(&json_input);
}

#[test]
fn geometry_complete_solid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_complete_solid.city.json"));
    assert_eq_roundtrip::<Geometry>(&json_input);
}

#[test]
fn geometry_material_solid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_material_solid.city.json"));
    assert_eq_roundtrip::<Geometry>(&json_input);
}

#[test]
fn geometry_texture_multisolid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_texture_multisolid.city.json"));
    assert_eq_roundtrip::<Geometry>(&json_input);
}

#[test]
fn geometry_texture_solid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_texture_solid.city.json"));
    assert_eq_roundtrip::<Geometry>(&json_input);
}

#[test]
fn geometry_texture_multisurface() {
    let json_input = read_to_string(DATA_DIR.join("geometry_texture_multisurface.city.json"));
    assert_eq_roundtrip::<Geometry>(&json_input);
}

#[test]
fn geometry_semantics_multisolid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multisolid.city.json"));
    assert_eq_roundtrip::<Geometry>(&json_input);
}

#[test]
fn geometry_semantics_solid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_solid.city.json"));
    assert_eq_roundtrip::<Geometry>(&json_input);
}

#[test]
fn geometry_semantics_multisurface() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multisurface.city.json"));
    assert_eq_roundtrip::<Geometry>(&json_input);
}

#[test]
fn geometry_semantics_multilinestring() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multilinestring.city.json"));
    assert_eq_roundtrip::<Geometry>(&json_input);
}

#[test]
fn geometry_semantics_multipoint() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multipoint.city.json"));
    assert_eq_roundtrip::<Geometry>(&json_input);
}

#[test]
fn appearance_minimal_complete() {
    let json_input = read_to_string(DATA_DIR.join("appearance_minimal_complete.city.json"));
    assert_eq_roundtrip::<Appearance>(&json_input);
}

#[test]
fn appearance_empty() {
    let json_input = read_to_string(DATA_DIR.join("appearance_empty.city.json"));
    assert_eq_roundtrip::<Appearance>(&json_input);
}

#[test]
fn material_minimal() {
    let json_input = read_to_string(DATA_DIR.join("material_minimal.city.json"));
    assert_eq_roundtrip::<Material>(&json_input);
}

#[test]
fn material_complete() {
    let json_input = read_to_string(DATA_DIR.join("material_complete.city.json"));
    assert_eq_roundtrip::<Material>(&json_input);
}

#[test]
fn texture_complete() {
    let json_input = read_to_string(DATA_DIR.join("texture_complete.city.json"));
    assert_eq_roundtrip::<Texture>(&json_input);
}

#[test]
fn texture_minimal() {
    let json_input = read_to_string(DATA_DIR.join("texture_minimal.city.json"));
    assert_eq_roundtrip::<Texture>(&json_input);
}

#[test]
fn geometry_templates() {
    let json_input = read_to_string(DATA_DIR.join("geometry_templates.city.json"));
    assert_eq_roundtrip::<GeometryTemplates>(&json_input);
}

#[test]
fn semantic_minimal() {
    let json_input = read_to_string(DATA_DIR.join("semantic_minimal.city.json"));
    assert_eq_roundtrip::<Semantic>(&json_input);
}

#[test]
fn semantic_extended() {
    let json_input = read_to_string(DATA_DIR.join("semantic_extended.city.json"));
    assert_eq_roundtrip::<Semantic>(&json_input);
}

#[test]
fn vertices() {
    let json_input = read_to_string(DATA_DIR.join("vertices.city.json"));
    assert_eq_roundtrip::<Vertices>(&json_input);
}

#[test]
fn metadata_empty() {
    let json_input = read_to_string(DATA_DIR.join("metadata_empty.city.json"));
    assert_eq_roundtrip::<Metadata>(&json_input);
}

#[test]
fn metadata_complete() {
    let json_input = read_to_string(DATA_DIR.join("metadata_complete.city.json"));
    assert_eq_roundtrip::<Metadata>(&json_input);
}

#[test]
fn metadata_poc_minimal() {
    let json_input = read_to_string(DATA_DIR.join("metadata_poc_minimal.city.json"));
    assert_eq_roundtrip::<Metadata>(&json_input);
}

#[test]
fn metadata_extra_properties() {
    let json_input = read_to_string(DATA_DIR.join("metadata_extra_properties.city.json"));
    assert_eq_roundtrip::<Metadata>(&json_input);
}

#[test]
fn extension() {
    let json_input = read_to_string(DATA_DIR.join("extension.city.json"));
    assert_eq_roundtrip::<Extension>(&json_input);
}

// #[test]
// fn objects() -> Result<(), String> {
//     let cityjson_path = "resources/data/downloaded/30gz1_04.json";
//     let mut file = File::open(cityjson_path).map_err(|e| e.to_string())?;
//     let mut cityjson_json = String::new();
//     file.read_to_string(&mut cityjson_json)
//         .map_err(|e| e.to_string())?;
//     let cm: CityModel = serde_json::from_str(&cityjson_json).map_err(|e| e.to_string())?;
//     println!("{:?}", &cm.version);
//     Ok(())
// }
