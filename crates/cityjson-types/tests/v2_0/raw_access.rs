use cityjson_types::error::Result;
use cityjson_types::raw::RawAccess;
use cityjson_types::resources::storage::OwnedStringStorage;
use cityjson_types::v2_0::{
    AttributeValue, Boundary, CityModel, CityModelType, CityObject, CityObjectIdentifier,
    CityObjectType, ImageType, Material, RealWorldCoordinate, Semantic, SemanticType, Texture,
    boundary::nested::{BoundaryNestedMultiOrCompositeSolid32, BoundaryNestedMultiPoint32},
};

const FLOAT_EPSILON: f64 = 1.0e-9;

fn assert_f64_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= FLOAT_EPSILON,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn raw_access_and_attribute_column_helpers_work() -> Result<()> {
    let mut model = CityModel::<u32, OwnedStringStorage>::new(CityModelType::CityJSON);

    let mut cityobject = CityObject::new(
        CityObjectIdentifier::new("co-1".to_string()),
        CityObjectType::Building,
    );
    cityobject
        .attributes_mut()
        .insert("height".to_string(), AttributeValue::Float(12.5));
    cityobject
        .attributes_mut()
        .insert("floors".to_string(), AttributeValue::Integer(3));
    cityobject
        .attributes_mut()
        .insert("name".to_string(), AttributeValue::String("A".to_string()));
    model.cityobjects_mut().add(cityobject)?;

    model.add_semantic(Semantic::new(SemanticType::RoofSurface))?;
    model.add_material(Material::new("default".to_string()))?;
    model.add_texture(Texture::new(
        "https://example.com/tex.png".to_string(),
        ImageType::Png,
    ))?;
    model.add_vertex(RealWorldCoordinate::new(1.0, 2.0, 3.0))?;

    let raw = model.raw();
    assert_eq!(raw.vertices().len(), 1);
    assert_eq!(raw.semantics().len(), 1);
    assert_eq!(raw.materials().len(), 1);
    assert_eq!(raw.textures().len(), 1);

    let raw_vertices = model.vertices_raw();
    assert_eq!(raw_vertices.len(), 1);
    assert_f64_eq(raw_vertices.get(0).expect("vertex 0").x(), 1.0);

    let (ids_h, heights) = model.extract_float_column("height");
    assert_eq!(ids_h.len(), 1);
    assert_eq!(heights.len(), 1);
    assert_f64_eq(heights[0], 12.5);

    let (ids_f, floors) = model.extract_integer_column("floors");
    assert_eq!(ids_f.len(), 1);
    assert_eq!(floors, vec![3]);

    let (ids_n, names) = model.extract_string_column("name");
    assert_eq!(ids_n.len(), 1);
    assert_eq!(names[0], "A");

    let keys = model.attribute_keys();
    assert!(keys.contains("height"));
    assert!(keys.contains("floors"));
    assert!(keys.contains("name"));

    Ok(())
}

#[test]
fn boundary_to_columnar_exposes_flat_views() {
    let nested: BoundaryNestedMultiPoint32 = vec![0, 1, 2];
    let boundary: Boundary<u32> = nested.into();
    let col = boundary.to_columnar();
    assert_eq!(col.vertices.len(), 3);
    assert!(col.ring_offsets.is_empty());
    assert!(col.surface_offsets.is_empty());
    assert!(col.shell_offsets.is_empty());
    assert!(col.solid_offsets.is_empty());
}

#[test]
fn boundary_raw_access_exposes_all_offset_layers() {
    let nested: BoundaryNestedMultiOrCompositeSolid32 = vec![
        vec![
            vec![
                vec![vec![0, 1, 2, 0]],
                vec![vec![3, 4, 5, 3], vec![6, 7, 8, 6]],
            ],
            vec![vec![vec![9, 10, 11, 9]]],
        ],
        vec![vec![vec![vec![12, 13, 14, 12]]]],
    ];
    let boundary: Boundary<u32> = nested.try_into().unwrap();
    let columnar = boundary.to_columnar();

    assert_eq!(
        &*boundary.vertices_raw(),
        &[
            0, 1, 2, 0, 3, 4, 5, 3, 6, 7, 8, 6, 9, 10, 11, 9, 12, 13, 14, 12
        ]
    );
    assert_eq!(&*boundary.rings_raw(), &[0, 4, 8, 12, 16]);
    assert_eq!(&*boundary.surfaces_raw(), &[0, 1, 3, 4]);
    assert_eq!(&*boundary.shells_raw(), &[0, 2, 3]);
    assert_eq!(&*boundary.solids_raw(), &[0, 2]);

    assert_eq!(columnar.vertices, boundary.vertices());
    assert_eq!(columnar.ring_offsets, boundary.rings());
    assert_eq!(columnar.surface_offsets, boundary.surfaces());
    assert_eq!(columnar.shell_offsets, boundary.shells());
    assert_eq!(columnar.solid_offsets, boundary.solids());
}
