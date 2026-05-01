# cityjson-lib-ffi-core

`cityjson-lib-ffi-core` is the shared low-level C ABI for the release-facing
bindings in this repository.

It intentionally stays small and explicit:

- opaque model handles with owned transfer helpers
- stable status and error categories
- parse, serialize, summary, and workflow entry points
- opaque model-selection handles for CityObject and geometry extraction workflows
- generated C headers under `include/cityjson_lib/cityjson_lib.h`

The C++ and Python bindings build on this crate. The wasm adapter uses the same
substrate internally, but it is not part of the public release surface yet.

## Selection and merge ABI

The C core exposes the same selection carrier used by the Rust operations API
as an opaque `cj_model_selection_t*`. Callers create selections from a model,
combine or expand them as needed, then pass them back to
`cj_model_extract_selection(...)` to materialize a new `cj_model_t*`.

```c
cj_string_view_t ids[] = {
    {.data = (const uint8_t *)"building-part-1", .len = 15},
};

cj_model_selection_t *selection = NULL;
cj_status_t status = cj_model_select_cityobjects_by_id(
    model,
    ids,
    1,
    &selection);

cj_model_selection_t *with_relatives = NULL;
status = cj_model_selection_include_relatives(selection, model, &with_relatives);

cj_model_t *extracted = NULL;
status = cj_model_extract_selection(model, with_relatives, &extracted);

cj_model_free(extracted);
cj_model_selection_free(with_relatives);
cj_model_selection_free(selection);
```

Geometry-level selection uses `cj_geometry_selection_spec_t`, pairing a
CityObject id with the geometry index inside that CityObject. Selection handles
can be combined with `cj_model_selection_union(...)` and
`cj_model_selection_intersection(...)`; `cj_model_selection_is_empty(...)`
reports whether a selection has no retained CityObjects or geometries.

Multiple complete models can be merged through `cj_model_merge_models(...)`.
The array must contain at least one non-null `cj_model_t*`; the returned model
is owned by the caller and must be released with `cj_model_free(...)`.

This crate is dual-licensed under MIT or Apache-2.0. See the package metadata
and the license files in this repository.
