#pragma once

#include <cstddef>
#include <cstdint>
#include <array>
#include <cstring>
#include <span>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

#include <cityjson_lib/cityjson_lib.h>

namespace cityjson_lib {

using Status = cj_status_t;
using ErrorKind = cj_error_kind_t;
using RootKind = cj_root_kind_t;
using Version = cj_version_t;
using ModelType = cj_model_type_t;
using GeometryType = cj_geometry_type_t;
using Probe = cj_probe_t;
using ModelSummary = cj_model_summary_t;
using ModelCapacities = cj_model_capacities_t;
using Vertex = cj_vertex_t;
using UV = cj_uv_t;

struct WriteOptions final {
  bool pretty = false;
  bool validate_default_themes = true;
};

struct Transform final {
  std::array<double, 3> scale{1.0, 1.0, 1.0};
  std::array<double, 3> translate{0.0, 0.0, 0.0};
};

struct GeometryBoundary final {
  GeometryType geometry_type;
  bool has_boundaries;
  std::vector<std::size_t> vertex_indices;
  std::vector<std::size_t> ring_offsets;
  std::vector<std::size_t> surface_offsets;
  std::vector<std::size_t> shell_offsets;
  std::vector<std::size_t> solid_offsets;
};

class StatusError final : public std::runtime_error {
 public:
  StatusError(Status status, ErrorKind kind, std::string message)
      : std::runtime_error(std::move(message)), status_(status), kind_(kind) {}

  [[nodiscard]] Status status() const noexcept { return status_; }
  [[nodiscard]] ErrorKind kind() const noexcept { return kind_; }

 private:
  Status status_;
  ErrorKind kind_;
};

inline std::string last_error_message() {
  const std::size_t len = cj_last_error_message_len();
  if (len == 0U) {
    return {};
  }

  std::vector<std::uint8_t> buffer(len + 1U, 0U);
  std::size_t copied = 0U;
  const auto status = cj_last_error_message_copy(buffer.data(), buffer.size(), &copied);
  if (status != CJ_STATUS_SUCCESS) {
    return "failed to retrieve cityjson_lib last-error message";
  }

  return std::string(reinterpret_cast<const char*>(buffer.data()), copied);
}

[[noreturn]] inline void throw_last_error(Status status) {
  throw StatusError(status, cj_last_error_kind(), last_error_message());
}

inline void check_status(Status status) {
  if (status != CJ_STATUS_SUCCESS) {
    throw_last_error(status);
  }
}

inline const std::uint8_t* span_data(std::span<const std::uint8_t> bytes) noexcept {
  return bytes.empty() ? nullptr : bytes.data();
}

inline cj_string_view_t to_view(std::string_view value) noexcept {
  return {
      .data = reinterpret_cast<const std::uint8_t*>(value.data()),
      .len = value.size(),
  };
}

inline cj_json_write_options_t to_native(const WriteOptions& options) noexcept {
  return {
      .pretty = options.pretty,
      .validate_default_themes = options.validate_default_themes,
  };
}

inline cj_transform_t to_native(const Transform& transform) noexcept {
  return {
      .scale_x = transform.scale[0],
      .scale_y = transform.scale[1],
      .scale_z = transform.scale[2],
      .translate_x = transform.translate[0],
      .translate_y = transform.translate[1],
      .translate_z = transform.translate[2],
  };
}

inline std::string take_string(cj_bytes_t bytes) {
  std::string value;
  if (bytes.len > 0U) {
    value.assign(reinterpret_cast<const char*>(bytes.data), bytes.len);
  }
  check_status(cj_bytes_free(bytes));
  return value;
}

inline std::vector<std::uint8_t> take_bytes(cj_bytes_t bytes) {
  std::vector<std::uint8_t> value;
  if (bytes.len > 0U) {
    value.assign(bytes.data, bytes.data + bytes.len);
  }
  check_status(cj_bytes_free(bytes));
  return value;
}

inline std::vector<Vertex> take_vertices(cj_vertices_t vertices) {
  std::vector<Vertex> value;
  if (vertices.len > 0U) {
    value.assign(vertices.data, vertices.data + vertices.len);
  }
  check_status(cj_vertices_free(vertices));
  return value;
}

inline std::vector<UV> take_uvs(cj_uvs_t uvs) {
  std::vector<UV> value;
  if (uvs.len > 0U) {
    value.assign(uvs.data, uvs.data + uvs.len);
  }
  check_status(cj_uvs_free(uvs));
  return value;
}

inline std::vector<std::size_t> copy_indices(cj_indices_t indices) {
  std::vector<std::size_t> value;
  if (indices.len > 0U) {
    value.assign(indices.data, indices.data + indices.len);
  }
  return value;
}

inline std::string copy_string(cj_bytes_t bytes) {
  std::string value;
  if (bytes.len > 0U) {
    value.assign(reinterpret_cast<const char*>(bytes.data), bytes.len);
  }
  return value;
}

inline GeometryBoundary take_geometry_boundary(cj_geometry_boundary_t boundary) {
  struct FreeGuard {
    cj_geometry_boundary_t boundary;
    ~FreeGuard() { static_cast<void>(cj_geometry_boundary_free(boundary)); }
  } guard{boundary};

  return GeometryBoundary{
      .geometry_type = boundary.geometry_type,
      .has_boundaries = boundary.has_boundaries,
      .vertex_indices = copy_indices(boundary.vertex_indices),
      .ring_offsets = copy_indices(boundary.ring_offsets),
      .surface_offsets = copy_indices(boundary.surface_offsets),
      .shell_offsets = copy_indices(boundary.shell_offsets),
      .solid_offsets = copy_indices(boundary.solid_offsets),
  };
}

class Model final {
 public:
  Model() = default;

