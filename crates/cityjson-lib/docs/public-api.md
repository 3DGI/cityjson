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
- `ErrorKind`
- `ops`
- `cityjson`, re-exported as a crate for advanced model access

`CityModel` should remain a thin owned wrapper around `cityjson::v2_0::OwnedCityModel`.
`cjlib` should avoid scattering lots of cherry-picked `cityjson-rs` items at the crate root.
The clean advanced path is `cjlib::cityjson::...`.

## Default Entry Point

The default path for CityJSON JSON input should stay on `CityModel`:

```rust
use std::io::Cursor;

use cjlib::CityModel;

let document = CityModel::from_file("rotterdam.city.json") ?;
let bytes = CityModel::from_slice(br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#) ?;
let stream = CityModel::from_stream(Cursor::new(std::fs::read("rotterdam.city.jsonl") ? )) ?;
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

```rust
use cjlib::{json, CityJSONVersion};

let bytes = std::fs::read("rotterdam.city.json") ?;
let probe = json::probe( & bytes) ?;
assert_eq!(probe.kind(), json::RootKind::CityJSON);
assert_eq!(probe.version(), Some(CityJSONVersion::V2_0));

let model = json::from_slice( & bytes) ?;
# Ok::<(), cjlib::Error>(())
```

For simplicity, the explicit `json` boundary should own all JSON-specific operations:

- `probe`
- `from_slice`
- `from_file`
- `from_stream`
- `to_vec`
- `to_string`
- `to_writer`

This gives a clean rule:

- reading convenience aliases live on `CityModel`
- format-boundary control and serialization live in `cjlib::json`

For non-JSON transport crates, the intended surface should stay smaller:

- `from_file`
- `to_file`

That is enough for a clean first public contract over sibling crates such as `cityarrow` and `cityparquet`.
If reader or writer variants become necessary later, they can be added after the backend crates settle.

## No Generic Format Registry

`cjlib` should prefer explicit modules over a generic codec registry or extension-sniffing dispatcher.

Preferred:

```rust
let model = cjlib::json::from_file("rotterdam.city.json")?;
#[cfg(feature = "arrow")]
cjlib::arrow::to_file("rotterdam.cjarrow", &model)?;
# Ok::<(), cjlib::Error>(())
```

Not preferred:

```rust
let model = cjlib::read("rotterdam.city.json")?;
cjlib::write("rotterdam.cjarrow", &model)?;
# Ok::<(), cjlib::Error>(())
```

The explicit-module design is simpler to teach, easier to maintain, and avoids a growing matrix of path sniffing rules.

## Model Boundary

`cjlib` should not mirror the whole `cityjson-rs` API, and `CityModel` should not pretend to be the whole inner model via implicit `Deref`.
The boundary should stay explicit:

```rust
let inner =
cjlib::cityjson::v2_0::OwnedCityModel::new(cjlib::cityjson::CityModelType::CityJSON);
let mut model = cjlib::CityModel::from(inner);

let borrowed: & cjlib::cityjson::v2_0::OwnedCityModel = model.as_inner();
let _ = borrowed;
let borrowed_mut: & mut cjlib::cityjson::v2_0::OwnedCityModel = model.as_inner_mut();
let _ = borrowed_mut;
let as_ref_model: & cjlib::cityjson::v2_0::OwnedCityModel = model.as_ref();
let _ = as_ref_model;
let as_mut_model: & mut cjlib::cityjson::v2_0::OwnedCityModel = model.as_mut();
let _ = as_mut_model;
let owned: cjlib::cityjson::v2_0::OwnedCityModel = model.into_inner();
# let _ = owned;
```

This keeps the split clean:

- `cjlib` is the facade
- `cityjson-rs` is the model
- conversions are explicit
- the root namespace stays small

For advanced work, callers should import from `cjlib::cityjson`, not from a `cjlib`-specific prelude.

## Error Surface

The public error API should be structured.

`Error` should remain the main error type, but callers should be able to branch on a small stable category enum such as `ErrorKind`.
The preferred taxonomy is intentionally small:

- `Io`
- `Syntax`
- `Version`
- `Shape`
- `Unsupported`
- `Model`

The goal is to support code like this:

```rust
use cjlib::{json, ErrorKind};

let error = json::from_slice(br#"{"type":"CityJSON","CityObjects":{},"vertices":[]}"#).unwrap_err();
assert_eq!(error.kind(), ErrorKind::Version);
```

That is a better public contract than matching on formatted error strings.
It is also simpler to maintain than a very granular error enum that mirrors every internal parsing branch.

## Higher-level Operations

`cjlib` should also own higher-level model operations that are useful to application code but do not belong in the core `cityjson-rs` data model.

Those operations should live under `cjlib::ops`, not as a large set of inherent `CityModel` methods.

That keeps the facade organized:

- `CityModel` stays focused on loading and wrapper-boundary access
- `cityjson-rs` stays focused on the model and its invariants
- `cjlib::ops` becomes the place for reusable higher-level workflows

Examples of the intended shape:

```rust
use cjlib::{ops, CityModel};

let mut model = CityModel::from_file("rotterdam.city.json")?;
let subset = ops::subset(&model, ops::Selection::from_ids(["bldg-1"]))?;
let merged = ops::merge([model, subset])?;
let _surface_area = ops::geometry::surface_area(&merged, "bldg-1")?;
# Ok::<(), cjlib::Error>(())
```

The initial operations namespace should cover:

- `merge`
- `subset`
- `upgrade`
- `lod::filter`
- `vertices::clean`
- `geometry::surface_area`
- `geometry::volume`
- `textures` helpers for texture-path relocation
- feature-gated `crs` helpers such as assign and reproject

## Relationship To `cjfake`

`cjfake` should not be re-exported as `cjlib::fake`.

The cleaner dependency direction is:

- `cjfake` depends on `cjlib`
- `cjlib` depends on sibling format crates and `cityjson-rs`

That way `cjfake` can generate `cityjson-rs` data and then emit any supported format by calling the explicit `cjlib` format modules.
`cjlib` stays focused on facade and format integration instead of absorbing test-data generation concerns.

## Alternative Format Modules

Arrow and Parquet integration should be feature-gated and explicit.

```rust
#[cfg(feature = "arrow")]
let model = cjlib::arrow::from_file("tiles.cjarrow") ?;

#[cfg(feature = "arrow")]
cjlib::arrow::to_file("tiles-out.cjarrow", &model) ?;

#[cfg(feature = "parquet")]
let model = cjlib::parquet::from_file("tiles.cjparquet") ?;

#[cfg(feature = "parquet")]
cjlib::parquet::to_file("tiles-out.cjparquet", &model) ?;
# Ok::<(), cjlib::Error>(())
```

Those modules are part of the intended public shape even if their implementation lands later than the JSON path.
