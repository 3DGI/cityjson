use cityjson::error::Result;
use cityjson::resources::CityObjectHandle;
use cityjson::resources::handles::{MaterialHandle, TextureHandle};
use cityjson::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};
use cityjson::v2_0::*;
use serde_json::Value;
use std::collections::HashMap;

type JsonObject = serde_json::Map<String, Value>;

#[derive(Clone, Copy)]
struct SharedVertices {
    v0: VertexIndex<u32>,
    v1: VertexIndex<u32>,
    v2: VertexIndex<u32>,
    v3: VertexIndex<u32>,
}

struct Appearance {
    material_irradiation: MaterialHandle,
    material_red: MaterialHandle,
    texture: TextureHandle,
    texture_theme: &'static str,
}

struct PendingCityObjects<SS: StringStorage> {
    building_part: CityObject<SS>,
    noise_building: CityObject<SS>,
    tree: CityObject<SS>,
    neighbourhood: CityObject<SS>,
}

const FLOAT_EPSILON: f64 = 1.0e-9;

pub(crate) trait FixtureStorage<'a>: StringStorage {
    fn store(value: &'a str) -> Self::String;
}

impl<'a> FixtureStorage<'a> for OwnedStringStorage {
    fn store(value: &'a str) -> Self::String {
        value.to_owned()
    }
}

impl<'a> FixtureStorage<'a> for BorrowedStringStorage<'a> {
    fn store(value: &'a str) -> Self::String {
        value
    }
}

pub(crate) fn load_fixture() -> Value {
    serde_json::from_str(include_str!(
        "../data/v2_0/cityjson_fake_complete.city.json"
    ))
    .expect("fake-complete fixture should be valid JSON")
}

pub(crate) fn build_model_from_fixture<'a, SS>(fixture: &'a Value) -> Result<CityModel<u32, SS>>
where
    SS: FixtureStorage<'a>,
{
    let mut model = CityModel::new(CityModelType::CityJSON);

    build_metadata(model.metadata_mut(), fixture);
    build_root_components(&mut model, fixture);

    let mut cityobjects = init_cityobjects::<SS>(fixture);
    let appearance = build_appearance(&mut model, fixture)?;
    let vertices = build_shared_vertices(&mut model, fixture)?;

    build_cityobject_id_1(
        &mut model,
        &mut cityobjects.building_part,
        fixture,
        vertices,
        &appearance,
    )?;
    build_cityobject_id_3(&mut cityobjects.noise_building, fixture);
    build_cityobject_tree(&mut model, &mut cityobjects.tree, fixture, vertices.v1)?;
    build_cityobject_neighbourhood(
        &mut model,
        &mut cityobjects.neighbourhood,
        fixture,
        vertices,
    )?;

    add_cityobjects_with_hierarchy(&mut model, cityobjects)?;

    Ok(model)
}

pub(crate) fn assert_model_matches_fixture<SS: StringStorage>(
    model: &CityModel<u32, SS>,
    fixture: &Value,
) {
    assert_eq!(model.type_citymodel(), CityModelType::CityJSON);
    assert_eq!(model.version(), Some(CityJSONVersion::V2_0));
    assert_eq!(
        model.cityobjects().len(),
        object(&fixture["CityObjects"]).len()
    );
    assert_eq!(model.vertices().len(), array(&fixture["vertices"]).len());
    assert_eq!(model.geometry_count(), 4);
    assert_eq!(
        model.geometry_template_count(),
        array(&fixture["geometry-templates"]["templates"]).len()
    );
    assert_eq!(
        model.semantic_count(),
        array(&fixture["CityObjects"]["id-1"]["geometry"][0]["semantics"]["surfaces"]).len()
    );

    assert_metadata_matches_fixture(model, &fixture["metadata"]);
    assert_root_components_match_fixture(model, fixture);
    assert_vertices_match_fixture(model, &fixture["vertices"]);
    assert_template_geometry_matches_fixture(model, &fixture["geometry-templates"]);
    assert_building_part_matches_fixture(model, fixture);
    assert_noise_building_matches_fixture(model, fixture);
    assert_tree_matches_fixture(model, fixture);
    assert_neighbourhood_matches_fixture(model, fixture);
}

fn build_metadata<'a, SS>(metadata: &mut Metadata<SS>, fixture: &'a Value)
where
    SS: FixtureStorage<'a>,
{
    let metadata_json = &fixture["metadata"];
    let contact = &metadata_json["pointOfContact"];

    metadata.set_geographical_extent(bbox(&metadata_json["geographicalExtent"]));
    metadata.set_identifier(CityModelIdentifier::new(SS::store(string(
        &metadata_json["identifier"],
    ))));
    metadata.set_reference_date(Date::new(SS::store(string(
        &metadata_json["referenceDate"],
    ))));
    metadata.set_reference_system(CRS::new(SS::store(string(
        &metadata_json["referenceSystem"],
    ))));
    metadata.set_title(SS::store(string(&metadata_json["title"])));
    let mut poc = Contact::<SS>::new();
    poc.set_contact_name(SS::store(string(&contact["contactName"])));
    poc.set_email_address(SS::store(string(&contact["emailAddress"])));
    poc.set_role(Some(contact_role(&contact["role"])));
    poc.set_website(Some(SS::store(string(&contact["website"]))));
    poc.set_contact_type(Some(contact_type(&contact["contactType"])));
    poc.set_phone(Some(SS::store(string(&contact["phone"]))));
    poc.set_organization(Some(SS::store(string(&contact["organization"]))));
    poc.set_address(Some(attributes_from_object::<SS>(&contact["address"])));
    metadata.set_point_of_contact(Some(poc));
    metadata.extra_mut().insert(
        SS::store("nospec_description"),
        attribute_value_from_fixture::<SS>(&metadata_json["nospec_description"]),
    );
}

