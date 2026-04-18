from __future__ import annotations

from cityjson_lib import (
    AffineTransform4x4,
    BBox,
    CityObjectDraft,
    CityModel,
    Contact,
    ContactRole,
    ContactType,
    GeometryDraft,
    ImageType,
    MaterialId,
    ModelType,
    RGBA,
    RGB,
    RingDraft,
    ShellDraft,
    SurfaceDraft,
    TextureType,
    Transform,
    UV,
    Value,
    Vertex,
    WrapMode,
)


def make_contact_address() -> Value:
    return (
        Value.object()
        .insert("city", Value.string("Den Haag"))
        .insert("country", Value.string("The Netherlands"))
    )


def make_census() -> Value:
    return (
        Value.object()
        .insert("percent_men", Value.number(49.5))
        .insert("percent_women", Value.number(51.5))
    )


def make_building_address(location_geometry) -> Value:
    entry = Value.object()
    entry.insert("Country", Value.string("Canada"))
    entry.insert("Locality", Value.string("Chibougamau"))
    entry.insert("ThoroughfareNumber", Value.string("1"))
    entry.insert("ThoroughfareName", Value.string("rue de la Patate"))
    entry.insert("Postcode", Value.string("H0H 0H0"))
    entry.insert("location", Value.geometry(location_geometry))

    return Value.array().push(entry)


def make_children_roles() -> Value:
    return (
        Value.array()
        .push(Value.string("residential building"))
        .push(Value.string("voting location"))
    )


def make_template_geometry() -> GeometryDraft:
    surface_0 = SurfaceDraft(
        RingDraft().push_vertex_index(0).push_vertex_index(3).push_vertex_index(2).push_vertex_index(1)
    )
    surface_1 = SurfaceDraft(
        RingDraft().push_vertex_index(1).push_vertex_index(2).push_vertex_index(0).push_vertex_index(3)
    )
    surface_2 = SurfaceDraft(
        RingDraft().push_vertex_index(0).push_vertex_index(1).push_vertex_index(3).push_vertex_index(2)
    )

    draft = GeometryDraft.multi_surface("2.1")
    draft.add_surface(surface_0)
    draft.add_surface(surface_1)
    draft.add_surface(surface_2)
    return draft


def _add_roof_materials(surface: SurfaceDraft, irradiation: MaterialId) -> SurfaceDraft:
    surface.add_material("irradiation", irradiation)
    surface.add_material("red", irradiation)
    return surface


