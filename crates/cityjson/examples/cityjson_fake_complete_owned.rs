use cityjson::v2_0::GeometryBuilder;
use cityjson::cityjson::core::attributes::OwnedAttributeValue;
use cityjson::prelude::*;
use cityjson::v2_0::{
    CityModel, CityObject, CityObjectType, Extension, Material, Semantic, SemanticType, Texture,
};
use std::collections::HashMap;

type OwnedModel = CityModel<u32, OwnedStringStorage>;
type OwnedCityObject = CityObject<OwnedStringStorage>;
type VertexRefs = [VertexIndex<u32>; 4];

/// Build a `CityModel` that uses the complete `CityJSON` v2.0 specifications with fake values.
/// Builds the same `CityModel` that is stored in
/// `tests/data/v2_0/cityjson_fake_complete.city.json`, with owned values.
fn main() -> Result<()> {
    let mut model = OwnedModel::new(CityModelType::CityJSON);
    configure_model(&mut model);
    let (mut co_1, mut co_3, co_tree, mut co_neighbourhood) = create_city_objects();
    let vertices = add_shared_vertices(&mut model)?;

    configure_co_1(&mut model, &mut co_1, vertices)?;
    configure_co_3(&mut co_3);
    configure_tree(&mut model, vertices[1])?;
    configure_neighbourhood(&mut model, &mut co_neighbourhood, vertices)?;
    link_semantics(&mut model);
    add_cityobjects_and_hierarchy(&mut model, co_1, co_3, co_tree, co_neighbourhood)?;

    println!("{model}");
    Ok(())
}

fn configure_model(model: &mut OwnedModel) {
    let metadata = model.metadata_mut();
    metadata.set_identifier(CityModelIdentifier::new(
        "eaeceeaa-3f66-429a-b81d-bbc6140b8c1c".to_string(),
    ));
    metadata.set_reference_system(CRS::new(
        "https://www.opengis.net/def/crs/EPSG/0/2355".to_string(),
    ));
    metadata.set_contact_name("3DGI");
    metadata.set_email_address("info@3dgi.nl");

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
    extra.insert("+census".to_string(), OwnedAttributeValue::Map(census_map));

    let transform = model.transform_mut();
    transform.set_scale([1.0, 1.0, 1.0]);
    transform.set_translate([0.0, 0.0, 0.0]);

    model.extensions_mut().add(Extension::new(
        "Noise".to_string(),
        "https://someurl.orgnoise.json".to_string(),
        "2.0".to_string(),
    ));
}

fn create_city_objects() -> (
    OwnedCityObject,
    OwnedCityObject,
    OwnedCityObject,
    OwnedCityObject,
) {
    let co_1 = CityObject::new(
        CityObjectIdentifier::new("id-1".to_string()),
        CityObjectType::BuildingPart,
    );
    let co_3 = CityObject::new(
        CityObjectIdentifier::new("id-3".to_string()),
        CityObjectType::Extension("+NoiseBuilding".to_string()),
    );
    let co_tree = CityObject::new(
        CityObjectIdentifier::new("a-tree".to_string()),
        CityObjectType::SolitaryVegetationObject,
    );
    let co_neighbourhood = CityObject::new(
        CityObjectIdentifier::new("my-neighbourhood".to_string()),
        CityObjectType::CityObjectGroup,
    );
    (co_1, co_3, co_tree, co_neighbourhood)
}

fn add_shared_vertices(model: &mut OwnedModel) -> Result<VertexRefs> {
    Ok([
        model.add_vertex(QuantizedCoordinate::new(102, 103, 1))?,
        model.add_vertex(QuantizedCoordinate::new(11, 910, 43))?,
        model.add_vertex(QuantizedCoordinate::new(25, 744, 22))?,
        model.add_vertex(QuantizedCoordinate::new(23, 88, 5))?,
    ])
}

fn configure_co_1(
    model: &mut OwnedModel,
    co_1: &mut OwnedCityObject,
    vertices: VertexRefs,
) -> Result<()> {
    co_1.set_geographical_extent(Some(BBox::new(
        84_710.1, 446_846.0, -5.3, 84_757.1, 446_944.0, 40.9,
    )));
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
    add_address_location(model, &mut address_map, vertices[0]);
    co_1.extra_mut().insert(
        "address".to_string(),
        OwnedAttributeValue::Vec(vec![Box::new(OwnedAttributeValue::Map(address_map))]),
    );

    let attrs = co_1.attributes_mut();
    attrs.insert(
        "measuredHeight".to_string(),
        OwnedAttributeValue::Float(22.3),
    );
    attrs.insert(
        "roofType".to_string(),
        OwnedAttributeValue::String("gable".to_string()),
    );
    attrs.insert("residential".to_string(), OwnedAttributeValue::Bool(true));
    attrs.insert("nr_doors".to_string(), OwnedAttributeValue::Integer(3));

    build_co_1_geometry(model, vertices)?;
    Ok(())
}