fn build_root_components<'a, SS>(model: &mut CityModel<u32, SS>, fixture: &'a Value)
where
    SS: FixtureStorage<'a>,
{
    model.extra_mut().insert(
        SS::store("+census"),
        attribute_value_from_fixture::<SS>(&fixture["+census"]),
    );

    let transform = model.transform_mut();
    transform.set_scale(array3_f64(&fixture["transform"]["scale"]));
    transform.set_translate(array3_f64(&fixture["transform"]["translate"]));

    let (extension_name, extension_json) = object(&fixture["extensions"])
        .iter()
        .next()
        .expect("fixture should contain one extension");
    model.extensions_mut().add(Extension::new(
        SS::store(extension_name),
        SS::store(string(&extension_json["url"])),
        SS::store(string(&extension_json["version"])),
    ));
}

fn init_cityobjects<'a, SS>(fixture: &'a Value) -> PendingCityObjects<SS>
where
    SS: FixtureStorage<'a>,
{
    PendingCityObjects {
        building_part: CityObject::new(
            CityObjectIdentifier::new(SS::store("id-1")),
            CityObjectType::BuildingPart,
        ),
        noise_building: CityObject::new(
            CityObjectIdentifier::new(SS::store("id-3")),
            CityObjectType::Extension(SS::store(string(&fixture["CityObjects"]["id-3"]["type"]))),
        ),
        tree: CityObject::new(
            CityObjectIdentifier::new(SS::store("a-tree")),
            CityObjectType::SolitaryVegetationObject,
        ),
        neighbourhood: CityObject::new(
            CityObjectIdentifier::new(SS::store("my-neighbourhood")),
            CityObjectType::CityObjectGroup,
        ),
    }
}

fn build_appearance<'a, SS>(
    model: &mut CityModel<u32, SS>,
    fixture: &'a Value,
) -> Result<Appearance>
where
    SS: FixtureStorage<'a>,
{
    let appearance = &fixture["appearance"];
    let materials = array(&appearance["materials"]);
    let irradiation_json = &materials[0];
    let red_json = &materials[1];

    let mut irradiation = Material::new(SS::store(string(&irradiation_json["name"])));
    irradiation.set_ambient_intensity(Some(number_f32(&irradiation_json["ambientIntensity"])));
    irradiation.set_diffuse_color(Some(rgb(&irradiation_json["diffuseColor"])));
    irradiation.set_emissive_color(Some(rgb(&irradiation_json["emissiveColor"])));
    irradiation.set_specular_color(Some(rgb(&irradiation_json["specularColor"])));
    irradiation.set_shininess(Some(number_f32(&irradiation_json["shininess"])));
    irradiation.set_transparency(Some(number_f32(&irradiation_json["transparency"])));
    irradiation.set_is_smooth(Some(boolean(&irradiation_json["isSmooth"])));

    let red = Material::new(SS::store(string(&red_json["name"])));

    let material_irradiation = model.add_material(irradiation)?;
    let material_red = model.add_material(red)?;
    model.set_default_material_theme(Some(ThemeName::new(SS::store(string(
        &appearance["default-theme-material"],
    )))));

    let texture_json = &array(&appearance["textures"])[0];
    let mut texture = Texture::new(
        SS::store(string(&texture_json["image"])),
        image_type(&texture_json["type"]),
    );
    texture.set_wrap_mode(Some(wrap_mode(&texture_json["wrapMode"])));
    texture.set_texture_type(Some(texture_type(&texture_json["textureType"])));
    texture.set_border_color(Some(rgba(&texture_json["borderColor"])));

    let texture_handle = model.add_texture(texture)?;
    model.set_default_texture_theme(Some(ThemeName::new(SS::store(string(
        &appearance["default-theme-texture"],
    )))));

    Ok(Appearance {
        material_irradiation,
        material_red,
        texture: texture_handle,
        texture_theme: "winter-textures",
    })
}

fn build_shared_vertices<SS: StringStorage>(
    model: &mut CityModel<u32, SS>,
    fixture: &Value,
) -> Result<SharedVertices> {
    let vertices = array(&fixture["vertices"]);
    Ok(SharedVertices {
        v0: model.add_vertex(real_world_coordinate(&vertices[0]))?,
        v1: model.add_vertex(real_world_coordinate(&vertices[1]))?,
        v2: model.add_vertex(real_world_coordinate(&vertices[2]))?,
        v3: model.add_vertex(real_world_coordinate(&vertices[3]))?,
    })
}

fn build_cityobject_id_1<'a, SS>(
    model: &mut CityModel<u32, SS>,
    building_part: &mut CityObject<SS>,
    fixture: &'a Value,
    vertices: SharedVertices,
    appearance: &Appearance,
) -> Result<()>
where
    SS: FixtureStorage<'a>,
{
    let building_json = &fixture["CityObjects"]["id-1"];
    building_part.set_geographical_extent(Some(bbox(&building_json["geographicalExtent"])));
    insert_building_address(
        model,
        building_part,
        &array(&building_json["address"])[0],
        vertices.v0,
    )?;
    insert_attributes_from_object::<SS>(
        building_part.attributes_mut(),
        &building_json["attributes"],
    );
    let geometry_ref = build_building_geometry(
        model,
        vertices,
        appearance,
        &array(&building_json["geometry"])[0],
    )?;
    building_part.add_geometry(geometry_ref);

    Ok(())
}