  explicit Model(cj_model_t* handle) : handle_(handle) {}

  Model(const Model&) = delete;
  Model& operator=(const Model&) = delete;

  Model(Model&& other) noexcept : handle_(std::exchange(other.handle_, nullptr)) {}

  Model& operator=(Model&& other) noexcept {
    if (this != &other) {
      reset();
      handle_ = std::exchange(other.handle_, nullptr);
    }
    return *this;
  }

  ~Model() { reset(); }

  [[nodiscard]] static Probe probe(std::span<const std::uint8_t> bytes) {
    Probe probe{};
    check_status(cj_probe_bytes(span_data(bytes), bytes.size(), &probe));
    return probe;
  }

  [[nodiscard]] static Model parse_document(std::span<const std::uint8_t> bytes) {
    cj_model_t* handle = nullptr;
    check_status(cj_model_parse_document_bytes(span_data(bytes), bytes.size(), &handle));
    return Model(handle);
  }

  [[nodiscard]] static Model parse_feature(std::span<const std::uint8_t> bytes) {
    cj_model_t* handle = nullptr;
    check_status(cj_model_parse_feature_bytes(span_data(bytes), bytes.size(), &handle));
    return Model(handle);
  }

  [[nodiscard]] static Model parse_feature_with_base(
      std::span<const std::uint8_t> feature_bytes,
      std::span<const std::uint8_t> base_bytes) {
    cj_model_t* handle = nullptr;
    check_status(cj_model_parse_feature_with_base_bytes(
        span_data(feature_bytes), feature_bytes.size(), span_data(base_bytes), base_bytes.size(),
        &handle));
    return Model(handle);
  }

  [[nodiscard]] static Model create(ModelType type) {
    cj_model_t* handle = nullptr;
    check_status(cj_model_create(type, &handle));
    return Model(handle);
  }

  [[nodiscard]] bool valid() const noexcept { return handle_ != nullptr; }

  void reset() noexcept {
    if (handle_ != nullptr) {
      static_cast<void>(cj_model_free(handle_));
      handle_ = nullptr;
    }
  }

