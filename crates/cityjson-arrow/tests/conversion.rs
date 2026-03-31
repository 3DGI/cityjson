use cityarrow::{from_parts, read_package_dir, to_parts, write_package_dir};
use cityjson::CityModelType;
use cityjson::v2_0::{
    AffineTransform3D, AttributeValue, Boundary, CityObject, CityObjectIdentifier, CityObjectType,
    Extension, Geometry, GeometryDraft, GeometryType, ImageType, LoD, OwnedCityModel,
    OwnedMaterial, OwnedSemantic, OwnedTexture, RGB, RGBA, RingDraft, SemanticMap, SemanticType,
    StoredGeometryInstance, StoredGeometryParts, SurfaceDraft, TextureType, ThemeName, WrapMode,
};
use serde_cityjson::to_string_validated;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tempfile::tempdir;

fn sample_model() -> OwnedCityModel {
    let mut model = OwnedCityModel::new(CityModelType::CityJSON);

    model
        .metadata_mut()
        .set_identifier(cityjson::v2_0::CityModelIdentifier::new(
            "sample-citymodel".to_string(),
        ));
    model
        .metadata_mut()
        .set_reference_date(cityjson::v2_0::Date::new("2026-03-30".to_string()));
    model
        .metadata_mut()
        .set_reference_system(cityjson::v2_0::CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/7415".to_string(),
        ));
    model.metadata_mut().set_title("Sample".to_string());
    model
        .metadata_mut()
        .set_geographical_extent(cityjson::v2_0::BBox::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0));
    model.metadata_mut().extra_mut().insert(
        "source".to_string(),
        AttributeValue::String("unit-test".to_string()),
    );
    model.extra_mut().insert(
        "+metadata-extended".to_string(),
        AttributeValue::Map(HashMap::from([(
            "textures".to_string(),
            AttributeValue::String("absent".to_string()),
        )])),
    );

    model.extensions_mut().add(Extension::new(
        "MetadataExtended".to_string(),
        "https://example.com/metadata-extended.ext.json".to_string(),
        "0.5".to_string(),
    ));

    let roof = model
        .add_semantic(OwnedSemantic::new(SemanticType::RoofSurface))
        .unwrap();
    let wall = model
        .add_semantic(OwnedSemantic::new(SemanticType::WallSurface))
        .unwrap();

    let vertices = [
        cityjson::v2_0::RealWorldCoordinate::new(0.0, 0.0, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(1.0, 0.0, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(1.0, 1.0, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(0.0, 1.0, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(0.5, 0.5, 1.0),
    ];
    for vertex in vertices {
        model.add_vertex(vertex).unwrap();
    }

    let boundary: Boundary<u32> = vec![
        vec![vec![0_u32, 1, 4, 0]],
        vec![vec![1_u32, 2, 4, 1]],
        vec![vec![2_u32, 3, 4, 2]],
        vec![vec![3_u32, 0, 4, 3]],
        vec![vec![0_u32, 3, 2, 1, 0]],
    ]
    .try_into()
    .unwrap();
    let mut semantics = SemanticMap::new();
    semantics.add_surface(Some(wall));
    semantics.add_surface(Some(wall));
    semantics.add_surface(Some(wall));
    semantics.add_surface(Some(wall));
    semantics.add_surface(Some(roof));

    let geometry = Geometry::from_stored_parts(StoredGeometryParts {
        type_geometry: GeometryType::MultiSurface,
        lod: Some(LoD::LoD2_2),
        boundaries: Some(boundary),
        semantics: Some(semantics),
        materials: None,
        textures: None,
        instance: None,
    });
    let geometry_handle = model.add_geometry(geometry).unwrap();

    let mut building = CityObject::new(
        CityObjectIdentifier::new("building-1".to_string()),
        CityObjectType::Building,
    );
    building.add_geometry(geometry_handle);
    building.attributes_mut().insert(
        "name".to_string(),
        AttributeValue::String("Sample Building".to_string()),
    );
    building
        .attributes_mut()
        .insert("nullable".to_string(), AttributeValue::Null);
    building.extra_mut().insert(
        "flags".to_string(),
        AttributeValue::Map(HashMap::from([(
            "demo".to_string(),
            AttributeValue::Bool(true),
        )])),
    );

    let mut part = CityObject::new(
        CityObjectIdentifier::new("building-1-part-0".to_string()),
        CityObjectType::BuildingPart,
    );
    part.attributes_mut()
        .insert("storeys".to_string(), AttributeValue::Unsigned(2));

    let building_handle = model.cityobjects_mut().add(building).unwrap();
    let part_handle = model.cityobjects_mut().add(part).unwrap();
    model
        .cityobjects_mut()
        .get_mut(building_handle)
        .unwrap()
        .add_child(part_handle);
    model
        .cityobjects_mut()
        .get_mut(part_handle)
        .unwrap()
        .add_parent(building_handle);

    model
}

fn normalized_json(model: &OwnedCityModel) -> JsonValue {
    serde_json::from_str(&to_string_validated(model).unwrap()).unwrap()
}

fn sample_model_with_template_instance() -> OwnedCityModel {
    let mut model = sample_model();

    let reference_point = model
        .add_vertex(cityjson::v2_0::RealWorldCoordinate::new(10.0, 20.0, 5.0))
        .unwrap();
    model
        .add_template_vertex(cityjson::v2_0::RealWorldCoordinate::new(0.0, 0.0, 0.0))
        .unwrap();
    model
        .add_template_vertex(cityjson::v2_0::RealWorldCoordinate::new(1.0, 0.0, 0.0))
        .unwrap();

    let template_boundary: Boundary<u32> = vec![0_u32, 1_u32].try_into().unwrap();
    let template_geometry = Geometry::from_stored_parts(StoredGeometryParts {
        type_geometry: GeometryType::MultiPoint,
        lod: Some(LoD::LoD1),
        boundaries: Some(template_boundary),
        semantics: None,
        materials: None,
        textures: None,
        instance: None,
    });
    let template_handle = model.add_geometry_template(template_geometry).unwrap();

    let instance_geometry = Geometry::from_stored_parts(StoredGeometryParts {
        type_geometry: GeometryType::GeometryInstance,
        lod: Some(LoD::LoD1),
        boundaries: None,
        semantics: None,
        materials: None,
        textures: None,
        instance: Some(StoredGeometryInstance {
            template: template_handle,
            reference_point,
            transformation: AffineTransform3D::from([
                1.0, 0.0, 0.0, 2.5, 0.0, 1.0, 0.0, 3.5, 0.0, 0.0, 1.0, 4.5, 0.0, 0.0, 0.0, 1.0,
            ]),
        }),
    });
    let instance_handle = model.add_geometry(instance_geometry).unwrap();

    let mut object = CityObject::new(
        CityObjectIdentifier::new("building-template-instance".to_string()),
        CityObjectType::Building,
    );
    object.add_geometry(instance_handle);
    object.attributes_mut().insert(
        "kind".to_string(),
        AttributeValue::String("instance".to_string()),
    );
    model.cityobjects_mut().add(object).unwrap();

    model
}

fn sample_model_with_appearance() -> OwnedCityModel {
    let mut model = OwnedCityModel::new(CityModelType::CityJSON);
    model
        .metadata_mut()
        .set_identifier(cityjson::v2_0::CityModelIdentifier::new(
            "appearance-citymodel".to_string(),
        ));

    let roof = model
        .add_semantic(OwnedSemantic::new(SemanticType::RoofSurface))
        .unwrap();
    let wall = model
        .add_semantic(OwnedSemantic::new(SemanticType::WallSurface))
        .unwrap();

    let mut material = OwnedMaterial::new("roof-tiles".to_string());
    material.set_ambient_intensity(Some(0.2));
    material.set_diffuse_color(Some(RGB::new(0.7, 0.2, 0.1)));
    material.set_specular_color(Some(RGB::new(0.9, 0.9, 0.9)));
    material.set_shininess(Some(0.6));
    material.set_transparency(Some(0.1));
    material.set_is_smooth(Some(true));
    let material = model.add_material(material).unwrap();

    let mut texture = OwnedTexture::new("textures/roof.png".to_string(), ImageType::Png);
    texture.set_wrap_mode(Some(WrapMode::Border));
    texture.set_texture_type(Some(TextureType::Specific));
    texture.set_border_color(Some(RGBA::new(1.0, 1.0, 1.0, 0.5)));
    let texture = model.add_texture(texture).unwrap();

    let theme = ThemeName::new("visual".to_string());
    model.set_default_material_theme(Some(theme.clone()));
    model.set_default_texture_theme(Some(theme.clone()));

    let surface_0 = SurfaceDraft::new(
        RingDraft::new([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.5, 1.0]]).with_texture(
            theme.clone(),
            texture,
            [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
        ),
        [],
    )
    .with_semantic(roof)
    .with_material(theme.clone(), material);

    let surface_1 = SurfaceDraft::new(
        RingDraft::new([[1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.5, 0.5, 1.0]]).with_texture(
            theme.clone(),
            texture,
            [[0.0, 0.2], [1.0, 0.2], [0.5, 1.0]],
        ),
        [],
    )
    .with_semantic(wall)
    .with_material(theme, material);

    let geometry = GeometryDraft::multi_surface(Some(LoD::LoD2_2), [surface_0, surface_1])
        .insert_into(&mut model)
        .unwrap();

    let mut building = CityObject::new(
        CityObjectIdentifier::new("appearance-building".to_string()),
        CityObjectType::Building,
    );
    building.add_geometry(geometry);
    building.attributes_mut().insert(
        "name".to_string(),
        AttributeValue::String("Appearance Building".to_string()),
    );
    model.cityobjects_mut().add(building).unwrap();

    model
}

#[test]
fn core_model_roundtrips_through_parts() {
    let model = sample_model();

    let parts = to_parts(&model).expect("to_parts should succeed");
    assert_eq!(parts.vertices.num_rows(), 5);
    assert_eq!(parts.geometries.num_rows(), 1);
    assert!(parts.semantics.is_some());
    assert!(parts.geometry_surface_semantics.is_some());
    assert!(parts.extensions.is_some());

    let reconstructed = from_parts(&parts).expect("from_parts should succeed");
    assert_eq!(normalized_json(&model), normalized_json(&reconstructed));
}

#[test]
fn geometry_templates_and_instances_roundtrip_through_parts() {
    let model = sample_model_with_template_instance();

    let parts = to_parts(&model).expect("to_parts should succeed");
    assert!(parts.geometry_instances.is_some());
    assert!(parts.template_vertices.is_some());
    assert!(parts.template_geometries.is_some());
    assert!(parts.template_geometry_boundaries.is_some());

    let reconstructed = from_parts(&parts).expect("from_parts should succeed");
    assert_eq!(normalized_json(&model), normalized_json(&reconstructed));
}

#[test]
fn appearances_roundtrip_through_parts_and_package() {
    let model = sample_model_with_appearance();

    let parts = to_parts(&model).expect("to_parts should succeed");
    assert!(parts.materials.is_some());
    assert!(parts.geometry_surface_materials.is_some());
    assert!(parts.textures.is_some());
    assert!(parts.texture_vertices.is_some());
    assert!(parts.geometry_ring_textures.is_some());

    let reconstructed = from_parts(&parts).expect("from_parts should succeed");
    assert_eq!(normalized_json(&model), normalized_json(&reconstructed));

    let dir = tempdir().expect("temp dir");
    write_package_dir(dir.path(), &parts).expect("package write should succeed");
    let package_parts = read_package_dir(dir.path()).expect("package read should succeed");
    let reconstructed = from_parts(&package_parts).expect("from_parts should succeed");
    assert_eq!(normalized_json(&model), normalized_json(&reconstructed));
}