fn insert_building_address<'a, SS>(
    model: &mut CityModel<u32, SS>,
    building_part: &mut CityObject<SS>,
    address_json: &'a Value,
    address_vertex: VertexIndex<u32>,
) -> Result<()>
where
    SS: FixtureStorage<'a>,
{
    let mut address = map_from_object::<SS>(address_json, Some("location"));
    let location_geometry = GeometryDraft::multi_point(
        Some(lod(&address_json["location"]["lod"])),
        [PointDraft::new(address_vertex)],
    )
    .insert_into(model)?;
    address.insert(
        SS::store("location"),
        Box::new(AttributeValue::Geometry(location_geometry)),
    );
    building_part.extra_mut().insert(
        SS::store("address"),
        AttributeValue::Vec(vec![Box::new(AttributeValue::Map(address))]),
    );
    Ok(())
}

fn build_building_geometry<'a, SS>(
    model: &mut CityModel<u32, SS>,
    vertices: SharedVertices,
    appearance: &Appearance,
    geometry_json: &'a Value,
) -> Result<cityjson::resources::GeometryHandle>
where
    SS: FixtureStorage<'a>,
{
    let semantics = &geometry_json["semantics"]["surfaces"];
    let roof_semantic = model.add_semantic(Semantic::new(SemanticType::RoofSurface))?;
    model
        .get_semantic_mut(roof_semantic)
        .expect("roof semantic should exist")
        .attributes_mut()
        .insert(
            SS::store("surfaceAttribute"),
            attribute_value_from_fixture::<SS>(&semantics[0]["surfaceAttribute"]),
        );

    let patio_semantic = model.add_semantic(Semantic::new(SemanticType::Extension(SS::store(
        string(&semantics[1]["type"]),
    ))))?;
    model
        .get_semantic_mut(roof_semantic)
        .expect("roof semantic should exist")
        .children_mut()
        .push(patio_semantic);
    model
        .get_semantic_mut(patio_semantic)
        .expect("patio semantic should exist")
        .set_parent(roof_semantic);

    let roof_ring = [vertices.v0, vertices.v3, vertices.v2, vertices.v1];
    let textured_ring = RingDraft::new(roof_ring).with_texture(
        ThemeName::new(SS::store(appearance.texture_theme)),
        appearance.texture,
        [[0.0, 0.5], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
    );

    let surface_0 = SurfaceDraft::new(textured_ring.clone(), [])
        .with_semantic(roof_semantic)
        .with_material(
            ThemeName::new(SS::store("irradiation")),
            appearance.material_irradiation,
        )
        .with_material(ThemeName::new(SS::store("red")), appearance.material_red);
    let surface_1 = SurfaceDraft::new(textured_ring, [])
        .with_semantic(roof_semantic)
        .with_material(
            ThemeName::new(SS::store("irradiation")),
            appearance.material_irradiation,
        )
        .with_material(ThemeName::new(SS::store("red")), appearance.material_red);
    let surface_2 = SurfaceDraft::new(RingDraft::new(roof_ring), [])
        .with_material(
            ThemeName::new(SS::store("irradiation")),
            appearance.material_irradiation,
        )
        .with_material(ThemeName::new(SS::store("red")), appearance.material_red);
    let surface_3 = SurfaceDraft::new(RingDraft::new(roof_ring), [])
        .with_semantic(patio_semantic)
        .with_material(ThemeName::new(SS::store("red")), appearance.material_red);
    let surface_4 = SurfaceDraft::new(
        RingDraft::new([vertices.v1, vertices.v2, vertices.v3, vertices.v0]),
        [RingDraft::new([
            vertices.v1,
            vertices.v2,
            vertices.v3,
            vertices.v0,
        ])],
    );

    GeometryDraft::solid(
        Some(lod(&geometry_json["lod"])),
        ShellDraft::new([surface_0, surface_1, surface_2, surface_3]),
        [ShellDraft::new([surface_4])],
    )
    .insert_into(model)
}

fn build_cityobject_id_3<'a, SS>(noise_building: &mut CityObject<SS>, fixture: &'a Value)
where
    SS: FixtureStorage<'a>,
{
    for (key, value) in object(&fixture["CityObjects"]["id-3"]["attributes"]) {
        noise_building
            .attributes_mut()
            .insert(SS::store(key), attribute_value_from_fixture::<SS>(value));
    }
}

fn build_cityobject_tree<SS: StringStorage>(
    model: &mut CityModel<u32, SS>,
    tree: &mut CityObject<SS>,
    fixture: &Value,
    reference_point: VertexIndex<u32>,
) -> Result<()> {
    let geometry_templates = &fixture["geometry-templates"];
    let template_vertices_json = array(&geometry_templates["vertices-templates"]);
    let mut template_vertices = Vec::with_capacity(template_vertices_json.len());
    for vertex in template_vertices_json {
        template_vertices.push(model.add_template_vertex(real_world_coordinate(vertex))?);
    }

    let template_geometry_json = &array(&geometry_templates["templates"])[0];
    let boundaries = array(&template_geometry_json["boundaries"]);
    let surface_0 = SurfaceDraft::new(
        RingDraft::new(vertex_quad(&template_vertices, &boundaries[0][0])),
        [],
    );
    let surface_1 = SurfaceDraft::new(
        RingDraft::new(vertex_quad(&template_vertices, &boundaries[1][0])),
        [],
    );
    let surface_2 = SurfaceDraft::new(
        RingDraft::new(vertex_quad(&template_vertices, &boundaries[2][0])),
        [],
    );

    let template_ref = GeometryDraft::multi_surface(
        Some(lod(&template_geometry_json["lod"])),
        [surface_0, surface_1, surface_2],
    )
    .insert_template_into(model)?;

    let instance_json = &array(&fixture["CityObjects"]["a-tree"]["geometry"])[0];
    let tree_geometry = GeometryDraft::instance(
        template_ref,
        reference_point,
        AffineTransform3D::new(matrix16(&instance_json["transformationMatrix"])),
    )
    .insert_into(model)?;
    tree.add_geometry(tree_geometry);

    Ok(())
}

