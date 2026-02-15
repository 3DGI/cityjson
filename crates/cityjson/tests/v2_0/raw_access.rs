use cityjson::prelude::*;
use cityjson::raw::RawAccess;
use cityjson::cityjson::core::boundary::nested::BoundaryNestedMultiPoint32;
use cityjson::v2_0::{
    CityModel, CityObject, CityObjectIdentifier, CityObjectType, Material, Semantic, SemanticType,
    Texture,
};

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
    model.add_vertex(QuantizedCoordinate::new(1, 2, 3))?;

    let raw = model.raw();
    assert_eq!(raw.vertices().len(), 1);
    assert_eq!(raw.semantics().len(), 1);
    assert_eq!(raw.materials().len(), 1);
    assert_eq!(raw.textures().len(), 1);

    let raw_vertices = model.vertices_raw();
    assert_eq!(raw_vertices.len(), 1);
    assert_eq!(raw_vertices.get(0).expect("vertex 0").x(), 1);

    let (ids_h, heights) = model.extract_float_column("height");
    assert_eq!(ids_h.len(), 1);
    assert_eq!(heights, vec![12.5]);

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
