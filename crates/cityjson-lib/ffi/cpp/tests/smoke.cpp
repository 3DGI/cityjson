#include <cassert>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <iterator>
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

  auto created = cjlib::Model::create(CJ_MODEL_TYPE_CITY_JSON_FEATURE);
  cjlib::ModelCapacities capacities{};
  capacities.vertices = 2U;
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

  return 0;
}
