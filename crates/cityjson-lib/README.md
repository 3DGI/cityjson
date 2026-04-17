# cityjson-lib

`cityjson_lib` is the publishable facade and binding host for the CityJSON
stack in this repository.

The release line is intentionally small:

- `cityjson-rs` owns the semantic CityJSON model
- `cityjson-json` owns CityJSON and CityJSONSeq parsing and serialization
- `cityjson-lib` owns the stable Rust facade, `ops`, `query`, and the shared
  FFI core used by Python and C++

Arrow and Parquet are not part of this published crate.

## Rust Quick Start

```rust
use cityjson_lib::{json, query};

let model = json::from_file("amsterdam.city.json")?;
let summary = query::summary(&model);
println!("{} cityobjects", summary.cityobject_count);

let bytes = json::to_vec(&model)?;
# let _ = bytes;
# Ok::<(), cityjson_lib::Error>(())
```

Use `cityjson_lib::json` when you need explicit boundary control:

```rust
use cityjson_lib::{json, CityJSONVersion};

let bytes = std::fs::read("amsterdam.city.json")?;
let probe = json::probe(&bytes)?;
assert_eq!(probe.kind(), json::RootKind::CityJSON);
assert_eq!(probe.version(), Some(CityJSONVersion::V2_0));
# Ok::<(), cityjson_lib::Error>(())
```

Use `cityjson_lib::ops` for reusable workflows above the model:

```rust
use cityjson_lib::{json, ops};

let model = json::from_file("amsterdam.city.json")?;
let cleaned = ops::cleanup(&model)?;
let subset = ops::extract(&cleaned, ["building-1"])?;
# let _ = subset;
# Ok::<(), cityjson_lib::Error>(())
```

## Bindings

This repository also ships:

- a shared Rust FFI core in `ffi/core`
- a Python package in `ffi/python`
- a C++ wrapper in `ffi/cpp`

The docs site under [`docs/`](docs/) is the canonical reference for the Rust,
Python, and C++ surfaces. The wasm adapter remains work in progress.

## Common Tasks

- `just test`
- `just ffi test`
- `just docs-build`
- `just docs-serve`
- `cargo publish --dry-run --allow-dirty`

## Documentation

Start with:

- [`docs/index.md`](docs/index.md)
- [`docs/guide.md`](docs/guide.md)
- [`docs/public-api.md`](docs/public-api.md)
- [`docs/ffi/api.md`](docs/ffi/api.md)
