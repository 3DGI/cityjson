# cityjson-index-ffi-core

Shared C ABI for `cityjson-index`.

The crate is the release-facing native core used by the Python package under
`ffi/python/`.

## Filtered feature reads

`cjx_index_read_filtered_features` reconstructs indexed feature refs and
applies a typed `cjx_feature_filter_t`:

- `has_cityobject_types=false` ignores `cityobject_types`; otherwise the
  `cjx_string_list_t` names the retained CityObject types.
- `default_lod` is a `cjx_lod_selection_t` with kind `ALL`, `HIGHEST`, or
  `EXACT`. Exact selections read `exact_lod` as UTF-8 bytes.
- `lods_by_type` is an array of `cjx_lod_by_type_t` entries overriding the
  default LoD selection for a CityObject type.

The function returns an owned `cjx_filtered_feature_t` array. Each item contains
serialized CityJSON feature bytes in `model_json` plus typed diagnostics:
available, retained, and ignored type lists; available and retained LoD maps;
missing exact LoD selections; and `retained_geometry_count`.

Call `cjx_filtered_features_free(features, count)` exactly once for a returned
non-null array. It recursively frees `model_json` and all nested diagnostics
lists/maps. Input filter strings and input refs remain caller-owned and are not
freed by the ABI.
