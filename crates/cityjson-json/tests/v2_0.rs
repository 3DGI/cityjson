use serde_json::value::RawValue;
use serde_json::{Value, json};

use cityjson::v2_0::{
    AffineTransform3D, BBox, CityModelType, CityObject, CityObjectIdentifier, CityObjectType,
    GeometryDraft, ImageType, LoD, OwnedCityModel, PointDraft, RealWorldCoordinate, Texture,
    UVCoordinate,
};
use cityjson_json::{
    as_json, from_feature_str, from_str_borrowed, from_str_owned, merge_cityjsonseq,
    read_cityjsonseq,
    v2_0::{FeatureObject, FeatureParts, from_feature_parts_with_base},
    write_cityjsonseq,
};
use common::*;

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

    let model = from_feature_parts_with_base(parts, &base.to_string()).unwrap();
    let vertices = model.vertices();
    let json: Value = serde_json::from_str(&as_json(&model).to_string().unwrap()).unwrap();

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

    let error = from_feature_parts_with_base(parts, &base.to_string()).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("duplicate CityObject id in feature parts")
    );
}

fn assert_vertex_eq(actual: [f64; 3], expected: [f64; 3]) {
    for (actual_coord, expected_coord) in actual.into_iter().zip(expected) {
        assert!((actual_coord - expected_coord).abs() < f64::EPSILON);
    }
}

