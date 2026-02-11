use cityjson::prelude::*;
use cityjson::v2_0::GeometryBuilder;
use cityjson::v2_0::*;

type BorrowedModel = CityModel<u32, BorrowedStringStorage<'static>>;
type BorrowedCityObject = CityObject<BorrowedStringStorage<'static>>;

struct TestInputs {
    model_id: &'static str,
    crs_url: &'static str,
    contact_name: &'static str,
    contact_email: &'static str,
    extension_name: &'static str,
    extension_url: &'static str,
    extension_version: &'static str,
    building_id: &'static str,
    attribute_key: &'static str,
    roof_type_key: &'static str,
    roof_type_value: &'static str,
}

fn test_inputs() -> TestInputs {
    TestInputs {
        model_id: "550e8400-e29b-41d4-a716-446655440000",
        crs_url: "https://www.opengis.net/def/crs/EPSG/0/7415",
        contact_name: "Test Organization",
        contact_email: "test@example.org",
        extension_name: "Noise",
        extension_url: "https://example.org/noise.json",
        extension_version: "2.0",
        building_id: "building-1",
        attribute_key: "yearOfConstruction",
        roof_type_key: "roofType",
        roof_type_value: "flat",
    }
}

#[test]
fn test_citymodel_with_borrowed_storage() -> Result<()> {
    let inputs = test_inputs();
    let (model, building_ref) = build_model(&inputs)?;

    assert_model_contents(&model, building_ref, &inputs);

    println!("Successfully created and verified CityModel with BorrowedStringStorage");
    println!(
        "Model contains {} vertices and {} city objects",
        model.vertices().len(),
        model.cityobjects().len()
    );

    Ok(())
}

fn build_model(inputs: &TestInputs) -> Result<(BorrowedModel, CityObjectRef)> {
    let mut model = BorrowedModel::new(CityModelType::CityJSON);
    configure_metadata(&mut model, inputs);
    configure_transform_and_extension(&mut model, inputs);
    let building_ref = add_building_cityobject(&mut model, inputs)?;
    Ok((model, building_ref))
}

fn configure_metadata(model: &mut BorrowedModel, inputs: &TestInputs) {
    let metadata = model.metadata_mut();
    metadata.set_geographical_extent(BBox::new(1000.0, 2000.0, 0.0, 1500.0, 2500.0, 100.0));
    metadata.set_identifier(CityModelIdentifier::new(inputs.model_id));
    metadata.set_reference_system(CRS::new(inputs.crs_url));
    metadata.set_contact_name(inputs.contact_name);
    metadata.set_email_address(inputs.contact_email);
}

fn configure_transform_and_extension(model: &mut BorrowedModel, inputs: &TestInputs) {
    let transform = model.transform_mut();
    transform.set_scale([1.0, 1.0, 1.0]);
    transform.set_translate([0.0, 0.0, 0.0]);

    model.extensions_mut().add(Extension::new(
        inputs.extension_name,
        inputs.extension_url,
        inputs.extension_version,
    ));
}

fn add_building_cityobject(
    model: &mut BorrowedModel,
    inputs: &TestInputs,
) -> Result<CityObjectRef> {
    let vertices = add_building_vertices(model)?;
    let mut building = CityObject::new(
        CityObjectIdentifier::new(inputs.building_id),
        CityObjectType::Building,
    );
    building
        .attributes_mut()
        .insert(inputs.attribute_key, AttributeValue::Integer(2020));
    building.attributes_mut().insert(
        inputs.roof_type_key,
        AttributeValue::String(inputs.roof_type_value),
    );

    add_building_geometry(model, &mut building, vertices)?;
    model.cityobjects_mut().add(building)
}

fn add_building_vertices(model: &mut BorrowedModel) -> Result<[VertexIndex<u32>; 8]> {
    Ok([
        model.add_vertex(QuantizedCoordinate::new(0, 0, 0))?,
        model.add_vertex(QuantizedCoordinate::new(100, 0, 0))?,
        model.add_vertex(QuantizedCoordinate::new(100, 100, 0))?,
        model.add_vertex(QuantizedCoordinate::new(0, 100, 0))?,
        model.add_vertex(QuantizedCoordinate::new(0, 0, 50))?,
        model.add_vertex(QuantizedCoordinate::new(100, 0, 50))?,
        model.add_vertex(QuantizedCoordinate::new(100, 100, 50))?,
        model.add_vertex(QuantizedCoordinate::new(0, 100, 50))?,
    ])
}

