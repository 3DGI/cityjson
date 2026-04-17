# Examples

The current example set mirrors the publishable core surface.

- `examples/json_document.rs`
  Load a CityJSON document from a file or byte slice.
- `examples/explicit_json_module.rs`
  Probe the input and then parse it through `cityjson_lib::json`.
- `examples/json_feature_stream.rs`
  Read a CityJSONSeq stream through the explicit feature-stream API.
- `examples/json_roundtrip.rs`
  Serialize a model back to document and feature forms.
- `examples/model_operations.rs`
  Use `ops::merge` on feature-sized models.

`examples/alternate_formats.rs` is an archived transport sketch and is not part
of the current published API story.