fn stream_items(bytes: &[u8]) -> Vec<Value> {
    serde_json::Deserializer::from_slice(bytes)
        .into_iter::<Value>()
        .collect::<serde_json::Result<Vec<_>>>()
        .unwrap()
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

    let json: Value = serde_json::from_str(&as_json(&model).to_string().unwrap()).unwrap();

    let root_vertices = json
        .get("vertices")
        .and_then(Value::as_array)
        .expect("root vertices should be present");
    assert_eq!(root_vertices.len(), 1);
    assert!(
        root_vertices[0]
            .as_array()
            .expect("vertex should be an array")
            .iter()
            .all(|coordinate| coordinate.is_i64() || coordinate.is_u64())
    );

    let template_vertices = json
        .get("geometry-templates")
        .and_then(Value::as_object)
        .and_then(|templates| templates.get("vertices-templates"))
        .and_then(Value::as_array)
        .expect("template vertices should be present");
    assert_eq!(template_vertices.len(), 1);
    assert!(
        template_vertices[0]
            .as_array()
            .expect("template vertex should be an array")
            .iter()
            .all(Value::is_f64)
    );

    let texture_vertices = json
        .get("appearance")
        .and_then(Value::as_object)
        .and_then(|appearance| appearance.get("vertices-texture"))
        .and_then(Value::as_array)
        .expect("texture vertices should be present");
    assert_eq!(texture_vertices.len(), 1);
    assert!(
        texture_vertices[0]
            .as_array()
            .expect("texture vertex should be an array")
            .iter()
            .all(Value::is_f64)
    );

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
fn serialize_omits_empty_appearance_and_geometry_templates_sections() {
    let model = OwnedCityModel::new(CityModelType::CityJSON);
    let json: Value = serde_json::from_str(&as_json(&model).to_string().unwrap()).unwrap();

    assert!(json.get("appearance").is_none());
    assert!(json.get("geometry-templates").is_none());
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

    let json: Value = serde_json::from_str(&as_json(&model).to_string().unwrap()).unwrap();
    let root_vertices = json
        .get("vertices")
        .and_then(Value::as_array)
        .expect("root vertices should be present");
    assert_eq!(root_vertices.len(), 1);
    assert!(
        root_vertices[0]
            .as_array()
            .expect("root vertex should be an array")
            .iter()
            .all(|coordinate| coordinate.is_i64() || coordinate.is_u64())
    );

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
    assert!(
        template_vertices[0]
            .as_array()
            .expect("template vertex should be an array")
            .iter()
            .all(Value::is_f64)
    );
}

#[test]
fn feature_constructor_rejects_full_documents() {
    let json_input = conformance_case_input("cityjson_minimal");
    let err = from_feature_str(&json_input).unwrap_err();
    assert!(format!("{err}").contains("CityJSON"));
}

fn assert_invalid_case_rejected(case_id: &str, expected_error_fragment: &str) {
    let json_input = invalid_case_input(case_id);

    let owned_err = from_str_owned(&json_input).unwrap_err();
    assert!(
        owned_err.to_string().contains(expected_error_fragment),
        "owned parser accepted invalid case '{case_id}' with unexpected error: {owned_err}"
    );

    let borrowed_err = from_str_borrowed(&json_input).unwrap_err();
    assert!(
        borrowed_err.to_string().contains(expected_error_fragment),
        "borrowed parser accepted invalid case '{case_id}' with unexpected error: {borrowed_err}"
    );
}

#[test]
fn invalid_missing_type_is_rejected() {
    assert_invalid_case_rejected("invalid_missing_type", "missing field `type`");
}

#[test]
fn invalid_out_of_range_vertex_index_is_rejected() {
    assert_invalid_case_rejected(
        "invalid_out_of_range_vertex_index",
        "geometry vertex index 99 out of range",
    );
}

#[test]
fn invalid_cityjsonfeature_root_id_unresolved_is_rejected() {
    assert_invalid_case_rejected(
        "invalid_cityjsonfeature_root_id_unresolved",
        "feature root id",
    );
}

#[test]
fn cityjsonfeature_minimal_is_typed_not_extra() {
    let json_input = conformance_case_input("cityjsonfeature_minimal");
    let model = from_feature_str(&json_input).unwrap();
    let serialized: Value = serde_json::from_str(&as_json(&model).to_string().unwrap()).unwrap();

    assert_eq!(model.type_citymodel(), CityModelType::CityJSONFeature);
    assert_eq!(
        model
            .id()
            .and_then(|handle| model.cityobjects().get(handle))
            .map(|cityobject| cityobject.id().to_string()),
        Some("myid".to_string())
    );
    assert!(model.extra().and_then(|extra| extra.get("id")).is_none());
    assert_eq!(serialized["id"], "myid");
}

#[test]
fn cityjsonfeature_root_id_must_resolve_to_a_cityobject() {
    let json_input = invalid_case_input("invalid_cityjsonfeature_root_id_unresolved");
    let err = from_feature_str(&json_input).unwrap_err();
    assert!(err.to_string().contains("feature root id"));
}

#[test]
fn strict_feature_stream_reads_self_contained_models() {
    let input = r#"{"type":"CityJSON","version":"2.0","transform":{"scale":[1.0,1.0,1.0],"translate":[0.0,0.0,0.0]},"CityObjects":{},"vertices":[]}
{"type":"CityJSONFeature","id":"feature-1","CityObjects":{"feature-1":{"type":"Building","geometry":[{"type":"MultiPoint","boundaries":[0,1]}]}},"vertices":[[0,0,0],[1,1,1]]}
"#;

    let mut models = read_cityjsonseq(std::io::Cursor::new(input))
        .unwrap()
        .collect::<cityjson_json::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(models.len(), 1);

    let model = models.pop().unwrap();
    assert_eq!(
        model.type_citymodel(),
        cityjson::CityModelType::CityJSONFeature
    );
    assert_eq!(
        model
            .id()
            .and_then(|handle| model.cityobjects().get(handle))
            .map(|cityobject| cityobject.id().to_string()),
        Some("feature-1".to_string())
    );
    assert_eq!(model.cityobjects().len(), 1);
    assert!(model.transform().is_some());
}

#[test]
fn strict_feature_stream_merges_into_one_document() {
    let input = r#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}
{"type":"CityJSONFeature","id":"feature-1","CityObjects":{"feature-1":{"type":"Building","geometry":[{"type":"MultiPoint","boundaries":[0,1]}]}},"vertices":[[0,0,0],[1,1,1]]}
{"type":"CityJSONFeature","id":"feature-2","CityObjects":{"feature-2":{"type":"Building","geometry":[{"type":"MultiLineString","boundaries":[[0,1,2]]}]}},"vertices":[[2,2,2],[3,3,3],[4,4,4]]}
"#;

    let model = merge_cityjsonseq(std::io::Cursor::new(input)).unwrap();
    assert_eq!(model.type_citymodel(), cityjson::CityModelType::CityJSON);
    assert_eq!(model.cityobjects().len(), 2);
    assert_eq!(model.vertices().len(), 5);
}

#[test]
fn strict_feature_stream_rejects_duplicate_ids() {
    let input = r#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}
{"type":"CityJSONFeature","id":"feature-1","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}
{"type":"CityJSONFeature","id":"feature-1","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}
"#;

    let err = merge_cityjsonseq(std::io::Cursor::new(input)).unwrap_err();
    assert!(format!("{err}").contains("duplicate CityObject id"));
}

#[test]
fn strict_cityjsonseq_writer_emits_header_and_stripped_feature_items() {
    let base_input = json!({
        "type": "CityJSON",
        "version": "2.0",
        "transform": {
            "scale": [1.0, 1.0, 1.0],
            "translate": [0.0, 0.0, 0.0]
        },
        "metadata": {
            "title": "base-root"
        },
        "CityObjects": {},
        "vertices": []
    })
    .to_string();
    let feature_input = json!({
        "type": "CityJSONFeature",
        "id": "feature-1",
        "CityObjects": {
            "feature-1": {
                "type": "Building",
                "geometry": [{
                    "type": "MultiPoint",
                    "boundaries": [0, 1]
                }]
            }
        },
        "vertices": [[10, 20, 30], [12, 22, 31]]
    })
    .to_string();
    let base_root = from_str_owned(&base_input).unwrap();
    let feature = cityjson_json::from_feature_str_with_base(&feature_input, &base_input).unwrap();
    let mut transform = cityjson::v2_0::Transform::new();
    transform.set_scale([0.5, 0.5, 1.0]);
    transform.set_translate([10.0, 20.0, 30.0]);

    let mut output = Vec::new();
    let report = write_cityjsonseq(&base_root, [&feature])
        .with_transform(&transform)
        .write(&mut output)
        .unwrap();

    assert_eq!(report.feature_count, 1);
    assert_eq!(report.cityobject_count, 1);
    assert_eq!(
        report.geographical_extent,
        Some(BBox::new(10.0, 20.0, 30.0, 12.0, 22.0, 31.0))
    );

    let items = stream_items(&output);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["type"], "CityJSON");
    assert_eq!(items[0]["transform"]["scale"], json!([0.5, 0.5, 1.0]));
    assert_eq!(
        items[0]["transform"]["translate"],
        json!([10.0, 20.0, 30.0])
    );
    assert_eq!(items[0]["metadata"]["title"], "base-root");
    assert_eq!(
        items[0]["metadata"]["geographicalExtent"],
        json!([10.0, 20.0, 30.0, 12.0, 22.0, 31.0])
    );

    assert_eq!(items[1]["type"], "CityJSONFeature");
    assert_eq!(items[1]["id"], "feature-1");
    assert!(items[1].get("version").is_none());
    assert!(items[1].get("transform").is_none());
    assert!(items[1].get("metadata").is_none());
    assert!(items[1].get("extensions").is_none());
    assert!(items[1].get("appearance").is_none());
    assert_eq!(items[1]["vertices"], json!([[0, 0, 0], [4, 4, 1]]));

    let models = read_cityjsonseq(std::io::Cursor::new(output))
        .unwrap()
        .collect::<cityjson_json::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].vertices().len(), 2);
    assert_eq!(
        models[0]
            .id()
            .and_then(|handle| models[0].cityobjects().get(handle))
            .map(|cityobject| cityobject.id().to_string()),
        Some("feature-1".to_string())
    );
    assert_eq!(
        models[0].metadata().and_then(|metadata| metadata.title()),
        Some("base-root")
    );
}