fn add_building_geometry(
    model: &mut BorrowedModel,
    building: &mut BorrowedCityObject,
    vertices: [VertexIndex<u32>; 8],
) -> Result<()> {
    let mut geometry_builder =
        GeometryBuilder::new(model, GeometryType::Solid, BuilderMode::Regular).with_lod(LoD::LoD1);

    let [v0, v1, v2, v3, v4, v5, v6, v7] = vertices;
    let bv0 = geometry_builder.add_vertex(v0);
    let bv1 = geometry_builder.add_vertex(v1);
    let bv2 = geometry_builder.add_vertex(v2);
    let bv3 = geometry_builder.add_vertex(v3);
    let bv4 = geometry_builder.add_vertex(v4);
    let bv5 = geometry_builder.add_vertex(v5);
    let bv6 = geometry_builder.add_vertex(v6);
    let bv7 = geometry_builder.add_vertex(v7);

    let ring_bottom = geometry_builder.add_ring(&[bv0, bv1, bv2, bv3])?;
    let surface_bottom = geometry_builder.start_surface();
    geometry_builder.add_surface_outer_ring(ring_bottom)?;
    geometry_builder.set_semantic_surface(
        None,
        Semantic::new(SemanticType::GroundSurface),
        false,
    )?;

    let ring_top = geometry_builder.add_ring(&[bv4, bv7, bv6, bv5])?;
    let surface_top = geometry_builder.start_surface();
    geometry_builder.add_surface_outer_ring(ring_top)?;
    geometry_builder.set_semantic_surface(None, Semantic::new(SemanticType::RoofSurface), false)?;

    let wall_semantic = Semantic::new(SemanticType::WallSurface);
    let ring_wall1 = geometry_builder.add_ring(&[bv0, bv4, bv5, bv1])?;
    let surface_wall1 = geometry_builder.start_surface();
    geometry_builder.add_surface_outer_ring(ring_wall1)?;
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

    geometry_builder.add_shell(&[
        surface_bottom,
        surface_top,
        surface_wall1,
        surface_wall2,
        surface_wall3,
        surface_wall4,
    ])?;

    let geometry_ref = geometry_builder.build_geometry()?;
    building.add_geometry(geometry_ref);
    Ok(())
}

fn assert_model_contents(model: &BorrowedModel, building_ref: CityObjectRef, inputs: &TestInputs) {
    assert_eq!(model.type_citymodel(), CityModelType::CityJSON);
    assert_eq!(model.version(), Some(CityJSONVersion::V2_0));
    assert_eq!(model.vertices().len(), 8);
    assert_eq!(model.geometry_count(), 1);
    assert_eq!(model.semantic_count(), 3);

    let metadata = model.metadata().expect("Metadata should exist");
    assert_eq!(
        metadata.identifier(),
        Some(&CityModelIdentifier::new(inputs.model_id))
    );
    assert_eq!(metadata.reference_system(), Some(&CRS::new(inputs.crs_url)));
    let contact = metadata.point_of_contact().expect("Contact should exist");
    assert_eq!(contact.contact_name(), inputs.contact_name);
    assert_eq!(contact.email_address(), inputs.contact_email);

    let extensions = model.extensions().expect("Extensions should exist");
    let noise_ext = extensions
        .get(inputs.extension_name)
        .expect("Extension should exist");
    assert_eq!(*noise_ext.name(), inputs.extension_name);

    let building_obj = model
        .cityobjects()
        .get(building_ref)
        .expect("Building should exist");
    assert_eq!(building_obj.id(), inputs.building_id);
    assert_eq!(building_obj.type_cityobject(), &CityObjectType::Building);

    let attrs = building_obj
        .attributes()
        .expect("Building should have attributes");
    match attrs
        .get(inputs.attribute_key)
        .expect("yearOfConstruction should exist")
    {
        AttributeValue::Integer(year) => assert_eq!(*year, 2020),
        _ => panic!("yearOfConstruction should be Integer"),
    }
    match attrs
        .get(inputs.roof_type_key)
        .expect("roofType should exist")
    {
        AttributeValue::String(rt) => assert_eq!(*rt, inputs.roof_type_value),
        _ => panic!("roofType should be String"),
    }

    let geometries = building_obj
        .geometry()
        .expect("Building should have geometry");
    assert_eq!(geometries.len(), 1);
    let geom = model
        .get_geometry(geometries[0])
        .expect("Geometry should exist");
    assert_eq!(geom.type_geometry(), &GeometryType::Solid);
    assert_eq!(geom.lod(), Some(&LoD::LoD1));

    let semantics = geom.semantics().expect("Geometry should have semantics");
    let semantic_surfaces = semantics.surfaces();
    assert_eq!(semantic_surfaces.len(), 6);

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
}
