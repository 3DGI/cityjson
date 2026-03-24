use std::path::PathBuf;

use once_cell::sync::Lazy;
use serde_json::{json, Map, Value};

use common::*;
use serde_cityjson::from_str_owned;

mod common;

static DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    cargo_workspace_directory()
        .unwrap()
        .join("tests")
        .join("data")
        .join("v2_0")
});

fn dummy_vertices(count: usize) -> Value {
    Value::Array(
        (0..count)
            .map(|index| json!([index as f64, 0.0, 0.0]))
            .collect(),
    )
}

fn dummy_uv_vertices(count: usize) -> Value {
    Value::Array(
        (0..count)
            .map(|index| json!([index as f64 / 100.0, 0.0]))
            .collect(),
    )
}

fn max_u64_in(value: &Value) -> Option<u64> {
    match value {
        Value::Array(values) => values.iter().filter_map(max_u64_in).max(),
        Value::Object(values) => values.values().filter_map(max_u64_in).max(),
        Value::Number(number) => number.as_u64(),
        _ => None,
    }
}

fn base_citymodel() -> Map<String, Value> {
    let mut root = Map::new();
    root.insert("type".to_owned(), Value::String("CityJSON".to_owned()));
    root.insert("version".to_owned(), Value::String("2.0".to_owned()));
    root.insert("CityObjects".to_owned(), Value::Object(Map::new()));
    root.insert("vertices".to_owned(), Value::Array(Vec::new()));
    root
}

fn wrap_transform(value: Value) -> Value {
    let mut root = base_citymodel();
    root.insert("transform".to_owned(), value);
    Value::Object(root)
}

fn wrap_cityobject(value: Value) -> Value {
    let mut root = base_citymodel();
    let mut cityobjects = Map::new();
    cityobjects.insert("fixture".to_owned(), value.clone());

    if let Some(object) = value.as_object() {
        for key in ["children", "parents"] {
            if let Some(ids) = object.get(key).and_then(Value::as_array) {
                for id in ids.iter().filter_map(Value::as_str) {
                    cityobjects
                        .entry(id.to_owned())
                        .or_insert_with(|| json!({ "type": "GenericCityObject" }));
                }
            }
        }
    }

    root.insert("CityObjects".to_owned(), Value::Object(cityobjects));
    Value::Object(root)
}

fn wrap_geometry(value: Value) -> Value {
    let vertex_count = max_u64_in(value.get("boundaries").unwrap_or(&Value::Null))
        .map(|max| max as usize + 1)
        .unwrap_or(1);

    let mut root = base_citymodel();
    root.insert("vertices".to_owned(), dummy_vertices(vertex_count));
    root.insert(
        "CityObjects".to_owned(),
        json!({
            "fixture": {
                "type": "GenericCityObject",
                "geometry": [value.clone()]
            }
        }),
    );

    if value.get("material").is_some() || value.get("texture").is_some() {
        let material_count = value
            .get("material")
            .and_then(max_u64_in)
            .map(|max| max as usize + 1)
            .unwrap_or(0);
        let texture_index_bound = value
            .get("texture")
            .and_then(max_u64_in)
            .map(|max| max as usize + 1)
            .unwrap_or(0);

        let mut appearance = Map::new();
        if material_count > 0 {
            appearance.insert(
                "materials".to_owned(),
                Value::Array(
                    (0..material_count)
                        .map(|index| json!({ "name": format!("material-{index}") }))
                        .collect(),
                ),
            );
        }
        if texture_index_bound > 0 {
            appearance.insert(
                "textures".to_owned(),
                Value::Array(
                    (0..texture_index_bound)
                        .map(|index| {
                            json!({
                                "type": "PNG",
                                "image": format!("texture-{index}.png")
                            })
                        })
                        .collect(),
                ),
            );
            appearance.insert(
                "vertices-texture".to_owned(),
                dummy_uv_vertices(texture_index_bound),
            );
        }
        root.insert("appearance".to_owned(), Value::Object(appearance));
    }

    if value.get("type").and_then(Value::as_str) == Some("GeometryInstance") {
        root.insert(
            "geometry-templates".to_owned(),
            json!({
                "templates": [{
                    "type": "MultiPoint",
                    "lod": "1",
                    "boundaries": [0]
                }],
                "vertices-templates": [[0.0, 0.0, 0.0]]
            }),
        );
    }

    Value::Object(root)
}

fn wrap_appearance(value: Value) -> Value {
    let mut root = base_citymodel();
    root.insert("appearance".to_owned(), value);
    Value::Object(root)
}

fn wrap_material(value: Value) -> Value {
    wrap_appearance(json!({ "materials": [value] }))
}

fn wrap_texture(value: Value) -> Value {
    wrap_appearance(json!({ "textures": [value], "vertices-texture": [] }))
}

fn wrap_geometry_templates(value: Value) -> Value {
    let mut root = base_citymodel();
    root.insert("geometry-templates".to_owned(), value);
    Value::Object(root)
}

