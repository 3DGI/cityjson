#include <array>
#include <cassert>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <iterator>
#include <string_view>
#include <vector>

#include <cjlib/cjlib.hpp>

namespace {

std::vector<std::uint8_t> read_file_bytes(const std::filesystem::path& path) {
  std::ifstream input(path, std::ios::binary);
  assert(input.good());

  return std::vector<std::uint8_t>(
      std::istreambuf_iterator<char>(input), std::istreambuf_iterator<char>());
}

}  // namespace

int main() {
  const auto fixture_path = std::filesystem::path{CJLIB_FIXTURE_PATH};
  const auto bytes = read_file_bytes(fixture_path);

  const auto probe = cjlib::Model::probe(bytes);
  assert(probe.root_kind == CJ_ROOT_KIND_CITY_JSON);
  assert(probe.version == CJ_VERSION_V2_0);
  assert(probe.has_version);

  auto model = cjlib::Model::parse_document(bytes);
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
  assert((boundary0.ring_offsets == std::vector<std::size_t>{0U}));
  assert((boundary0.surface_offsets == std::vector<std::size_t>{0U}));
  assert(boundary0.shell_offsets.empty());
  assert(boundary0.solid_offsets.empty());

  const auto boundary0_coordinates = model.geometry_boundary_coordinates(0U);
  assert(boundary0_coordinates.size() == 5U);
  assert(boundary0_coordinates[0].x == 10.0);
  assert(boundary0_coordinates[0].y == 20.0);
  assert(boundary0_coordinates[2].x == 11.0);
  assert(boundary0_coordinates[2].y == 21.0);

  const auto boundary1 = model.geometry_boundary(1U);
  assert(boundary1.geometry_type == CJ_GEOMETRY_TYPE_MULTI_POINT);
  assert(boundary1.has_boundaries);
  assert((boundary1.vertex_indices == std::vector<std::size_t>{4U}));
  assert(boundary1.ring_offsets.empty());
  assert(boundary1.surface_offsets.empty());
  assert(boundary1.shell_offsets.empty());
  assert(boundary1.solid_offsets.empty());

  const auto boundary1_coordinates = model.geometry_boundary_coordinates(1U);
  assert(boundary1_coordinates.size() == 1U);
  assert(boundary1_coordinates[0].x == 12.0);
  assert(boundary1_coordinates[0].y == 22.0);

  const auto vertices = model.vertices();
  assert(vertices.size() == 5U);
  assert(vertices[0].x == 10.0);
  assert(vertices[0].y == 20.0);
  assert(vertices[4].x == 12.0);
  assert(vertices[4].y == 22.0);

  const auto uvs = model.uv_coordinates();
  assert(uvs.size() == 4U);
  assert(uvs[2].u == 1.0F);
  assert(uvs[2].v == 1.0F);

  const auto serialized = model.serialize_document();
  assert(!serialized.empty());
  const auto serialized_bytes = model.serialize_document_bytes();
  assert(!serialized_bytes.empty());

  auto created = cjlib::Model::create(CJ_MODEL_TYPE_CITY_JSON_FEATURE);
  cjlib::ModelCapacities capacities{};
  capacities.cityobjects = 2U;
  capacities.vertices = 2U;
  capacities.geometries = 2U;
  capacities.template_vertices = 1U;
  capacities.uv_coordinates = 1U;
  created.reserve_import(capacities);
  assert(created.add_vertex(cjlib::Vertex{1.0, 2.0, 3.0}) == 0U);
  assert(created.add_template_vertex(cjlib::Vertex{4.0, 5.0, 6.0}) == 0U);
  assert(created.add_uv_coordinate(cjlib::UV{0.25F, 0.75F}) == 0U);
  const auto created_summary = created.summary();
  assert(created_summary.model_type == CJ_MODEL_TYPE_CITY_JSON_FEATURE);
  assert(created_summary.vertex_count == 1U);
  assert(created_summary.template_vertex_count == 1U);
  assert(created_summary.uv_coordinate_count == 1U);

  created.set_metadata_title("Wrapper Smoke");
  created.set_metadata_identifier("wrapper-smoke");
  created.set_transform(cjlib::Transform{
      .scale = {2.0, 2.0, 1.0},
      .translate = {1.0, 2.0, 3.0},
  });
  created.add_cityobject("cityobject-1", "Building");
  created.add_cityobject("cityobject-temp", "BuildingPart");
  created.remove_cityobject("cityobject-temp");

  const auto point_boundary = cjlib::GeometryBoundary{
      .geometry_type = CJ_GEOMETRY_TYPE_MULTI_POINT,
      .has_boundaries = true,
      .vertex_indices = {0U},
      .ring_offsets = {},
      .surface_offsets = {},
      .shell_offsets = {},
      .solid_offsets = {},
  };
  const auto point_geometry_index = created.add_geometry_from_boundary(point_boundary);
  created.attach_geometry_to_cityobject("cityobject-1", point_geometry_index);

  const auto transformed_feature = created.serialize_feature(cjlib::WriteOptions{
      .pretty = true,
      .validate_default_themes = false,
  });
  assert(transformed_feature.find("Wrapper Smoke") != std::string::npos);
  assert(transformed_feature.find("\n") != std::string::npos);
  const auto transformed_feature_bytes = created.serialize_feature_bytes();
  assert(!transformed_feature_bytes.empty());

  created.clear_transform();
  const auto cleared_feature = created.serialize_feature();
  assert(transformed_feature != cleared_feature);

  created.cleanup();
  const auto cleaned_summary = created.summary();
  assert(cleaned_summary.cityobject_count == 1U);
  assert(cleaned_summary.geometry_count == 1U);

  auto left = cjlib::Model::create(CJ_MODEL_TYPE_CITY_JSON_FEATURE);
  cjlib::ModelCapacities left_capacities{};
  left_capacities.cityobjects = 1U;
  left_capacities.vertices = 1U;
  left_capacities.geometries = 1U;
  left.reserve_import(left_capacities);
  static_cast<void>(left.add_vertex(cjlib::Vertex{0.0, 0.0, 0.0}));
  left.add_cityobject("left", "Building");
  const auto left_geometry = left.add_geometry_from_boundary(point_boundary);
  left.attach_geometry_to_cityobject("left", left_geometry);

  auto right = cjlib::Model::create(CJ_MODEL_TYPE_CITY_JSON_FEATURE);
  cjlib::ModelCapacities right_capacities{};
  right_capacities.cityobjects = 1U;
  right_capacities.vertices = 1U;
  right_capacities.geometries = 1U;
  right.reserve_import(right_capacities);
  static_cast<void>(right.add_vertex(cjlib::Vertex{1.0, 0.0, 0.0}));
  right.add_cityobject("right", "BuildingPart");
  const auto right_geometry = right.add_geometry_from_boundary(point_boundary);
  right.attach_geometry_to_cityobject("right", right_geometry);

  left.append_model(right);
  const auto appended_summary = left.summary();
  assert(appended_summary.cityobject_count == 2U);
  assert(appended_summary.geometry_count == 2U);
  assert(appended_summary.vertex_count == 2U);

  const auto extracted = left.extract_cityobjects(std::array{std::string_view{"right"}});
  const auto extracted_summary = extracted.summary();
  assert(extracted_summary.cityobject_count == 1U);
  assert(extracted.cityobject_ids()[0] == "right");

  const std::array<const cjlib::Model* const, 2> stream_models{&created, &right};
  const auto stream = cjlib::Model::serialize_feature_stream(stream_models);
  assert(!stream.empty());
  assert(stream.back() == '\n');

  const auto document_json = created.serialize_document();
  assert(!document_json.empty());
  const auto document_bytes = created.serialize_document_bytes();
  assert(!document_bytes.empty());

  return 0;
}
