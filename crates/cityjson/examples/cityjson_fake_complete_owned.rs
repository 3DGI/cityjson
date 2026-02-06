use cityjson::cityjson::core::attributes::OwnedAttributeValue;
use cityjson::prelude::*;
use cityjson::backend::default::geometry::GeometryBuilder;
use cityjson::v2_0::*;
use std::collections::HashMap;

/// Build a CityModel that uses the complete CityJSON v2.0 specifications with fake
/// values.
/// Builds the same CityModel that is stored in
/// `tests/data/v2_0/cityjson_fake_complete.city.json`, with owned values.
fn main() -> Result<()> {
    // A CityModel for CityJSON v2.0 that uses u32 indices and owned strings.
    let mut model =
        CityModel::<u32, OwnedStringStorage>::new(CityModelType::CityJSON);

    // Set metadata
    let metadata = model.metadata_mut();
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
    let mut census_map = HashMap::new();
    census_map.insert(
        "percent_men".to_string(),
        Box::new(OwnedAttributeValue::Float(49.5)),
    );
    census_map.insert(
        "percent_women".to_string(),
        Box::new(OwnedAttributeValue::Float(51.5)),
    );
    extra.insert(
        "+census".to_string(),
        OwnedAttributeValue::Map(census_map),
    );

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
    let mut co_1 = CityObject::new(CityObjectIdentifier::new(co_1_id.clone()), CityObjectType::BuildingPart);
    let co_3_id = "id-3".to_string();
    let mut co_3 = CityObject::new(CityObjectIdentifier::new(co_3_id.clone()),
        CityObjectType::Extension("+NoiseBuilding".to_string()),
    );
    let co_tree_id = "a-tree".to_string();
    let co_tree = CityObject::new(CityObjectIdentifier::new(co_tree_id.clone()), CityObjectType::SolitaryVegetationObject);
    let co_neighbourhood_id = "my-neighbourhood".to_string();
    let mut co_neighbourhood =
        CityObject::new(CityObjectIdentifier::new(co_neighbourhood_id.clone()), CityObjectType::CityObjectGroup);

    // Create materials
    let mut material_irradiation = Material::new("irradiation".to_string());
    material_irradiation.set_ambient_intensity(Some(0.2000));
    material_irradiation.set_diffuse_color(Some([0.9000, 0.1000, 0.7500].into()));
    material_irradiation.set_emissive_color(Some([0.9000, 0.1000, 0.7500].into()));
    material_irradiation.set_specular_color(Some([0.9000, 0.1000, 0.7500].into()));
    material_irradiation.set_shininess(Some(0.2));
    material_irradiation.set_transparency(Some(0.5));
    material_irradiation.set_is_smooth(Some(false));
    let material_red = Material::new("red".to_string());

    // Create textures
    let mut texture_0 = Texture::new(
        "http://www.someurl.org/filename.jpg".to_string(),
        ImageType::Png,
    );
    texture_0.set_wrap_mode(Some(WrapMode::Wrap));
    texture_0.set_texture_type(Some(TextureType::Specific));
    texture_0.set_border_color(Some([1.0, 1.0, 1.0, 1.0].into()));

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
            Box::new(OwnedAttributeValue::String("Canada".to_string())),
        );
        address_map.insert(
            "Locality".to_string(),
            Box::new(OwnedAttributeValue::String("Chibougamau".to_string())),
        );
        address_map.insert(
            "ThoroughfareNumber".to_string(),
            Box::new(OwnedAttributeValue::String("1".to_string())),
        );
        address_map.insert(
            "ThoroughfareName".to_string(),
            Box::new(OwnedAttributeValue::String("rue de la Patate".to_string())),
        );
        address_map.insert(
            "Postcode".to_string(),
            Box::new(OwnedAttributeValue::String("H0H 0H0".to_string())),
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
                    Box::new(OwnedAttributeValue::Geometry(GeometryRef::from_parts(location_geometry_ref.index(), location_geometry_ref.generation()))),
                );
            }
        }

        // Per CityJSON specifications, we can have multiple addresses assigned to a single CityObject.
        let addresses_vec = vec![Box::new(OwnedAttributeValue::Map(address_map))];
        co_1_extra.insert(
            "address".to_string(),
            OwnedAttributeValue::Vec(addresses_vec),
        );

        // Set regular attributes that will be stored in the "attributes" member of the CityObject.
        let co_1_attrs = co_1.attributes_mut();
        co_1_attrs.insert(
            "measuredHeight".to_string(),
            OwnedAttributeValue::Float(22.3),
        );
        co_1_attrs.insert(
            "roofType".to_string(),
            OwnedAttributeValue::String("gable".to_string()),
        );
        co_1_attrs.insert(
            "residential".to_string(),
            OwnedAttributeValue::Bool(true),
        );
        co_1_attrs.insert(
            "nr_doors".to_string(),
            OwnedAttributeValue::Integer(3),
        );

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
            sem_attr.insert(
                "surfaceAttribute".to_string(),
                OwnedAttributeValue::Bool(true),
            );
            geometry_builder.set_semantic_surface(None, roof_semantic.clone(), false)?;
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
            geometry_builder.set_semantic_surface(None, roof_semantic, false)?;
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
                "theme-texture".to_string(),
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
            let _geometry_ref = geometry_builder.build()?;
        }
    }

    // Build CityObject "id-3".
    {
        let co_3_attrs = co_3.attributes_mut();
        co_3_attrs.insert(
            "buildingLDenMin".to_string(),
            OwnedAttributeValue::Float(1.0),
        );
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
        GeometryBuilder::new(
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
    }

    // Build CityObject "my-neighbourhood"
    {
        let co_neigh_attrs = co_neighbourhood.attributes_mut();
        co_neigh_attrs.insert(
            "location".to_string(),
            OwnedAttributeValue::String("Magyarkanizsa".to_string()),
        );
        let co_neigh_extra = co_neighbourhood.extra_mut();
        let children_roles_vec = vec![
            Box::new(OwnedAttributeValue::String("residential building".to_string())),
            Box::new(OwnedAttributeValue::String("voting location".to_string())),
        ];
        co_neigh_extra.insert(
            "children_roles".to_string(),
            OwnedAttributeValue::Vec(children_roles_vec),
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
            let _geometry_ref = geometry_builder.build()?;
        }
    }

    // Add a parent/children relation between semantic surfaces for schema coverage.
    let mut roof_semantic_ref = None;
    let mut patio_door_semantic_ref = None;
    for (semantic_ref, semantic) in model.iter_semantics() {
        if roof_semantic_ref.is_none() && semantic.type_semantic() == &SemanticType::RoofSurface {
            roof_semantic_ref = Some(semantic_ref);
        }
        if patio_door_semantic_ref.is_none() {
            if let SemanticType::Extension(ext) = semantic.type_semantic() {
                if ext == "+PatioDoor" {
                    patio_door_semantic_ref = Some(semantic_ref);
                }
            }
        }
    }
    if let (Some(roof), Some(patio)) = (roof_semantic_ref, patio_door_semantic_ref) {
        model
            .get_semantic_mut(roof)
            .expect("roof semantic should exist")
            .children_mut()
            .push(patio);
        model
            .get_semantic_mut(patio)
            .expect("patio door semantic should exist")
            .set_parent(roof);
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
        .add_parent(co_3_ref);
    cityobjects
        .get_mut(co_1_ref)
        .unwrap()
        .add_parent(co_neigh_ref);
    cityobjects
        .get_mut(co_3_ref)
        .unwrap()
        .add_child(co_1_ref);
    cityobjects
        .get_mut(co_3_ref)
        .unwrap()
        .add_parent(co_neigh_ref);
    cityobjects
        .get_mut(co_neigh_ref)
        .unwrap()
        .add_child(co_1_ref);
    cityobjects
        .get_mut(co_neigh_ref)
        .unwrap()
        .add_child(co_3_ref);

    println!("{}", &model);
    Ok(())
}