fn wrap_semantic_minimal(value: Value) -> Value {
    let mut root = base_citymodel();
    root.insert("vertices".to_owned(), dummy_vertices(3));
    root.insert(
        "CityObjects".to_owned(),
        json!({
            "fixture": {
                "type": "GenericCityObject",
                "geometry": [{
                    "type": "MultiSurface",
                    "lod": "1",
                    "boundaries": [[[0, 1, 2]]],
                    "semantics": {
                        "surfaces": [value],
                        "values": [0]
                    }
                }]
            }
        }),
    );
    Value::Object(root)
}

fn wrap_semantic_extended(value: Value) -> Value {
    let mut surfaces = vec![json!({ "type": "WallSurface" }); 38];
    surfaces[1] = value;

    let mut root = base_citymodel();
    root.insert("vertices".to_owned(), dummy_vertices(3));
    root.insert(
        "CityObjects".to_owned(),
        json!({
            "fixture": {
                "type": "GenericCityObject",
                "geometry": [{
                    "type": "MultiSurface",
                    "lod": "1",
                    "boundaries": [[[0, 1, 2]]],
                    "semantics": {
                        "surfaces": surfaces,
                        "values": [1]
                    }
                }]
            }
        }),
    );
    Value::Object(root)
}

fn wrap_vertices(value: Value) -> Value {
    let mut root = base_citymodel();
    root.insert("vertices".to_owned(), value);
    Value::Object(root)
}

fn wrap_metadata(value: Value) -> Value {
    let mut root = base_citymodel();
    root.insert("metadata".to_owned(), value);
    Value::Object(root)
}

fn wrap_extension(value: Value) -> Value {
    let mut root = base_citymodel();
    root.insert("extensions".to_owned(), json!({ "Noise": value }));
    Value::Object(root)
}

fn extract_transform(value: &Value) -> Value {
    value.get("transform").cloned().unwrap()
}

fn extract_cityobject(value: &Value) -> Value {
    value
        .get("CityObjects")
        .and_then(Value::as_object)
        .and_then(|cityobjects| cityobjects.get("fixture"))
        .cloned()
        .unwrap()
}

fn extract_geometry(value: &Value) -> Value {
    value
        .get("CityObjects")
        .and_then(Value::as_object)
        .and_then(|cityobjects| cityobjects.get("fixture"))
        .and_then(|fixture| fixture.get("geometry"))
        .and_then(Value::as_array)
        .and_then(|geometry| geometry.first())
        .cloned()
        .unwrap()
}

fn extract_appearance(value: &Value) -> Value {
    value.get("appearance").cloned().unwrap_or(Value::Null)
}

fn extract_material(value: &Value) -> Value {
    value
        .get("appearance")
        .and_then(|appearance| appearance.get("materials"))
        .and_then(Value::as_array)
        .and_then(|materials| materials.first())
        .cloned()
        .unwrap()
}

fn extract_texture(value: &Value) -> Value {
    value
        .get("appearance")
        .and_then(|appearance| appearance.get("textures"))
        .and_then(Value::as_array)
        .and_then(|textures| textures.first())
        .cloned()
        .unwrap()
}

fn extract_geometry_templates(value: &Value) -> Value {
    value.get("geometry-templates").cloned().unwrap()
}

fn extract_semantic_minimal(value: &Value) -> Value {
    value
        .get("CityObjects")
        .and_then(Value::as_object)
        .and_then(|cityobjects| cityobjects.get("fixture"))
        .and_then(|fixture| fixture.get("geometry"))
        .and_then(Value::as_array)
        .and_then(|geometry| geometry.first())
        .and_then(|geometry| geometry.get("semantics"))
        .and_then(|semantics| semantics.get("surfaces"))
        .and_then(Value::as_array)
        .and_then(|surfaces| surfaces.first())
        .cloned()
        .unwrap()
}

fn extract_semantic_extended(value: &Value) -> Value {
    value
        .get("CityObjects")
        .and_then(Value::as_object)
        .and_then(|cityobjects| cityobjects.get("fixture"))
        .and_then(|fixture| fixture.get("geometry"))
        .and_then(Value::as_array)
        .and_then(|geometry| geometry.first())
        .and_then(|geometry| geometry.get("semantics"))
        .and_then(|semantics| semantics.get("surfaces"))
        .and_then(Value::as_array)
        .and_then(|surfaces| surfaces.get(1))
        .cloned()
        .unwrap()
}

fn extract_vertices(value: &Value) -> Value {
    value.get("vertices").cloned().unwrap()
}

fn extract_metadata(value: &Value) -> Value {
    value.get("metadata").cloned().unwrap()
}

fn extract_extension(value: &Value) -> Value {
    value
        .get("extensions")
        .and_then(|extensions| extensions.get("Noise"))
        .cloned()
        .unwrap()
}

