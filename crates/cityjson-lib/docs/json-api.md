# JSON Boundary

`cityjson_lib::json` is the explicit boundary layer over `cityjson-json`.

It is not a second JSON implementation.

## Surface

The current public surface includes:

- `probe`
- `from_slice`
- `from_file`
- `from_feature_slice`
- `from_feature_file`
- `read_feature_stream`
- `write_feature_stream`
- `to_vec`
- `to_string`
- `to_writer`
- `to_feature_vec`
- `to_feature_string`
- `to_feature_writer`
- `staged::*`

It also re-exports the lower-level `cityjson-json` read and write option types
that matter for advanced callers.

## Boundary Rules

- `CityJSON` and `CityJSONFeature` are wire forms
- `CityModel` is still the semantic unit returned to callers
- feature streams stay explicit instead of hiding behind document-oriented
  helpers
- probing is available when the caller needs it, but ordinary parsing still
  starts with `from_file` or `from_slice`

## Staged Reconstruction

`json::staged` exists for callers that already own:

- raw feature bytes
- a cached base document
- preassembled feature fragments

Those advanced helpers still return `CityModel`, not transport-specific public
types.

## Error Translation

`cityjson-json` owns the JSON-aware implementation details.
`cityjson-lib` translates them into the stable `Error` and `ErrorKind`
categories exposed by this crate.
