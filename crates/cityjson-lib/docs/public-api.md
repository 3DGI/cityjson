# Public API Overview

This page summarizes the stable Rust-facing shape that `cityjson_lib` is trying to
preserve.

## Root Surface

The crate root should stay small:

- `CityModel`
- `CityJSONVersion`
- `Error`
- `ErrorKind`
- `json`, enabled by default through the `json` feature
- `ops`, enabled by default with `json`
- `cityjson`

The public rule is:

- common document loading lives in `cityjson_lib::json`
- explicit boundary work lives in explicit modules
- advanced model work happens through `cityjson_lib::cityjson`

## `CityModel`

`CityModel` is a direct alias for the owned `cityjson-rs` model at the Rust
boundary. The facade contract stays the same:

- owned by default
- explicit boundary functions live in `cityjson_lib::json`
- no `Deref`-based API blur
- one semantic model type regardless of whether the data is a whole document,
  a subset, or a feature-sized package

```rust
use cityjson_lib::json;
use cityjson_lib::CityModel;

let from_file = json::from_file("tests/data/v2_0/minimal.city.json")?;
let from_slice = json::from_slice(
    br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#,
)?;

let borrowed = &from_file;
let _owned: CityModel = from_slice;
# let _ = borrowed;
# Ok::<(), cityjson_lib::Error>(())
```

## Explicit Boundary Modules

`cityjson_lib::json` owns explicit JSON and JSONL work:

- probing
- document parsing
- feature parsing
- feature-stream reading
- document and feature serialization
- feature-stream writing

The transport-specific branch keeps the Arrow and Parquet experiments available
without making them part of the core publishable crate.

## `cityjson_lib::ops`

`ops` is the home for reusable workflows above the semantic model:

- selection and subset helpers
- merge and upgrade workflows
- cleanup and maintenance helpers
- geometry measurements
- feature-gated CRS helpers

Those operations should build on `cityjson-rs` semantics rather than redefine
them.

## `cityjson_lib::cityjson`

The advanced escape hatch is the re-exported model crate:

```rust
use cityjson_lib::cityjson;
```

That keeps the facade teachable without pretending that `cityjson_lib` owns the whole
semantic surface.

## Relationship To FFI

The Rust facade and the FFI work are parallel layers, not competing ones:

- Rust users call `cityjson_lib` directly.
- Foreign bindings share one low-level core documented under
  [FFI and Bindings](ffi/index.md).
- Binding-specific APIs stay target-specific for bulk interchange and use scalar
  or single-item helpers for inspection and editing.