#[test]
fn strict_cityjsonseq_writer_auto_transform_uses_extent_minimal() {
    let base_input = json!({
        "type": "CityJSON",
        "version": "2.0",
        "metadata": {
            "title": "base-root"
        },
        "CityObjects": {},
        "vertices": []
    })
    .to_string();
    let feature_a = cityjson_json::from_feature_str_with_base(
        &json!({
            "type": "CityJSONFeature",
            "id": "feature-a",
            "CityObjects": {
                "feature-a": {
                    "type": "Building",
                    "geometry": [{
                        "type": "MultiPoint",
                        "boundaries": [0, 1]
                    }]
                }
            },
            "vertices": [[10, 20, 30], [12, 23, 35]]
        })
        .to_string(),
        &base_input,
    )
    .unwrap();
    let feature_b = cityjson_json::from_feature_str_with_base(
        &json!({
            "type": "CityJSONFeature",
            "id": "feature-b",
            "CityObjects": {
                "feature-b": {
                    "type": "Building",
                    "geometry": [{
                        "type": "MultiPoint",
                        "boundaries": [0]
                    }]
                }
            },
            "vertices": [[9, 21, 40]]
        })
        .to_string(),
        &base_input,
    )
    .unwrap();
    let base_root = from_str_owned(&base_input).unwrap();

    let mut output = Vec::new();
    let report = write_cityjsonseq(&base_root, [&feature_a, &feature_b])
        .auto_transform([0.5, 1.0, 5.0])
        .write(&mut output)
        .unwrap();

    assert_eq!(
        report.geographical_extent,
        Some(BBox::new(9.0, 20.0, 30.0, 12.0, 23.0, 40.0))
    );
    assert_vertex_eq(report.transform.scale(), [0.5, 1.0, 5.0]);
    assert_vertex_eq(report.transform.translate(), [9.0, 20.0, 30.0]);

    let items = stream_items(&output);
    assert_eq!(items[0]["transform"]["translate"], json!([9.0, 20.0, 30.0]));
    assert_eq!(
        items[0]["metadata"]["geographicalExtent"],
        json!([9.0, 20.0, 30.0, 12.0, 23.0, 40.0])
    );
}

