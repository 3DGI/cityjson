use std::collections::HashMap;

use cityjson::CityModelType;
use cityjson::v2_0::appearance::ImageType;
use cityjson::v2_0::geometry::semantic::SemanticType;
use cityjson::v2_0::{
    AttributeValue, CityModelIdentifier, CityObject, CityObjectIdentifier, CityObjectType,
    GeometryDraft, OwnedCityModel, OwnedMaterial, OwnedSemantic, OwnedTexture, RingDraft,
    SurfaceDraft,
};
use cityjson_arrow::{
    ExportOptions, ImportOptions, export_reader, import_batches, read_stream, write_stream,
};

fn assert_stream_roundtrip(model: &OwnedCityModel) {
    let mut bytes = Vec::new();
    write_stream(&mut bytes, model, &ExportOptions::default()).expect("encode stream");
    let decoded = read_stream(bytes.as_slice(), &ImportOptions::default()).expect("decode stream");
    if model.type_citymodel() == CityModelType::CityJSONFeature {
        assert_feature_model(&decoded);
    } else {
        assert_appearance_model(&decoded);
    }
}

fn assert_batch_roundtrip(model: &OwnedCityModel) {
    let reader = export_reader(model, &ExportOptions::default()).expect("export canonical batches");
    let header = reader.header().clone();
    let projection = reader.projection().clone();
    let batches = reader.collect::<Vec<_>>();
    let decoded = import_batches(header, projection, batches, &ImportOptions::default())
        .expect("import canonical batches");
    assert_appearance_model(&decoded);
}

fn assert_feature_model(model: &OwnedCityModel) {
    assert_eq!(model.type_citymodel(), CityModelType::CityJSONFeature);
    assert_eq!(model.cityobjects().len(), 2);
    assert_eq!(
        model
            .metadata()
            .and_then(|metadata| metadata.identifier())
            .map(ToString::to_string),
        Some("feature-model".to_string())
    );
    assert_eq!(
        model.extra().and_then(|extra| extra.get("dataset")),
        Some(&AttributeValue::String("local-fixture".to_string()))
    );

    let root_handle = model.id().expect("feature root handle");
    let root = model.cityobjects().get(root_handle).expect("feature root");
    assert_eq!(root.id().to_string(), "feature-root");
    assert_eq!(root.type_cityobject(), &CityObjectType::Building);
    assert_eq!(root.children().map(<[_]>::len), Some(1));

    let child_handle = root
        .children()
        .and_then(|children| children.first())
        .copied();
    let child = child_handle
        .and_then(|handle| model.cityobjects().get(handle))
        .expect("feature child");
    assert_eq!(child.id().to_string(), "feature-child");
    assert_eq!(child.type_cityobject(), &CityObjectType::BuildingPart);
}