fn add_address_location(
    model: &mut OwnedModel,
    address_map: &mut HashMap<String, Box<OwnedAttributeValue>>,
    v0: VertexIndex<u32>,
) {
    let mut location_builder =
        GeometryBuilder::new(model, GeometryType::MultiPoint, BuilderMode::Regular)
            .with_lod(LoD::LoD1);
    let _location_p = location_builder.add_vertex(v0);
    if let Ok(location_geometry_ref) = location_builder.build_geometry() {
        address_map.insert(
            "location".to_string(),
            Box::new(OwnedAttributeValue::Geometry(location_geometry_ref)),
        );
    }
}

fn build_co_1_geometry(model: &mut OwnedModel, [v0, v1, v2, v3]: VertexRefs) -> Result<()> {
    let mut material_irradiation = Material::new("irradiation".to_string());
    material_irradiation.set_ambient_intensity(Some(0.2));
    material_irradiation.set_diffuse_color(Some([0.9, 0.1, 0.75].into()));
    material_irradiation.set_emissive_color(Some([0.9, 0.1, 0.75].into()));
    material_irradiation.set_specular_color(Some([0.9, 0.1, 0.75].into()));
    material_irradiation.set_shininess(Some(0.2));
    material_irradiation.set_transparency(Some(0.5));
    material_irradiation.set_is_smooth(Some(false));
    let material_red = Material::new("red".to_string());

    let mut texture_0 = Texture::new(
        "http://www.someurl.org/filename.jpg".to_string(),
        ImageType::Png,
    );
    texture_0.set_wrap_mode(Some(WrapMode::Wrap));
    texture_0.set_texture_type(Some(TextureType::Specific));
    texture_0.set_border_color(Some([1.0, 1.0, 1.0, 1.0].into()));

    let mut geometry_builder =
        GeometryBuilder::new(model, GeometryType::Solid, BuilderMode::Regular)
            .with_lod(LoD::LoD2_1);
    let [bv0, bv1, bv2, bv3] = [
        geometry_builder.add_vertex(v0),
        geometry_builder.add_vertex(v1),
        geometry_builder.add_vertex(v2),
        geometry_builder.add_vertex(v3),
    ];
    let [uv0, uv1, uv2, uv3] = [
        geometry_builder.add_uv_coordinate(0.0, 0.5),
        geometry_builder.add_uv_coordinate(1.0, 0.0),
        geometry_builder.add_uv_coordinate(1.0, 1.0),
        geometry_builder.add_uv_coordinate(0.0, 1.0),
    ];

    let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
    roof_semantic.attributes_mut().insert(
        "surfaceAttribute".to_string(),
        OwnedAttributeValue::Bool(true),
    );

    let mut add_surface = |semantic: Option<Semantic<OwnedStringStorage>>,
                           add_irradiation: bool,
                           texture_theme: Option<&str>|
     -> Result<usize> {
        let ring = geometry_builder.add_ring(&[bv0, bv3, bv2, bv1])?;
        let surface = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring)?;
        if let Some(semantic) = semantic {
            geometry_builder.set_semantic_surface(None, semantic, false)?;
        }
        if add_irradiation {
            geometry_builder.set_material_surface(
                None,
                material_irradiation.clone(),
                "irradiation".to_string(),
                true,
            )?;
        }
        geometry_builder.set_material_surface(
            None,
            material_red.clone(),
            "red".to_string(),
            true,
        )?;
        if let Some(theme) = texture_theme {
            geometry_builder.map_vertex_to_uv(bv0, uv0);
            geometry_builder.map_vertex_to_uv(bv1, uv1);
            geometry_builder.map_vertex_to_uv(bv2, uv2);
            geometry_builder.map_vertex_to_uv(bv3, uv3);
            geometry_builder.set_texture_ring(None, texture_0.clone(), theme.to_string(), true)?;
        }
        Ok(surface)
    };

    let surface_0 = add_surface(Some(roof_semantic.clone()), true, Some("winter-textures"))?;
    let surface_1 = add_surface(Some(roof_semantic), true, Some("theme-texture"))?;
    let surface_2 = add_surface(None, true, None)?;
    let surface_3 = add_surface(
        Some(Semantic::new(SemanticType::Extension(
            "+PatioDoor".to_string(),
        ))),
        false,
        None,
    )?;
    geometry_builder.add_shell(&[surface_0, surface_1, surface_2, surface_3])?;

    let surface_4 = geometry_builder.start_surface();
    let ring4 = geometry_builder.add_ring(&[bv1, bv2, bv3, bv0])?;
    geometry_builder.add_surface_outer_ring(ring4)?;
    let ring5 = geometry_builder.add_ring(&[bv1, bv2, bv3, bv0])?;
    geometry_builder.add_surface_inner_ring(ring5)?;
    geometry_builder.add_shell(&[surface_4])?;
    let _geometry_ref = geometry_builder.build_geometry()?;
    Ok(())
}