#[test]
fn citymodel_fake_complete() {
    let json_input = read_to_string(DATA_DIR.join("cityjson_fake_complete.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn citymodel_fake_complete_deserialize() {
    let json_input = read_to_string(DATA_DIR.join("cityjson_fake_complete.city.json"));
    let cm = from_str_owned(&json_input).unwrap();
    assert!(cm.vertices().len() > 0);
    assert!(cm.extensions().is_some());
    assert!(cm.metadata().is_some());
    assert!(cm.cityobjects().len() > 0);
    assert!(
        cm.material_count() > 0
            || cm.texture_count() > 0
            || cm.default_material_theme().is_some()
            || cm.default_texture_theme().is_some()
            || !cm.vertices_texture().is_empty()
    );
    assert!(cm.geometry_template_count() > 0);
}

#[test]
fn citymodel_minimal_complete() {
    let json_input = read_to_string(DATA_DIR.join("cityjson_minimal_complete.city.json"));
    let cm = from_str_owned(&json_input).unwrap();
    assert!(cm.extra().is_none());
    assert_eq_roundtrip(&json_input);
}

#[test]
fn cityjsonfeature_minimal_complete() {
    let json_input = read_to_string(DATA_DIR.join("cityjsonfeature_minimal_complete.city.jsonl"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn transform() {
    let json_input = read_to_string(DATA_DIR.join("transform.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_transform, extract_transform);
}

#[test]
fn cityobject_complete() {
    let json_input = read_to_string(DATA_DIR.join("cityobject_complete.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_cityobject, extract_cityobject);
}

#[test]
fn cityobject_extended() {
    let json_input = read_to_string(DATA_DIR.join("cityobject_extended.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_cityobject, extract_cityobject);
}

#[test]
fn geometry_instance() {
    let json_input = read_to_string(DATA_DIR.join("geometry_instance.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_geometry, extract_geometry);
}

#[test]
fn geometry_complete_solid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_complete_solid.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_geometry, extract_geometry);
}

#[test]
fn geometry_material_solid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_material_solid.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_geometry, extract_geometry);
}

#[test]
fn geometry_texture_multisolid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_texture_multisolid.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_geometry, extract_geometry);
}

#[test]
fn geometry_texture_solid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_texture_solid.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_geometry, extract_geometry);
}

#[test]
fn geometry_texture_multisurface() {
    let json_input = read_to_string(DATA_DIR.join("geometry_texture_multisurface.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_geometry, extract_geometry);
}

#[test]
fn geometry_semantics_multisolid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multisolid.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_geometry, extract_geometry);
}

#[test]
fn geometry_semantics_solid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_solid.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_geometry, extract_geometry);
}

#[test]
fn geometry_semantics_multisurface() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multisurface.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_geometry, extract_geometry);
}

#[test]
fn geometry_semantics_multilinestring() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multilinestring.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_geometry, extract_geometry);
}

#[test]
fn geometry_semantics_multipoint() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multipoint.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_geometry, extract_geometry);
}

#[test]
fn appearance_minimal_complete() {
    let json_input = read_to_string(DATA_DIR.join("appearance_minimal_complete.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_appearance, extract_appearance);
}

#[test]
fn appearance_empty() {
    let json_input = read_to_string(DATA_DIR.join("appearance_empty.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_appearance, extract_appearance);
}

#[test]
fn material_minimal() {
    let json_input = read_to_string(DATA_DIR.join("material_minimal.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_material, extract_material);
}

#[test]
fn material_complete() {
    let json_input = read_to_string(DATA_DIR.join("material_complete.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_material, extract_material);
}

#[test]
fn texture_complete() {
    let json_input = read_to_string(DATA_DIR.join("texture_complete.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_texture, extract_texture);
}

#[test]
fn texture_minimal() {
    let json_input = read_to_string(DATA_DIR.join("texture_minimal.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_texture, extract_texture);
}

#[test]
fn geometry_templates() {
    let json_input = read_to_string(DATA_DIR.join("geometry_templates.city.json"));
    assert_eq_roundtrip_wrapped(
        &json_input,
        wrap_geometry_templates,
        extract_geometry_templates,
    );
}

#[test]
fn semantic_minimal() {
    let json_input = read_to_string(DATA_DIR.join("semantic_minimal.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_semantic_minimal, extract_semantic_minimal);
}

#[test]
fn semantic_extended() {
    let json_input = read_to_string(DATA_DIR.join("semantic_extended.city.json"));
    assert_eq_roundtrip_wrapped(
        &json_input,
        wrap_semantic_extended,
        extract_semantic_extended,
    );
}

#[test]
fn vertices() {
    let json_input = read_to_string(DATA_DIR.join("vertices.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_vertices, extract_vertices);
}

#[test]
fn metadata_empty() {
    let json_input = read_to_string(DATA_DIR.join("metadata_empty.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_metadata, extract_metadata);
}

#[test]
fn metadata_complete() {
    let json_input = read_to_string(DATA_DIR.join("metadata_complete.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_metadata, extract_metadata);
}

#[test]
fn metadata_poc_minimal() {
    let json_input = read_to_string(DATA_DIR.join("metadata_poc_minimal.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_metadata, extract_metadata);
}

#[test]
fn metadata_extra_properties() {
    let json_input = read_to_string(DATA_DIR.join("metadata_extra_properties.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_metadata, extract_metadata);
}

#[test]
fn extension() {
    let json_input = read_to_string(DATA_DIR.join("extension.city.json"));
    assert_eq_roundtrip_wrapped(&json_input, wrap_extension, extract_extension);
}