fn build_cityobject_neighbourhood<'a, SS>(
    model: &mut CityModel<u32, SS>,
    neighbourhood: &mut CityObject<SS>,
    fixture: &'a Value,
    vertices: SharedVertices,
) -> Result<()>
where
    SS: FixtureStorage<'a>,
{
    let neighbourhood_json = &fixture["CityObjects"]["my-neighbourhood"];
    let attributes = object(&neighbourhood_json["attributes"]);
    for (key, value) in attributes {
        neighbourhood
            .attributes_mut()
            .insert(SS::store(key), attribute_value_from_fixture::<SS>(value));
    }
    neighbourhood.extra_mut().insert(
        SS::store("children_roles"),
        attribute_value_from_fixture::<SS>(&neighbourhood_json["children_roles"]),
    );

    let geometry_json = &array(&neighbourhood_json["geometry"])[0];
    let neighbourhood_geometry = GeometryDraft::multi_surface(
        Some(lod(&geometry_json["lod"])),
        [SurfaceDraft::new(
            RingDraft::new([vertices.v0, vertices.v1, vertices.v2, vertices.v3]),
            [],
        )],
    )
    .insert_into(model)?;
    neighbourhood.add_geometry(neighbourhood_geometry);

    Ok(())
}

fn add_cityobjects_with_hierarchy<SS: StringStorage>(
    model: &mut CityModel<u32, SS>,
    pending: PendingCityObjects<SS>,
) -> Result<()> {
    let PendingCityObjects {
        building_part,
        noise_building,
        tree,
        neighbourhood,
    } = pending;

    let cityobjects = model.cityobjects_mut();
    let building_part = cityobjects.add(building_part)?;
    let noise_building = cityobjects.add(noise_building)?;
    cityobjects.add(tree)?;
    let neighbourhood = cityobjects.add(neighbourhood)?;

    cityobjects
        .get_mut(building_part)
        .expect("building part should exist")
        .add_parent(noise_building);
    cityobjects
        .get_mut(building_part)
        .expect("building part should exist")
        .add_parent(neighbourhood);
    cityobjects
        .get_mut(noise_building)
        .expect("noise building should exist")
        .add_child(building_part);
    cityobjects
        .get_mut(noise_building)
        .expect("noise building should exist")
        .add_parent(neighbourhood);
    cityobjects
        .get_mut(neighbourhood)
        .expect("neighbourhood should exist")
        .add_child(building_part);
    cityobjects
        .get_mut(neighbourhood)
        .expect("neighbourhood should exist")
        .add_child(noise_building);

    Ok(())
}

fn assert_metadata_matches_fixture<SS: StringStorage>(
    model: &CityModel<u32, SS>,
    metadata_json: &Value,
) {
    let metadata = model.metadata().expect("metadata should exist");
    let contact = metadata
        .point_of_contact()
        .expect("point of contact should exist");

    assert_eq!(
        metadata.geographical_extent(),
        Some(&bbox(&metadata_json["geographicalExtent"]))
    );
    assert_eq!(
        metadata
            .identifier()
            .expect("identifier should exist")
            .to_string(),
        string(&metadata_json["identifier"])
    );
    assert_eq!(
        metadata
            .reference_date()
            .expect("reference date should exist")
            .to_string(),
        string(&metadata_json["referenceDate"])
    );
    assert_eq!(metadata.title(), Some(string(&metadata_json["title"])));
    assert_eq!(
        metadata
            .reference_system()
            .expect("reference system should exist")
            .to_string(),
        string(&metadata_json["referenceSystem"])
    );

    assert_eq!(
        contact.contact_name(),
        string(&metadata_json["pointOfContact"]["contactName"])
    );
    assert_eq!(
        contact.email_address(),
        string(&metadata_json["pointOfContact"]["emailAddress"])
    );
    assert_eq!(
        contact.role(),
        Some(contact_role(&metadata_json["pointOfContact"]["role"]))
    );
    assert_eq!(
        contact.website().as_deref(),
        Some(string(&metadata_json["pointOfContact"]["website"]))
    );
    assert_eq!(
        contact.contact_type(),
        Some(contact_type(
            &metadata_json["pointOfContact"]["contactType"]
        ))
    );
    assert_eq!(
        contact.phone().as_deref(),
        Some(string(&metadata_json["pointOfContact"]["phone"]))
    );
    assert_eq!(
        contact.organization().as_deref(),
        Some(string(&metadata_json["pointOfContact"]["organization"]))
    );
    assert_attributes_match_json(
        contact.address().expect("contact address should exist"),
        &metadata_json["pointOfContact"]["address"],
    );
    assert_attributes_match_json(
        metadata.extra().expect("metadata extra should exist"),
        &json_object_with(
            "nospec_description",
            metadata_json["nospec_description"].clone(),
        ),
    );
}

