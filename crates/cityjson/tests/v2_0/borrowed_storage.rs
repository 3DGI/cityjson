use cityjson::prelude::*;
use cityjson::backend::default::geometry::GeometryBuilder;
use cityjson::v2_0::*;

/// Test that demonstrates how CityModel works with BorrowedStringStorage.
/// This test shows that we can create a CityModel that borrows string data
/// instead of owning it, which can be more memory-efficient when processing
/// data from external sources that remain in memory.
#[test]
fn test_citymodel_with_borrowed_storage() -> Result<()> {
    // Define all string data upfront with 'static lifetime for simplicity
    // In a real scenario, these could be borrowed from a deserialization buffer
    // or any other source that outlives the CityModel
    let model_id = "550e8400-e29b-41d4-a716-446655440000";
    let crs_url = "https://www.opengis.net/def/crs/EPSG/0/7415";
    let contact_name = "Test Organization";
    let contact_email = "test@example.org";
    let extension_name = "Noise";
    let extension_url = "https://example.org/noise.json";
    let extension_version = "2.0";
    let building_id = "building-1";
    let attribute_key = "yearOfConstruction";
    let roof_type_key = "roofType";
    let roof_type_value = "flat";

    // Create a CityModel for CityJSON v2.0 that uses:
    // - u32 indices for vertices
    // - ResourceId32 for resource references
    // - BorrowedStringStorage for string data (with 'static lifetime)
    let mut model = CityModel::<u32, BorrowedStringStorage<'static>>::new(
        CityModelType::CityJSON,
    );

    // Set up metadata using borrowed strings
    let metadata = model.metadata_mut();
    metadata.set_geographical_extent(BBox::new(1000.0, 2000.0, 0.0, 1500.0, 2500.0, 100.0));
    metadata.set_identifier(CityModelIdentifier::new(model_id));
    metadata.set_reference_system(CRS::new(crs_url));
    metadata.set_contact_name(contact_name);
    metadata.set_email_address(contact_email);

    // Set transform
    let transform = model.transform_mut();
    transform.set_scale([1.0, 1.0, 1.0]);
    transform.set_translate([0.0, 0.0, 0.0]);

    // Add an extension
    let extensions = model.extensions_mut();
    extensions.add(Extension::new(
        extension_name,
        extension_url,
        extension_version,
    ));

    // Create vertices
    let v0 = model.add_vertex(QuantizedCoordinate::new(0, 0, 0))?;
    let v1 = model.add_vertex(QuantizedCoordinate::new(100, 0, 0))?;
    let v2 = model.add_vertex(QuantizedCoordinate::new(100, 100, 0))?;
    let v3 = model.add_vertex(QuantizedCoordinate::new(0, 100, 0))?;
    let v4 = model.add_vertex(QuantizedCoordinate::new(0, 0, 50))?;
    let v5 = model.add_vertex(QuantizedCoordinate::new(100, 0, 50))?;
    let v6 = model.add_vertex(QuantizedCoordinate::new(100, 100, 50))?;
    let v7 = model.add_vertex(QuantizedCoordinate::new(0, 100, 50))?;

    // Create a building CityObject with borrowed string ID
    let mut building = CityObject::new(CityObjectIdentifier::new(building_id), CityObjectType::Building);

    // Add inline attributes
    let building_attrs = building.attributes_mut();
    building_attrs.insert(
        attribute_key,
        AttributeValue::Integer(2020),
    );
    building_attrs.insert(
        roof_type_key,
        AttributeValue::String(roof_type_value),
    );

    // Build a simple Solid geometry
    {
        let mut geometry_builder =
            GeometryBuilder::new(&mut model, GeometryType::Solid, BuilderMode::Regular)
                .with_lod(LoD::LoD1);

        // Add vertices to the builder
        let bv0 = geometry_builder.add_vertex(v0);
        let bv1 = geometry_builder.add_vertex(v1);
        let bv2 = geometry_builder.add_vertex(v2);
        let bv3 = geometry_builder.add_vertex(v3);
        let bv4 = geometry_builder.add_vertex(v4);
        let bv5 = geometry_builder.add_vertex(v5);
        let bv6 = geometry_builder.add_vertex(v6);
        let bv7 = geometry_builder.add_vertex(v7);

        // Create bottom face
        let ring_bottom = geometry_builder.add_ring(&[bv0, bv1, bv2, bv3])?;
        let surface_bottom = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring_bottom)?;
        let ground_semantic = Semantic::new(SemanticType::GroundSurface);
        geometry_builder.set_semantic_surface(None, ground_semantic, false)?;

        // Create top face
        let ring_top = geometry_builder.add_ring(&[bv4, bv7, bv6, bv5])?;
        let surface_top = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring_top)?;
        let roof_semantic = Semantic::new(SemanticType::RoofSurface);
        geometry_builder.set_semantic_surface(None, roof_semantic, false)?;

        // Create wall faces
        let ring_wall1 = geometry_builder.add_ring(&[bv0, bv4, bv5, bv1])?;
        let surface_wall1 = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring_wall1)?;
        let wall_semantic = Semantic::new(SemanticType::WallSurface);
        geometry_builder.set_semantic_surface(None, wall_semantic.clone(), true)?;

        let ring_wall2 = geometry_builder.add_ring(&[bv1, bv5, bv6, bv2])?;
        let surface_wall2 = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring_wall2)?;
        geometry_builder.set_semantic_surface(None, wall_semantic.clone(), true)?;

        let ring_wall3 = geometry_builder.add_ring(&[bv2, bv6, bv7, bv3])?;
        let surface_wall3 = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring_wall3)?;
        geometry_builder.set_semantic_surface(None, wall_semantic.clone(), true)?;

        let ring_wall4 = geometry_builder.add_ring(&[bv3, bv7, bv4, bv0])?;
        let surface_wall4 = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring_wall4)?;
        geometry_builder.set_semantic_surface(None, wall_semantic, true)?;

        // Create the shell and build the geometry
        geometry_builder.add_shell(&[
            surface_bottom,
            surface_top,
            surface_wall1,
            surface_wall2,
            surface_wall3,
            surface_wall4,
        ])?;

        let geometry_ref = geometry_builder.build()?;
        building.add_geometry(GeometryRef::from_parts(geometry_ref.index(), geometry_ref.generation()));
    }

    // Add the building to the model
    let building_ref = model.cityobjects_mut().add(building);

    // === Verify the model works correctly with borrowed storage ===

    // Test that we can access the borrowed strings through the model
    assert_eq!(model.type_citymodel(), CityModelType::CityJSON);
    assert_eq!(model.version(), Some(CityJSONVersion::V2_0));
    assert_eq!(model.vertices().len(), 8);
    assert_eq!(model.geometry_count(), 1);
    assert_eq!(model.semantic_count(), 3); // ground, roof, wall

    // Verify metadata with borrowed strings
    let metadata = model.metadata().expect("Metadata should exist");
    assert_eq!(
        metadata.identifier(),
        Some(&CityModelIdentifier::new(model_id))
    );
    assert_eq!(metadata.reference_system(), Some(&CRS::new(crs_url)));
    let contact = metadata.point_of_contact().expect("Contact should exist");
    assert_eq!(contact.contact_name(), contact_name);
    assert_eq!(contact.email_address(), contact_email);

    // Verify extension
    let extensions = model.extensions().expect("Extensions should exist");
    let noise_ext = extensions
        .get(extension_name)
        .expect("Extension should exist");
    assert_eq!(*noise_ext.name(), extension_name);

    // Verify building CityObject
    let building_obj = model
        .cityobjects()
        .get(building_ref)
        .expect("Building should exist");
    assert_eq!(building_obj.id(), building_id);
    assert_eq!(building_obj.type_cityobject(), &CityObjectType::Building);

    // Verify attributes
    let attrs = building_obj
        .attributes()
        .expect("Building should have attributes");

    // Get year attribute and verify
    let year_attr = attrs
        .get(attribute_key)
        .expect("yearOfConstruction should exist");
    match year_attr {
        AttributeValue::Integer(year) => assert_eq!(*year, 2020),
        _ => panic!("yearOfConstruction should be Integer"),
    }

    // Get roof type attribute and verify
    let roof_type_attr = attrs.get(roof_type_key).expect("roofType should exist");
    match roof_type_attr {
        AttributeValue::String(rt) => assert_eq!(*rt, roof_type_value),
        _ => panic!("roofType should be String"),
    }

    // Verify geometry
    let geometries = building_obj
        .geometry()
        .expect("Building should have geometry");
    assert_eq!(geometries.len(), 1);
    let geom = model
        .get_geometry(geometries[0])
        .expect("Geometry should exist");
    assert_eq!(geom.type_geometry(), &GeometryType::Solid);
    assert_eq!(geom.lod(), Some(&LoD::LoD1));

    // Verify semantics
    let semantics = geom.semantics().expect("Geometry should have semantics");
    let semantic_surfaces = semantics.surfaces();
    assert_eq!(semantic_surfaces.len(), 6); // bottom, top, 4 walls

    // Check semantic types
    let sem0 = model
        .get_semantic(semantic_surfaces[0].unwrap())
        .expect("Semantic should exist");
    assert_eq!(sem0.type_semantic(), &SemanticType::GroundSurface);

    let sem1 = model
        .get_semantic(semantic_surfaces[1].unwrap())
        .expect("Semantic should exist");
    assert_eq!(sem1.type_semantic(), &SemanticType::RoofSurface);

    let sem2 = model
        .get_semantic(semantic_surfaces[2].unwrap())
        .expect("Semantic should exist");
    assert_eq!(sem2.type_semantic(), &SemanticType::WallSurface);

    println!("Successfully created and verified CityModel with BorrowedStringStorage");
    println!(
        "Model contains {} vertices and {} city objects",
        model.vertices().len(),
        model.cityobjects().len()
    );

    Ok(())
}
