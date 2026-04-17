# cityjson-json

`cityjson-json` is the `CityJSON` 2.0 JSON boundary crate around the [`cityjson`](https://crates.io/crates/cityjson) crate. It reads and writes owned `CityJSON` models with explicit document and feature-stream APIs.

It also exposes the JSON-aware helper layer consumed by `cityjson-lib`:

- `probe` and `RootKind` for cheap root sniffing
- `staged` reconstruction helpers for feature-plus-base workflows
- JSON-backed `cleanup`, `extract`, `append`, and `merge` helpers

## Benchmarks

Read benchmarks against `serde_json::Value` for acquired real-world data and
synthetic stress cases. The table below is a historical snapshot; refresh the
current suite with `just bench`.

<!-- benchmark-summary:start -->
**Acquired data**

| Case | cityjson-json | `serde_json::Value` | Factor |
| --- | --- | --- | --- |
| `io_basisvoorziening_3d_cityjson` | 282.9 MiB/s | 272.2 MiB/s | 1.04x |
| `io_3dbag_cityjson_cluster_4x` | 186.9 MiB/s | 324.4 MiB/s | 0.58x |
| `io_3dbag_cityjson` | 193.5 MiB/s | 340.5 MiB/s | 0.57x |

**Stress cases**

| Case | cityjson-json | `serde_json::Value` | Factor |
| --- | --- | --- | --- |
| `stress_attribute_heavy` | 179.9 MiB/s | 226.8 MiB/s | 0.79x |
| `stress_boundary_heavy` | 320.7 MiB/s | 213.5 MiB/s | 1.50x |
| `stress_geometry_heavy` | 280.8 MiB/s | 219.8 MiB/s | 1.28x |
| `stress_hierarchy_heavy` | 195.6 MiB/s | 233.1 MiB/s | 0.84x |
| `stress_resource_heavy` | 150.7 MiB/s | 228.2 MiB/s | 0.66x |
| `stress_vertex_heavy` | 363.1 MiB/s | 243.0 MiB/s | 1.49x |
<!-- benchmark-summary:end -->

Full benchmark tables and plots are written to `benches/results/benchmark_summary.md`.
Use `just bench-local /path/to/file-or-directory` for ad hoc local inputs without
rewriting this README snapshot.

## Installation

```shell
cargo add cityjson-json
```

## Getting Started

### Read A Document

```rust
use cityjson_json::v2_0::{ReadOptions, read_model};

let json_bytes = br#"{
  "type": "CityJSON",
  "version": "2.0",
  "transform": {"scale": [1.0, 1.0, 1.0], "translate": [0.0, 0.0, 0.0]},
  "CityObjects": {},
  "vertices": []
}"#;

let model = read_model(json_bytes, &ReadOptions::default())?;
# Ok::<(), cityjson_json::Error>(())
```

### Write A Document

```rust
use cityjson_json::v2_0::{WriteOptions, to_vec};

let bytes = to_vec(&model, &WriteOptions::default())?;
# Ok::<(), cityjson_json::Error>(())
```

### `CityJSONSeq`

Read a newline-delimited `CityJSONSeq` stream. The first item must be a
`CityJSON` header; each subsequent item is a self-contained `CityJSONFeature`:

```rust
use std::io::Cursor;
use cityjson_json::v2_0::{ReadOptions, read_feature_stream};

let seq = concat!(
    r#"{"type":"CityJSON","version":"2.0","transform":{"scale":[0.001,0.001,0.001],"translate":[0.0,0.0,0.0]},"CityObjects":{},"vertices":[]}"#, "\n",
    r#"{"type":"CityJSONFeature","id":"f1","CityObjects":{"f1":{"type":"Building"}},"vertices":[]}"#, "\n",
);
let features = read_feature_stream(Cursor::new(seq.as_bytes()), &ReadOptions::default())?
    .collect::<cityjson_json::Result<Vec<_>>>()?;
// each element is an OwnedCityModel (CityJSONFeature) with the header transform merged in
# Ok::<(), cityjson_json::Error>(())
```

Write a strict `CityJSONSeq` stream from feature models. The writer derives the
header from their shared root state and quantizes vertices with explicit
options:

```rust
use cityjson_json::v2_0::{
    CityJsonSeqWriteOptions, FeatureStreamTransform, ReadOptions, read_feature_with_base,
    read_model, write_feature_stream,
};
use cityjson::v2_0::Transform;

let base_input = br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#;
let base = read_model(base_input, &ReadOptions::default())?;
let feature = read_feature_with_base(
    br#"{"type":"CityJSONFeature","id":"f1","CityObjects":{"f1":{"type":"Building","geometry":[{"type":"MultiPoint","boundaries":[0]}]}},"vertices":[[10,20,30]]}"#,
    &base,
    &ReadOptions::default(),
)?;

let mut output: Vec<u8> = Vec::new();
let report = write_feature_stream(
    &mut output,
    [feature],
    &CityJsonSeqWriteOptions {
        transform: FeatureStreamTransform::Explicit(Transform::new()),
        ..CityJsonSeqWriteOptions::default()
    },
)?;
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
| `v2_0`   | `CityJSON` 2.0 read/write entry points and feature-stream helpers |
| `errors` | `Error` and `Result` types surfaced by the adapter                                             |
| (root)   | Convenience re-exports: `read_*`, `write_*`, option types, and model types                     |

Core types re-exported from `cityjson`:

- **`OwnedCityModel`**: A `CityJSON` model with owned `String` storage. Self-contained and doesn't depend on external lifetimes.

## Design

The adapter is optimized around a small number of core decisions:

- deserialization is split into root preparation and streamed model construction
- `CityObjects` and geometry boundaries avoid large intermediate JSON structures
- attributes deserialize directly into backend value types
- serialization streams from the model with a shared write context instead of building a DOM first
- the public JSON boundary is explicitly owned-model oriented

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
