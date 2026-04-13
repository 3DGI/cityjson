# cityjson-json

`cityjson-json` is a CityJSON 2.0 serde adapter around the [`cityjson`](https://crates.io/crates/cityjson) crate. It provides efficient serialization and deserialization of CityJSON documents with both owned and borrowed string storage options.

## Installation

```shell
cargo add cityjson-json
```

## Getting Started

### Imports

```rust
use cityjson_json::{from_str_owned, from_str_borrowed, to_string, to_string_validated};
use cityjson_json::{OwnedCityModel, BorrowedCityModel, SerializableCityModel};
```

### Owned Deserialization

For simple use cases, deserialize into an owned model:

```rust
use cityjson_json::from_str_owned;

let json_str = r#"{
  "type": "CityJSON",
  "version": "2.0",
  "CityObjects": {},
  "vertices": []
}"#;

let model = from_str_owned(json_str)?;
```

### Borrowed Deserialization

For performance-critical applications, use borrowed deserialization to avoid allocations:

```rust
use cityjson_json::from_str_borrowed;

let json_str = r#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#;
let model = from_str_borrowed(json_str)?;
// model holds references to json_str
```

### Serialization

Serialize models back to JSON:

```rust
use cityjson_json::to_string;

let json_output = to_string(&model)?;
```

For validated serialization (checks default theme references):

```rust
use cityjson_json::to_string_validated;

let json_output = to_string_validated(&model)?;
```

## Validation Policy

The library provides two serialization paths to balance performance and safety:

- **`to_string()`**: Fast path. Does not validate that default theme names (for materials and textures) actually reference existing themes in the appearance section.
- **`to_string_validated()`**: Strict path. Validates default theme references before serialization to ensure document consistency.

Use `to_string_validated()` when you need guaranteed valid CityJSON output, especially when serializing user-provided models.

## Documentation

- [CityJSON 2.0](https://www.cityjson.org/specs/2.0.1/)
- [`cityjson` crate documentation](https://docs.rs/cityjson/)

todo: link to docs.rs

## Library Layout

| Module   | Contents                                                                                       |
|----------|------------------------------------------------------------------------------------------------|
| `v2_0`   | CityJSON 2.0 (de)serialization entry points, feature-stream helpers, and `SerializableCityModel` |
| `errors` | `Error` and `Result` types surfaced by the adapter                                             |
| (root)   | Convenience re-exports: `from_str_*`, `to_string*`, `to_vec*`, `to_writer*`, model types       |

Core types re-exported from `cityjson`:

- **`OwnedCityModel`**: A CityJSON model with owned `String` storage. Self-contained and doesn't depend on external lifetimes.
- **`BorrowedCityModel`**: A CityJSON model with borrowed string references. More memory efficient but requires careful lifetime management.

## Design

todo: link to design docs

## API Stability

This crate follows semantic versioning (`MAJOR.MINOR.PATCH`):

- `MAJOR`: incompatible API changes
- `MINOR`: backwards-compatible feature additions
- `PATCH`: backwards-compatible fixes

## Minimum Rust Version

The minimum supported rustc version is `1.93.0`.

## Development

### Running Tests

```bash
cargo test
cargo test --test v2_0
```

The corpus-backed correctness tests read fixture IDs from the shared
`cityjson-benchmarks` checkout at
`../cityjson-benchmarks/artifacts/correctness-index.json` by default. Override
the shared root with `CITYJSON_JSON_SHARED_CORPUS_ROOT` or the index path with
`CITYJSON_JSON_CORRECTNESS_INDEX` if your checkout lives elsewhere.

### Running Benchmarks

The benchmark corpus lives in the shared `cityjson-benchmarks` repository.
`cityjson-json` benchmarks the CityJSON artifacts listed in each workload's
`artifacts[]` array and reads the shared benchmark index from
`../cityjson-benchmarks/artifacts/benchmark-index.json` by default. Override
the shared root with `CITYJSON_JSON_SHARED_CORPUS_ROOT` or the index path with
`CITYJSON_JSON_BENCHMARK_INDEX` if your checkout lives elsewhere.

```bash
just bench-read
just bench-write
just bench-report
```

The benchmarks use Criterion. Read throughput is based on input bytes; write
throughput is based on output bytes. README benchmark tables are generated
from the shared corpus and should be refreshed from current benchmark output,
not edited by hand.

## Contributing

todo: add contributing guidelines

## License

Licensed under either:

- Apache License, Version 2.0 (`LICENSE-APACHE`)
- MIT license (`LICENSE-MIT`)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
cityjson-json by you, as defined in the Apache-2.0 license, shall be dual licensed as above,
without additional terms or conditions.

## Use of AI in this project

This crate was originally developed without the use of AI.
Since then, it underwent multiple significant refactors and various LLM models (Claude, `ChatGPT`) were used for experimenting with alternative designs, in particular for the (de)serialization strategies and borrowed-parsing paths.
LLM generated code is also used for improving the test coverage and documentation and mechanical improvements.
Code correctness and performance are verified by carefully curated test cases and benchmarks that cover the CityJSON 2.0 specification.