fn assert_root_components_match_fixture<SS: StringStorage>(
    model: &CityModel<u32, SS>,
    fixture: &Value,
) {
    assert_attributes_match_json(
        model.extra().expect("root extra should exist"),
        &json_object_with("+census", fixture["+census"].clone()),
    );

    let transform = model.transform().expect("transform should exist");
    assert_f64_slice_eq(
        &transform.scale(),
        &array3_f64(&fixture["transform"]["scale"]),
    );
    assert_f64_slice_eq(
        &transform.translate(),
        &array3_f64(&fixture["transform"]["translate"]),
    );

    let (extension_name, extension_json) = object(&fixture["extensions"])
        .iter()
        .next()
        .expect("fixture should contain one extension");
    let extension = model
        .extensions()
        .expect("extensions should exist")
        .get(extension_name)
        .expect("fixture extension should exist");
    assert_eq!(extension.name().as_ref(), extension_name);
    assert_eq!(extension.url().as_ref(), string(&extension_json["url"]));
    assert_eq!(
        extension.version().as_ref(),
        string(&extension_json["version"])
    );

    assert_eq!(
        model.default_material_theme().map(AsRef::as_ref),
        Some(string(&fixture["appearance"]["default-theme-material"]))
    );
    assert_eq!(
        model.default_texture_theme().map(AsRef::as_ref),
        Some(string(&fixture["appearance"]["default-theme-texture"]))
    );
}

fn assert_vertices_match_fixture<SS: StringStorage>(
    model: &CityModel<u32, SS>,
    vertices_json: &Value,
) {
    let vertices = model.vertices().as_slice();
    let expected = array(vertices_json);
    assert_eq!(vertices.len(), expected.len());
    for (actual, expected) in vertices.iter().zip(expected.iter()) {
        let expected = array3_f64(expected);
        assert_f64_eq(actual.x(), expected[0]);
        assert_f64_eq(actual.y(), expected[1]);
        assert_f64_eq(actual.z(), expected[2]);
    }
}

fn assert_template_geometry_matches_fixture<SS: StringStorage>(
    model: &CityModel<u32, SS>,
    templates_json: &Value,
) {
    let expected_templates = array(&templates_json["templates"]);
    let expected_vertices = array(&templates_json["vertices-templates"]);

    assert_eq!(model.geometry_template_count(), expected_templates.len());
    assert_eq!(model.template_vertices().len(), expected_vertices.len());

    for (actual, expected) in model
        .template_vertices()
        .as_slice()
        .iter()
        .zip(expected_vertices.iter())
    {
        let expected = array3_f64(expected);
        assert_f64_eq(actual.x(), expected[0]);
        assert_f64_eq(actual.y(), expected[1]);
        assert_f64_eq(actual.z(), expected[2]);
    }

    let (_, template) = model
        .iter_geometry_templates()
        .next()
        .expect("template geometry should exist");
    let expected_template = &expected_templates[0];
    assert_eq!(template.type_geometry(), &GeometryType::MultiSurface);
    assert_eq!(template.lod(), Some(&lod(&expected_template["lod"])));
    assert_eq!(
        template
            .boundaries()
            .expect("template boundaries should exist")
            .surfaces()
            .len(),
        array(&expected_template["boundaries"]).len()
    );
}

fn assert_building_part_matches_fixture<SS: StringStorage>(
    model: &CityModel<u32, SS>,
    fixture: &Value,
) {
    let expected = &fixture["CityObjects"]["id-1"];
    let (_, building_part) = find_cityobject(model, "id-1");

    assert_building_part_base(model, building_part, expected);
    assert_building_part_address(model, building_part, &array(&expected["address"])[0]);
    assert_building_part_geometry(model, building_part, &expected["geometry"][0]);
}

fn assert_building_part_base<SS: StringStorage>(
    model: &CityModel<u32, SS>,
    building_part: &CityObject<SS>,
    expected: &Value,
) {
    assert_eq!(
        building_part.type_cityobject(),
        &CityObjectType::BuildingPart
    );
    assert_eq!(
        building_part.geographical_extent(),
        Some(&bbox(&expected["geographicalExtent"]))
    );
    assert_attributes_match_json(
        building_part
            .attributes()
            .expect("building attributes should exist"),
        &expected["attributes"],
    );
    let parents = sorted_cityobject_ids(
        model,
        building_part
            .parents()
            .expect("building parents should exist"),
    );
    let expected_parents = array(&expected["parents"])
        .iter()
        .map(|value| string(value).to_owned())
        .collect::<Vec<_>>();
    assert_eq!(parents, expected_parents);
}

fn assert_building_part_address<SS: StringStorage>(
    model: &CityModel<u32, SS>,
    building_part: &CityObject<SS>,
    expected_address: &Value,
) {
    let address_values = match building_part
        .extra()
        .expect("building extra should exist")
        .get("address")
        .expect("address should exist")
    {
        AttributeValue::Vec(values) => values,
        other => panic!("expected address vector, got {other:?}"),
    };
    assert_eq!(address_values.len(), 1);
    let address_map = match &*address_values[0] {
        AttributeValue::Map(values) => values,
        other => panic!("expected address map, got {other:?}"),
    };
    for (key, value) in object(expected_address) {
        if key == "location" {
            continue;
        }
        assert_attribute_matches_json(
            address_map
                .get(key.as_str())
                .expect("address key should exist")
                .as_ref(),
            value,
        );
    }

    let location_handle = match address_map
        .get("location")
        .expect("address location should exist")
        .as_ref()
    {
        AttributeValue::Geometry(handle) => *handle,
        other => panic!("expected address location geometry, got {other:?}"),
    };
    let location_geometry = model
        .get_geometry(location_handle)
        .expect("address geometry should exist");
    assert_eq!(location_geometry.type_geometry(), &GeometryType::MultiPoint);
    assert_eq!(
        location_geometry.lod(),
        Some(&lod(&expected_address["location"]["lod"]))
    );
    assert_eq!(
        location_geometry
            .boundaries()
            .expect("location boundaries should exist")
            .vertices(),
        &[0u32.into()]
    );
}

