# cjlib

`cjlib` is the user-facing facade for the CityJSON crates in this repository.

The current rewrite keeps the implemented surface deliberately small:

- `cityjson-rs` owns the one semantic model
- `serde_cityjson` owns the CityJSON JSON and JSONL boundary
- `cjlib` owns the ergonomic facade, explicit format modules, and version-level
  dispatch where needed

The semantic rule is:

- one semantic model: `cityjson::v2_0::OwnedCityModel`
- one facade wrapper: `cjlib::CityModel`
- one semantic interchange unit: a self-contained `CityModel`
- many format boundaries: JSON, JSONL, Arrow, Parquet, and future raw/staged
  APIs

For the full synthesis, see [`docs/architecture.md`](docs/architecture.md).

The future public API is centered on:

- `cjlib::CityModel`
- `cjlib::CityJSONVersion`
- `cjlib::Error`
- `cjlib::ErrorKind`
- `cjlib::json`
- `cjlib::ops`, currently one illustrative `todo!()` function
- `cjlib::arrow`, currently one illustrative `todo!()` function
- `cjlib::parquet`, currently one illustrative `todo!()` function
- `cjlib::cityjson` for advanced model access

## Default Path

For single-document CityJSON input, the default entry points stay on
`CityModel`:

```rust
use cjlib::CityModel;

let document = CityModel::from_file("rotterdam.city.json")?;
let bytes = CityModel::from_slice(br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#)?;
# Ok::<(), cjlib::Error>(())
```

## Explicit Format Modules

The top-level constructors are only the convenience path for CityJSON JSON.

Serialization, feature handling, and model streams should be explicit and
format-qualified:

```rust
use cjlib::{json, CityModel};

let model = CityModel::from_file("rotterdam.city.json")?;
let bytes = json::to_vec(&model)?;
let text = json::to_string(&model)?;
let feature_text = json::to_feature_string(&model)?;
# let _ = (bytes, text, feature_text);
# Ok::<(), cjlib::Error>(())
```

Alternative encodings and containers should live in explicit modules:

- `cjlib::json`
- `cjlib::arrow`
- `cjlib::parquet`

That keeps the facade predictable:

- `CityModel::from_*` means the common single-document CityJSON path
- explicit modules mean explicit formats

Within `cjlib::json`, the intended surface is:

- `probe`
- `from_slice`
- `from_file`
- `from_feature_slice`
- `read_feature_stream`
- `write_feature_stream`
- `to_vec`
- `to_string`
- `to_writer`
- `to_feature_string`

`json::from_file` is document-oriented.
Feature streams should be handled explicitly through
`json::read_feature_stream`.

## Higher-level Operations

Higher-level workflows that do not belong in the core `cityjson-rs` model should live under `cjlib::ops`.
Right now that namespace is intentionally reduced to one unimplemented
illustrative function so the intended boundary is visible without hiding the
missing work behind no-op behavior.

## Relationship To `cjfake`

`cjfake` should remain a sibling crate above `cjlib`, not part of the `cjlib` root API.

That keeps the dependency direction clean:

- `cjfake` generates model data
- `cjfake` uses `cjlib` format modules to emit JSON, Arrow, Parquet, and future formats
- `cjlib` stays focused on facade, format integration, and operations

For advanced model work, `cjlib` should stay explicit rather than proxying `cityjson-rs` through `Deref`.
The intended path is to use `CityModel::as_inner`, `as_inner_mut`, `into_inner`, `AsRef`, `AsMut`, and then work through `cjlib::cityjson`.

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
- `just fmt`
- `just lint`
- `just test`
- `just doc`
- `just docs-build`
- `just docs-serve`

The MkDocs site is intended to be the main documentation home for the Rust facade, future FFI surface, and language bindings.

## Status

This repository is currently being rewritten in a docs-first, tests-first style.
Unimplemented areas are intentionally marked with `todo!()` and covered by
failing tests so the remaining work stays visible.
