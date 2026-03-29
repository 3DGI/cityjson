# Public API Overview

This page summarizes the stable Rust-facing shape that `cjlib` is trying to
preserve.

## Root Surface

The crate root should stay small:

- `CityModel`
- `CityJSONVersion`
- `Error`
- `ErrorKind`
- `json`
- `ops`
- optional sibling modules such as `arrow` and `parquet`
- `cityjson`

The public rule is:

- common document loading lives on `CityModel`
- explicit boundary work lives in explicit modules
- advanced model work happens through `cjlib::cityjson`

## `CityModel`

`CityModel` is the owned default wrapper at the Rust boundary.
The concrete `cityjson-rs` model instantiation behind it may evolve where that
helps implementation, but the facade contract stays the same:

- owned by default
- explicit boundary accessors
- no `Deref`-based API blur
- one wrapper type regardless of whether the model is a whole document, a
  subset, or a feature-sized package

```rust
use cjlib::CityModel;

let from_file = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;
let from_slice = CityModel::from_slice(
    br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#,
)?;

let borrowed = from_file.as_inner();
let _owned = from_slice.into_inner();
# let _ = borrowed;
# Ok::<(), cjlib::Error>(())
```

## Explicit Boundary Modules

`cjlib::json` owns explicit JSON and JSONL work:

- probing
- document parsing
- feature parsing
- feature-stream reading
- document and feature serialization
- feature-stream writing

Sibling format modules follow the same rule: explicit module, explicit format,
semantic item type still `CityModel`.

## `cjlib::ops`

`ops` is the home for reusable workflows above the semantic model:

- selection and subset helpers
- merge and upgrade workflows
- cleanup and maintenance helpers
- geometry measurements
- feature-gated CRS helpers

Those operations should build on `cityjson-rs` semantics rather than redefine
them.

## `cjlib::cityjson`

The advanced escape hatch is the re-exported model crate:

```rust
use cjlib::cityjson;
```

That keeps the facade teachable without pretending that `cjlib` owns the whole
semantic surface.

## Relationship To FFI

The Rust facade and the FFI work are parallel layers, not competing ones:

- Rust users call `cjlib` directly.
- Foreign bindings share one low-level core documented under
  [FFI and Bindings](ffi/index.md).
- Binding-specific APIs are free to be more C++-like, Python-like, or
  wasm-friendly than the Rust facade.