fn assert_building_part_geometry<SS: StringStorage>(
    model: &CityModel<u32, SS>,
    building_part: &CityObject<SS>,
    expected_geometry: &Value,
) {
    let geometry_handle = building_part
        .geometry()
        .expect("building geometry should exist")[0];
    let geometry = model
        .get_geometry(geometry_handle)
        .expect("building geometry should exist");
    assert_eq!(geometry.type_geometry(), &GeometryType::Solid);
    assert_eq!(geometry.lod(), Some(&lod(&expected_geometry["lod"])));

    let semantics = geometry
        .semantics()
        .expect("building semantics should exist");
    assert_eq!(model.semantic_count(), 2);
    assert_eq!(semantics.surfaces().len(), 5);

    let roof_handle = semantics.surfaces()[0].expect("roof semantic should exist");
    let roof = model
        .get_semantic(roof_handle)
        .expect("roof semantic should exist");
    assert_eq!(roof.type_semantic(), &SemanticType::RoofSurface);
    assert_attributes_match_json(
        roof.attributes().expect("roof attributes should exist"),
        &json_object_with(
            "surfaceAttribute",
            expected_geometry["semantics"]["surfaces"][0]["surfaceAttribute"].clone(),
        ),
    );

    let patio_handle = semantics.surfaces()[3].expect("patio semantic should exist");
    assert_eq!(roof.children(), Some(&[patio_handle] as &[_]));
    let patio = model
        .get_semantic(patio_handle)
        .expect("patio semantic should exist");
    match patio.type_semantic() {
        SemanticType::Extension(value) => {
            assert_eq!(
                value.as_ref(),
                string(&expected_geometry["semantics"]["surfaces"][1]["type"])
            );
        }
        other => panic!("expected patio extension semantic, got {other:?}"),
    }
    assert_eq!(patio.parent(), Some(roof_handle));

    let materials = geometry
        .materials()
        .expect("building materials should exist");
    assert_eq!(
        materials.len(),
        object(&expected_geometry["material"]).len()
    );
    assert!(
        materials
            .iter()
            .any(|(name, _)| name.as_ref() == "irradiation")
    );
    assert!(materials.iter().any(|(name, _)| name.as_ref() == "red"));

    let textures = geometry.textures().expect("building textures should exist");
    assert_eq!(textures.len(), object(&expected_geometry["texture"]).len());
    let texture_theme = object(&expected_geometry["texture"])
        .keys()
        .next()
        .expect("texture theme should exist");
    let texture = textures
        .iter()
        .find(|(name, _)| name.as_ref() == texture_theme.as_str())
        .expect("building texture theme should exist")
        .1;
    assert_eq!(
        texture.rings(),
        geometry
            .boundaries()
            .expect("building boundaries should exist")
            .rings()
    );
}

fn assert_noise_building_matches_fixture<SS: StringStorage>(
    model: &CityModel<u32, SS>,
    fixture: &Value,
) {
    let expected = &fixture["CityObjects"]["id-3"];
    let (_, noise_building) = find_cityobject(model, "id-3");

    match noise_building.type_cityobject() {
        CityObjectType::Extension(value) => assert_eq!(value.as_ref(), string(&expected["type"])),
        other => panic!("expected extension cityobject, got {other:?}"),
    }
    assert_attributes_match_json(
        noise_building
            .attributes()
            .expect("noise building attributes should exist"),
        &expected["attributes"],
    );
    assert_eq!(
        sorted_cityobject_ids(
            model,
            noise_building
                .children()
                .expect("noise building children should exist"),
        ),
        array(&expected["children"])
            .iter()
            .map(|value| string(value).to_owned())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        sorted_cityobject_ids(
            model,
            noise_building
                .parents()
                .expect("noise building parents should exist"),
        ),
        array(&expected["parents"])
            .iter()
            .map(|value| string(value).to_owned())
            .collect::<Vec<_>>()
    );
    assert!(noise_building.geometry().is_none());
}

fn assert_tree_matches_fixture<SS: StringStorage>(model: &CityModel<u32, SS>, fixture: &Value) {
    let expected = &fixture["CityObjects"]["a-tree"];
    let (_, tree) = find_cityobject(model, "a-tree");

    assert_eq!(
        tree.type_cityobject(),
        &CityObjectType::SolitaryVegetationObject
    );
    assert!(tree.attributes().is_none());
    assert!(tree.extra().is_none());
    assert!(tree.parents().is_none());
    assert!(tree.children().is_none());

    let geometry_handle = tree.geometry().expect("tree geometry should exist")[0];
    let geometry = model
        .get_geometry(geometry_handle)
        .expect("tree geometry should exist");
    assert_eq!(geometry.type_geometry(), &GeometryType::GeometryInstance);
    let instance = geometry.instance().expect("tree instance should exist");
    let expected_matrix = matrix16(&expected["geometry"][0]["transformationMatrix"]);
    assert_f64_slice_eq(instance.transformation().as_array(), &expected_matrix);

    let reference = model
        .get_vertex(instance.reference_point())
        .expect("instance reference point should exist");
    let expected_reference = array3_f64(&fixture["vertices"][1]);
    assert_f64_eq(reference.x(), expected_reference[0]);
    assert_f64_eq(reference.y(), expected_reference[1]);
    assert_f64_eq(reference.z(), expected_reference[2]);

    let template = model
        .get_geometry_template(instance.template())
        .expect("template geometry should exist");
    assert_eq!(template.type_geometry(), &GeometryType::MultiSurface);
}

