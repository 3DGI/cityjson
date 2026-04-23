#include <array>
#include <cstdint>
#include <iostream>
#include <utility>
#include <vector>

#include <cityjson_lib/cityjson_lib.hpp>

namespace {

using cityjson_lib::BBox;
using cityjson_lib::GeometryDraft;
using cityjson_lib::Model;
using cityjson_lib::RingDraft;
using cityjson_lib::ShellDraft;
using cityjson_lib::SurfaceDraft;
using cityjson_lib::Value;
using cityjson_lib::Vertex;

Value make_contact_address() {
  return Value::object()
      .insert("city", Value::string("Den Haag"))
      .insert("country", Value::string("The Netherlands"));
}

Value make_census() {
  return Value::object()
      .insert("percent_men", Value::number(49.5))
      .insert("percent_women", Value::number(51.5));
}

Value make_building_address(cityjson_lib::GeometryId location_geometry) {
  auto entry = Value::object();
  entry.insert("Country", Value::string("Canada"));
  entry.insert("Locality", Value::string("Chibougamau"));
  entry.insert("ThoroughfareNumber", Value::string("1"));
  entry.insert("ThoroughfareName", Value::string("rue de la Patate"));
  entry.insert("Postcode", Value::string("H0H 0H0"));
  entry.insert("location", Value::geometry(location_geometry));

  auto addresses = Value::array();
  addresses.push(std::move(entry));
  return addresses;
}

Value make_children_roles() {
  return Value::array()
      .push(Value::string("residential building"))
      .push(Value::string("voting location"));
}

GeometryDraft make_template_geometry() {
  auto surface_0 = SurfaceDraft(
      RingDraft{}.push_vertex_index(0U).push_vertex_index(3U).push_vertex_index(2U).push_vertex_index(1U));
  auto surface_1 = SurfaceDraft(
      RingDraft{}.push_vertex_index(1U).push_vertex_index(2U).push_vertex_index(0U).push_vertex_index(3U));
  auto surface_2 = SurfaceDraft(
      RingDraft{}.push_vertex_index(0U).push_vertex_index(1U).push_vertex_index(3U).push_vertex_index(2U));

  auto draft = GeometryDraft::multi_surface("2.1");
  draft.add_surface(std::move(surface_0));
  draft.add_surface(std::move(surface_1));
  draft.add_surface(std::move(surface_2));
  return draft;
}

Model build_fake_complete_model() {
  auto model = Model::create(CJ_MODEL_TYPE_CITY_JSON);

  model.set_metadata_geographical_extent(BBox{
      .min_x = 84710.1,
      .min_y = 446846.0,
      .min_z = -5.3,
      .max_x = 84757.1,
      .max_y = 446944.0,
      .max_z = 40.9,
  });
  model.set_metadata_identifier("eaeceeaa-3f66-429a-b81d-bbc6140b8c1c");
  model.set_metadata_reference_date("2026-01-26");
  model.set_metadata_reference_system("https://www.opengis.net/def/crs/EPSG/0/2355");
  model.set_metadata_title("Complete and fake CityJSON");
  model.set_metadata_contact(
      cityjson_lib::Contact{}
          .set_name("Kitalált Név")
          .set_email("spam@3dgi.nl")
          .set_role(CJ_CONTACT_ROLE_AUTHOR)
          .set_website("https://3dgi.nl")
          .set_type(CJ_CONTACT_TYPE_ORGANIZATION)
          .set_phone("+36612345678")
          .set_organization("3DGI")
          .set_address(make_contact_address()));
  model.set_metadata_extra(
      "nospec_description",
      Value::string("A CityJSON object with all existing properties set to a fake value"));

  model.set_root_extra("+census", make_census());
  model.add_extension("Noise", "https://someurl.org/noise.ext.json", "0.5");
  model.set_transform(cityjson_lib::Transform{
      .scale = {1.0, 1.0, 1.0},
      .translate = {0.0, 0.0, 0.0},
  });

  const auto irradiation = model.add_material("irradiation");
  model.set_material_ambient_intensity(irradiation, 0.2F);
  model.set_material_diffuse_color(irradiation, cityjson_lib::Rgb{.r = 0.9F, .g = 0.1F, .b = 0.75F});
  model.set_material_emissive_color(irradiation, cityjson_lib::Rgb{.r = 0.9F, .g = 0.1F, .b = 0.75F});
  model.set_material_specular_color(irradiation, cityjson_lib::Rgb{.r = 0.9F, .g = 0.1F, .b = 0.75F});
  model.set_material_shininess(irradiation, 0.2F);
  model.set_material_transparency(irradiation, 0.5F);
  model.set_material_is_smooth(irradiation, false);
  const auto red = model.add_material("red");
  const auto texture =
      model.add_texture("http://www.someurl.org/filename.jpg", CJ_IMAGE_TYPE_PNG);
  model.set_texture_wrap_mode(texture, CJ_WRAP_MODE_WRAP);
  model.set_texture_type(texture, CJ_TEXTURE_TYPE_SPECIFIC);
  model.set_texture_border_color(
      texture, cityjson_lib::Rgba{.r = 1.0F, .g = 1.0F, .b = 1.0F, .a = 1.0F});
  model.set_default_material_theme("irradiation");
  model.set_default_texture_theme("summer-textures");

  const std::array<cityjson_lib::UV, 4> roof_uvs{{
      {.u = 0.0F, .v = 0.5F},
      {.u = 1.0F, .v = 0.0F},
      {.u = 1.0F, .v = 1.0F},
      {.u = 0.0F, .v = 1.0F},
  }};

  const auto v0 = model.add_vertex(Vertex{.x = 102.0, .y = 103.0, .z = 1.0});
  const auto v1 = model.add_vertex(Vertex{.x = 11.0, .y = 910.0, .z = 43.0});
  const auto v2 = model.add_vertex(Vertex{.x = 25.0, .y = 744.0, .z = 22.0});
  const auto v3 = model.add_vertex(Vertex{.x = 23.0, .y = 88.0, .z = 5.0});

  auto location = GeometryDraft::multi_point("1");
  location.add_point(v0);
  const auto location_geometry = model.add_geometry(std::move(location));

  const auto roof = model.add_semantic("RoofSurface");
  model.set_semantic_extra(roof, "surfaceAttribute", Value::boolean(true));
  const auto patio = model.add_semantic("+PatioDoor");
  model.set_semantic_parent(patio, roof);

  auto textured_ring =
      RingDraft{}
          .push_vertex_index(v0)
          .push_vertex_index(v3)
          .push_vertex_index(v2)
          .push_vertex_index(v1)
          .add_texture("winter-textures", texture, roof_uvs);

  auto surface_0 = SurfaceDraft(std::move(textured_ring));
  surface_0.set_semantic(roof).add_material("irradiation", irradiation).add_material("red", irradiation);

  auto surface_1 = SurfaceDraft(
      RingDraft{}
          .push_vertex_index(v0)
          .push_vertex_index(v3)
          .push_vertex_index(v2)
          .push_vertex_index(v1)
          .add_texture("winter-textures", texture, roof_uvs));
  surface_1.set_semantic(roof).add_material("irradiation", irradiation).add_material("red", irradiation);

  auto surface_2 = SurfaceDraft(
      RingDraft{}.push_vertex_index(v0).push_vertex_index(v3).push_vertex_index(v2).push_vertex_index(v1));
  surface_2.add_material("irradiation", irradiation).add_material("red", irradiation);

  auto surface_3 = SurfaceDraft(
      RingDraft{}.push_vertex_index(v0).push_vertex_index(v3).push_vertex_index(v2).push_vertex_index(v1));
  surface_3.set_semantic(patio).add_material("red", irradiation);

  auto surface_4 = SurfaceDraft(
      RingDraft{}.push_vertex_index(v1).push_vertex_index(v2).push_vertex_index(v3).push_vertex_index(v0));
  surface_4.add_inner_ring(
      RingDraft{}.push_vertex_index(v1).push_vertex_index(v2).push_vertex_index(v3).push_vertex_index(v0));
  surface_4.add_material("red", irradiation);

  auto outer_shell = ShellDraft{};
  outer_shell.add_surface(std::move(surface_0))
      .add_surface(std::move(surface_1))
      .add_surface(std::move(surface_2))
      .add_surface(std::move(surface_3));
  auto inner_shell = ShellDraft{};
  inner_shell.add_surface(std::move(surface_4));

  auto building_geometry = GeometryDraft::solid("2.1");
  std::vector<ShellDraft> inner_shells;
  inner_shells.push_back(std::move(inner_shell));
  building_geometry.add_solid(std::move(outer_shell), std::move(inner_shells));
  const auto building_geometry_id = model.add_geometry(std::move(building_geometry));

  static_cast<void>(model.add_template_vertex(Vertex{.x = 0.0, .y = 0.5, .z = 0.0}));
  static_cast<void>(model.add_template_vertex(Vertex{.x = 1.0, .y = 1.0, .z = 0.0}));
  static_cast<void>(model.add_template_vertex(Vertex{.x = 0.0, .y = 1.0, .z = 0.0}));
  static_cast<void>(model.add_template_vertex(Vertex{.x = 2.1, .y = 4.2, .z = 1.2}));
  const auto template_id = model.add_geometry_template(make_template_geometry());

  auto tree_instance = GeometryDraft::instance(
      template_id,
      v1,
      cityjson_lib::AffineTransform4x4{
          .elements = {2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0,
                       0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0},
      });
  const auto tree_geometry_id = model.add_geometry(std::move(tree_instance));

  auto neighbourhood_geometry = GeometryDraft::multi_surface("2");
  neighbourhood_geometry.add_surface(
      SurfaceDraft(RingDraft{}
                       .push_vertex_index(v0)
                       .push_vertex_index(v1)
                       .push_vertex_index(v2)
                       .push_vertex_index(v3)));
  const auto neighbourhood_geometry_id = model.add_geometry(std::move(neighbourhood_geometry));

  auto building = cityjson_lib::CityObjectDraft("id-1", "BuildingPart");
  building.set_geographical_extent(BBox{
      .min_x = 84710.1,
      .min_y = 446846.0,
      .min_z = -5.3,
      .max_x = 84757.1,
      .max_y = 446944.0,
      .max_z = 40.9,
  });
  building.set_attribute("measuredHeight", Value::number(22.3));
  building.set_attribute("roofType", Value::string("gable"));
  building.set_attribute("residential", Value::boolean(true));
  building.set_attribute("nr_doors", Value::integer(3));
  building.set_extra("address", make_building_address(location_geometry));
  const auto building_id = model.add_cityobject(std::move(building));
  model.add_cityobject_geometry(building_id, building_geometry_id);

  auto noise = cityjson_lib::CityObjectDraft("id-3", "+NoiseBuilding");
  noise.set_attribute("buildingLDenMin", Value::integer(1));
  const auto noise_id = model.add_cityobject(std::move(noise));

  const auto tree_id =
      model.add_cityobject(cityjson_lib::CityObjectDraft("a-tree", "SolitaryVegetationObject"));
  model.add_cityobject_geometry(tree_id, tree_geometry_id);

  auto neighbourhood = cityjson_lib::CityObjectDraft("my-neighbourhood", "CityObjectGroup");
  neighbourhood.set_attribute("location", Value::string("Magyarkanizsa"));
  neighbourhood.set_extra("children_roles", make_children_roles());
  const auto neighbourhood_id = model.add_cityobject(std::move(neighbourhood));
  model.add_cityobject_geometry(neighbourhood_id, neighbourhood_geometry_id);

  model.add_cityobject_parent(building_id, noise_id);
  model.add_cityobject_parent(building_id, neighbourhood_id);
  model.add_cityobject_parent(noise_id, neighbourhood_id);

  return model;
}

}  // namespace

int main() {
  auto model = build_fake_complete_model();
  std::cout << model.serialize_document(cityjson_lib::WriteOptions{
      .pretty = true,
      .validate_default_themes = false,
  });
  return 0;
}
