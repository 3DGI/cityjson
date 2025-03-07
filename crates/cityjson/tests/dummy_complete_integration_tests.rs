use cityjson::prelude::*;
use cityjson::v1_1::*;
use std::collections::HashMap;

#[test]
fn build_dummy_complete_owned() -> Result<()> {
    // A CityModel for CityJSON v1.1, that uses u32 indices and owned strings.
    let mut model =
        CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

    // Set metadata
    let metadata = model.metadata_mut();
    metadata.set_identifier("eaeceeaa-3f66-429a-b81d-bbc6140b8c1c");
    metadata.set_reference_system("https://www.opengis.net/def/crs/EPSG/0/2355");
    metadata.set_contact_name("3DGI");
    metadata.set_email_address("info@3dgi.nl");

    // Set extra root properties (see https://www.cityjson.org/specs/1.1.3/#case-1-adding-new-properties-at-the-root-of-a-document)
    let extra = model.extra_mut();
    let mut census_map = HashMap::new();
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
    let co_tree = CityObject::new(co_tree_id.clone(), CityObjectType::SolitaryVegetationObject);
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

    // Create textures
    // let texture_0 = Texture::new("http://www.someurl.org/filename.jpg".to_string(), ImageType::Png);

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
                GeometryBuilder::new(&mut model, GeometryType::MultiPoint).with_lod(LoD::LoD1);
            let _location_p = location_builder.add_vertex(102.0, 103.0, 1.0);
            if let Ok(_location_geometry_ref) = location_builder.build() {
                // todo: AttributeValue::Geometry
            }
        }

        // Per CityJSON specifications, we can have multiple addresses assigned to a single CityObject.
        let addresses_vec = AttributeValue::Vec(vec![Box::new(AttributeValue::Map(address_map))]);
        co_1_extra.insert("address".to_string(), addresses_vec);

        // Set regular attributes, that will be stored in the "attributes" member of the CityObject.
        let co_1_attrs = co_1.attributes_mut();
        co_1_attrs.insert("measuredHeight".to_string(), AttributeValue::Float(22.3));
        co_1_attrs.insert(
            "roofType".to_string(),
            AttributeValue::String("gable".to_string()),
        );
        co_1_attrs.insert("residential".to_string(), AttributeValue::Bool(true));
        co_1_attrs.insert("nr_doors".to_string(), AttributeValue::Integer(3));

        // Set CityObject family
        co_1.parents_mut().push(co_3_id.clone()); // todo: change CityObject.parents,children to use ResourceRef, since we use a ResourcePool for CityObjects
        co_1.parents_mut().push(co_neighbourhood_id.clone());

        // Use a block scope to limit the lifetime of the GeometryBuilder, because it takes
        // a mutable borrow to the CityModel.
        {
            let mut geometry_builder = GeometryBuilder::new(&mut model, GeometryType::Solid).with_lod(LoD::LoD2_1);
            let v0 = geometry_builder.add_vertex(102.0, 103.0, 1.0); // todo: should be able to reuse a vertex from the pool
            let v1 = geometry_builder.add_vertex(11.0, 910.0, 43.0);
            let v2 = geometry_builder.add_vertex(25.0, 744.0, 22.0);
            let v3 = geometry_builder.add_vertex(23.0, 88.0, 5.0);

            // For a Solid, we don't need the shell index, just need to start the one
            // shell that it has.
            let _shell_i = geometry_builder.start_shell();

            // 0th Surface ---
            // Geometry
            let surface_0 = geometry_builder.start_surface(Some(SemanticType::RoofSurface)); // todo: probably not necessary to set semantic type here
            geometry_builder.set_surface_outer_ring(&[v0, v3, v2, v1])?; // todo: how to handle errors properly?
            // Semantic
            let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
            let sem_attr = roof_semantic.attributes_mut();
            sem_attr.insert("surfaceAttribute".to_string(), AttributeValue::Bool(true));
            geometry_builder.set_surface_semantic(roof_semantic.clone())?;
            // Material
            geometry_builder.set_surface_material(material_irradiation.clone())?;
            geometry_builder.set_surface_material(material_red.clone())?;
            // Add the surface to the shell
            geometry_builder.add_shell_outer_surface(surface_0)?; // todo: set_* and add_* methods are confusing
            // todo: figure out setting texture to vertices etc.
            // todo: set texture theme on geometry "winter-textures"

            // 1st Surface ---
            let surface_1 = geometry_builder.start_surface(Some(SemanticType::RoofSurface));
            geometry_builder.set_surface_outer_ring(&[v0, v3, v2, v1])?;
            // We reuse the previously created Semantic
            geometry_builder.set_surface_semantic(roof_semantic)?;
            geometry_builder.set_surface_material(material_irradiation.clone())?;
            geometry_builder.set_surface_material(material_red.clone())?;
            geometry_builder.add_shell_outer_surface(surface_1)?;

            // 2nd Surface ---
            // This surface does not have Semantic
            let surface_2 = geometry_builder.start_surface(None);
            geometry_builder.set_surface_outer_ring(&[v0, v3, v2, v1])?;
            geometry_builder.set_surface_material(material_irradiation.clone())?;
            geometry_builder.set_surface_material(material_red.clone())?;
            geometry_builder.add_shell_outer_surface(surface_2)?;

            // 3rd Surface ---
            // This surface has a type from an Extension
            let semantic_extension_type = "+PatioDoor".to_string();
            let surface_3 = geometry_builder.start_surface(Some(SemanticType::Extension(semantic_extension_type.clone()))); // todo: probably not necessary to set semantic type here
            geometry_builder.set_surface_outer_ring(&[v0, v3, v2, v1])?;
            let patio_door_semantic = Semantic::new(SemanticType::Extension(semantic_extension_type.clone()));
            geometry_builder.set_surface_semantic(patio_door_semantic.clone())?;
            // This surface does not have the "irradiation" material
            geometry_builder.set_surface_material(material_red.clone())?;
            geometry_builder.add_shell_outer_surface(surface_3)?;

            // Inner shell
            let _inner_shell_i = geometry_builder.start_shell();
            let surface_4 = geometry_builder.start_surface(None);
            geometry_builder.set_surface_outer_ring(&[v1, v2, v3, v0])?;
            geometry_builder.add_surface_inner_ring(&[v1, v2, v3, v0])?;
            geometry_builder.add_shell_outer_surface(surface_4)?;

            // Consume the builder by building a Geometry and adding it to the CityModel
            let geometry_ref = geometry_builder.build()?;

            // For debug only
            let geom_nested = model.geometries().get(geometry_ref).unwrap().clone().boundaries().unwrap().to_nested_solid().unwrap();
            println!("{:?}", geom_nested);
        }
    }

    // Build CityObject "id-3".
    {
        let co_3_attrs = co_3.attributes_mut();
        co_3_attrs.insert("buildingLDenMin".to_string(), AttributeValue::Float(1.0));
        co_3.children_mut().push(co_1_id.clone());
        co_3.parents_mut().push(co_neighbourhood_id.clone());
    }

    // Build CityObject "a-tree".
    {
        // todo: Sort out GeometryInstance
    }

    // Build CityObject "my-neighbourhood"
    {
        let co_neigh_attrs = co_neighbourhood.attributes_mut();
        co_neigh_attrs.insert("location".to_string(), AttributeValue::String("Magyarkanizsa".to_string()));
        co_neighbourhood.children_mut().push(co_1_id.clone());
        co_neighbourhood.children_mut().push(co_3_id.clone());
        // todo: add children_roles, probably as "extra"
        {
            let mut geometry_builder = GeometryBuilder::new(&mut model, GeometryType::MultiSurface).with_lod(LoD::LoD2);
            let _surface_i = geometry_builder.start_surface(None);
            let p1 = geometry_builder.add_vertex(102.0, 103.0, 1.0);
            let p2 = geometry_builder.add_vertex(23.0, 88.0, 5.0);
            let p3 = geometry_builder.add_vertex(25.0, 744.0, 22.0);
            let p4 = geometry_builder.add_vertex(11.0, 910.0, 43.0);
            // todo: builder.add_ring() ? what's that for?
            geometry_builder.set_surface_outer_ring(&[p1, p4, p3, p2])?;
            let _geometry_ref = geometry_builder.build()?;
        }
    }

    let cityobjects = model.cityobjects_mut();
    cityobjects.add(co_1);
    cityobjects.add(co_3);
    cityobjects.add(co_tree);
    cityobjects.add(co_neighbourhood);

    println!("{}", &model);
    Ok(())
}