  [[nodiscard]] ModelSummary summary() const {
    ModelSummary summary{};
    check_status(cj_model_get_summary(handle_, &summary));
    return summary;
  }

  [[nodiscard]] std::string metadata_title() const {
    cj_bytes_t bytes{};
    check_status(cj_model_get_metadata_title(handle_, &bytes));
    return take_string(bytes);
  }

  [[nodiscard]] std::string metadata_identifier() const {
    cj_bytes_t bytes{};
    check_status(cj_model_get_metadata_identifier(handle_, &bytes));
    return take_string(bytes);
  }

  void set_metadata_title(std::string_view title) {
    check_status(cj_model_set_metadata_title(handle_, to_view(title)));
  }

  void set_metadata_identifier(std::string_view identifier) {
    check_status(cj_model_set_metadata_identifier(handle_, to_view(identifier)));
  }

  void set_transform(const Transform& transform) {
    check_status(cj_model_set_transform(handle_, to_native(transform)));
  }

  void clear_transform() {
    check_status(cj_model_clear_transform(handle_));
  }

  [[nodiscard]] std::vector<std::string> cityobject_ids() const {
    const auto model_summary = summary();
    std::vector<std::string> ids;
    ids.reserve(model_summary.cityobject_count);

    for (std::size_t index = 0; index < model_summary.cityobject_count; ++index) {
      cj_bytes_t bytes{};
      check_status(cj_model_get_cityobject_id(handle_, index, &bytes));
      ids.push_back(take_string(bytes));
    }

    return ids;
  }

  void add_cityobject(std::string_view id, std::string_view cityobject_type) {
    check_status(cj_model_add_cityobject(handle_, to_view(id), to_view(cityobject_type)));
  }

  void remove_cityobject(std::string_view id) {
    check_status(cj_model_remove_cityobject(handle_, to_view(id)));
  }

  void attach_geometry_to_cityobject(std::string_view cityobject_id, std::size_t geometry_index) {
    check_status(cj_model_attach_geometry_to_cityobject(
        handle_, to_view(cityobject_id), geometry_index));
  }

  void clear_cityobject_geometry(std::string_view cityobject_id) {
    check_status(cj_model_clear_cityobject_geometry(handle_, to_view(cityobject_id)));
  }

  [[nodiscard]] std::vector<GeometryType> geometry_types() const {
    const auto model_summary = summary();
    std::vector<GeometryType> types;
    types.reserve(model_summary.geometry_count);

    for (std::size_t index = 0; index < model_summary.geometry_count; ++index) {
      GeometryType type{};
      check_status(cj_model_get_geometry_type(handle_, index, &type));
      types.push_back(type);
    }

    return types;
  }

  [[nodiscard]] std::vector<UV> uv_coordinates() const {
    cj_uvs_t uvs{};
    check_status(cj_model_copy_uv_coordinates(handle_, &uvs));
    return take_uvs(uvs);
  }

  [[nodiscard]] GeometryBoundary geometry_boundary(std::size_t index) const {
    cj_geometry_boundary_t boundary{};
    check_status(cj_model_copy_geometry_boundary(handle_, index, &boundary));
    return take_geometry_boundary(boundary);
  }

  [[nodiscard]] std::vector<Vertex> geometry_boundary_coordinates(std::size_t index) const {
    cj_vertices_t vertices{};
    check_status(cj_model_copy_geometry_boundary_coordinates(handle_, index, &vertices));
    return take_vertices(vertices);
  }

  [[nodiscard]] std::size_t add_geometry_from_boundary(
      const GeometryBoundary& boundary,
      std::string_view lod = {}) {
    const auto build_view = [](const std::vector<std::size_t>& values) {
      return cj_indices_view_t{
          .data = values.empty() ? nullptr : values.data(),
          .len = values.size(),
      };
    };

    cj_geometry_boundary_view_t view{
        .geometry_type = boundary.geometry_type,
        .vertex_indices = build_view(boundary.vertex_indices),
        .ring_offsets = build_view(boundary.ring_offsets),
        .surface_offsets = build_view(boundary.surface_offsets),
        .shell_offsets = build_view(boundary.shell_offsets),
        .solid_offsets = build_view(boundary.solid_offsets),
    };

    std::size_t index = 0U;
    check_status(cj_model_add_geometry_from_boundary(
        handle_, view, to_view(lod), &index));
    return index;
  }