#[allow(clippy::too_many_lines)]
fn assert_appearance_model(model: &OwnedCityModel) {
    assert_eq!(model.type_citymodel(), CityModelType::CityJSON);
    assert_eq!(model.cityobjects().len(), 1);
    assert_eq!(model.semantic_count(), 2);
    assert_eq!(model.material_count(), 1);
    assert_eq!(model.texture_count(), 1);
    assert_eq!(model.vertices_texture().as_slice().len(), 3);

    let building = model.cityobjects().iter().next().expect("building").1;
    assert_eq!(building.id().to_string(), "appearance-building");
    assert_eq!(building.type_cityobject(), &CityObjectType::Building);

    let geometry_handle = building
        .geometry()
        .and_then(|geometry| geometry.first())
        .copied()
        .expect("geometry handle");
    let geometry = model.get_geometry(geometry_handle).expect("geometry");
    assert_eq!(geometry.type_geometry().to_string(), "MultiSurface");

    let semantic_types = geometry
        .semantics()
        .expect("geometry semantics")
        .surfaces()
        .iter()
        .map(|semantic| {
            semantic.map(|handle| {
                model
                    .get_semantic(handle)
                    .expect("semantic")
                    .type_semantic()
                    .to_string()
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        semantic_types,
        vec![
            Some("WallSurface".to_string()),
            Some("RoofSurface".to_string()),
        ]
    );

    let mut material_themes = geometry
        .materials()
        .expect("geometry materials")
        .iter()
        .map(|(theme, map)| {
            let surfaces = map
                .surfaces()
                .iter()
                .map(|material| {
                    material
                        .map(|handle| model.get_material(handle).expect("material").name().clone())
                })
                .collect::<Vec<_>>();
            (theme.to_string(), surfaces)
        })
        .collect::<Vec<_>>();
    material_themes.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(
        material_themes,
        vec![
            ("roof".to_string(), vec![None, Some("brick".to_string())]),
            ("wall".to_string(), vec![Some("brick".to_string()), None]),
        ]
    );

    let mut texture_themes = geometry
        .textures()
        .expect("geometry textures")
        .iter()
        .map(|(theme, map)| {
            let ring_textures = map
                .ring_textures()
                .iter()
                .map(|texture| {
                    texture
                        .map(|handle| model.get_texture(handle).expect("texture").image().clone())
                })
                .collect::<Vec<_>>();
            (theme.to_string(), ring_textures)
        })
        .collect::<Vec<_>>();
    texture_themes.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(
        texture_themes,
        vec![
            (
                "roof".to_string(),
                vec![None, Some("brick.png".to_string())]
            ),
            (
                "wall".to_string(),
                vec![Some("brick.png".to_string()), None]
            ),
        ]
    );
}

fn build_feature_model() -> OwnedCityModel {
    let mut model = OwnedCityModel::new(CityModelType::CityJSONFeature);
    model
        .metadata_mut()
        .set_identifier(CityModelIdentifier::new("feature-model".to_string()));
    model.metadata_mut().set_title("Feature model".to_string());
    model.extra_mut().insert(
        "dataset".to_string(),
        AttributeValue::String("local-fixture".to_string()),
    );

    let root_geometry = GeometryDraft::multi_surface(
        None,
        [SurfaceDraft::new(
            RingDraft::new([
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ]),
            [],
        )],
    )
    .insert_into(&mut model)
    .expect("insert root geometry");

    let mut root = CityObject::new(
        CityObjectIdentifier::new("feature-root".to_string()),
        CityObjectType::Building,
    );
    root.add_geometry(root_geometry);
    root.attributes_mut().insert(
        "name".to_string(),
        AttributeValue::String("Feature Root".to_string()),
    );
    root.attributes_mut().insert(
        "stats".to_string(),
        AttributeValue::Map(HashMap::from([
            ("levels".to_string(), AttributeValue::Unsigned(3)),
            ("active".to_string(), AttributeValue::Bool(true)),
        ])),
    );

    let mut child = CityObject::new(
        CityObjectIdentifier::new("feature-child".to_string()),
        CityObjectType::BuildingPart,
    );
    child.attributes_mut().insert(
        "tags".to_string(),
        AttributeValue::Vec(vec![
            AttributeValue::String("annex".to_string()),
            AttributeValue::String("surveyed".to_string()),
        ]),
    );

    let root_handle = model
        .cityobjects_mut()
        .add(root)
        .expect("insert root object");
    let child_handle = model
        .cityobjects_mut()
        .add(child)
        .expect("insert child object");
    model
        .cityobjects_mut()
        .get_mut(root_handle)
        .expect("root object")
        .add_child(child_handle);
    model
        .cityobjects_mut()
        .get_mut(child_handle)
        .expect("child object")
        .add_parent(root_handle);
    model.set_id(Some(root_handle));

    model
}

fn build_appearance_model() -> OwnedCityModel {
    let mut model = OwnedCityModel::new(CityModelType::CityJSON);
    model
        .metadata_mut()
        .set_identifier(CityModelIdentifier::new("appearance-model".to_string()));

    let roof = model
        .add_semantic(OwnedSemantic::new(SemanticType::RoofSurface))
        .expect("insert roof semantic");
    let wall = model
        .add_semantic(OwnedSemantic::new(SemanticType::WallSurface))
        .expect("insert wall semantic");
    let material = model
        .add_material(OwnedMaterial::new("brick".to_string()))
        .expect("insert material");
    let texture = model
        .add_texture(OwnedTexture::new("brick.png".to_string(), ImageType::Png))
        .expect("insert texture");

    let geometry = GeometryDraft::multi_surface(
        None,
        [
            SurfaceDraft::new(
                RingDraft::new([[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 1.0, 1.0]]).with_texture(
                    "wall".to_string(),
                    texture,
                    [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
                ),
                [],
            )
            .with_semantic(wall)
            .with_material("wall".to_string(), material),
            SurfaceDraft::new(
                RingDraft::new([[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.0, 2.0, 0.0]]).with_texture(
                    "roof".to_string(),
                    texture,
                    [[0.0, 0.0], [0.5, 1.0], [1.0, 0.0]],
                ),
                [],
            )
            .with_semantic(roof)
            .with_material("roof".to_string(), material),
        ],
    )
    .insert_into(&mut model)
    .expect("insert appearance geometry");

    let mut building = CityObject::new(
        CityObjectIdentifier::new("appearance-building".to_string()),
        CityObjectType::Building,
    );
    building.add_geometry(geometry);
    building.attributes_mut().insert(
        "usage".to_string(),
        AttributeValue::String("mixed-use".to_string()),
    );

    model
        .cityobjects_mut()
        .add(building)
        .expect("insert appearance building");

    model
}

#[test]
fn stream_roundtrip_feature_model() {
    assert_stream_roundtrip(&build_feature_model());
}

#[test]
fn stream_roundtrip_appearance_model() {
    assert_stream_roundtrip(&build_appearance_model());
}

#[test]
fn batch_roundtrip_appearance_model() {
    assert_batch_roundtrip(&build_appearance_model());
}
