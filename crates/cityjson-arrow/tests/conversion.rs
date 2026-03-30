use cityarrow::{from_parts, to_parts};
use cityjson::CityModelType;
use cityjson::v2_0::{
    AttributeValue, Boundary, CityObject, CityObjectIdentifier, CityObjectType, Extension, Geometry,
    GeometryType, LoD, OwnedCityModel, OwnedSemantic, SemanticMap, SemanticType, StoredGeometryParts,
};
use serde_cityjson::to_string_validated;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

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
    model
        .metadata_mut()
        .set_title("Sample".to_string());
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
    part.attributes_mut().insert(
        "storeys".to_string(),
        AttributeValue::Unsigned(2),
    );

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
