use cityarrow::{CityArrowPackageVersion, ModelDecoder, ModelEncoder};
use cityjson::CityModelType;
use cityjson::v2_0::{
    AttributeValue, Boundary, CityObject, CityObjectIdentifier, CityObjectType, Contact,
    ContactRole, ContactType, Geometry, ImageType, LoD, OwnedCityModel, OwnedMaterial,
    OwnedSemantic, OwnedTexture, SemanticMap, SemanticType, StoredGeometryParts, TextureType,
    WrapMode,
};
use cityparquet::{PackageReader, PackageWriter};
use serde_cityjson::to_string_validated;
use serde_json::Value as JsonValue;
use tempfile::tempdir;

fn sample_model() -> OwnedCityModel {
    let mut model = OwnedCityModel::new(CityModelType::CityJSON);
    model
        .metadata_mut()
        .set_identifier(cityjson::v2_0::CityModelIdentifier::new(
            "sample-citymodel".to_string(),
        ));
    model.metadata_mut().set_title("Sample".to_string());
    model
        .metadata_mut()
        .set_reference_date(cityjson::v2_0::Date::new("2026-04-02".to_string()));
    let mut contact = Contact::new();
    contact.set_contact_name("Example Contact".to_string());
    contact.set_email_address("contact@example.test".to_string());
    contact.set_role(Some(ContactRole::PointOfContact));
    contact.set_contact_type(Some(ContactType::Organization));
    contact.set_phone(Some("+31-20-1234567".to_string()));
    contact.set_organization(Some("CityArrow".to_string()));
    let mut address = cityjson::v2_0::OwnedAttributes::default();
    address.insert(
        "locality".to_string(),
        AttributeValue::String("Amsterdam".to_string()),
    );
    address.insert(
        "postalCode".to_string(),
        AttributeValue::String("1011AA".to_string()),
    );
    contact.set_address(Some(address));
    model.metadata_mut().set_point_of_contact(Some(contact));

    let mut material = OwnedMaterial::new("Facade".to_string());
    material.set_diffuse_color(Some([0.8, 0.7, 0.6].into()));
    material.set_emissive_color(Some([0.1, 0.1, 0.1].into()));
    material.set_specular_color(Some([0.9, 0.9, 0.9].into()));
    model.add_material(material).unwrap();

    let mut texture = OwnedTexture::new("textures/facade.png".to_string(), ImageType::Png);
    texture.set_wrap_mode(Some(WrapMode::Mirror));
    texture.set_texture_type(Some(TextureType::Specific));
    texture.set_border_color(Some([0.0, 0.0, 0.0, 1.0].into()));
    model.add_texture(texture).unwrap();

    let roof = model
        .add_semantic(OwnedSemantic::new(SemanticType::RoofSurface))
        .unwrap();
    let wall = model
        .add_semantic(OwnedSemantic::new(SemanticType::WallSurface))
        .unwrap();

    for vertex in [
        cityjson::v2_0::RealWorldCoordinate::new(0.0, 0.0, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(1.0, 0.0, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(1.0, 1.0, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(0.0, 1.0, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(0.5, 0.5, 1.0),
    ] {
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
        type_geometry: cityjson::v2_0::GeometryType::MultiSurface,
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
    model.cityobjects_mut().add(building).unwrap();

    model
}

fn normalized_json(model: &OwnedCityModel) -> JsonValue {
    serde_json::from_str(&to_string_validated(model).unwrap()).unwrap()
}

#[test]
fn live_arrow_stream_roundtrips_a_model() {
    let model = sample_model();
    let mut bytes = Vec::new();

    ModelEncoder.encode(&model, &mut bytes).unwrap();
    let decoded = ModelDecoder.decode(bytes.as_slice()).unwrap();

    assert_eq!(normalized_json(&model), normalized_json(&decoded));
}

#[test]
fn single_file_package_roundtrips_a_model_and_exposes_manifest() {
    let model = sample_model();
    let dir = tempdir().unwrap();
    let path = dir.path().join("sample.cityarrow");

    let manifest = PackageWriter.write_file(&path, &model).unwrap();
    assert_eq!(manifest.package_schema, CityArrowPackageVersion::V3Alpha1);
    assert!(!manifest.tables.is_empty());

    let inspected = PackageReader.read_manifest(&path).unwrap();
    assert_eq!(inspected.package_schema, CityArrowPackageVersion::V3Alpha1);
    assert_eq!(inspected.citymodel_id, manifest.citymodel_id);

    let decoded = PackageReader.read_file(&path).unwrap();
    assert_eq!(normalized_json(&model), normalized_json(&decoded));
}