fn configure_co_3(co_3: &mut OwnedCityObject) {
    co_3.attributes_mut().insert(
        "buildingLDenMin".to_string(),
        OwnedAttributeValue::Float(1.0),
    );
}

fn configure_tree(model: &mut OwnedModel, reference_vertex: VertexIndex<u32>) -> Result<()> {
    let mut template_builder =
        GeometryBuilder::new(model, GeometryType::MultiSurface, BuilderMode::Template)
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

    let template_ref = template_builder.build_template()?;
    GeometryBuilder::new(model, GeometryType::GeometryInstance, BuilderMode::Regular)
        .with_template_ref(template_ref)?
        .with_transformation_matrix([
            2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])?
        .with_reference_vertex(reference_vertex)
        .build_geometry()?;
    Ok(())
}

fn configure_neighbourhood(
    model: &mut OwnedModel,
    co_neighbourhood: &mut OwnedCityObject,
    [v0, v1, v2, v3]: VertexRefs,
) -> Result<()> {
    co_neighbourhood.attributes_mut().insert(
        "location".to_string(),
        OwnedAttributeValue::String("Magyarkanizsa".to_string()),
    );
    co_neighbourhood.extra_mut().insert(
        "children_roles".to_string(),
        OwnedAttributeValue::Vec(vec![
            Box::new(OwnedAttributeValue::String(
                "residential building".to_string(),
            )),
            Box::new(OwnedAttributeValue::String("voting location".to_string())),
        ]),
    );

    let mut geometry_builder =
        GeometryBuilder::new(model, GeometryType::MultiSurface, BuilderMode::Regular)
            .with_lod(LoD::LoD2);
    let _surface_i = geometry_builder.start_surface();
    let [p1, p2, p3, p4] = [
        geometry_builder.add_vertex(v0),
        geometry_builder.add_vertex(v3),
        geometry_builder.add_vertex(v2),
        geometry_builder.add_vertex(v1),
    ];
    let ring0 = geometry_builder.add_ring(&[p1, p4, p3, p2])?;
    geometry_builder.add_surface_outer_ring(ring0)?;
    let _geometry_ref = geometry_builder.build_geometry()?;
    Ok(())
}

fn link_semantics(model: &mut OwnedModel) {
    let mut roof_semantic_ref = None;
    let mut patio_door_semantic_ref = None;
    for (semantic_ref, semantic) in model.iter_semantics() {
        if roof_semantic_ref.is_none() && semantic.type_semantic() == &SemanticType::RoofSurface {
            roof_semantic_ref = Some(semantic_ref);
        }
        if patio_door_semantic_ref.is_none()
            && let SemanticType::Extension(ext) = semantic.type_semantic()
            && ext == "+PatioDoor"
        {
            patio_door_semantic_ref = Some(semantic_ref);
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
}

fn add_cityobjects_and_hierarchy(
    model: &mut OwnedModel,
    co_1: OwnedCityObject,
    co_3: OwnedCityObject,
    co_tree: OwnedCityObject,
    co_neighbourhood: OwnedCityObject,
) -> Result<()> {
    let cityobjects = model.cityobjects_mut();
    let co_1_ref = cityobjects.add(co_1)?;
    let co_3_ref = cityobjects.add(co_3)?;
    let _co_tree_ref = cityobjects.add(co_tree)?;
    let co_neigh_ref = cityobjects.add(co_neighbourhood)?;

    cityobjects.get_mut(co_1_ref).unwrap().add_parent(co_3_ref);
    cityobjects
        .get_mut(co_1_ref)
        .unwrap()
        .add_parent(co_neigh_ref);
    cityobjects.get_mut(co_3_ref).unwrap().add_child(co_1_ref);
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
    Ok(())
}
