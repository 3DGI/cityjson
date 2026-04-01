use std::path::PathBuf;

use serde_json::value::RawValue;
use serde_json::{json, Value};

use cityjson::v2_0::{
    AffineTransform3D, BBox, CityModelType, CityObject, CityObjectIdentifier, CityObjectType,
    GeometryDraft, ImageType, LoD, OwnedCityModel, PointDraft, RealWorldCoordinate, Texture,
    UVCoordinate,
};
use common::*;
use serde_cityjson::{
    from_feature_str_owned, from_str_borrowed, from_str_owned, merge_feature_stream,
    read_feature_stream, to_string,
    v2_0::{from_feature_parts_owned_with_base, FeatureObject, FeatureParts},
};

mod common;

static DATA_DIR: std::sync::LazyLock<PathBuf> = std::sync::LazyLock::new(|| {
    cargo_workspace_directory()
        .unwrap()
        .join("tests")
        .join("data")
        .join("v2_0")
});

#[test]
fn feature_parts_with_base_materializes_a_self_contained_feature_model() {
    let base = json!({
        "type": "CityJSON",
        "version": "2.0",
        "transform": {
            "scale": [0.5, 0.5, 1.0],
            "translate": [10.0, 20.0, 30.0]
        },
        "metadata": {
            "title": "base-root"
        },
        "CityObjects": {},
        "vertices": []
    });
    let object = RawValue::from_string(
        json!({
            "type": "Building",
            "geometry": [{
                "type": "MultiSurface",
                "lod": "0",
                "boundaries": [[[0, 2, 1]]]
            }]
        })
        .to_string(),
    )
    .expect("raw CityObject");
    let cityobjects = [FeatureObject {
        id: "feature-1",
        object: object.as_ref(),
    }];
    let vertices = [[0, 0, 0], [2, 0, 0], [1, 0, 0]];
    let parts = FeatureParts {
        id: "feature-1",
        cityobjects: &cityobjects,
        vertices: &vertices,
    };

    let model = from_feature_parts_owned_with_base(parts, &base.to_string()).unwrap();
    let vertices = model.vertices();
    let json: Value = serde_json::from_str(&to_string(&model).unwrap()).unwrap();

    assert_eq!(json["metadata"]["title"], "base-root");
    assert_eq!(json["transform"], base["transform"]);
    assert_eq!(json["vertices"], json!([[0, 0, 0], [2, 0, 0], [1, 0, 0]]));
    assert_eq!(
        json["CityObjects"]["feature-1"]["geometry"][0]["boundaries"],
        json!([[[0, 2, 1]]])
    );
    assert_eq!(vertices.as_slice()[0].to_array(), [10.0, 20.0, 30.0]);
    assert_eq!(vertices.as_slice()[2].to_array(), [10.5, 20.0, 30.0]);
}

#[test]
fn feature_parts_with_base_rejects_duplicate_cityobject_ids() {
    let base = json!({
        "type": "CityJSON",
        "version": "2.0",
        "CityObjects": {},
        "vertices": []
    });
    let object = RawValue::from_string(json!({ "type": "Building" }).to_string()).unwrap();
    let cityobjects = [
        FeatureObject {
            id: "feature-1",
            object: object.as_ref(),
        },
        FeatureObject {
            id: "feature-1",
            object: object.as_ref(),
        },
    ];
    let parts = FeatureParts {
        id: "feature-1",
        cityobjects: &cityobjects,
        vertices: &[],
    };

    let error = from_feature_parts_owned_with_base(parts, &base.to_string()).unwrap_err();
    assert!(error
        .to_string()
        .contains("duplicate CityObject id in feature parts"));
}


