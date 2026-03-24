# Examples

These examples describe the intended `cjlib` API for the rewrite.

- `examples/json_document.rs`
  Read a single CityJSON document through `CityModel::from_file` and `CityModel::from_slice`.
- `examples/json_feature_stream.rs`
  Aggregate a strict `CityJSON` plus `CityJSONFeature` stream with `CityModel::from_stream`.
- `examples/explicit_json_module.rs`
  Use the future `cjlib::json` module for explicit probing and parsing.
- `examples/json_roundtrip.rs`
  Use the explicit JSON boundary module for serialization and round-tripping.
- `examples/alternate_formats.rs`
  Show the intended feature-gated module pattern for Arrow and Parquet backends.

The examples are the contract for the public API.
Some of them intentionally target APIs that are not implemented yet.
