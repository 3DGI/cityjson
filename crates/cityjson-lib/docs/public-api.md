# cjlib

`cjlib` is the user-facing facade for the CityJSON crates in this repository.

This document describes the intended public API of the rewrite.
It is deliberately future-facing: the examples and tests are allowed to get ahead of the implementation.

## Design

`cjlib` should stay small.

- `cityjson-rs` owns the in-memory model and model invariants
- `serde_cityjson` owns CityJSON JSON and JSONL parsing/serialization
- `cjlib` owns convenience constructors, version dispatch, format integration, and a stable user-facing boundary

The crate should not grow a second model, a second importer stack, or a public indexed-geometry API.

## Primary Types

The core user-facing surface should be:

- `CityModel`
- `CityJSONVersion`
- `Error`
- re-exports of `cityjson-rs` for advanced model access

`CityModel` should remain a thin owned wrapper around `cityjson::v2_0::OwnedCityModel`.

## Default Entry Point

The default path for CityJSON JSON input should stay on `CityModel`:

```rust,ignore
use std::io::Cursor;

use cjlib::CityModel;

let document = CityModel::from_file("rotterdam.city.json")?;
let bytes = CityModel::from_slice(br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#)?;
let stream = CityModel::from_stream(Cursor::new(std::fs::read("rotterdam.city.jsonl")?))?;
# Ok::<(), cjlib::Error>(())
```

The intent is simple:

- `from_slice` for already-loaded bytes
- `from_file` for path-based document import
- `from_stream` for strict `CityJSON` plus `CityJSONFeature` streams

## Explicit Format Modules

The top-level methods should only cover the common CityJSON JSON path.

Format-specific behavior should move into explicit modules:

- `cjlib::json`
- `cjlib::arrow`
- `cjlib::parquet`

That yields a predictable rule:

- top-level constructors mean CityJSON JSON / JSONL
- module-qualified constructors mean explicit format work

Example:

```rust,ignore
use cjlib::{json, CityJSONVersion};

let bytes = std::fs::read("rotterdam.city.json")?;
assert_eq!(json::detect_version(&bytes)?, CityJSONVersion::V2_0);

let model = json::from_slice(&bytes)?;
# Ok::<(), cjlib::Error>(())
```

## Relationship To `cityjson-rs`

`cjlib` should not mirror the whole `cityjson-rs` API.
Once a model is loaded, callers should be able to drop down to the re-exported model crate directly.

```rust,ignore
use cjlib::{CityModel, CityModelType};

let model = CityModel::new(CityModelType::CityJSON);
let inner: &cjlib::cityjson::v2_0::OwnedCityModel = model.as_inner();
let _ = inner;
```

This keeps the split clean:

- `cjlib` is the facade
- `cityjson-rs` is the model

## Alternative Format Modules

Arrow and Parquet integration should be feature-gated and explicit.

```rust,ignore
#[cfg(feature = "arrow")]
let model = cjlib::arrow::from_file("tiles.cjarrow")?;

#[cfg(feature = "parquet")]
let model = cjlib::parquet::from_file("tiles.cjparquet")?;
# Ok::<(), cjlib::Error>(())
```

Those modules are part of the intended public shape even if their implementation lands later than the JSON path.