#[test]
fn serialize_quantizes_root_vertices_only() {
    let mut model = OwnedCityModel::new(CityModelType::CityJSON);
    model.transform_mut();
    model
        .metadata_mut()
        .set_geographical_extent(BBox::new(1.1, 2.2, 3.3, 4.4, 5.5, 6.6));
    model
        .add_vertex(RealWorldCoordinate::new(1.25, 2.5, 3.75))
        .unwrap();
    model
        .add_template_vertex(RealWorldCoordinate::new(4.125, 5.25, 6.875))
        .unwrap();
    model
        .add_uv_coordinate(UVCoordinate::new(0.125, 0.875))
        .unwrap();
    model
        .add_texture(Texture::new("texture.png".to_string(), ImageType::Png))
        .unwrap();

    let json: Value = serde_json::from_str(&to_string(&model).unwrap()).unwrap();

    let root_vertices = json
        .get("vertices")
        .and_then(Value::as_array)
        .expect("root vertices should be present");
    assert_eq!(root_vertices.len(), 1);
    assert!(root_vertices[0]
        .as_array()
        .expect("vertex should be an array")
        .iter()
        .all(|coordinate| coordinate.is_i64() || coordinate.is_u64()));

    let template_vertices = json
        .get("geometry-templates")
        .and_then(Value::as_object)
        .and_then(|templates| templates.get("vertices-templates"))
        .and_then(Value::as_array)
        .expect("template vertices should be present");
    assert_eq!(template_vertices.len(), 1);
    assert!(template_vertices[0]
        .as_array()
        .expect("template vertex should be an array")
        .iter()
        .all(Value::is_f64));

    let texture_vertices = json
        .get("appearance")
        .and_then(Value::as_object)
        .and_then(|appearance| appearance.get("vertices-texture"))
        .and_then(Value::as_array)
        .expect("texture vertices should be present");
    assert_eq!(texture_vertices.len(), 1);
    assert!(texture_vertices[0]
        .as_array()
        .expect("texture vertex should be an array")
        .iter()
        .all(Value::is_f64));

    let extent = json
        .get("metadata")
        .and_then(Value::as_object)
        .and_then(|metadata| metadata.get("geographicalExtent"))
        .and_then(Value::as_array)
        .expect("geographical extent should be present");
    assert_eq!(extent.len(), 6);
    assert!(extent.iter().all(Value::is_f64));
}

#[test]
fn serialize_geometry_instance_keeps_float_sections() {
    let mut model = OwnedCityModel::new(CityModelType::CityJSON);

    let template = GeometryDraft::multi_point(
        Some(LoD::LoD1),
        [PointDraft::new(RealWorldCoordinate::new(0.25, 0.5, 0.75))],
    )
    .insert_template_into(&mut model)
    .unwrap();

    let geometry = GeometryDraft::instance(
        template,
        RealWorldCoordinate::new(1.25, 2.5, 3.75),
        AffineTransform3D::default(),
    )
    .insert_into(&mut model)
    .unwrap();

    let mut cityobject = CityObject::new(
        CityObjectIdentifier::new("instance-1".to_string()),
        CityObjectType::Building,
    );
    cityobject.add_geometry(geometry);
    model.cityobjects_mut().add(cityobject).unwrap();

    let json: Value = serde_json::from_str(&to_string(&model).unwrap()).unwrap();
    let root_vertices = json
        .get("vertices")
        .and_then(Value::as_array)
        .expect("root vertices should be present");
    assert_eq!(root_vertices.len(), 1);
    assert!(root_vertices[0]
        .as_array()
        .expect("root vertex should be an array")
        .iter()
        .all(|coordinate| coordinate.is_i64() || coordinate.is_u64()));

    let geometry = json
        .get("CityObjects")
        .and_then(Value::as_object)
        .and_then(|cityobjects| cityobjects.get("instance-1"))
        .and_then(|object| object.get("geometry"))
        .and_then(Value::as_array)
        .and_then(|geometry| geometry.first())
        .cloned()
        .expect("geometry instance should be serialized");

    let template = geometry
        .get("template")
        .expect("template index should be serialized");
    assert!(template.is_i64() || template.is_u64());

    let boundary = geometry
        .get("boundaries")
        .and_then(Value::as_array)
        .and_then(|boundaries| boundaries.first())
        .expect("instance boundary should be serialized");
    assert!(boundary.is_i64() || boundary.is_u64());

    let transform = geometry
        .get("transformationMatrix")
        .and_then(Value::as_array)
        .expect("instance transform should be serialized");
    assert_eq!(transform.len(), 16);
    assert!(transform.iter().all(Value::is_f64));

    let template_vertices = json
        .get("geometry-templates")
        .and_then(Value::as_object)
        .and_then(|templates| templates.get("vertices-templates"))
        .and_then(Value::as_array)
        .expect("template vertices should be present");
    assert_eq!(template_vertices.len(), 1);
    assert!(template_vertices[0]
        .as_array()
        .expect("template vertex should be an array")
        .iter()
        .all(Value::is_f64));
}