def build_fake_complete_model() -> CityModel:
    model = CityModel.create(model_type=ModelType.CITY_JSON)

    model.set_metadata_geographical_extent(
        BBox(
            min_x=84710.1,
            min_y=446846.0,
            min_z=-5.3,
            max_x=84757.1,
            max_y=446944.0,
            max_z=40.9,
        )
    )
    model.set_metadata_identifier("eaeceeaa-3f66-429a-b81d-bbc6140b8c1c")
    model.set_metadata_reference_date("2026-01-26")
    model.set_metadata_reference_system("https://www.opengis.net/def/crs/EPSG/0/2355")
    model.set_metadata_title("Complete and fake CityJSON")
    model.set_metadata_contact(
        Contact()
        .set_name("Kitalált Név")
        .set_email("spam@3dgi.nl")
        .set_role(ContactRole.AUTHOR)
        .set_website("https://3dgi.nl")
        .set_type(ContactType.ORGANIZATION)
        .set_phone("+36612345678")
        .set_organization("3DGI")
        .set_address(make_contact_address())
    )
    model.set_metadata_extra(
        "nospec_description",
        Value.string("A CityJSON object with all existing properties set to a fake value"),
    )

    model.set_root_extra("+census", make_census())
    model.add_extension("Noise", "https://someurl.org/noise.ext.json", "0.5")
    model.set_transform(Transform(scale=(1.0, 1.0, 1.0), translate=(0.0, 0.0, 0.0)))

    irradiation = model.add_material("irradiation")
    model.set_material_ambient_intensity(irradiation, 0.2)
    model.set_material_diffuse_color(irradiation, RGB(r=0.9, g=0.1, b=0.75))
    model.set_material_emissive_color(irradiation, RGB(r=0.9, g=0.1, b=0.75))
    model.set_material_specular_color(irradiation, RGB(r=0.9, g=0.1, b=0.75))
    model.set_material_shininess(irradiation, 0.2)
    model.set_material_transparency(irradiation, 0.5)
    model.set_material_is_smooth(irradiation, False)
    _unused_red = model.add_material("red")
    texture = model.add_texture("http://www.someurl.org/filename.jpg", ImageType.PNG)
    model.set_texture_wrap_mode(texture, WrapMode.WRAP)
    model.set_texture_type(texture, TextureType.SPECIFIC)
    model.set_texture_border_color(texture, RGBA(r=1.0, g=1.0, b=1.0, a=1.0))
    model.set_default_material_theme("irradiation")
    model.set_default_texture_theme("summer-textures")

    roof_uvs = [
        UV(u=0.0, v=0.5),
        UV(u=1.0, v=0.0),
        UV(u=1.0, v=1.0),
        UV(u=0.0, v=1.0),
    ]

    v0 = model.add_vertex(Vertex(x=102.0, y=103.0, z=1.0))
    v1 = model.add_vertex(Vertex(x=11.0, y=910.0, z=43.0))
    v2 = model.add_vertex(Vertex(x=25.0, y=744.0, z=22.0))
    v3 = model.add_vertex(Vertex(x=23.0, y=88.0, z=5.0))

    location = GeometryDraft.multi_point("1")
    location.add_point(v0)
    location_geometry = model.add_geometry(location)

    roof = model.add_semantic("RoofSurface")
    model.set_semantic_extra(roof, "surfaceAttribute", Value.boolean(True))
    patio = model.add_semantic("+PatioDoor")
    model.set_semantic_parent(patio, roof)

    textured_ring = (
        RingDraft()
        .push_vertex_index(v0)
        .push_vertex_index(v3)
        .push_vertex_index(v2)
        .push_vertex_index(v1)
        .add_texture_uvs("winter-textures", texture, roof_uvs)
    )

    surface_0 = SurfaceDraft(textured_ring)
    _add_roof_materials(surface_0.set_semantic(roof), irradiation)

    surface_1 = SurfaceDraft(
        RingDraft()
        .push_vertex_index(v0)
        .push_vertex_index(v3)
        .push_vertex_index(v2)
        .push_vertex_index(v1)
        .add_texture_uvs("winter-textures", texture, roof_uvs)
    )
    _add_roof_materials(surface_1.set_semantic(roof), irradiation)

    surface_2 = SurfaceDraft(
        RingDraft().push_vertex_index(v0).push_vertex_index(v3).push_vertex_index(v2).push_vertex_index(v1)
    )
    _add_roof_materials(surface_2, irradiation)

    surface_3 = SurfaceDraft(
        RingDraft().push_vertex_index(v0).push_vertex_index(v3).push_vertex_index(v2).push_vertex_index(v1)
    )
    surface_3.set_semantic(patio).add_material("red", irradiation)

    surface_4 = SurfaceDraft(
        RingDraft().push_vertex_index(v1).push_vertex_index(v2).push_vertex_index(v3).push_vertex_index(v0)
    )
    surface_4.add_inner_ring(
        RingDraft().push_vertex_index(v1).push_vertex_index(v2).push_vertex_index(v3).push_vertex_index(v0)
    )
    surface_4.add_material("red", irradiation)

    outer_shell = ShellDraft()
    outer_shell.add_surface(surface_0)
    outer_shell.add_surface(surface_1)
    outer_shell.add_surface(surface_2)
    outer_shell.add_surface(surface_3)

    inner_shell = ShellDraft()
    inner_shell.add_surface(surface_4)

    building_geometry = GeometryDraft.solid("2.1")
    building_geometry.add_solid(outer_shell, [inner_shell])
    building_geometry_id = model.add_geometry(building_geometry)

    model.add_template_vertex(Vertex(x=0.0, y=0.5, z=0.0))
    model.add_template_vertex(Vertex(x=1.0, y=1.0, z=0.0))
    model.add_template_vertex(Vertex(x=0.0, y=1.0, z=0.0))
    model.add_template_vertex(Vertex(x=2.1, y=4.2, z=1.2))
    template_id = model.add_geometry_template(make_template_geometry())

    tree_instance = GeometryDraft.instance(
        template_id,
        v1,
        AffineTransform4x4(
            elements=(
                2.0,
                0.0,
                0.0,
                0.0,
                0.0,
                2.0,
                0.0,
                0.0,
                0.0,
                0.0,
                2.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            )
        ),
    )
    tree_geometry_id = model.add_geometry(tree_instance)

    neighbourhood_geometry = GeometryDraft.multi_surface("2")
    neighbourhood_geometry.add_surface(
        SurfaceDraft(
            RingDraft().push_vertex_index(v0).push_vertex_index(v1).push_vertex_index(v2).push_vertex_index(v3)
        )
    )
    neighbourhood_geometry_id = model.add_geometry(neighbourhood_geometry)

    building = CityObjectDraft("id-1", "BuildingPart")
    building.set_geographical_extent(
        BBox(
            min_x=84710.1,
            min_y=446846.0,
            min_z=-5.3,
            max_x=84757.1,
            max_y=446944.0,
            max_z=40.9,
        )
    )
    building.set_attribute("measuredHeight", Value.number(22.3))
    building.set_attribute("roofType", Value.string("gable"))
    building.set_attribute("residential", Value.boolean(True))
    building.set_attribute("nr_doors", Value.integer(3))
    building.set_extra("address", make_building_address(location_geometry))
    building_id = model.add_cityobject(building)
    model.add_cityobject_geometry(building_id, building_geometry_id)

    noise = CityObjectDraft("id-3", "+NoiseBuilding")
    noise.set_attribute("buildingLDenMin", Value.integer(1))
    noise_id = model.add_cityobject(noise)

    tree_id = model.add_cityobject(CityObjectDraft("a-tree", "SolitaryVegetationObject"))
    model.add_cityobject_geometry(tree_id, tree_geometry_id)

    neighbourhood = CityObjectDraft("my-neighbourhood", "CityObjectGroup")
    neighbourhood.set_attribute("location", Value.string("Magyarkanizsa"))
    neighbourhood.set_extra("children_roles", make_children_roles())
    neighbourhood_id = model.add_cityobject(neighbourhood)
    model.add_cityobject_geometry(neighbourhood_id, neighbourhood_geometry_id)

    model.add_cityobject_parent(building_id, noise_id)
    model.add_cityobject_parent(building_id, neighbourhood_id)
    model.add_cityobject_parent(noise_id, neighbourhood_id)

    return model