  [[nodiscard]] std::vector<std::uint8_t> serialize_document_bytes(
      const WriteOptions& options = {}) const {
    cj_bytes_t bytes{};
    check_status(cj_model_serialize_document_with_options(handle_, to_native(options), &bytes));
    return take_bytes(bytes);
  }

  [[nodiscard]] std::string serialize_document(const WriteOptions& options = {}) const {
    const auto bytes = serialize_document_bytes(options);
    return std::string(bytes.begin(), bytes.end());
  }

  [[nodiscard]] std::vector<std::uint8_t> serialize_feature_bytes(
      const WriteOptions& options = {}) const {
    cj_bytes_t bytes{};
    check_status(cj_model_serialize_feature_with_options(handle_, to_native(options), &bytes));
    return take_bytes(bytes);
  }

  [[nodiscard]] std::string serialize_feature(const WriteOptions& options = {}) const {
    const auto bytes = serialize_feature_bytes(options);
    return std::string(bytes.begin(), bytes.end());
  }

  [[nodiscard]] static std::vector<std::uint8_t> serialize_feature_stream(
      std::span<const Model* const> models,
      const WriteOptions& options = {}) {
    std::vector<const cj_model_t*> handles;
    handles.reserve(models.size());
    for (const Model* model : models) {
      handles.push_back(model->raw_handle());
    }

    cj_bytes_t bytes{};
    check_status(cj_model_serialize_feature_stream(
        handles.empty() ? nullptr : handles.data(), handles.size(), to_native(options), &bytes));
    return take_bytes(bytes);
  }

  [[nodiscard]] static Model merge_feature_stream(std::span<const std::uint8_t> bytes) {
    cj_model_t* handle = nullptr;
    check_status(cj_model_parse_feature_stream_merge_bytes(
        span_data(bytes), bytes.size(), &handle));
    return Model(handle);
  }

  [[nodiscard]] Model extract_cityobjects(std::span<const std::string_view> ids) const {
    std::vector<cj_string_view_t> views;
    views.reserve(ids.size());
    for (const std::string_view id : ids) {
      views.push_back(to_view(id));
    }

    cj_model_t* handle = nullptr;
    check_status(cj_model_extract_cityobjects(
        handle_, views.empty() ? nullptr : views.data(), views.size(), &handle));
    return Model(handle);
  }

  void append_model(const Model& source) {
    check_status(cj_model_append_model(handle_, source.raw_handle()));
  }

  void cleanup() {
    check_status(cj_model_cleanup(handle_));
  }

  void reserve_import(const ModelCapacities& capacities) {
    check_status(cj_model_reserve_import(handle_, capacities));
  }

  [[nodiscard]] std::size_t add_vertex(const Vertex& vertex) {
    std::size_t index = 0U;
    check_status(cj_model_add_vertex(handle_, vertex, &index));
    return index;
  }

  [[nodiscard]] std::size_t add_template_vertex(const Vertex& vertex) {
    std::size_t index = 0U;
    check_status(cj_model_add_template_vertex(handle_, vertex, &index));
    return index;
  }

  [[nodiscard]] std::size_t add_uv_coordinate(const UV& uv) {
    std::size_t index = 0U;
    check_status(cj_model_add_uv_coordinate(handle_, uv, &index));
    return index;
  }

  [[nodiscard]] cj_model_t* raw_handle() const noexcept { return handle_; }

 private:
  cj_model_t* handle_ = nullptr;
};

}  // namespace cityjson_lib