fn assert_neighbourhood_matches_fixture<SS: StringStorage>(
    model: &CityModel<u32, SS>,
    fixture: &Value,
) {
    let expected = &fixture["CityObjects"]["my-neighbourhood"];
    let (_, neighbourhood) = find_cityobject(model, "my-neighbourhood");

    assert_eq!(
        neighbourhood.type_cityobject(),
        &CityObjectType::CityObjectGroup
    );
    assert_attributes_match_json(
        neighbourhood
            .attributes()
            .expect("neighbourhood attributes should exist"),
        &expected["attributes"],
    );
    assert_attributes_match_json(
        neighbourhood
            .extra()
            .expect("neighbourhood extra should exist"),
        &json_object_with("children_roles", expected["children_roles"].clone()),
    );
    assert_eq!(
        sorted_cityobject_ids(
            model,
            neighbourhood
                .children()
                .expect("neighbourhood children should exist"),
        ),
        array(&expected["children"])
            .iter()
            .map(|value| string(value).to_owned())
            .collect::<Vec<_>>()
    );
    assert!(neighbourhood.parents().is_none());

    let geometry_handle = neighbourhood
        .geometry()
        .expect("neighbourhood geometry should exist")[0];
    let geometry = model
        .get_geometry(geometry_handle)
        .expect("neighbourhood geometry should exist");
    assert_eq!(geometry.type_geometry(), &GeometryType::MultiSurface);
    assert_eq!(geometry.lod(), Some(&lod(&expected["geometry"][0]["lod"])));
    assert!(geometry.semantics().is_none());
    assert!(geometry.materials().is_none());
    assert!(geometry.textures().is_none());
}

fn attribute_value_from_fixture<'a, SS>(value: &'a Value) -> AttributeValue<SS>
where
    SS: FixtureStorage<'a>,
{
    match value {
        Value::Null => AttributeValue::Null,
        Value::Bool(value) => AttributeValue::Bool(*value),
        Value::Number(value) if value.is_f64() => {
            AttributeValue::Float(value.as_f64().expect("float should parse"))
        }
        Value::Number(value) => AttributeValue::Integer(value.as_i64().unwrap_or_else(|| {
            i64::try_from(value.as_u64().expect("integer should parse"))
                .expect("fixture integer should fit in i64")
        })),
        Value::String(value) => AttributeValue::String(SS::store(value)),
        Value::Array(values) => AttributeValue::Vec(
            values
                .iter()
                .map(|value| Box::new(attribute_value_from_fixture::<SS>(value)))
                .collect(),
        ),
        Value::Object(values) => AttributeValue::Map(
            values
                .iter()
                .map(|(key, value)| {
                    (
                        SS::store(key),
                        Box::new(attribute_value_from_fixture::<SS>(value)),
                    )
                })
                .collect::<HashMap<_, _>>(),
        ),
    }
}

fn attributes_from_object<'a, SS>(value: &'a Value) -> Attributes<SS>
where
    SS: FixtureStorage<'a>,
{
    let mut attributes = Attributes::new();
    insert_attributes_from_object::<SS>(&mut attributes, value);
    attributes
}

fn insert_attributes_from_object<'a, SS>(attributes: &mut Attributes<SS>, value: &'a Value)
where
    SS: FixtureStorage<'a>,
{
    for (key, value) in object(value) {
        attributes.insert(SS::store(key), attribute_value_from_fixture::<SS>(value));
    }
}

fn map_from_object<'a, SS>(
    value: &'a Value,
    skip_key: Option<&str>,
) -> HashMap<SS::String, Box<AttributeValue<SS>>>
where
    SS: FixtureStorage<'a>,
{
    object(value)
        .iter()
        .filter(|(key, _)| Some(key.as_str()) != skip_key)
        .map(|(key, value)| {
            (
                SS::store(key),
                Box::new(attribute_value_from_fixture::<SS>(value)),
            )
        })
        .collect()
}

fn find_cityobject<'a, SS: StringStorage>(
    model: &'a CityModel<u32, SS>,
    id: &str,
) -> (CityObjectHandle, &'a CityObject<SS>) {
    model
        .cityobjects()
        .iter()
        .find(|(_, cityobject)| cityobject.id() == id)
        .expect("cityobject should exist")
}

