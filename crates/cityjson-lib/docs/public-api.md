# Public API Overview

This page summarizes the current stable Rust-facing shape of `cityjson_lib`.

## Crate Root

The crate root is intentionally small:

- `CityModel`
- `CityJSONVersion`
- `Error`
- `ErrorKind`
- `json`
- `arrow` *(feature `arrow`)*
- `parquet` *(feature `parquet`)*
- `ops`
- `query`

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

## `arrow` *(feature `arrow`)*

`arrow` is the optional columnar I/O boundary backed by `cityjson-arrow`.
Enable it with `features = ["arrow"]` in your `Cargo.toml`.

It owns:

- reading from bytes, a reader, or a file
- writing to bytes, a writer, or a file
- lower-level `read_stream` / `write_stream` primitives
- re-exported `ExportOptions`, `ImportOptions`, `SchemaVersion`, `WriteReport`

```rust
# #[cfg(feature = "arrow")]
# {
use cityjson_lib::{arrow, json};

let model = json::from_file("tests/data/v2_0/minimal.city.json")?;
let bytes = arrow::to_vec(&model)?;
let roundtrip = arrow::from_bytes(&bytes)?;
# let _ = roundtrip;
# }
# Ok::<(), cityjson_lib::Error>(())
```

## `parquet` *(feature `parquet`)*

`parquet` is the optional Parquet I/O boundary backed by `cityjson-parquet`.
Enable it with `features = ["parquet"]` in your `Cargo.toml`.
Enabling `parquet` also enables `arrow`.

It owns:

- reading and writing as a self-contained package file (`.cityjson-parquet`)
- reading and writing as a bare dataset directory
- re-exported `PackageManifest`, `ParquetDatasetManifest`

```rust
# #[cfg(feature = "parquet")]
# {
use cityjson_lib::{json, parquet};

let model = json::from_file("tests/data/v2_0/minimal.city.json")?;
let dir = tempfile::tempdir()?;
let path = dir.path().join("model.cityjson-parquet");
let manifest = parquet::to_file(&path, &model)?;
assert!(!manifest.tables.is_empty());
let roundtrip = parquet::from_file(&path)?;
# let _ = roundtrip;
# }
# Ok::<(), cityjson_lib::Error>(())
```

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
