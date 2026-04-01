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

macro_rules! conformance_roundtrip_tests {
    ($assert_fn:ident; $($case_id:ident),+ $(,)?) => {
        $(
            #[test]
            fn $case_id() {
                let json_input = conformance_case_input(stringify!($case_id));
                $assert_fn(&json_input);
            }
        )+
    };
}

macro_rules! conformance_roundtrip_tests_named {
    ($assert_fn:ident; $($test_name:ident => $case_id:ident),+ $(,)?) => {
        $(
            #[test]
            fn $test_name() {
                let json_input = conformance_case_input(stringify!($case_id));
                $assert_fn(&json_input);
            }
        )+
    };
}

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
    assert_vertex_eq(vertices.as_slice()[0].to_array(), [10.0, 20.0, 30.0]);
    assert_vertex_eq(vertices.as_slice()[2].to_array(), [10.5, 20.0, 30.0]);
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

fn assert_vertex_eq(actual: [f64; 3], expected: [f64; 3]) {
    for (actual_coord, expected_coord) in actual.into_iter().zip(expected) {
        assert!((actual_coord - expected_coord).abs() < f64::EPSILON);
    }
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
    let json_input = conformance_case_input("cityjson_fake_complete");
    assert_eq_roundtrip(&json_input);
}

#[test]
fn cityjson_fake_complete_deserialize() {
    let json_input = conformance_case_input("cityjson_fake_complete");
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
    let json_input = conformance_case_input("cityjson_minimal_complete");
    let cm = from_str_owned(&json_input).unwrap();
    assert!(cm.extra().is_none());
    assert_eq_roundtrip(&json_input);
}

#[test]
fn cityjsonfeature_minimal_complete() {
    let json_input = conformance_case_input("cityjsonfeature_minimal_complete");
    assert_eq_roundtrip(&json_input);
}

#[test]
fn feature_constructor_rejects_full_documents() {
    let json_input = conformance_case_input("cityjson_minimal_complete");
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

conformance_roundtrip_tests!(
    assert_eq_roundtrip;
    transform,
    cityobject_complete,
    cityobject_extended,
    geometry_instance,
    geometry_complete_solid,
    geometry_material_solid,
    geometry_texture_multisolid,
    geometry_texture_solid,
    geometry_texture_multisurface,
    geometry_semantics_multisolid,
    geometry_semantics_solid,
    geometry_semantics_multisurface,
    geometry_semantics_multilinestring,
    geometry_semantics_multipoint,
    appearance_minimal_complete,
    appearance_empty,
    material_minimal,
    material_complete,
    texture_complete,
    texture_minimal,
    geometry_templates,
    semantic_minimal,
    semantic_extended,
    vertices,
    metadata_empty,
    metadata_complete,
    metadata_poc_minimal,
    metadata_extra_properties,
    extension,
);

// ---------------------------------------------------------------------------
// Borrowed-mode mirrors: same fixtures, same assertions, borrowed storage
// ---------------------------------------------------------------------------

#[test]
fn cityjson_fake_complete_borrowed() {
    let json_input = conformance_case_input("cityjson_fake_complete");
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn cityjson_fake_complete_deserialize_borrowed() {
    let json_input = conformance_case_input("cityjson_fake_complete");
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
    let json_input = conformance_case_input("cityjson_minimal_complete");
    let cm = from_str_borrowed(&json_input).unwrap();
    assert!(cm.extra().is_none());
    assert_eq_roundtrip_borrowed(&json_input);
}

#[test]
fn cityjsonfeature_minimal_complete_borrowed() {
    let json_input = conformance_case_input("cityjsonfeature_minimal_complete");
    assert_eq_roundtrip_borrowed(&json_input);
}

conformance_roundtrip_tests_named!(
    assert_eq_roundtrip_borrowed;
    transform_borrowed => transform,
    cityobject_complete_borrowed => cityobject_complete,
    cityobject_extended_borrowed => cityobject_extended,
    geometry_instance_borrowed => geometry_instance,
    geometry_complete_solid_borrowed => geometry_complete_solid,
    geometry_material_solid_borrowed => geometry_material_solid,
    geometry_texture_multisolid_borrowed => geometry_texture_multisolid,
    geometry_texture_solid_borrowed => geometry_texture_solid,
    geometry_texture_multisurface_borrowed => geometry_texture_multisurface,
    geometry_semantics_multisolid_borrowed => geometry_semantics_multisolid,
    geometry_semantics_solid_borrowed => geometry_semantics_solid,
    geometry_semantics_multisurface_borrowed => geometry_semantics_multisurface,
    geometry_semantics_multilinestring_borrowed => geometry_semantics_multilinestring,
    geometry_semantics_multipoint_borrowed => geometry_semantics_multipoint,
    appearance_minimal_complete_borrowed => appearance_minimal_complete,
    appearance_empty_borrowed => appearance_empty,
    material_minimal_borrowed => material_minimal,
    material_complete_borrowed => material_complete,
    texture_complete_borrowed => texture_complete,
    texture_minimal_borrowed => texture_minimal,
    geometry_templates_borrowed => geometry_templates,
    semantic_minimal_borrowed => semantic_minimal,
    semantic_extended_borrowed => semantic_extended,
    vertices_borrowed => vertices,
    metadata_empty_borrowed => metadata_empty,
    metadata_complete_borrowed => metadata_complete,
    metadata_poc_minimal_borrowed => metadata_poc_minimal,
    metadata_extra_properties_borrowed => metadata_extra_properties,
    extension_borrowed => extension,
);

// ---------------------------------------------------------------------------
// Parity tests: owned and borrowed must produce identical output
// ---------------------------------------------------------------------------

conformance_roundtrip_tests_named!(
    assert_eq_roundtrip_parity;
    cityjson_fake_complete_parity => cityjson_fake_complete,
    transform_parity => transform,
    cityobject_complete_parity => cityobject_complete,
    geometry_instance_parity => geometry_instance,
    appearance_minimal_complete_parity => appearance_minimal_complete,
    geometry_templates_parity => geometry_templates,
    metadata_complete_parity => metadata_complete,
    extension_parity => extension,
);
