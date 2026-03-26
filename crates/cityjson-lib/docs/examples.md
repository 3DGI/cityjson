# Examples

These examples describe the intended `cjlib` API for the rewrite.

- `examples/json_document.rs`
  Read a single CityJSON document through `CityModel::from_file` and `CityModel::from_slice`.
- `examples/json_feature_stream.rs`
  Read or aggregate a strict `CityJSON` plus `CityJSONFeature` stream through the explicit `cjlib::json` boundary.
- `examples/explicit_json_module.rs`
  Use `cjlib::json` for probing, feature handling, and explicit boundary control.
- `examples/json_roundtrip.rs`
  Use the explicit JSON boundary module for document and feature serialization.
- `examples/alternate_formats.rs`
  Show the intended feature-gated module pattern for Arrow and Parquet backends while preserving `CityModel` as the semantic unit.
- `examples/model_operations.rs`
  Show the intended split between `cityjson-rs` model semantics and `cjlib::ops` workflow helpers.

The examples are the contract for the public API.
Some of them intentionally target APIs that are not implemented yet.
