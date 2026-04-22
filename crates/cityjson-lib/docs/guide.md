# Guide

This guide describes the current Rust-facing `cityjson_lib` surface.

## Start With `json`

For ordinary document loading, start in `cityjson_lib::json`.

```rust
use cityjson_lib::{json, query};

let model = json::from_file("amsterdam.city.json")?;
let summary = query::summary(&model);
println!("{} cityobjects", summary.cityobject_count);
# Ok::<(), cityjson_lib::Error>(())
```

Use:

- `json::from_file` for file-backed input
- `json::from_slice` for bytes already in memory
- `json::probe` when you need explicit root-kind or version checks first

## Keep Boundary Work Explicit

`cityjson_lib` does not hide format choice behind generic `read` and `write`
entry points.

If you need feature parsing, feature-stream handling, or explicit document and
feature serialization, stay on the `json` module:

```rust
use cityjson_lib::{json, CityJSONVersion};

let bytes = std::fs::read("amsterdam.city.json")?;
let probe = json::probe(&bytes)?;
assert_eq!(probe.kind(), json::RootKind::CityJSON);
assert_eq!(probe.version(), Some(CityJSONVersion::V2_0));

let model = json::from_slice(&bytes)?;
let encoded = json::to_vec(&model)?;
# let _ = encoded;
# Ok::<(), cityjson_lib::Error>(())
```

## Use `ops` For Shared Workflows

`ops` is where reusable workflows live today:

- `cleanup`
- `subset`
- `append`
- `merge`

```rust
use cityjson_lib::{json, ops};

let first = json::from_feature_file("tests/data/v2_0/feature-1.city.json")?;
let second = json::from_feature_file("tests/data/v2_0/feature-2.city.json")?;

let merged = ops::merge([first, second])?;
let subset = ops::subset(&merged, ["building-1"], false)?;
let cleaned = ops::cleanup(&subset)?;
# let _ = cleaned;
# Ok::<(), cityjson_lib::Error>(())
```

These helpers are part of the stable facade, with semantic workflows owned by
`cityjson-lib` and JSON-aware implementation details living in `cityjson-json`.

## Handle Errors By Category

The stable contract is `ErrorKind`, not display text.

```rust
use cityjson_lib::{json, ErrorKind};

let error = json::from_slice(br#"{"type":"CityJSON","CityObjects":{},"vertices":[]}"#).unwrap_err();
assert_eq!(error.kind(), ErrorKind::Version);
```