fn sorted_cityobject_ids<SS: StringStorage>(
    model: &CityModel<u32, SS>,
    handles: &[CityObjectHandle],
) -> Vec<String> {
    let mut ids = handles
        .iter()
        .map(|handle| {
            model
                .cityobjects()
                .get(*handle)
                .expect("cityobject handle should exist")
                .id()
                .to_owned()
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn assert_attributes_match_json<SS: StringStorage>(attributes: &Attributes<SS>, expected: &Value) {
    let expected = object(expected);
    assert_eq!(attributes.len(), expected.len());
    for (key, expected_value) in expected {
        let actual = attributes
            .get(key)
            .unwrap_or_else(|| panic!("missing attribute {key}"));
        assert_attribute_matches_json(actual, expected_value);
    }
}

fn assert_attribute_matches_json<SS: StringStorage>(actual: &AttributeValue<SS>, expected: &Value) {
    match (actual, expected) {
        (AttributeValue::Null, Value::Null) => {}
        (AttributeValue::Bool(actual), Value::Bool(expected)) => assert_eq!(actual, expected),
        (AttributeValue::Integer(actual), Value::Number(_)) => {
            assert_eq!(*actual, integer(expected));
        }
        (AttributeValue::Float(actual), Value::Number(_)) => {
            assert_f64_eq(*actual, number(expected));
        }
        (AttributeValue::String(actual), Value::String(expected)) => {
            assert_eq!(actual.as_ref(), expected);
        }
        (AttributeValue::Vec(actual), Value::Array(expected)) => {
            assert_eq!(actual.len(), expected.len());
            for (actual, expected) in actual.iter().zip(expected.iter()) {
                assert_attribute_matches_json(actual, expected);
            }
        }
        (AttributeValue::Map(actual), Value::Object(expected)) => {
            assert_eq!(actual.len(), expected.len());
            for (key, expected_value) in expected {
                let actual = actual
                    .get(key.as_str())
                    .unwrap_or_else(|| panic!("missing map entry {key}"));
                assert_attribute_matches_json(actual, expected_value);
            }
        }
        (AttributeValue::Geometry(_), Value::Object(_)) => {
            panic!("geometry attributes must be asserted separately");
        }
        _ => panic!("attribute type mismatch: actual {actual:?}, expected {expected:?}"),
    }
}

fn object(value: &Value) -> &JsonObject {
    value.as_object().expect("fixture object expected")
}

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("fixture array expected")
}

fn string(value: &Value) -> &str {
    value.as_str().expect("fixture string expected")
}

fn boolean(value: &Value) -> bool {
    value.as_bool().expect("fixture bool expected")
}

fn integer(value: &Value) -> i64 {
    value.as_i64().unwrap_or_else(|| {
        i64::try_from(value.as_u64().expect("fixture integer expected"))
            .expect("fixture integer should fit in i64")
    })
}

fn number(value: &Value) -> f64 {
    value
        .as_number()
        .expect("fixture number expected")
        .to_string()
        .parse::<f64>()
        .expect("fixture number should parse as f64")
}

fn number_f32(value: &Value) -> f32 {
    value
        .as_number()
        .expect("fixture number expected")
        .to_string()
        .parse::<f32>()
        .expect("fixture number should parse as f32")
}

fn bbox(value: &Value) -> BBox {
    let values = array(value);
    BBox::new(
        number(&values[0]),
        number(&values[1]),
        number(&values[2]),
        number(&values[3]),
        number(&values[4]),
        number(&values[5]),
    )
}

fn array3_f64(value: &Value) -> [f64; 3] {
    let values = array(value);
    [number(&values[0]), number(&values[1]), number(&values[2])]
}

fn real_world_coordinate(value: &Value) -> RealWorldCoordinate {
    let [x, y, z] = array3_f64(value);
    RealWorldCoordinate::new(x, y, z)
}

fn rgb(value: &Value) -> RGB {
    let [r, g, b] = array3_f64(value);
    RGB::new(
        r.to_string()
            .parse::<f32>()
            .expect("rgb value should parse as f32"),
        g.to_string()
            .parse::<f32>()
            .expect("rgb value should parse as f32"),
        b.to_string()
            .parse::<f32>()
            .expect("rgb value should parse as f32"),
    )
}

fn rgba(value: &Value) -> RGBA {
    let values = array(value);
    RGBA::new(
        number_f32(&values[0]),
        number_f32(&values[1]),
        number_f32(&values[2]),
        number_f32(&values[3]),
    )
}

fn image_type(value: &Value) -> ImageType {
    match string(value) {
        "PNG" => ImageType::Png,
        other => panic!("unsupported fixture image type {other}"),
    }
}

fn wrap_mode(value: &Value) -> WrapMode {
    match string(value) {
        "wrap" => WrapMode::Wrap,
        other => panic!("unsupported fixture wrap mode {other}"),
    }
}

fn texture_type(value: &Value) -> TextureType {
    match string(value) {
        "specific" => TextureType::Specific,
        other => panic!("unsupported fixture texture type {other}"),
    }
}

fn lod(value: &Value) -> LoD {
    match string(value) {
        "1" => LoD::LoD1,
        "2" => LoD::LoD2,
        "2.1" => LoD::LoD2_1,
        "2.2" => LoD::LoD2_2,
        other => panic!("unsupported fixture lod {other}"),
    }
}

fn contact_role(value: &Value) -> ContactRole {
    match string(value) {
        "author" => ContactRole::Author,
        other => panic!("unsupported fixture contact role {other}"),
    }
}

fn contact_type(value: &Value) -> ContactType {
    match string(value) {
        "organization" => ContactType::Organization,
        other => panic!("unsupported fixture contact type {other}"),
    }
}

fn matrix16(value: &Value) -> [f64; 16] {
    let values = array(value);
    let mut matrix = [0.0; 16];
    for (index, value) in values.iter().enumerate() {
        matrix[index] = number(value);
    }
    matrix
}

fn vertex_quad<VR: VertexRef>(
    vertices: &[VertexIndex<VR>],
    ring_json: &Value,
) -> [VertexIndex<VR>; 4] {
    let ring = array(ring_json);
    [
        vertices[usize::try_from(integer(&ring[0])).expect("fixture index should fit usize")],
        vertices[usize::try_from(integer(&ring[1])).expect("fixture index should fit usize")],
        vertices[usize::try_from(integer(&ring[2])).expect("fixture index should fit usize")],
        vertices[usize::try_from(integer(&ring[3])).expect("fixture index should fit usize")],
    ]
}

fn json_object_with(key: &str, value: Value) -> Value {
    let mut map = JsonObject::new();
    map.insert(key.to_owned(), value);
    Value::Object(map)
}

fn assert_f64_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= FLOAT_EPSILON,
        "expected {expected}, got {actual}"
    );
}

fn assert_f64_slice_eq(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected.iter()) {
        assert_f64_eq(*actual, *expected);
    }
}
