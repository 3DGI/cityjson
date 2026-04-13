#pragma once

#include <cstddef>
#include <cstdint>

#include <cityjson_lib/cityjson_lib.h>

struct cj_string_view_t {
  const std::uint8_t* data;
  std::size_t len;
};

struct cj_indices_view_t {
  const std::size_t* data;
  std::size_t len;
};

struct cj_json_write_options_t {
  bool pretty;
  bool validate_default_themes;
};

struct cj_transform_t {
  double scale_x;
  double scale_y;
  double scale_z;
  double translate_x;
  double translate_y;
  double translate_z;
};

struct cj_geometry_boundary_view_t {
  cj_geometry_type_t geometry_type;
  cj_indices_view_t vertex_indices;
  cj_indices_view_t ring_offsets;
  cj_indices_view_t surface_offsets;
  cj_indices_view_t shell_offsets;
  cj_indices_view_t solid_offsets;
};

extern "C" {

cj_status_t cj_model_set_metadata_title(cj_model_t* model, cj_string_view_t title);
cj_status_t cj_model_set_metadata_identifier(cj_model_t* model, cj_string_view_t identifier);
cj_status_t cj_model_set_transform(cj_model_t* model, cj_transform_t transform);
cj_status_t cj_model_clear_transform(cj_model_t* model);
cj_status_t cj_model_add_cityobject(cj_model_t* model, cj_string_view_t id,
                                    cj_string_view_t cityobject_type);
cj_status_t cj_model_remove_cityobject(cj_model_t* model, cj_string_view_t id);
cj_status_t cj_model_attach_geometry_to_cityobject(cj_model_t* model,
                                                   cj_string_view_t cityobject_id,
                                                   std::size_t geometry_index);
cj_status_t cj_model_clear_cityobject_geometry(cj_model_t* model,
                                               cj_string_view_t cityobject_id);
cj_status_t cj_model_add_geometry_from_boundary(cj_model_t* model,
                                                cj_geometry_boundary_view_t boundary,
                                                cj_string_view_t lod,
                                                std::size_t* out_index);
cj_status_t cj_model_cleanup(cj_model_t* model);
cj_status_t cj_model_append_model(cj_model_t* target_model, const cj_model_t* source_model);
cj_status_t cj_model_extract_cityobjects(const cj_model_t* model,
                                         const cj_string_view_t* cityobject_ids,
                                         std::size_t cityobject_count,
                                         cj_model_t** out_model);
cj_status_t cj_model_serialize_document_with_options(const cj_model_t* model,
                                                     cj_json_write_options_t options,
                                                     cj_bytes_t* out_bytes);
cj_status_t cj_model_serialize_feature_with_options(const cj_model_t* model,
                                                    cj_json_write_options_t options,
                                                    cj_bytes_t* out_bytes);
cj_status_t cj_model_parse_feature_stream_merge_bytes(const std::uint8_t* data,
                                                      std::size_t len,
                                                      cj_model_t** out_model);
cj_status_t cj_model_serialize_feature_stream(const cj_model_t** models,
                                              std::size_t model_count,
                                              cj_json_write_options_t options,
                                              cj_bytes_t* out_bytes);

}
