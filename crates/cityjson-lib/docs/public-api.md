# Public API Overview

This page summarizes the current stable Rust-facing shape of `cityjson_lib`.

## Crate Root

The crate root is intentionally small:

- `CityModel`
- `CityJSONVersion`
- `Error`
- `ErrorKind`
- `json`
- `ops`
- `query`
- `cityjson`

## `CityModel`

`CityModel` is the owned semantic model type re-exported from `cityjson-rs`.
It is the same type whether the payload represents:

- a full document
- a feature-sized self-contained model
- a merged or extracted subset

```rust
use cityjson_lib::{json, CityModel};

let model: CityModel = json::from_file("tests/data/v2_0/minimal.city.json")?;
# let _ = model;
# Ok::<(), cityjson_lib::Error>(())
```

## `json`

`json` is the default-on boundary module.
It owns:

- probing
- document parsing
- feature parsing
- feature-stream reading and writing
- document and feature serialization
- staged feature reconstruction helpers

The implementation lives in `cityjson-json`.
`cityjson-lib` keeps the public contract and error/version translation.

## `ops`

`ops` exposes the workflow helpers currently shipped on the release line:

- `cleanup`
- `extract`
- `append`
- `merge`

Those helpers are part of the stable facade, but their JSON-aware implementation
is delegated to `cityjson-json`.

## `query`

`query` exposes summary-style read helpers over `CityModel` without turning the
model type into a large method bag.

```rust
use cityjson_lib::{json, query};

let model = json::from_file("tests/data/v2_0/minimal.city.json")?;
let summary = query::summary(&model);
assert!(summary.cityobject_count >= 1);
# Ok::<(), cityjson_lib::Error>(())
```

## `cityjson`

`cityjson` is the explicit drop-down path to the deeper semantic model API.
Use it when the facade surface is intentionally smaller than the underlying
model crate.
