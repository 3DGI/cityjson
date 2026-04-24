# cityjson-lib

`cityjson_lib` is the publishable CityJSON facade for Rust, plus the shared FFI
surface used by the Python and C++ bindings in this repository.

The current published surface is intentionally small:

- Rust document and feature-stream IO through `cityjson_lib::json`
- Rust workflow helpers through `cityjson_lib::ops`
- Rust summary helpers through `cityjson_lib::query`
- Python and C++ bindings over the shared FFI core

Arrow, Parquet, and wasm are not part of the current release-facing surface.

## Quick Start

```rust
use cityjson_lib::{json, query};

let model = json::from_file("amsterdam.city.json") ?;
let summary = query::summary( & model);
println!("{} cityobjects", summary.cityobject_count);

let bytes = json::to_vec( & model) ?;
# let _ = bytes;
# Ok::<(), cityjson_lib::Error>(())
```

## Docs

The minimal docs set lives under [`docs/`](docs/).
Start with:

- [`docs/index.md`](docs/index.md)
- [`docs/guide.md`](docs/guide.md)
- [`docs/public-api.md`](docs/public-api.md)
- [`docs/ffi/api.md`](docs/ffi/api.md)
- [`docs/ffi/performance.md`](docs/ffi/performance.md)

## Acknowledgements

`cityjson-lib`'s native `subset` and `merge` workflows were ported from [`cjio`](https://github.com/cityjson/cjio), the CityJSON/io project, which is licensed under MIT.

## Contributing

This crate follows the workspace contract. See
[`CONTRIBUTING.md`](../../CONTRIBUTING.md) for PR guidelines and
[`docs/development.md`](../../docs/development.md) for tooling, lints,
and release flow.

Crate-specific notes: the `ffi/` subtree ships FFI cores for Python, C++,
and wasm; see the `just ffi` helper and [`docs/ffi/`](docs/ffi/).

## License

Dual-licensed under MIT or Apache-2.0, at your option. See
[`LICENSE`](LICENSE) and [`LICENSE-APACHE`](LICENSE-APACHE).
