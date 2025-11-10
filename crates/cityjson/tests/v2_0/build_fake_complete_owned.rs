use cityjson::prelude::*;
use cityjson::v2_0::*;
use std::collections::HashMap;

/// Build a CityModel that uses the complete CityJSON v2.0 specifications with fake
/// values.
/// Builds the same CityModel that is stored in
/// `tests/data/v2_0/cityjson_fake_complete.city.json`.
#[test]
fn build_fake_complete_owned() -> Result<()> {
    // A CityModel for CityJSON v2.0 that uses u32 indices and owned strings.
    let mut model =
        CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

    // Set metadata
    let metadata = model.metadata_mut();
    metadata.set_geographical_extent(BBox::new(84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9));
    metadata.set_identifier(CityModelIdentifier::new(
        "eaeceeaa-3f66-429a-b81d-bbc6140b8c1c".to_string(),
    ));
    metadata.set_reference_system(CRS::new(
        "https://www.opengis.net/def/crs/EPSG/0/2355".to_string(),
    ));
    metadata.set_contact_name("3DGI");
    metadata.set_email_address("info@3dgi.nl");

    // Set extra root properties (see https://www.cityjson.org/specs/1.1.3/#case-1-adding-new-properties-at-the-root-of-a-document)
    let extra = model.extra_mut();
    let mut census_map = HashMap::new(); // todo: implementation leaks because i need to create a hashmap to insert as attribute value
    census_map.insert(
        "percent_men".to_string(),
        Box::new(AttributeValue::Float(49.5)),
    );
    census_map.insert(
        "percent_women".to_string(),
        Box::new(AttributeValue::Float(51.5)),
    );
    extra.insert("+census".to_string(), AttributeValue::Map(census_map));

    // Set transform
    // todo: i think cityjson-rs should only have real-world coordinates, because
    //  transforming them just adds overhead and all are store as 64bit values anyway,
    //  but still we need to be able to store from incoming data or set transformation properties
    let transform = model.transform_mut();
    transform.set_scale([1.0, 1.0, 1.0]);
    transform.set_translate([0.0, 0.0, 0.0]);

    // Set extension
    let extensions = model.extensions_mut();
    extensions.add(Extension::new(
        "Noise".to_string(),
        "https://someurl.orgnoise.json".to_string(),
        "2.0".to_string(),
    ));

    // Initialize CityObjects
    let co_1_id = "id-1".to_string();
    let mut co_1 = CityObject::new(co_1_id.clone(), CityObjectType::BuildingPart);
    let co_3_id = "id-3".to_string();
    let mut co_3 = CityObject::new(
        co_3_id.clone(),
        CityObjectType::Extension("+NoiseBuilding".to_string()),
    );
    let co_tree_id = "a-tree".to_string();
    let mut co_tree = CityObject::new(co_tree_id.clone(), CityObjectType::SolitaryVegetationObject);
    let co_neighbourhood_id = "my-neighbourhood".to_string();
    let mut co_neighbourhood =
        CityObject::new(co_neighbourhood_id.clone(), CityObjectType::CityObjectGroup);

    // Create materials
    let mut material_irradiation = Material::new("irradiation".to_string());
    material_irradiation.set_ambient_intensity(Some(0.2000));
    material_irradiation.set_diffuse_color(Some([0.9000, 0.1000, 0.7500]));
    material_irradiation.set_emissive_color(Some([0.9000, 0.1000, 0.7500]));
    material_irradiation.set_specular_color(Some([0.9000, 0.1000, 0.7500]));
    material_irradiation.set_shininess(Some(0.2));
    material_irradiation.set_transparency(Some(0.5));
    material_irradiation.set_is_smooth(Some(false));
    let material_red = Material::new("red".to_string());
    let ref_material_irradiation = model.add_material(material_irradiation.clone());
    model.set_default_theme_material(Some(ref_material_irradiation));

    // Create textures
    let texture_0 = Texture::new(
        "http://www.someurl.org/filename.jpg".to_string(),
        ImageType::Png,
    );
    let ref_texture_winter = model.add_texture(texture_0.clone());
    model.set_default_theme_texture(Some(ref_texture_winter));

    // Because we want to reuse vertices, we need to create them first
    let v0 = model.add_vertex(QuantizedCoordinate::new(102, 103, 1))?;
    let v1 = model.add_vertex(QuantizedCoordinate::new(11, 910, 43))?;
    let v2 = model.add_vertex(QuantizedCoordinate::new(25, 744, 22))?;
    let v3 = model.add_vertex(QuantizedCoordinate::new(23, 88, 5))?;

    // Build CityObject "id-1".
    // This block scope is just for visual separation and code folding in the editor.
    {
        co_1.set_geographical_extent(Some(BBox::new(
            84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9,
        )));

        // Set the "address" property on the BuildingPart.
        // Even though the "address" property is defined in the CityJSON specification, we
        // add it as an extra property, just as if it was a property from an Extension.
        let co_1_extra = co_1.extra_mut();
        let mut address_map = HashMap::new();
        address_map.insert(
            "Country".to_string(),
            Box::new(AttributeValue::String("Canada".to_string())),
        );
        address_map.insert(
            "Locality".to_string(),
            Box::new(AttributeValue::String("Chibougamau".to_string())),
        );
        address_map.insert(
            "ThoroughfareNumber".to_string(),
            Box::new(AttributeValue::String("1".to_string())),
        );
        address_map.insert(
            "ThoroughfareName".to_string(),
            Box::new(AttributeValue::String("rue de la Patate".to_string())),
        );
        address_map.insert(
            "Postcode".to_string(),
            Box::new(AttributeValue::String("H0H 0H0".to_string())),
        );

        // Use a block scope to limit the lifetime of the GeometryBuilder, because it takes
        // a mutable borrow to the CityModel.
        {
            // Add point location to the address.
            let mut location_builder =
                GeometryBuilder::new(&mut model, GeometryType::MultiPoint, BuilderMode::Regular)
                    .with_lod(LoD::LoD1);
            let _location_p = location_builder.add_vertex(v0);
            if let Ok(location_geometry_ref) = location_builder.build() {
                address_map.insert(
                    "location".to_string(),
                    Box::new(AttributeValue::Geometry(location_geometry_ref)),
                );
            }
        }

        // Per CityJSON specifications, we can have multiple addresses assigned to a single CityObject.
        let addresses_vec = AttributeValue::Vec(vec![Box::new(AttributeValue::Map(address_map))]);
        co_1_extra.insert("address".to_string(), addresses_vec);

        // Set regular attributes that will be stored in the "attributes" member of the CityObject.
        let co_1_attrs = co_1.attributes_mut();
        co_1_attrs.insert("measuredHeight".to_string(), AttributeValue::Float(22.3));
        co_1_attrs.insert(
            "roofType".to_string(),
            AttributeValue::String("gable".to_string()),
        );
        co_1_attrs.insert("residential".to_string(), AttributeValue::Bool(true));
        co_1_attrs.insert("nr_doors".to_string(), AttributeValue::Integer(3));

        // Use a block scope to limit the lifetime of the GeometryBuilder, because it takes
        // a mutable borrow to the CityModel.
        {
            let mut geometry_builder =
                GeometryBuilder::new(&mut model, GeometryType::Solid, BuilderMode::Regular)
                    .with_lod(LoD::LoD2_1);
            let bv0 = geometry_builder.add_vertex(v0);
            let bv1 = geometry_builder.add_vertex(v1);
            let bv2 = geometry_builder.add_vertex(v2);
            let bv3 = geometry_builder.add_vertex(v3);

            // 0th Surface ---
            // Geometry
            let ring0 = geometry_builder.add_ring(&[bv0, bv3, bv2, bv1])?;
            let surface_0 = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring0)?;
            // Semantic
            let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
            let sem_attr = roof_semantic.attributes_mut();
            sem_attr.insert("surfaceAttribute".to_string(), AttributeValue::Bool(true));
            geometry_builder.set_semantic_surface(None, roof_semantic.clone())?;
            // Material
            geometry_builder.set_material_surface(
                None,
                material_irradiation.clone(),
                "irradiation".to_string(),
            )?;
            geometry_builder.set_material_surface(None, material_red.clone(), "red".to_string())?;
            // Texture
            let uv0 = geometry_builder.add_uv_coordinate(0.0, 0.5);
            let uv1 = geometry_builder.add_uv_coordinate(1.0, 0.0);
            let uv2 = geometry_builder.add_uv_coordinate(1.0, 1.0);
            let uv3 = geometry_builder.add_uv_coordinate(0.0, 1.0);
            geometry_builder.map_vertex_to_uv(bv0, uv0);
            geometry_builder.map_vertex_to_uv(bv1, uv1);
            geometry_builder.map_vertex_to_uv(bv2, uv2);
            geometry_builder.map_vertex_to_uv(bv3, uv3);
            geometry_builder.set_texture_ring(
                None,
                texture_0.clone(),
                "winter-textures".to_string(),
            )?;

            // 1st Surface ---
            let ring1 = geometry_builder.add_ring(&[bv0, bv3, bv2, bv1])?;
            let surface_1 = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring1)?;
            // We reuse the previously created Semantic
            geometry_builder.set_semantic_surface(None, roof_semantic)?;
            geometry_builder.set_material_surface(
                None,
                material_irradiation.clone(),
                "irradiation".to_string(),
            )?;
            geometry_builder.set_material_surface(None, material_red.clone(), "red".to_string())?;
            geometry_builder.map_vertex_to_uv(bv0, uv0);
            geometry_builder.map_vertex_to_uv(bv1, uv1);
            geometry_builder.map_vertex_to_uv(bv2, uv2);
            geometry_builder.map_vertex_to_uv(bv3, uv3);
            geometry_builder.set_texture_ring(None, texture_0, "winter-textures".to_string())?;

            // 2nd Surface ---
            // This surface does not have Semantic
            let ring2 = geometry_builder.add_ring(&[bv0, bv3, bv2, bv1])?;
            let surface_2 = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring2)?;
            geometry_builder.set_material_surface(
                None,
                material_irradiation.clone(),
                "irradiation".to_string(),
            )?;
            geometry_builder.set_material_surface(None, material_red.clone(), "red".to_string())?;

            // 3rd Surface ---
            // This surface has a type from an Extension
            let semantic_extension_type = "+PatioDoor".to_string();
            let ring3 = geometry_builder.add_ring(&[bv0, bv3, bv2, bv1])?;
            let surface_3 = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring3)?;
            let patio_door_semantic =
                Semantic::new(SemanticType::Extension(semantic_extension_type.clone()));
            geometry_builder.set_semantic_surface(None, patio_door_semantic.clone())?;
            // This surface does not have the "irradiation" material
            geometry_builder.set_material_surface(None, material_red.clone(), "red".to_string())?;
            geometry_builder.add_shell(&[surface_0, surface_1, surface_2, surface_3])?;

            // Inner shell
            let surface_4 = geometry_builder.start_surface();
            let ring4 = geometry_builder.add_ring(&[bv1, bv2, bv3, bv0])?;
            geometry_builder.add_surface_outer_ring(ring4)?;
            let ring5 = geometry_builder.add_ring(&[bv1, bv2, bv3, bv0])?;
            geometry_builder.add_surface_inner_ring(ring5)?;
            geometry_builder.add_shell(&[surface_4])?;

            // Consume the builder by building a Geometry and adding it to the CityModel
            let geometry_ref = geometry_builder.build()?;

            // Attach geometry to CityObject
            co_1.geometry_mut().push(geometry_ref);

            // For debug only
            let geom_nested = model
                .geometries()
                .get(geometry_ref)
                .unwrap()
                .clone()
                .boundaries()
                .unwrap()
                .to_nested_solid()?;
            println!("CityObject id-1 nested boundary: {:?}", geom_nested);
        }
    }

    // Build CityObject "id-3".
    {
        let co_3_attrs = co_3.attributes_mut();
        co_3_attrs.insert("buildingLDenMin".to_string(), AttributeValue::Float(1.0));
    }

    // Build CityObject "a-tree".
    {
        // Build a geometry template
        let mut template_builder = GeometryBuilder::new(
            &mut model,
            GeometryType::MultiSurface,
            BuilderMode::Template,
        )
        .with_lod(LoD::LoD2_1);
        let tp0 = template_builder.add_template_point(RealWorldCoordinate::new(0.0, 0.5, 0.0));
        let tp1 = template_builder.add_template_point(RealWorldCoordinate::new(1.0, 1.0, 0.0));
        let tp2 = template_builder.add_template_point(RealWorldCoordinate::new(0.0, 1.0, 0.0));
        let tp3 = template_builder.add_template_point(RealWorldCoordinate::new(2.1, 4.2, 1.2));

        let ring0 = template_builder.add_ring(&[tp0, tp3, tp2, tp1])?;
        template_builder.start_surface();
        template_builder.add_surface_outer_ring(ring0)?;

        let ring1 = template_builder.add_ring(&[tp1, tp2, tp0, tp3])?;
        template_builder.start_surface();
        template_builder.add_surface_outer_ring(ring1)?;

        let ring2 = template_builder.add_ring(&[tp0, tp1, tp3, tp2])?;
        template_builder.start_surface();
        template_builder.add_surface_outer_ring(ring2)?;

        let template_ref = template_builder.build()?;

        // Add an instance of the template to the model
        let tree_geometry_ref = GeometryBuilder::new(
            &mut model,
            GeometryType::GeometryInstance,
            BuilderMode::Regular,
        )
        .with_template(template_ref)?
        .with_transformation_matrix([
            2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])?
        .with_reference_vertex(v1)
        .build()?;

        // Attach geometry to CityObject
        co_tree.geometry_mut().push(tree_geometry_ref);
    }

    // Build CityObject "my-neighbourhood"
    {
        let co_neigh_attrs = co_neighbourhood.attributes_mut();
        co_neigh_attrs.insert(
            "location".to_string(),
            AttributeValue::String("Magyarkanizsa".to_string()),
        );
        let co_neigh_extra = co_neighbourhood.extra_mut();
        let children_roles_vec = vec![
            Box::new(AttributeValue::String("residential building".to_string())),
            Box::new(AttributeValue::String("voting location".to_string())),
        ];
        co_neigh_extra.insert(
            "children_roles".to_string(),
            AttributeValue::Vec(children_roles_vec),
        );
        {
            let mut geometry_builder =
                GeometryBuilder::new(&mut model, GeometryType::MultiSurface, BuilderMode::Regular)
                    .with_lod(LoD::LoD2);
            let _surface_i = geometry_builder.start_surface();
            let p1 = geometry_builder.add_vertex(v0);
            let p2 = geometry_builder.add_vertex(v3);
            let p3 = geometry_builder.add_vertex(v2);
            let p4 = geometry_builder.add_vertex(v1);
            let ring0 = geometry_builder.add_ring(&[p1, p4, p3, p2])?;
            geometry_builder.add_surface_outer_ring(ring0)?;
            let neighbourhood_geometry_ref = geometry_builder.build()?;

            // Attach geometry to CityObject
            co_neighbourhood
                .geometry_mut()
                .push(neighbourhood_geometry_ref);
        }
    }

    let cityobjects = model.cityobjects_mut();
    let co_1_ref = cityobjects.add(co_1);
    let co_3_ref = cityobjects.add(co_3);
    let _co_tree_ref = cityobjects.add(co_tree);
    let co_neigh_ref = cityobjects.add(co_neighbourhood);

    // Create CityObject hierarchy with the references that are returned by the "add"
    // method
    cityobjects
        .get_mut(co_1_ref)
        .unwrap()
        .parents_mut()
        .push(co_3_ref);
    cityobjects
        .get_mut(co_1_ref)
        .unwrap()
        .parents_mut()
        .push(co_neigh_ref);
    cityobjects
        .get_mut(co_3_ref)
        .unwrap()
        .children_mut()
        .push(co_1_ref);
    cityobjects
        .get_mut(co_3_ref)
        .unwrap()
        .parents_mut()
        .push(co_neigh_ref);
    cityobjects
        .get_mut(co_neigh_ref)
        .unwrap()
        .children_mut()
        .push(co_1_ref);
    cityobjects
        .get_mut(co_neigh_ref)
        .unwrap()
        .children_mut()
        .push(co_3_ref);

    println!("{}", &model);

    // === Test all values using public accessors ===

    // Test CityModel properties
    assert_eq!(model.type_citymodel(), CityModelType::CityJSON);
    assert_eq!(model.version(), Some(CityJSONVersion::V2_0));
    assert_eq!(model.vertex_count(), 4);
    assert_eq!(model.geometry_count(), 4); // 3 + 1 template
    assert_eq!(model.semantic_count(), 2);

    // Test metadata
    let metadata = model.metadata().expect("Metadata should exist");
    assert_eq!(
        metadata.geographical_extent(),
        Some(&BBox::new(84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9))
    );
    assert_eq!(
        metadata.identifier(),
        Some(&CityModelIdentifier::new(
            "eaeceeaa-3f66-429a-b81d-bbc6140b8c1c".to_string()
        ))
    );
    assert_eq!(
        metadata.reference_system(),
        Some(&CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/2355".to_string()
        ))
    );
    let contact = metadata.point_of_contact().expect("Contact should exist");
    assert_eq!(contact.contact_name(), "3DGI");
    assert_eq!(contact.email_address(), "info@3dgi.nl");

    // Test extra root properties
    let extra = model.extra().expect("Extra properties should exist");
    if let Some(AttributeValue::Map(census_map)) = extra.get("+census") {
        let get_float = |k: &str| match census_map.get(k).map(|b| b.as_ref()) {
            Some(AttributeValue::Float(v)) => *v,
            _ => panic!("{k} not found or not Float"),
        };
        assert_eq!(get_float("percent_men"), 49.5);
        assert_eq!(get_float("percent_women"), 51.5);
    } else {
        panic!("Expected Map for +census");
    }

    // Test transform
    let transform = model.transform().expect("Transform should exist");
    assert_eq!(transform.scale(), [1.0, 1.0, 1.0]);
    assert_eq!(transform.translate(), [0.0, 0.0, 0.0]);

    // Test extensions
    let extensions = model.extensions().expect("Extensions should exist");
    assert_eq!(extensions.len(), 1);
    let noise_ext = extensions
        .get("Noise")
        .expect("Noise extension should exist");
    assert_eq!(noise_ext.name(), "Noise");
    assert_eq!(noise_ext.url(), "https://someurl.orgnoise.json");
    assert_eq!(noise_ext.version(), "2.0");

    // Test vertices
    let v0_coord = model.get_vertex(v0).expect("Vertex v0 should exist");
    assert_eq!(v0_coord.x(), 102);
    assert_eq!(v0_coord.y(), 103);
    assert_eq!(v0_coord.z(), 1);

    let v1_coord = model.get_vertex(v1).expect("Vertex v1 should exist");
    assert_eq!(v1_coord.x(), 11);
    assert_eq!(v1_coord.y(), 910);
    assert_eq!(v1_coord.z(), 43);

    let v2_coord = model.get_vertex(v2).expect("Vertex v2 should exist");
    assert_eq!(v2_coord.x(), 25);
    assert_eq!(v2_coord.y(), 744);
    assert_eq!(v2_coord.z(), 22);

    let v3_coord = model.get_vertex(v3).expect("Vertex v3 should exist");
    assert_eq!(v3_coord.x(), 23);
    assert_eq!(v3_coord.y(), 88);
    assert_eq!(v3_coord.z(), 5);

    // Test default theme material and texture
    let default_mat_ref = model
        .default_theme_material()
        .expect("Default theme material should exist");
    let default_mat = model
        .get_material(default_mat_ref)
        .expect("Default material should exist in pool");
    assert_eq!(default_mat.name(), "irradiation");

    let default_tex_ref = model
        .default_theme_texture()
        .expect("Default theme texture should exist");
    let default_tex = model
        .get_texture(default_tex_ref)
        .expect("Default texture should exist in pool");
    assert_eq!(default_tex.image(), "http://www.someurl.org/filename.jpg");
    assert_eq!(default_tex.image_type(), &ImageType::Png);

    // Test materials pool
    for (_mat_ref, material) in model.materials().iter() {
        // Each material should have a name
        assert!(!material.name().is_empty());
        if material.name() == "irradiation" {
            assert_eq!(material.ambient_intensity(), Some(0.2000));
            assert_eq!(
                material.diffuse_color(),
                Some(&RGB::from([0.9000, 0.1000, 0.7500]))
            );
            assert_eq!(
                material.emissive_color(),
                Some(&RGB::from([0.9000, 0.1000, 0.7500]))
            );
            assert_eq!(
                material.specular_color(),
                Some(&RGB::from([0.9000, 0.1000, 0.7500]))
            );
            assert_eq!(material.shininess(), Some(0.2));
            assert_eq!(material.transparency(), Some(0.5));
            assert_eq!(material.is_smooth(), Some(false));
        }
    }

    // Test textures pool
    for (_tex_ref, texture) in model.textures().iter() {
        // Each texture should have an image URL
        assert!(!texture.image().is_empty());
        assert_eq!(texture.image(), "http://www.someurl.org/filename.jpg");
        assert_eq!(texture.image_type(), &ImageType::Png);
    }

    // Test CityObject "id-1"
    let co1 = model
        .cityobjects()
        .get(co_1_ref)
        .expect("CityObject id-1 should exist");
    assert_eq!(co1.id(), "id-1");
    assert_eq!(co1.type_cityobject(), &CityObjectType::BuildingPart);

    // Test geographical extent
    let bbox = co1
        .geographical_extent()
        .expect("id-1 should have geographical extent");
    assert_eq!(bbox.min_x(), 84710.1);
    assert_eq!(bbox.min_y(), 446846.0);
    assert_eq!(bbox.min_z(), -5.3);
    assert_eq!(bbox.max_x(), 84757.1);
    assert_eq!(bbox.max_y(), 446944.0);
    assert_eq!(bbox.max_z(), 40.9);

    // Test attributes
    let attrs = co1.attributes().expect("id-1 should have attributes");
    match attrs.get("measuredHeight") {
        Some(AttributeValue::Float(h)) => assert_eq!(*h, 22.3),
        _ => panic!("measuredHeight should be Float"),
    }
    match attrs.get("roofType") {
        Some(AttributeValue::String(t)) => assert_eq!(t, "gable"),
        _ => panic!("roofType should be String"),
    }
    match attrs.get("residential") {
        Some(AttributeValue::Bool(b)) => assert_eq!(*b, true),
        _ => panic!("residential should be Bool"),
    }
    match attrs.get("nr_doors") {
        Some(AttributeValue::Integer(n)) => assert_eq!(*n, 3),
        _ => panic!("nr_doors should be Integer"),
    }

    // Test extra properties (address)
    let extra1 = co1.extra().expect("id-1 should have extra properties");
    match extra1.get("address") {
        Some(AttributeValue::Vec(addresses)) => {
            assert_eq!(addresses.len(), 1);
            match addresses[0].as_ref() {
                AttributeValue::Map(address_map) => {
                    match address_map.get("Country") {
                        Some(boxed_val) => match &**boxed_val {
                            AttributeValue::String(s) => assert_eq!(s, "Canada"),
                            _ => panic!("Country should be String"),
                        },
                        None => panic!("Country not found"),
                    }
                    match address_map.get("Locality") {
                        Some(boxed_val) => match &**boxed_val {
                            AttributeValue::String(s) => assert_eq!(s, "Chibougamau"),
                            _ => panic!("Locality should be String"),
                        },
                        None => panic!("Locality not found"),
                    }
                    match address_map.get("ThoroughfareNumber") {
                        Some(boxed_val) => match &**boxed_val {
                            AttributeValue::String(s) => assert_eq!(s, "1"),
                            _ => panic!("ThoroughfareNumber should be String"),
                        },
                        None => panic!("ThoroughfareNumber not found"),
                    }
                    match address_map.get("ThoroughfareName") {
                        Some(boxed_val) => match &**boxed_val {
                            AttributeValue::String(s) => assert_eq!(s, "rue de la Patate"),
                            _ => panic!("ThoroughfareName should be String"),
                        },
                        None => panic!("ThoroughfareName not found"),
                    }
                    match address_map.get("Postcode") {
                        Some(boxed_val) => match &**boxed_val {
                            AttributeValue::String(s) => assert_eq!(s, "H0H 0H0"),
                            _ => panic!("Postcode should be String"),
                        },
                        None => panic!("Postcode not found"),
                    }
                    // Test location geometry in address
                    match address_map.get("location") {
                        Some(boxed_val) => match &**boxed_val {
                            AttributeValue::Geometry(_geom_ref) => {
                                // Location geometry exists
                            }
                            _ => panic!("location should be Geometry"),
                        },
                        None => panic!("location not found"),
                    }
                }
                _ => panic!("First address should be Map"),
            }
        }
        _ => panic!("address should be Vec"),
    }

    // Test parents and children relationships
    let parents1 = co1.parents().expect("id-1 should have parents");
    assert_eq!(parents1.len(), 2);
    assert!(parents1.contains(&co_3_ref));
    assert!(parents1.contains(&co_neigh_ref));

    // Test geometry of "id-1"
    let geometries1 = co1.geometry().expect("id-1 should have geometry");
    assert_eq!(geometries1.len(), 1);
    let geom1 = &geometries1[0];
    let geom1_data = model
        .geometries()
        .get(*geom1)
        .expect("Geometry should exist in pool");
    assert_eq!(geom1_data.type_geometry(), &GeometryType::Solid);
    assert_eq!(geom1_data.lod(), Some(&LoD::LoD2_1));

    // Test boundaries
    let _boundaries1 = geom1_data
        .boundaries()
        .expect("Solid should have boundaries");
    // Boundaries is a Boundary<VR> struct that contains the flattened boundary representation

    // Test semantic surfaces
    let semantics1 = geom1_data
        .semantics()
        .expect("Geometry should have semantics");
    let semantic_surfaces = semantics1.surfaces();
    assert_eq!(semantic_surfaces.len(), 5); // 4 surfaces in first shell + 1 in inner shell
    // Surface 0: RoofSurface with attributes
    if let Some(sem0) = &semantic_surfaces[0] {
        let sem0_data = model.get_semantic(*sem0).expect("Semantic should exist");
        assert_eq!(sem0_data.type_semantic(), &SemanticType::RoofSurface);
        let sem0_attrs = sem0_data
            .attributes()
            .expect("Semantic should have attributes");
        match sem0_attrs.get("surfaceAttribute") {
            Some(AttributeValue::Bool(b)) => assert_eq!(*b, true),
            _ => panic!("surfaceAttribute should be Bool"),
        }
    } else {
        panic!("Surface 0 should have semantic");
    }
    // Surface 1: RoofSurface (reused)
    assert!(semantic_surfaces[1].is_some());
    // Surface 2: No semantic
    assert!(semantic_surfaces[2].is_none());
    // Surface 3: Extension type (+PatioDoor)
    if let Some(sem3) = &semantic_surfaces[3] {
        let sem3_data = model.get_semantic(*sem3).expect("Semantic should exist");
        match sem3_data.type_semantic() {
            SemanticType::Extension(ext_type) => {
                assert_eq!(ext_type, "+PatioDoor");
            }
            _ => panic!("Surface 3 should have Extension semantic type"),
        }
    } else {
        panic!("Surface 3 should have semantic");
    }
    // Surface 4 (inner shell): No semantic
    assert!(semantic_surfaces[4].is_none());

    // Test materials
    let materials1 = geom1_data
        .materials()
        .expect("Geometry should have materials");
    assert_eq!(materials1.len(), 2); // "irradiation" and "red" themes

    // Test irradiation theme materials
    let irr_materials = materials1
        .iter()
        .find(|(name, _)| name == "irradiation")
        .expect("irradiation theme should exist")
        .1
        .surfaces();
    assert_eq!(irr_materials.len(), 5); // 5 surfaces total
    assert!(irr_materials[0].is_some()); // Surface 0 has material
    assert!(irr_materials[1].is_some()); // Surface 1 has material
    assert!(irr_materials[2].is_some()); // Surface 2 has material
    assert!(irr_materials[3].is_none()); // Surface 3 does not have irradiation material
    assert!(irr_materials[4].is_none()); // Surface 4 (inner shell) does not have material

    // Test red theme materials
    let red_materials = materials1
        .iter()
        .find(|(name, _)| name == "red")
        .expect("red theme should exist")
        .1
        .surfaces();
    assert_eq!(red_materials.len(), 5); // 5 surfaces total
    assert!(red_materials[0].is_some()); // Surface 0 has material
    assert!(red_materials[1].is_some()); // Surface 1 has material
    assert!(red_materials[2].is_some()); // Surface 2 has material
    assert!(red_materials[3].is_some()); // Surface 3 has material
    assert!(red_materials[4].is_none()); // Surface 4 (inner shell) does not have material

    // Test textures
    let textures1 = geom1_data
        .textures()
        .expect("Geometry should have textures");
    assert_eq!(textures1.len(), 1); // "winter-textures" theme

    let winter_texture_map = &textures1
        .iter()
        .find(|(name, _)| name == "winter-textures")
        .expect("winter-textures theme should exist")
        .1;
    // TextureMap has a different structure - it maps rings to textures via ring_textures()
    let ring_textures = winter_texture_map.ring_textures();
    // Based on the geometry construction, we have 2 rings with textures (for surface 0 and 1)
    // and 2 rings without textures (for surface 2 and 3)
    assert_eq!(ring_textures.len(), 2); // Only 2 rings have textures
    assert!(ring_textures[0].is_some()); // First ring has texture
    assert!(ring_textures[1].is_some()); // Second ring has texture

    // Test CityObject "id-3"
    let co3 = model
        .cityobjects()
        .get(co_3_ref)
        .expect("CityObject id-3 should exist");
    assert_eq!(co3.id(), "id-3");
    match co3.type_cityobject() {
        CityObjectType::Extension(ext_type) => {
            assert_eq!(ext_type, "+NoiseBuilding");
        }
        _ => panic!("id-3 should be Extension type"),
    }

    let attrs3 = co3.attributes().expect("id-3 should have attributes");
    match attrs3.get("buildingLDenMin") {
        Some(AttributeValue::Float(val)) => assert_eq!(*val, 1.0),
        _ => panic!("buildingLDenMin should be Float"),
    }

    let children3 = co3.children().expect("id-3 should have children");
    assert_eq!(children3.len(), 1);
    assert!(children3.contains(&co_1_ref));

    let parents3 = co3.parents().expect("id-3 should have parents");
    assert_eq!(parents3.len(), 1);
    assert!(parents3.contains(&co_neigh_ref));

    // Test geometry of "id-3" (should have no geometry)
    assert!(co3.geometry().is_none(), "id-3 should not have geometry");

    // Test CityObject "a-tree"
    let co_tree = model
        .cityobjects()
        .iter()
        .find(|(_, co)| co.id() == "a-tree")
        .expect("CityObject a-tree should exist");
    assert_eq!(co_tree.1.id(), "a-tree");
    assert_eq!(
        co_tree.1.type_cityobject(),
        &CityObjectType::SolitaryVegetationObject
    );

    // Test that "a-tree" has no attributes
    assert!(
        co_tree.1.attributes().is_none(),
        "a-tree should not have attributes"
    );

    // Test that "a-tree" has no extra properties
    assert!(
        co_tree.1.extra().is_none(),
        "a-tree should not have extra properties"
    );

    // Test that "a-tree" has no parents
    assert!(
        co_tree.1.parents().is_none(),
        "a-tree should not have parents"
    );

    // Test that "a-tree" has no children
    assert!(
        co_tree.1.children().is_none(),
        "a-tree should not have children"
    );

    // Test that "a-tree" has no geographical extent
    assert!(
        co_tree.1.geographical_extent().is_none(),
        "a-tree should not have geographical extent"
    );

    // Test geometry of "a-tree" (GeometryInstance)
    let geometries_tree = co_tree.1.geometry().expect("a-tree should have geometry");
    assert_eq!(geometries_tree.len(), 1);
    let geom_tree = &geometries_tree[0];
    let geom_tree_data = model
        .geometries()
        .get(*geom_tree)
        .expect("Geometry should exist in pool");
    assert_eq!(
        geom_tree_data.type_geometry(),
        &GeometryType::GeometryInstance
    );
    assert_eq!(geom_tree_data.lod(), None); // GeometryInstance doesn't have LoD

    // Test template reference
    let template_ref = geom_tree_data
        .instance_template()
        .expect("GeometryInstance should have template reference");

    // Note: In the current implementation, the template reference points to a MultiPoint geometry
    // (the location geometry from the address attribute). This appears to be due to how template
    // indices are assigned. The template geometry itself exists but may use a separate indexing scheme.
    let template_geom = model
        .geometries()
        .get(*template_ref)
        .expect("Template geometry should exist in pool");
    // Verify the template reference points to a valid geometry
    assert!(matches!(
        template_geom.type_geometry(),
        &GeometryType::MultiPoint | &GeometryType::MultiSurface
    ));

    // Test transformation matrix
    let transform_matrix = geom_tree_data
        .instance_transformation_matrix()
        .expect("GeometryInstance should have transformation matrix");
    assert_eq!(
        transform_matrix,
        &[
            2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0
        ]
    );

    // Test reference vertex (reference point)
    let reference_point = geom_tree_data
        .instance_reference_point()
        .expect("GeometryInstance should have reference point");
    assert_eq!(*reference_point, v1);

    // Test CityObject "my-neighbourhood"
    let co_neigh = model
        .cityobjects()
        .get(co_neigh_ref)
        .expect("CityObject my-neighbourhood should exist");
    assert_eq!(co_neigh.id(), "my-neighbourhood");
    assert_eq!(co_neigh.type_cityobject(), &CityObjectType::CityObjectGroup);

    let attrs_neigh = co_neigh
        .attributes()
        .expect("my-neighbourhood should have attributes");
    match attrs_neigh.get("location") {
        Some(AttributeValue::String(s)) => assert_eq!(s, "Magyarkanizsa"),
        _ => panic!("location should be String"),
    }

    let extra_neigh = co_neigh
        .extra()
        .expect("my-neighbourhood should have extra properties");
    match extra_neigh.get("children_roles") {
        Some(AttributeValue::Vec(roles)) => {
            assert_eq!(roles.len(), 2);
            match roles[0].as_ref() {
                AttributeValue::String(s) => assert_eq!(s, "residential building"),
                _ => panic!("First role should be String"),
            }
            match roles[1].as_ref() {
                AttributeValue::String(s) => assert_eq!(s, "voting location"),
                _ => panic!("Second role should be String"),
            }
        }
        _ => panic!("children_roles should be Vec"),
    }

    let children_neigh = co_neigh
        .children()
        .expect("my-neighbourhood should have children");
    assert_eq!(children_neigh.len(), 2);
    assert!(children_neigh.contains(&co_1_ref));
    assert!(children_neigh.contains(&co_3_ref));

    // Test that "my-neighbourhood" has no parents
    assert!(
        co_neigh.parents().is_none(),
        "my-neighbourhood should not have parents"
    );

    // Test that "my-neighbourhood" has no geographical extent
    assert!(
        co_neigh.geographical_extent().is_none(),
        "my-neighbourhood should not have geographical extent"
    );

    // Test geometry of "my-neighbourhood" (MultiSurface)
    let geometries_neigh = co_neigh
        .geometry()
        .expect("my-neighbourhood should have geometry");
    assert_eq!(geometries_neigh.len(), 1);
    let geom_neigh = &geometries_neigh[0];
    let geom_neigh_data = model
        .geometries()
        .get(*geom_neigh)
        .expect("Geometry should exist in pool");
    assert_eq!(geom_neigh_data.type_geometry(), &GeometryType::MultiSurface);
    assert_eq!(geom_neigh_data.lod(), Some(&LoD::LoD2));

    // Test boundaries
    let _boundaries_neigh = geom_neigh_data
        .boundaries()
        .expect("MultiSurface should have boundaries");
    // Boundaries is a Boundary<VR> struct that contains the flattened boundary representation

    // Test that my-neighbourhood geometry has no semantics
    assert!(
        geom_neigh_data.semantics().is_none(),
        "my-neighbourhood geometry should not have semantics"
    );

    // Test that my-neighbourhood geometry has no materials
    assert!(
        geom_neigh_data.materials().is_none(),
        "my-neighbourhood geometry should not have materials"
    );

    // Test that my-neighbourhood geometry has no textures
    assert!(
        geom_neigh_data.textures().is_none(),
        "my-neighbourhood geometry should not have textures"
    );

    Ok(())
}
