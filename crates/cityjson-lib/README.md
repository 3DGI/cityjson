# cityjson-lib

`cityjson_lib` is the user-facing facade for the CityJSON crates in this repository.

If you are new to the broader project family, start with
[`docs/ecosystem-overview.md`](docs/ecosystem-overview.md).
It explains what each repository does, how the responsibilities are split, and
which crate to use for which kind of task.

The current rewrite keeps the implemented surface deliberately small:

- `cityjson-rs` owns the one semantic model
- `serde_cityjson` owns the CityJSON JSON and JSONL boundary
- `cityjson_lib` owns the ergonomic facade, explicit format modules, and version-level
  dispatch where needed

The semantic rule is:

- one semantic model: `cityjson::v2_0::OwnedCityModel`
- one facade wrapper: `cityjson_lib::CityModel`
- one semantic interchange unit: a self-contained `CityModel`
- explicit format boundaries: JSON, JSONL, Arrow, and Parquet
- no binding-level cityobject projection wrappers on the hot path

For the full synthesis, see [`docs/architecture.md`](docs/architecture.md).

The future public API is centered on:

- `cityjson_lib::CityModel`
- `cityjson_lib::CityJSONVersion`
- `cityjson_lib::Error`
- `cityjson_lib::ErrorKind`
- `cityjson_lib::json`
- `cityjson_lib::ops`, currently one illustrative `todo!()` function
- `cityjson_lib::arrow`, backed by the sibling `cityarrow` transport crate
- `cityjson_lib::parquet`, backed by the sibling `cityparquet` transport crate
- `cityjson_lib::cityjson` for advanced model access

## Default Path

For single-document CityJSON input, the default entry points stay on
`CityModel`:

```rust
use cityjson_lib::CityModel;

let document = CityModel::from_file("rotterdam.city.json")?;
let bytes = CityModel::from_slice(br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#)?;
# Ok::<(), cityjson_lib::Error>(())
```

Current practical status:

- `cityjson_lib` is usable today for ordinary `CityJSON` document files
- the implemented document path is `CityJSON` v2.0 through `CityModel::from_*`
- explicit feature and feature-stream helpers exist under `cityjson_lib::json`
- explicit live Arrow IPC stream transport exists under `cityjson_lib::arrow`
- explicit Arrow batch export/import exists under `cityjson_lib::arrow`
- explicit cityparquet package-file transport exists under `cityjson_lib::parquet`
- `tyler` 0.4.0 now dogfoods `cityjson_lib` for CityJSON reading
- higher-level workflows such as `ops::merge` are still intentionally unimplemented

## Explicit Format Modules

The top-level constructors are only the convenience path for CityJSON JSON.

Serialization, feature handling, and model streams should be explicit and
format-qualified:

```rust
use cityjson_lib::{json, CityModel};

let model = CityModel::from_file("rotterdam.city.json")?;
let bytes = json::to_vec(&model)?;
let text = json::to_string(&model)?;
let feature_text = json::to_feature_string(&model)?;
# let _ = (bytes, text, feature_text);
# Ok::<(), cityjson_lib::Error>(())
```

Alternative encodings and containers should live in explicit modules:

- `cityjson_lib::json`
- `cityjson_lib::arrow`, which owns live Arrow IPC stream I/O and explicit batch export/import
- `cityjson_lib::parquet`, which owns persistent package-file I/O

That keeps the facade predictable:

- `CityModel::from_*` means the common single-document CityJSON path
- explicit modules mean explicit formats
- expensive Arrow conversion is explicit in the API name

Within `cityjson_lib::json`, the intended surface is:

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
- `to_feature_string`
- `to_feature_writer`

Advanced staged reconstruction paths live under `cityjson_lib::json::staged`:

- `from_feature_slice_with_base`
- `from_feature_file_with_base`
- `from_feature_assembly_with_base`

`json::from_file` is document-oriented.
Feature streams should be handled explicitly through
`json::read_feature_stream`.

## Higher-level Operations

Higher-level workflows that do not belong in the core `cityjson-rs` model should live under `cityjson_lib::ops`.
Right now that namespace is intentionally reduced to one unimplemented
illustrative function so the intended boundary is visible without hiding the
missing work behind no-op behavior.

## Relationship To `cjfake`

`cjfake` should remain a sibling crate above `cityjson_lib`, not part of the `cityjson_lib` root API.

That keeps the dependency direction clean:

- `cjfake` generates model data
- `cjfake` uses `cityjson_lib` format modules to emit JSON, Arrow, Parquet, and future formats
- `cityjson_lib` stays focused on facade, format integration, and operations

For advanced model work, `cityjson_lib` should stay explicit rather than proxying `cityjson-rs` through `Deref`.
The intended path is to use `CityModel::as_inner`, `as_inner_mut`, `into_inner`, `AsRef`, `AsMut`, and then work through `cityjson_lib::cityjson`.

`ErrorKind` should also stay intentionally small. The intended stable categories are:

- `Io`
- `Syntax`
- `Version`
- `Shape`
- `Unsupported`
- `Model`

## Repository Tasks

The repository now has a small `justfile` in the same style as `cityjson-rs`.
The main tasks are:

- `just check`
- `just clean`
- `just clippy`
- `just fmt`
- `just lint`
- `just ci`
- `just test`
- `just doc`
- `just docs-build`
- `just docs-serve`

The MkDocs site is intended to be the main documentation home for the Rust facade, future FFI surface, and language bindings.

## Status

This repository is currently being rewritten in a docs-first, tests-first style.
Unimplemented areas are intentionally marked with `todo!()` where that is still
the deliberate contract, and implemented boundaries are covered by direct
roundtrip tests so the remaining gaps stay visible.
