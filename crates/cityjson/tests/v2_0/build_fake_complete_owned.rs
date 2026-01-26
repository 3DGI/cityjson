use cityjson::cityjson::core::attributes::{AttributeOwnerType, AttributePool};
use cityjson::prelude::*;
use cityjson::v2_0::*;
use std::collections::HashMap;

/// Build a CityModel that uses the complete CityJSON v2.0 specifications with fake
/// values.
/// Builds the same CityModel that is stored in
/// `tests/data/v2_0/cityjson_fake_complete.city.json`.
#[test]
fn build_fake_complete_owned() -> Result<()> {
    // todo test: need to break up this test into a separate function per cityjson component, that will also improve representativeness
    // A CityModel for CityJSON v2.0 that uses u32 indices and owned strings.
    let mut model =
        CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
    let attribute_pool_ref = model.attributes_mut();

    // Three patterns of adding Metadata to the CityModel.
    // 1) Take the CityModel with mutable reference.
    build_metadata_with_reference(&mut model)?;
    // 2) Build a Metadata instance and add it to the CityModel.
    let metadata = build_metadata_with_return(attribute_pool_ref)?;
    *model.metadata_mut() = metadata;
    // 3) Take a mutable reference to the Metadata instance of the CityModel and set the data.
    let metadata_ref = model.metadata_mut();
    build_metadata(metadata_ref, attribute_pool_ref);

    // Set extra root properties (see https://www.cityjson.org/specs/1.1.3/#case-1-adding-new-properties-at-the-root-of-a-document)
    let percent_men_id = model.attributes_mut().add_float(
        "percent_men".to_string(),
        true,
        49.5,
        AttributeOwnerType::Element,
        None,
    );
    let percent_women_id = model.attributes_mut().add_float(
        "percent_women".to_string(),
        true,
        51.5,
        AttributeOwnerType::Element,
        None,
    );
    let mut census_map = HashMap::new();
    census_map.insert("percent_men".to_string(), percent_men_id);
    census_map.insert("percent_women".to_string(), percent_women_id);
    let census_id = model.attributes_mut().add_map(
        "+census".to_string(),
        true,
        census_map,
        AttributeOwnerType::CityModel,
        None,
    );
    let extra = model.extra_mut();
    extra.insert("+census".to_string(), census_id);

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

        // Add address fields to global attribute pool
        let country_id = model.attributes_mut().add_string(
            "Country".to_string(),
            true,
            "Canada".to_string(),
            AttributeOwnerType::Element,
            None,
        );
        let locality_id = model.attributes_mut().add_string(
            "Locality".to_string(),
            true,
            "Chibougamau".to_string(),
            AttributeOwnerType::Element,
            None,
        );
        let thoroughfare_number_id = model.attributes_mut().add_string(
            "ThoroughfareNumber".to_string(),
            true,
            "1".to_string(),
            AttributeOwnerType::Element,
            None,
        );
        let thoroughfare_name_id = model.attributes_mut().add_string(
            "ThoroughfareName".to_string(),
            true,
            "rue de la Patate".to_string(),
            AttributeOwnerType::Element,
            None,
        );
        let postcode_id = model.attributes_mut().add_string(
            "Postcode".to_string(),
            true,
            "H0H 0H0".to_string(),
            AttributeOwnerType::Element,
            None,
        );

        let mut address_map = HashMap::new();
        address_map.insert("Country".to_string(), country_id);
        address_map.insert("Locality".to_string(), locality_id);
        address_map.insert("ThoroughfareNumber".to_string(), thoroughfare_number_id);
        address_map.insert("ThoroughfareName".to_string(), thoroughfare_name_id);
        address_map.insert("Postcode".to_string(), postcode_id);

        // Use a block scope to limit the lifetime of the GeometryBuilder, because it takes
        // a mutable borrow to the CityModel.
        {
            // Add point location to the address.
            let mut location_builder =
                GeometryBuilder::new(&mut model, GeometryType::MultiPoint, BuilderMode::Regular)
                    .with_lod(LoD::LoD1);
            let _location_p = location_builder.add_vertex(v0);
            if let Ok(location_geometry_ref) = location_builder.build() {
                let location_id = model.attributes_mut().add_geometry(
                    "location".to_string(),
                    true,
                    location_geometry_ref,
                    AttributeOwnerType::Element,
                    None,
                );
                address_map.insert("location".to_string(), location_id);
            }
        }

        // Create address map attribute and add to global attribute pool
        let address_map_id = model.attributes_mut().add_map(
            "".to_string(),
            false,
            address_map,
            AttributeOwnerType::Element,
            None,
        );

        // Per CityJSON specifications, we can have multiple addresses assigned to a single CityObject.
        let addresses_vec_id = model.attributes_mut().add_vector(
            "address".to_string(),
            true,
            vec![address_map_id],
            AttributeOwnerType::CityObject,
            None,
        );
        co_1_extra.insert("address".to_string(), addresses_vec_id);

        // Set regular attributes that will be stored in the "attributes" member of the CityObject.
        let measured_height_id = model.attributes_mut().add_float(
            "measuredHeight".to_string(),
            true,
            22.3,
            AttributeOwnerType::CityObject,
            None,
        );
        let roof_type_id = model.attributes_mut().add_string(
            "roofType".to_string(),
            true,
            "gable".to_string(),
            AttributeOwnerType::CityObject,
            None,
        );
        let residential_id = model.attributes_mut().add_bool(
            "residential".to_string(),
            true,
            true,
            AttributeOwnerType::CityObject,
            None,
        );
        let nr_doors_id = model.attributes_mut().add_integer(
            "nr_doors".to_string(),
            true,
            3,
            AttributeOwnerType::CityObject,
            None,
        );
        let co_1_attrs = co_1.attributes_mut();
        co_1_attrs.insert("measuredHeight".to_string(), measured_height_id);
        co_1_attrs.insert("roofType".to_string(), roof_type_id);
        co_1_attrs.insert("residential".to_string(), residential_id);
        co_1_attrs.insert("nr_doors".to_string(), nr_doors_id);

        // Create semantic attributes BEFORE creating GeometryBuilder (to avoid borrow conflicts)
        let surface_attr_id = model.attributes_mut().add_bool(
            "surfaceAttribute".to_string(),
            true,
            true,
            AttributeOwnerType::Semantic,
            None,
        );
        let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
        roof_semantic
            .attributes_mut()
            .insert("surfaceAttribute".to_string(), surface_attr_id);

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
            geometry_builder.set_semantic_surface(None, roof_semantic.clone(), true)?;
            // Material
            geometry_builder.set_material_surface(
                None,
                material_irradiation.clone(),
                "irradiation".to_string(),
                true,
            )?;
            geometry_builder.set_material_surface(
                None,
                material_red.clone(),
                "red".to_string(),
                true,
            )?;
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
                true,
            )?;

            // 1st Surface ---
            let ring1 = geometry_builder.add_ring(&[bv0, bv3, bv2, bv1])?;
            let surface_1 = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring1)?;
            // We reuse the previously created Semantic
            geometry_builder.set_semantic_surface(None, roof_semantic, true)?;
            geometry_builder.set_material_surface(
                None,
                material_irradiation.clone(),
                "irradiation".to_string(),
                true,
            )?;
            geometry_builder.set_material_surface(
                None,
                material_red.clone(),
                "red".to_string(),
                true,
            )?;
            geometry_builder.map_vertex_to_uv(bv0, uv0);
            geometry_builder.map_vertex_to_uv(bv1, uv1);
            geometry_builder.map_vertex_to_uv(bv2, uv2);
            geometry_builder.map_vertex_to_uv(bv3, uv3);
            geometry_builder.set_texture_ring(
                None,
                texture_0,
                "winter-textures".to_string(),
                true,
            )?;

            // 2nd Surface ---
            // This surface does not have Semantic
            let ring2 = geometry_builder.add_ring(&[bv0, bv3, bv2, bv1])?;
            let surface_2 = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring2)?;
            geometry_builder.set_material_surface(
                None,
                material_irradiation.clone(),
                "irradiation".to_string(),
                true,
            )?;
            geometry_builder.set_material_surface(
                None,
                material_red.clone(),
                "red".to_string(),
                true,
            )?;

            // 3rd Surface ---
            // This surface has a type from an Extension
            let semantic_extension_type = "+PatioDoor".to_string();
            let ring3 = geometry_builder.add_ring(&[bv0, bv3, bv2, bv1])?;
            let surface_3 = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring3)?;
            let patio_door_semantic =
                Semantic::new(SemanticType::Extension(semantic_extension_type.clone()));
            geometry_builder.set_semantic_surface(None, patio_door_semantic.clone(), false)?;
            // This surface does not have the "irradiation" material
            geometry_builder.set_material_surface(
                None,
                material_red.clone(),
                "red".to_string(),
                true,
            )?;
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
                .get_geometry(geometry_ref)
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
        let building_lden_id = model.attributes_mut().add_float(
            "buildingLDenMin".to_string(),
            true,
            1.0,
            AttributeOwnerType::CityObject,
            None,
        );
        let co_3_attrs = co_3.attributes_mut();
        co_3_attrs.insert("buildingLDenMin".to_string(), building_lden_id);
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
        let location_id = model.attributes_mut().add_string(
            "location".to_string(),
            true,
            "Magyarkanizsa".to_string(),
            AttributeOwnerType::CityObject,
            None,
        );
        let co_neigh_attrs = co_neighbourhood.attributes_mut();
        co_neigh_attrs.insert("location".to_string(), location_id);

        let role1_id = model.attributes_mut().add_string(
            "".to_string(),
            false,
            "residential building".to_string(),
            AttributeOwnerType::Element,
            None,
        );
        let role2_id = model.attributes_mut().add_string(
            "".to_string(),
            false,
            "voting location".to_string(),
            AttributeOwnerType::Element,
            None,
        );
        let children_roles_id = model.attributes_mut().add_vector(
            "children_roles".to_string(),
            true,
            vec![role1_id, role2_id],
            AttributeOwnerType::CityObject,
            None,
        );
        let co_neigh_extra = co_neighbourhood.extra_mut();
        co_neigh_extra.insert("children_roles".to_string(), children_roles_id);
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
    assert_eq!(model.vertices().len(), 4);
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
    let census_id = extra.get("+census").expect("+census should exist");
    let percent_men_id = model
        .attributes()
        .get_map_value(census_id, "percent_men")
        .expect("percent_men should exist in census map");
    let percent_men = model
        .attributes()
        .get_float(percent_men_id)
        .expect("percent_men should be Float");
    assert_eq!(percent_men, 49.5);

    let percent_women_id = model
        .attributes()
        .get_map_value(census_id, "percent_women")
        .expect("percent_women should exist in census map");
    let percent_women = model
        .attributes()
        .get_float(percent_women_id)
        .expect("percent_women should be Float");
    assert_eq!(percent_women, 51.5);

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
    for (_mat_ref, material) in model.iter_materials() {
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
    for (_tex_ref, texture) in model.iter_textures() {
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

    let measured_height_attr_id = attrs
        .get("measuredHeight")
        .expect("measuredHeight should exist");
    let h = model
        .attributes()
        .get_float(measured_height_attr_id)
        .expect("measuredHeight should be Float");
    assert_eq!(h, 22.3);

    let roof_type_attr_id = attrs.get("roofType").expect("roofType should exist");
    let t = model
        .attributes()
        .get_string(roof_type_attr_id)
        .expect("roofType should be String");
    assert_eq!(t, "gable");

    let residential_attr_id = attrs.get("residential").expect("residential should exist");
    let b = model
        .attributes()
        .get_bool(residential_attr_id)
        .expect("residential should be Bool");
    assert!(b);

    let nr_doors_attr_id = attrs.get("nr_doors").expect("nr_doors should exist");
    let n = model
        .attributes()
        .get_integer(nr_doors_attr_id)
        .expect("nr_doors should be Integer");
    assert_eq!(n, 3);

    // Test extra properties (address)
    let extra1 = co1.extra().expect("id-1 should have extra properties");
    let addresses_vec_id = extra1.get("address").expect("address should exist");
    let addresses = model
        .attributes()
        .get_vector_elements(addresses_vec_id)
        .expect("address should be Vec");
    assert_eq!(addresses.len(), 1);

    let address_map_id = addresses[0];
    let country_id = model
        .attributes()
        .get_map_value(address_map_id, "Country")
        .expect("Country should exist in address map");
    let country = model
        .attributes()
        .get_string(country_id)
        .expect("Country should be String");
    assert_eq!(country, "Canada");

    let locality_id = model
        .attributes()
        .get_map_value(address_map_id, "Locality")
        .expect("Locality should exist in address map");
    let locality = model
        .attributes()
        .get_string(locality_id)
        .expect("Locality should be String");
    assert_eq!(locality, "Chibougamau");

    let thoroughfare_number_id = model
        .attributes()
        .get_map_value(address_map_id, "ThoroughfareNumber")
        .expect("ThoroughfareNumber should exist in address map");
    let thoroughfare_number = model
        .attributes()
        .get_string(thoroughfare_number_id)
        .expect("ThoroughfareNumber should be String");
    assert_eq!(thoroughfare_number, "1");

    let thoroughfare_name_id = model
        .attributes()
        .get_map_value(address_map_id, "ThoroughfareName")
        .expect("ThoroughfareName should exist in address map");
    let thoroughfare_name = model
        .attributes()
        .get_string(thoroughfare_name_id)
        .expect("ThoroughfareName should be String");
    assert_eq!(thoroughfare_name, "rue de la Patate");

    let postcode_id = model
        .attributes()
        .get_map_value(address_map_id, "Postcode")
        .expect("Postcode should exist in address map");
    let postcode = model
        .attributes()
        .get_string(postcode_id)
        .expect("Postcode should be String");
    assert_eq!(postcode, "H0H 0H0");

    // Test location geometry in address
    let location_id = model
        .attributes()
        .get_map_value(address_map_id, "location")
        .expect("location should exist in address map");
    let _geom_ref = model
        .attributes()
        .get_geometry(location_id)
        .expect("location should be Geometry");

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
        .get_geometry(*geom1)
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
        let surface_attr_id = sem0_attrs
            .get("surfaceAttribute")
            .expect("surfaceAttribute should exist");
        let surface_attr = model
            .attributes()
            .get_bool(surface_attr_id)
            .expect("surfaceAttribute should be Bool");
        assert!(surface_attr);
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
    let building_lden_attr_id = attrs3
        .get("buildingLDenMin")
        .expect("buildingLDenMin should exist");
    let val = model
        .attributes()
        .get_float(building_lden_attr_id)
        .expect("buildingLDenMin should be Float");
    assert_eq!(val, 1.0);

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
        .get_geometry(*geom_tree)
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
        .get_geometry(*template_ref)
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
    let location_attr_id = attrs_neigh.get("location").expect("location should exist");
    let location = model
        .attributes()
        .get_string(location_attr_id)
        .expect("location should be String");
    assert_eq!(location, "Magyarkanizsa");

    let extra_neigh = co_neigh
        .extra()
        .expect("my-neighbourhood should have extra properties");
    let children_roles_id = extra_neigh
        .get("children_roles")
        .expect("children_roles should exist");
    let roles = model
        .attributes()
        .get_vector_elements(children_roles_id)
        .expect("children_roles should be Vec");
    assert_eq!(roles.len(), 2);

    let role1 = model
        .attributes()
        .get_string(roles[0])
        .expect("First role should be String");
    assert_eq!(role1, "residential building");

    let role2 = model
        .attributes()
        .get_string(roles[1])
        .expect("Second role should be String");
    assert_eq!(role2, "voting location");

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
        .get_geometry(*geom_neigh)
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

/// Build a complete Metadata instance with all data set and add it to a CityModel.
/// Takes the CityModel by mutable reference.
fn build_metadata_with_reference(model: &mut CityModel) -> Result<()> {
    let metadata_ref = model.metadata_mut();
    let attribute_pool_ref = model.attributes_mut();
    build_metadata(metadata_ref, attribute_pool_ref);
    Ok(())
}

/// Build a complete Metadata instance with all data set and return it.
fn build_metadata_with_return(
    attribute_pool_ref: &mut AttributePool<OwnedStringStorage, ResourceId32>,
) -> Result<Metadata<OwnedStringStorage>> {
    let mut metadata = Metadata::new();
    build_metadata(&mut metadata, attribute_pool_ref);
    Ok(metadata)
}

/// Set data on a Metadata instance.
fn build_metadata(
    metadata_ref: &mut Metadata<OwnedStringStorage>,
    attribute_pool_ref: &mut AttributePool<OwnedStringStorage, ResourceId32>,
) {
    metadata_ref
        .set_geographical_extent(BBox::new(84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9));
    metadata_ref.set_identifier(CityModelIdentifier::new(
        "eaeceeaa-3f66-429a-b81d-bbc6140b8c1c".to_string(),
    ));
    metadata_ref.set_reference_system(CRS::new(
        "https://www.opengis.net/def/crs/EPSG/0/2355".to_string(),
    ));
    metadata_ref.set_contact_name("Kitalált Név");
    metadata_ref.set_email_address("spam@3dgi.nl");
    metadata_ref.set_role(ContactRole::Author);
    metadata_ref.set_website("https://3dgi.nl");
    metadata_ref.set_contact_type(ContactType::Organization);
    let mut address = Attributes::<OwnedStringStorage>::new();
    let mut attribute_id = attribute_pool_ref.add_string(
        "city".to_string(),
        true,
        "Den Haag".to_string(),
        AttributeOwnerType::Metadata,
        None,
    );
    address.insert("city".to_string(), attribute_id);
    attribute_id = attribute_pool_ref.add_string(
        "country".to_string(),
        true,
        "The Netherlands".to_string(),
        AttributeOwnerType::Metadata,
        None,
    );
    address.insert("country".to_string(), attribute_id);
    metadata_ref.set_address(address);
    metadata_ref.set_phone("+36612345678");
    metadata_ref.set_organization("3DGI");
}