#[test]
fn strict_cityjsonseq_writer_rejects_incompatible_root_state() {
    let base_input = json!({
        "type": "CityJSON",
        "version": "2.0",
        "metadata": {
            "title": "base-root"
        },
        "CityObjects": {},
        "vertices": []
    })
    .to_string();
    let feature_input = json!({
        "type": "CityJSONFeature",
        "id": "feature-1",
        "metadata": {
            "title": "different-root"
        },
        "CityObjects": {
            "feature-1": {
                "type": "Building"
            }
        },
        "vertices": []
    })
    .to_string();
    let base_root = from_str_owned(&base_input).unwrap();
    let feature = from_str_owned(&feature_input).unwrap();

    let err = write_cityjsonseq(&base_root, [&feature])
        .with_transform(&cityjson::v2_0::Transform::new())
        .write(Vec::new())
        .unwrap_err();
    assert!(err.to_string().contains("incompatible root state"));
}

#[test]
fn strict_cityjsonseq_writer_accepts_feature_root_id_as_feature_local_state() {
    let base_input = json!({
        "type": "CityJSON",
        "version": "2.0",
        "metadata": {
            "title": "base-root"
        },
        "CityObjects": {},
        "vertices": []
    })
    .to_string();
    let base_root = from_str_owned(&base_input).unwrap();
    let feature_a = cityjson_json::from_feature_str_with_base(
        &json!({
            "type": "CityJSONFeature",
            "id": "building-1",
            "CityObjects": {
                "building-1": {
                    "type": "Building"
                }
            },
            "vertices": []
        })
        .to_string(),
        &base_input,
    )
    .unwrap();
    let feature_b = cityjson_json::from_feature_str_with_base(
        &json!({
            "type": "CityJSONFeature",
            "id": "building-2",
            "CityObjects": {
                "building-2": {
                    "type": "Building"
                }
            },
            "vertices": []
        })
        .to_string(),
        &base_input,
    )
    .unwrap();

    let mut output = Vec::new();
    write_cityjsonseq(&base_root, [&feature_a, &feature_b])
        .with_transform(&cityjson::v2_0::Transform::new())
        .write(&mut output)
        .unwrap();

    let items = stream_items(&output);
    assert_eq!(items[1]["id"], "building-1");
    assert_eq!(items[2]["id"], "building-2");
}

conformance_roundtrip_tests!(
    assert_eq_roundtrip;
    appearance_complete,
    cityobject_building_address,
    cityobject_complete,
    cityobject_extended,
    cityobject_all_types,
    coordinates_precision_ecef,
    coordinates_precision_local,
    coordinates_precision_stateplane,
    coordinates_precision_utm,
    coordinates_precision_wgs84,
    coordinates_precision_worst,
    geometry_instance,
    geometry_material_solid,
    geometry_material_multisolid,
    geometry_material_multisurface,
    geometry_texture_solid,
    geometry_texture_multisolid,
    geometry_texture_multisurface,
    geometry_semantics_solid,
    geometry_semantics_multisolid,
    geometry_semantics_multisurface,
    geometry_semantics_multilinestring,
    geometry_semantics_multipoint,
    cityjson_extended,
    cityjsonfeature_minimal,
    cityjson_fake_complete,
    cityjson_minimal,
    metadata_complete,
    metadata_extra_properties,
    semantic_all_types,
    semantic_complete,
    semantic_extended,
    vertices,
    extension,
    spec_geometry_matrix,
);

#[test]
fn cityjson_fake_complete_borrowed() {
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
