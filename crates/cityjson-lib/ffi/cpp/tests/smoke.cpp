#include <array>
#include <cassert>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <iterator>
#include <string_view>
#include <vector>

#include <cityjson_lib/cityjson_lib.hpp>

namespace {

std::vector<std::uint8_t> read_file_bytes(const std::filesystem::path& path) {
  std::ifstream input(path, std::ios::binary);
  assert(input.good());

  return std::vector<std::uint8_t>(
      std::istreambuf_iterator<char>(input), std::istreambuf_iterator<char>());
}

}  // namespace

int main() {
  const auto fixture_path = std::filesystem::path{CITYJSON_LIB_FIXTURE_PATH};
  const auto bytes = read_file_bytes(fixture_path);

  const auto probe = cityjson_lib::Model::probe(bytes);
  assert(probe.root_kind == CJ_ROOT_KIND_CITY_JSON);
  assert(probe.version == CJ_VERSION_V2_0);
  assert(probe.has_version);

  auto model = cityjson_lib::Model::parse_document(bytes);
  const auto summary = model.summary();
  assert(summary.model_type == CJ_MODEL_TYPE_CITY_JSON);
  assert(summary.cityobject_count == 2U);
  assert(summary.geometry_count == 2U);
  assert(summary.vertex_count == 5U);
  assert(summary.uv_coordinate_count == 4U);

  const auto title = model.metadata_title();
  const auto identifier = model.metadata_identifier();
  assert(title == "Facade Fixture");
  assert(identifier == "fixture-1");

  const auto ids = model.cityobject_ids();
  assert(ids.size() == 2U);
  assert(ids[0] == "building-1");
  assert(ids[1] == "building-part-1");

  const auto geometry_types = model.geometry_types();
  assert(geometry_types.size() == 2U);
  assert(geometry_types[0] == CJ_GEOMETRY_TYPE_MULTI_SURFACE);
  assert(geometry_types[1] == CJ_GEOMETRY_TYPE_MULTI_POINT);

  const auto boundary0 = model.geometry_boundary(0U);
  assert(boundary0.geometry_type == CJ_GEOMETRY_TYPE_MULTI_SURFACE);
  assert(boundary0.has_boundaries);
  assert((boundary0.vertex_indices == std::vector<std::size_t>{0U, 1U, 2U, 3U, 0U}));

  const auto boundary0_coordinates = model.geometry_boundary_coordinates(0U);
  assert(boundary0_coordinates.size() == 5U);
  assert(boundary0_coordinates[0].x == 10.0);
  assert(boundary0_coordinates[0].y == 20.0);

  const auto boundary1 = model.geometry_boundary(1U);
  assert(boundary1.geometry_type == CJ_GEOMETRY_TYPE_MULTI_POINT);
  assert(boundary1.has_boundaries);
  assert((boundary1.vertex_indices == std::vector<std::size_t>{4U}));

  const auto boundary1_coordinates = model.geometry_boundary_coordinates(1U);
  assert(boundary1_coordinates.size() == 1U);
  assert(boundary1_coordinates[0].x == 12.0);
  assert(boundary1_coordinates[0].y == 22.0);

  const auto uvs = model.uv_coordinates();
  assert(uvs.size() == 4U);
  assert(uvs[2].u == 1.0F);
  assert(uvs[2].v == 1.0F);

  const auto serialized = model.serialize_document();
  assert(!serialized.empty());
  const auto serialized_bytes = model.serialize_document_bytes();
  assert(!serialized_bytes.empty());

  auto created = cityjson_lib::Model::create(CJ_MODEL_TYPE_CITY_JSON);
  cityjson_lib::ModelCapacities capacities{};
  capacities.cityobjects = 1U;
  capacities.vertices = 4U;
  capacities.geometries = 1U;
  capacities.template_vertices = 1U;
  capacities.uv_coordinates = 4U;
  capacities.semantics = 1U;
  capacities.materials = 1U;
  capacities.textures = 1U;
  created.reserve_import(capacities);

  assert(created.add_vertex(cityjson_lib::Vertex{1.0, 2.0, 3.0}) == 0U);
  assert(created.add_vertex(cityjson_lib::Vertex{4.0, 5.0, 6.0}) == 1U);
  assert(created.add_vertex(cityjson_lib::Vertex{7.0, 8.0, 9.0}) == 2U);
  assert(created.add_vertex(cityjson_lib::Vertex{10.0, 11.0, 12.0}) == 3U);
  assert(created.add_template_vertex(cityjson_lib::Vertex{4.0, 5.0, 6.0}) == 0U);
  assert(created.add_uv_coordinate(cityjson_lib::UV{0.25F, 0.75F}) == 0U);
  assert(created.add_uv_coordinate(cityjson_lib::UV{1.0F, 0.0F}) == 1U);
  assert(created.add_uv_coordinate(cityjson_lib::UV{1.0F, 1.0F}) == 2U);
  assert(created.add_uv_coordinate(cityjson_lib::UV{0.0F, 1.0F}) == 3U);

  created.set_metadata_title("Wrapper Smoke");
  created.set_metadata_identifier("wrapper-smoke");
  created.set_metadata_geographical_extent(cityjson_lib::BBox{
      .min_x = 1.0,
      .min_y = 2.0,
      .min_z = 3.0,
      .max_x = 4.0,
      .max_y = 5.0,
      .max_z = 6.0,
  });
  created.set_metadata_reference_date("2026-01-26");
  created.set_metadata_reference_system("EPSG:7415");
  created.set_metadata_contact(
      cityjson_lib::Contact{}
          .set_name("Smoke Author")
          .set_email("smoke@example.com")
          .set_role(CJ_CONTACT_ROLE_AUTHOR)
          .set_type(CJ_CONTACT_TYPE_ORGANIZATION)
          .set_address(
              cityjson_lib::Value::object().insert("city", cityjson_lib::Value::string("Leiden"))));
  created.set_metadata_extra("note", cityjson_lib::Value::string("typed"));
  created.set_root_extra(
      "+root",
      cityjson_lib::Value::object().insert("answer", cityjson_lib::Value::integer(42)));
  created.add_extension("Noise", "https://example.com/noise.ext.json", "0.5");
  created.set_transform(cityjson_lib::Transform{
      .scale = {2.0, 2.0, 1.0},
      .translate = {1.0, 2.0, 3.0},
  });

  const auto roof = created.add_semantic("RoofSurface");
  created.set_semantic_extra(roof, "surfaceAttribute", cityjson_lib::Value::boolean(true));

  const auto red = created.add_material("red");
  created.set_material_diffuse_color(red, cityjson_lib::Rgb{.r = 0.9F, .g = 0.1F, .b = 0.75F});

  const auto texture =
      created.add_texture("http://www.someurl.org/filename.jpg", CJ_IMAGE_TYPE_PNG);
  created.set_texture_wrap_mode(texture, CJ_WRAP_MODE_WRAP);
  created.set_texture_type(texture, CJ_TEXTURE_TYPE_SPECIFIC);
  created.set_texture_border_color(
      texture, cityjson_lib::Rgba{.r = 1.0F, .g = 1.0F, .b = 1.0F, .a = 1.0F});
  created.set_default_material_theme("red");
  created.set_default_texture_theme("winter-textures");

  auto ring = cityjson_lib::RingDraft{};
  ring.push_vertex_index(0U).push_vertex_index(1U).push_vertex_index(2U).push_vertex_index(3U);
  ring.add_texture(
      "winter-textures",
      texture,
      std::array<cityjson_lib::UV, 4>{
          cityjson_lib::UV{.u = 0.25F, .v = 0.75F},
          cityjson_lib::UV{.u = 1.0F, .v = 0.0F},
          cityjson_lib::UV{.u = 1.0F, .v = 1.0F},
          cityjson_lib::UV{.u = 0.0F, .v = 1.0F},
      });

  auto surface = cityjson_lib::SurfaceDraft(std::move(ring));
  surface.set_semantic(roof).add_material("red", red);

  auto point_geometry = cityjson_lib::GeometryDraft::multi_point("1");
  point_geometry.add_point(0U);
  const auto point_geometry_id = created.add_geometry(std::move(point_geometry));

  auto surface_geometry = cityjson_lib::GeometryDraft::multi_surface("2");
  surface_geometry.add_surface(std::move(surface));
  const auto surface_geometry_id = created.add_geometry(std::move(surface_geometry));

  auto cityobject = cityjson_lib::CityObjectDraft("cityobject-1", "Building");
  cityobject.set_attribute("height", cityjson_lib::Value::number(3.5));
  cityobject.set_extra("geoms", cityjson_lib::Value::array().push(cityjson_lib::Value::geometry(point_geometry_id)));
  const auto cityobject_id = created.add_cityobject(std::move(cityobject));
  created.add_cityobject_geometry(cityobject_id, point_geometry_id);
  created.add_cityobject_geometry(cityobject_id, surface_geometry_id);

  const auto created_summary = created.summary();
  assert(created_summary.cityobject_count == 1U);
  assert(created_summary.geometry_count == 2U);
  assert(created_summary.vertex_count == 4U);
  assert(created_summary.semantic_count == 1U);
  assert(created_summary.material_count == 1U);
  assert(created_summary.texture_count == 1U);

  const auto transformed_document = created.serialize_document(cityjson_lib::WriteOptions{
      .pretty = true,
      .validate_default_themes = false,
  });
  assert(transformed_document.find("Wrapper Smoke") != std::string::npos);
  assert(transformed_document.find("\"+root\"") != std::string::npos);
  assert(transformed_document.find("\"winter-textures\"") != std::string::npos);
  assert(transformed_document.find("\n") != std::string::npos);

  created.clear_transform();
  const auto cleared_document = created.serialize_document();
  assert(transformed_document != cleared_document);

  auto left = cityjson_lib::Model::create(CJ_MODEL_TYPE_CITY_JSON);
  cityjson_lib::ModelCapacities left_capacities{};
  left_capacities.cityobjects = 1U;
  left_capacities.vertices = 1U;
  left_capacities.geometries = 1U;
  left.reserve_import(left_capacities);
  static_cast<void>(left.add_vertex(cityjson_lib::Vertex{0.0, 0.0, 0.0}));
  auto left_geom = cityjson_lib::GeometryDraft::multi_point("1");
  left_geom.add_point(0U);
  const auto left_geometry = left.add_geometry(std::move(left_geom));
  const auto left_cityobject = left.add_cityobject(cityjson_lib::CityObjectDraft("left", "Building"));
  left.add_cityobject_geometry(left_cityobject, left_geometry);

  auto right = cityjson_lib::Model::create(CJ_MODEL_TYPE_CITY_JSON);
  cityjson_lib::ModelCapacities right_capacities{};
  right_capacities.cityobjects = 2U;
  right_capacities.vertices = 1U;
  right_capacities.geometries = 1U;
  right.reserve_import(right_capacities);
  static_cast<void>(right.add_vertex(cityjson_lib::Vertex{1.0, 0.0, 0.0}));
  auto right_geom = cityjson_lib::GeometryDraft::multi_point("1");
  right_geom.add_point(0U);
  const auto right_geometry = right.add_geometry(std::move(right_geom));
  const auto parent = right.add_cityobject(cityjson_lib::CityObjectDraft("parent", "Building"));
  const auto child = right.add_cityobject(cityjson_lib::CityObjectDraft("right", "BuildingPart"));
  right.add_cityobject_parent(child, parent);
  right.add_cityobject_geometry(child, right_geometry);

  left.append_model(right);
  const auto appended_summary = left.summary();
  assert(appended_summary.cityobject_count == 3U);
  assert(appended_summary.geometry_count == 2U);
  assert(appended_summary.vertex_count == 2U);

  try {
    left.append_model(left);
    assert(false && "self-append should throw");
  } catch (const cityjson_lib::StatusError& error) {
    assert(error.status() == CJ_STATUS_INVALID_ARGUMENT);
  }

  const auto extracted = left.extract_cityobjects(std::array{std::string_view{"right"}});
  const auto extracted_summary = extracted.summary();
  assert(extracted_summary.cityobject_count == 1U);
  assert(extracted.cityobject_ids()[0] == "right");

  const auto feature_fixture_bytes =
      read_file_bytes(fixture_path.parent_path() / "minimal.city.jsonl");
  auto feature_model = cityjson_lib::Model::parse_feature(feature_fixture_bytes);
  auto feature_copy = cityjson_lib::Model::parse_feature(feature_fixture_bytes);
  const auto feature_text = feature_model.serialize_feature(cityjson_lib::WriteOptions{
      .pretty = true,
      .validate_default_themes = false,
  });
  assert(feature_text.find("\"type\": \"CityJSONFeature\"") != std::string::npos);
  const auto feature_bytes = feature_model.serialize_feature_bytes();
  assert(!feature_bytes.empty());

  const std::array<const cityjson_lib::Model* const, 2> stream_models{&feature_model, &feature_copy};
  const auto stream = cityjson_lib::Model::serialize_feature_stream(stream_models);
  assert(!stream.empty());
  assert(stream.back() == '\n');

  return 0;
}
