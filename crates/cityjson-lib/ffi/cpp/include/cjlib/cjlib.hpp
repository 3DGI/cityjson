#pragma once

#include <cstdint>
#include <cstring>
#include <span>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

#include <cjlib/cjlib.h>

namespace cjlib {

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
    return "failed to retrieve cjlib last-error message";
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

  [[nodiscard]] std::vector<Vertex> vertices() const {
    cj_vertices_t vertices{};
    check_status(cj_model_copy_vertices(handle_, &vertices));
    return take_vertices(vertices);
  }

  [[nodiscard]] std::vector<Vertex> template_vertices() const {
    cj_vertices_t vertices{};
    check_status(cj_model_copy_template_vertices(handle_, &vertices));
    return take_vertices(vertices);
  }

  [[nodiscard]] std::vector<UV> uv_coordinates() const {
    cj_uvs_t uvs{};
    check_status(cj_model_copy_uv_coordinates(handle_, &uvs));
    return take_uvs(uvs);
  }

  [[nodiscard]] std::string serialize_document() const {
    cj_bytes_t bytes{};
    check_status(cj_model_serialize_document(handle_, &bytes));
    return take_string(bytes);
  }

  [[nodiscard]] std::string serialize_feature() const {
    cj_bytes_t bytes{};
    check_status(cj_model_serialize_feature(handle_, &bytes));
    return take_string(bytes);
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

}  // namespace cjlib