#[test]
fn cityjson_fake_complete() {
    let json_input = read_to_string(DATA_DIR.join("cityjson_fake_complete.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn cityjson_fake_complete_deserialize() {
    let json_input = read_to_string(DATA_DIR.join("cityjson_fake_complete.city.json"));
    let cm = from_str_owned(&json_input).unwrap();
    assert!(!cm.vertices().is_empty());
    assert!(cm.extensions().is_some());
    assert!(cm.metadata().is_some());
    assert!(!cm.cityobjects().is_empty());
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
fn cityjson_minimal_complete() {
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
fn feature_constructor_rejects_full_documents() {
    let json_input = read_to_string(DATA_DIR.join("cityjson_minimal_complete.city.json"));
    let err = from_feature_str_owned(&json_input).unwrap_err();
    assert!(format!("{err}").contains("CityJSON"));
}

#[test]
fn strict_feature_stream_reads_self_contained_models() {
    let input = r#"{"type":"CityJSON","version":"2.0","transform":{"scale":[1.0,1.0,1.0],"translate":[0.0,0.0,0.0]},"CityObjects":{},"vertices":[]}
{"type":"CityJSONFeature","CityObjects":{"feature-1":{"type":"Building","geometry":[{"type":"MultiPoint","boundaries":[0,1]}]}},"vertices":[[0,0,0],[1,1,1]]}
"#;

    let mut models = read_feature_stream(std::io::Cursor::new(input))
        .unwrap()
        .collect::<serde_cityjson::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(models.len(), 1);

    let model = models.pop().unwrap();
    assert_eq!(
        model.type_citymodel(),
        cityjson::CityModelType::CityJSONFeature
    );
    assert_eq!(model.cityobjects().len(), 1);
    assert!(model.transform().is_some());
}

#[test]
fn strict_feature_stream_merges_into_one_document() {
    let input = r#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}
{"type":"CityJSONFeature","CityObjects":{"feature-1":{"type":"Building","geometry":[{"type":"MultiPoint","boundaries":[0,1]}]}},"vertices":[[0,0,0],[1,1,1]]}
{"type":"CityJSONFeature","CityObjects":{"feature-2":{"type":"BuildingPart","parents":["feature-1"],"geometry":[{"type":"MultiLineString","boundaries":[[0,1,2]]}]}},"vertices":[[2,2,2],[3,3,3],[4,4,4]]}
"#;

    let model = merge_feature_stream(std::io::Cursor::new(input)).unwrap();
    assert_eq!(model.type_citymodel(), cityjson::CityModelType::CityJSON);
    assert_eq!(model.cityobjects().len(), 2);
    assert_eq!(model.vertices().len(), 5);
}

#[test]
fn strict_feature_stream_rejects_duplicate_ids() {
    let input = r#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}
{"type":"CityJSONFeature","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}
{"type":"CityJSONFeature","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}
"#;

    let err = merge_feature_stream(std::io::Cursor::new(input)).unwrap_err();
    assert!(format!("{err}").contains("duplicate CityObject id"));
}

#[test]
fn transform() {
    let json_input = read_to_string(DATA_DIR.join("transform.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn cityobject_complete() {
    let json_input = read_to_string(DATA_DIR.join("cityobject_complete.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn cityobject_extended() {
    let json_input = read_to_string(DATA_DIR.join("cityobject_extended.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn geometry_instance() {
    let json_input = read_to_string(DATA_DIR.join("geometry_instance.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn geometry_complete_solid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_complete_solid.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn geometry_material_solid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_material_solid.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn geometry_texture_multisolid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_texture_multisolid.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn geometry_texture_solid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_texture_solid.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn geometry_texture_multisurface() {
    let json_input = read_to_string(DATA_DIR.join("geometry_texture_multisurface.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn geometry_semantics_multisolid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multisolid.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn geometry_semantics_solid() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_solid.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn geometry_semantics_multisurface() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multisurface.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn geometry_semantics_multilinestring() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multilinestring.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn geometry_semantics_multipoint() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multipoint.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn appearance_minimal_complete() {
    let json_input = read_to_string(DATA_DIR.join("appearance_minimal_complete.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn appearance_empty() {
    let json_input = read_to_string(DATA_DIR.join("appearance_empty.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn material_minimal() {
    let json_input = read_to_string(DATA_DIR.join("material_minimal.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn material_complete() {
    let json_input = read_to_string(DATA_DIR.join("material_complete.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn texture_complete() {
    let json_input = read_to_string(DATA_DIR.join("texture_complete.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn texture_minimal() {
    let json_input = read_to_string(DATA_DIR.join("texture_minimal.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn geometry_templates() {
    let json_input = read_to_string(DATA_DIR.join("geometry_templates.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn semantic_minimal() {
    let json_input = read_to_string(DATA_DIR.join("semantic_minimal.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn semantic_extended() {
    let json_input = read_to_string(DATA_DIR.join("semantic_extended.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn vertices() {
    let json_input = read_to_string(DATA_DIR.join("vertices.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn metadata_empty() {
    let json_input = read_to_string(DATA_DIR.join("metadata_empty.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn metadata_complete() {
    let json_input = read_to_string(DATA_DIR.join("metadata_complete.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn metadata_poc_minimal() {
    let json_input = read_to_string(DATA_DIR.join("metadata_poc_minimal.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn metadata_extra_properties() {
    let json_input = read_to_string(DATA_DIR.join("metadata_extra_properties.city.json"));
    assert_eq_roundtrip(&json_input);
}

#[test]
fn extension() {
    let json_input = read_to_string(DATA_DIR.join("extension.city.json"));
    assert_eq_roundtrip(&json_input);
}

// ---------------------------------------------------------------------------
// Borrowed-mode mirrors: same fixtures, same assertions, borrowed storage
// ---------------------------------------------------------------------------

#[test]
fn cityjson_fake_complete_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("cityjson_fake_complete.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn cityjson_fake_complete_deserialize_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("cityjson_fake_complete.city.json"));
    let cm = from_str_borrowed(&json_input).unwrap();
    assert!(!cm.vertices().is_empty());
    assert!(cm.extensions().is_some());
    assert!(cm.metadata().is_some());
    assert!(!cm.cityobjects().is_empty());
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
fn cityjson_minimal_complete_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("cityjson_minimal_complete.city.json"));
    let cm = from_str_borrowed(&json_input).unwrap();
    assert!(cm.extra().is_none());
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn cityjsonfeature_minimal_complete_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("cityjsonfeature_minimal_complete.city.jsonl"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn transform_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("transform.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn cityobject_complete_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("cityobject_complete.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn cityobject_extended_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("cityobject_extended.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn geometry_instance_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("geometry_instance.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn geometry_complete_solid_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("geometry_complete_solid.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn geometry_material_solid_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("geometry_material_solid.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn geometry_texture_multisolid_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("geometry_texture_multisolid.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn geometry_texture_solid_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("geometry_texture_solid.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn geometry_texture_multisurface_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("geometry_texture_multisurface.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn geometry_semantics_multisolid_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multisolid.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn geometry_semantics_solid_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_solid.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn geometry_semantics_multisurface_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multisurface.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn geometry_semantics_multilinestring_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multilinestring.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn geometry_semantics_multipoint_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("geometry_semantics_multipoint.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn appearance_minimal_complete_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("appearance_minimal_complete.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn appearance_empty_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("appearance_empty.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn material_minimal_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("material_minimal.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn material_complete_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("material_complete.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn texture_complete_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("texture_complete.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn texture_minimal_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("texture_minimal.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn geometry_templates_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("geometry_templates.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn semantic_minimal_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("semantic_minimal.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn semantic_extended_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("semantic_extended.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn vertices_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("vertices.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn metadata_empty_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("metadata_empty.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn metadata_complete_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("metadata_complete.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn metadata_poc_minimal_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("metadata_poc_minimal.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn metadata_extra_properties_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("metadata_extra_properties.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn extension_borrowed() {
    let json_input = read_to_string(DATA_DIR.join("extension.city.json"));
    assert_eq_roundtrip_borrowed(&json_input);
}

// ---------------------------------------------------------------------------
// Parity tests: owned and borrowed must produce identical output
// ---------------------------------------------------------------------------

#[test]
fn cityjson_fake_complete_parity() {
    let json_input = read_to_string(DATA_DIR.join("cityjson_fake_complete.city.json"));
    assert_eq_roundtrip_parity(&json_input);
}

#[test]
fn transform_parity() {
    let json_input = read_to_string(DATA_DIR.join("transform.city.json"));
    assert_eq_roundtrip_parity(&json_input);
}

#[test]
fn cityobject_complete_parity() {
    let json_input = read_to_string(DATA_DIR.join("cityobject_complete.city.json"));
    assert_eq_roundtrip_parity(&json_input);
}

#[test]
fn geometry_instance_parity() {
    let json_input = read_to_string(DATA_DIR.join("geometry_instance.city.json"));
    assert_eq_roundtrip_parity(&json_input);
}

#[test]
fn appearance_minimal_complete_parity() {
    let json_input = read_to_string(DATA_DIR.join("appearance_minimal_complete.city.json"));
    assert_eq_roundtrip_parity(&json_input);
}

#[test]
fn geometry_templates_parity() {
    let json_input = read_to_string(DATA_DIR.join("geometry_templates.city.json"));
    assert_eq_roundtrip_parity(&json_input);
}

#[test]
fn metadata_complete_parity() {
    let json_input = read_to_string(DATA_DIR.join("metadata_complete.city.json"));
    assert_eq_roundtrip_parity(&json_input);
}

#[test]
fn extension_parity() {
    let json_input = read_to_string(DATA_DIR.join("extension.city.json"));
    assert_eq_roundtrip_parity(&json_input);
}
