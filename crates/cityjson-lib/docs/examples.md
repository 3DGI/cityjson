# Examples

The example programs mirror the intended public `cityjson_lib` surface.

- `examples/json_document.rs`
  Read one CityJSON document through `cityjson_lib::json::from_file` and
  `cityjson_lib::json::from_slice`.
- `examples/explicit_json_module.rs`
  Use `cityjson_lib::json` for probing, feature handling, and explicit boundary
  control.
- `examples/json_feature_stream.rs`
  Read and write a `CityJSONFeature` stream through the explicit JSON module.
- `examples/json_roundtrip.rs`
  Serialize documents and feature-sized models through `cityjson_lib::json`.
- `examples/alternate_formats.rs`
  Show the explicit-module pattern for Arrow and Parquet backends while keeping
  `CityModel` as the semantic unit.
- `examples/model_operations.rs`
  Show the intended split between `cityjson-rs` model semantics and
  `cityjson_lib::ops` workflow helpers.

Some examples cover extension points that may still be filled in gradually, but
the examples should all reinforce the same boundary rules:

- root constructors for the common document path
- explicit modules for explicit formats
- explicit access to the underlying model
