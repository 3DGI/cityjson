# cjlib

`cjlib` is the user-facing facade for the CityJSON crates in this repository.

The intended shape is deliberately small:

- `cityjson-rs` owns the in-memory model
- `serde_cityjson` owns CityJSON JSON and JSONL parsing/serialization
- `cjlib` owns the ergonomic entry points, version dispatch, and format-level integration

The future public API is centered on:

- `cjlib::CityModel`
- `cjlib::CityJSONVersion`
- `cjlib::Error`
- `cjlib::ErrorKind`
- `cjlib::json`
- feature-gated format modules such as `cjlib::arrow` and `cjlib::parquet`
- `cjlib::cityjson` for advanced model access

## Default Path

For CityJSON JSON input, the default entry points stay on `CityModel`:

```rust
use std::io::Cursor;

use cjlib::CityModel;

let document = CityModel::from_file("rotterdam.city.json")?;
let stream = CityModel::from_stream(Cursor::new(std::fs::read("rotterdam.city.jsonl")?))?;
let bytes = CityModel::from_slice(br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#)?;
# Ok::<(), cjlib::Error>(())
```

## Explicit Format Modules

The top-level constructors are only the convenience path for CityJSON JSON.

Serialization should be explicit and format-qualified:

```rust
use cjlib::{json, CityModel};

let model = CityModel::from_file("rotterdam.city.json")?;
let bytes = json::to_vec(&model)?;
let text = json::to_string(&model)?;
# let _ = (bytes, text);
# Ok::<(), cjlib::Error>(())
```

Alternative encodings and containers should live in explicit modules:

- `cjlib::json`
- `cjlib::arrow`
- `cjlib::parquet`

That keeps the facade predictable:

- `CityModel::from_*` means CityJSON JSON / JSONL
- explicit modules mean explicit formats

Within `cjlib::json`, the intended surface is:

- `probe`
- `from_slice`
- `from_file`
- `from_stream`
- `to_vec`
- `to_string`
- `to_writer`

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
The documents and integration tests describe the target API, even when the implementation is still catching up.
