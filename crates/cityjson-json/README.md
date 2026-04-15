# cityjson-json

`cityjson-json` is a `CityJSON` 2.0 serde adapter around the [`cityjson`](https://crates.io/crates/cityjson) crate. It provides efficient serialization and deserialization of `CityJSON` documents with both owned and borrowed string storage options.

## Installation

```shell
cargo add cityjson-json
```

## Getting Started

### Imports

```rust
use cityjson_json::{from_str_owned, from_str_borrowed, as_json};
use cityjson_json::{OwnedCityModel, BorrowedCityModel, SerializableCityModel};
```

### Owned Deserialization

For simple use cases, deserialize into an owned model:

```rust
use cityjson_json::from_str_owned;

let json_str = r#"{
  "type": "CityJSON",
  "version": "2.0",
  "transform": {"scale": [1.0, 1.0, 1.0], "translate": [0.0, 0.0, 0.0]},
  "CityObjects": {},
  "vertices": []
}"#;

let model = from_str_owned(json_str)?;
# Ok::<(), cityjson_json::Error>(())
```

### Borrowed Deserialization

For performance-critical applications, use borrowed deserialization to avoid allocations:

```rust
use cityjson_json::from_str_borrowed;

let json_str = r#"{"type":"CityJSON","version":"2.0","transform":{"scale":[1.0,1.0,1.0],"translate":[0.0,0.0,0.0]},"CityObjects":{},"vertices":[]}"#;
let model = from_str_borrowed(json_str)?;
// model holds references to json_str
# Ok::<(), cityjson_json::Error>(())
```

### Serialization

Serialize models back to JSON using the `as_json` builder:

```rust,ignore
use cityjson_json::as_json;

let json_output = as_json(&model).to_string()?;
```

The same builder works for other output targets:

```rust,ignore
use cityjson_json::as_json;

let bytes = as_json(&model).to_vec()?;
as_json(&model).validate().to_writer(&mut writer)?;
```

### `CityJSONSeq`

Read a newline-delimited `CityJSONSeq` stream. The first line must be a `CityJSON` header; each subsequent line is a self-contained `CityJSONFeature`:

```rust
use std::io::BufReader;
use cityjson_json::read_cityjsonseq;

let seq = concat!(
    r#"{"type":"CityJSON","version":"2.0","transform":{"scale":[0.001,0.001,0.001],"translate":[0.0,0.0,0.0]},"CityObjects":{},"vertices":[]}"#, "\n",
    r#"{"type":"CityJSONFeature","id":"f1","CityObjects":{"f1":{"type":"Building"}},"vertices":[]}"#, "\n",
);
let features = read_cityjsonseq(BufReader::new(seq.as_bytes()))?
    .collect::<cityjson_json::Result<Vec<_>>>()?;
// each element is an OwnedCityModel (CityJSONFeature) with the header transform merged in
# Ok::<(), cityjson_json::Error>(())
```

Write a strict `CityJSONSeq` stream. Supply a `CityJSON` base root and one or more feature models; the builder quantizes vertices and computes the geographical extent:

```rust
use cityjson_json::{from_str_owned, from_feature_str_with_base, write_cityjsonseq};

let base_input = r#"{"type":"CityJSON","version":"2.0","transform":{"scale":[1.0,1.0,1.0],"translate":[0.0,0.0,0.0]},"CityObjects":{},"vertices":[]}"#;
let base_root = from_str_owned(base_input)?;
let feature = from_feature_str_with_base(
    r#"{"type":"CityJSONFeature","id":"f1","CityObjects":{"f1":{"type":"Building","geometry":[{"type":"MultiPoint","boundaries":[0]}]}},"vertices":[[10,20,30]]}"#,
    base_input,
)?;

let mut output: Vec<u8> = Vec::new();
let report = write_cityjsonseq(&base_root, [&feature])
    .auto_transform([0.001, 0.001, 0.001])
    .write(&mut output)?;
// report.feature_count == 1, report.geographical_extent covers all feature vertices
# Ok::<(), cityjson_json::Error>(())
```

## Documentation

- [CityJSON 2.0](https://www.cityjson.org/specs/2.0.1/)
- [`cityjson` crate documentation](https://docs.rs/cityjson/)
- [Design notes](docs/design.md)
- [Development guide](docs/development.md)

todo: link to docs.rs

## Library Layout

| Module   | Contents                                                                                       |
|----------|------------------------------------------------------------------------------------------------|
| `v2_0`   | `CityJSON` 2.0 (de)serialization entry points, feature-stream helpers, and `SerializableCityModel` |
| `errors` | `Error` and `Result` types surfaced by the adapter                                             |
| (root)   | Convenience re-exports: `from_str_*`, `as_json`, `write_cityjsonseq`, model types              |

Core types re-exported from `cityjson`:

- **`OwnedCityModel`**: A `CityJSON` model with owned `String` storage. Self-contained and doesn't depend on external lifetimes.
- **`BorrowedCityModel`**: A `CityJSON` model with borrowed string references. More memory efficient but requires careful lifetime management.

## Design

The adapter is optimized around a small number of core decisions:

- deserialization is split into root preparation and streamed model construction
- `CityObjects` and geometry boundaries avoid large intermediate JSON structures
- attributes deserialize directly into backend value types
- serialization streams from the model with a shared write context instead of building a DOM first
- both owned and borrowed string storage are supported through the same parsing pipeline

The full design description lives in [docs/design.md](docs/design.md).

## API Stability

This crate follows semantic versioning (`MAJOR.MINOR.PATCH`):

- `MAJOR`: incompatible API changes
- `MINOR`: backwards-compatible feature additions
- `PATCH`: backwards-compatible fixes

## Minimum Rust Version

The minimum supported rustc version is `1.93.0`.

## Development

Development setup, test configuration, and benchmark workflow are documented in
[docs/development.md](docs/development.md).

## Contributing

Contributions are welcome in all forms.
Please open an issue to discuss any potential changes before working on a patch.
You can submit LLM-generated PRs for bug fixes and documentation improvements.
Regardless of handwritten or LLM-generated code, the PR should follow these guidelines:

- relatively small, focused changes, otherwise I won't be able to review it,
- follow the existing style and conventions,
- include unit tests and documentation for new features and bug fixes,
- the patched code should pass:
  - `just ci`
- if you remove or merge tests or examples or benchmarks, please explain why and update the documentation accordingly.

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
Code correctness and performance are verified by carefully curated test cases and benchmarks that cover the `CityJSON` 2.0 specification.

## Roadmap

There are no major features planned for the near future, beyond bug fixes, test coverage, performance optimization, and  documentation improvements.
