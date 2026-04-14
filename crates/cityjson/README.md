# cityjson-rs

`cityjson-rs` implements the [CityJSON 2.0](https://www.cityjson.org/specs/2.0.1/) data model in Rust.

Serialization is implemented by downstream crates:
- json: [cityjson-json]
- arrow: [cityjson-arrow]
- parquet: [cityjson-parquet]

## Installation

```shell
cargo add cityjson
```

## Getting Started

### Imports

```rust
use cityjson::v2_0::*;     // all CityJSON v2.0 types
use cityjson::prelude::*;  // handles, storage strategies, error types
```

The `prelude` re-exports crate-wide types (handles, errors, storage strategies) but not the cityjson-domain types from `v2_0`.

### Example

```rust
use cityjson::v2_0::{CityJSONVersion, CityModel, CityModelType};

fn main() {
    let model = CityModel::<u32>::new(CityModelType::CityJSON);

    assert_eq!(model.version(), Some(CityJSONVersion::V2_0));
    assert!(model.cityobjects().is_empty());

    assert_eq!(model.iter_geometries().count(), 0);
    assert_eq!(model.iter_geometry_templates().count(), 0);
    assert!(model.template_vertices().is_empty());
    assert_eq!(model.iter_semantics().count(), 0);
    assert_eq!(model.iter_materials().count(), 0);
    assert_eq!(model.iter_textures().count(), 0);
    assert!(model.vertices_texture().is_empty());

    assert!(model.vertices().is_empty());
    assert_eq!(model.transform(), None);

    assert_eq!(model.metadata(), None);
    assert_eq!(model.extra(), None);
    assert_eq!(model.extensions(), None);
}
```

## Data Model

`cityjson-rs` uses a flat, columnar internal representation that differs from the nested JSON structure of the `CityJSON` specification:

- **Geometry boundaries**: stored as sibling offset arrays (`surfaces`, `shells`, `solids` with corresponding vertex/ring/surface/shell offsets) instead of nested coordinate arrays
- **Resource pools**: global pools for semantics, materials, textures, and UV coordinates; geometry-local maps store handle references into these pools instead of dense array indices
- **Semantic and material assignments**: flat primitive-assignment arrays (one per surface/point/linestring) instead of nested index structures
- **Texture assignments**: per-ring resource references with flat UV coordinate arrays, rather than nested per-ring texture entries

This representation is more efficient for traversal and serialization to columnar formats (Arrow, Parquet) while maintaining round-trip fidelity with the spec format. See [Geometry Mappings](docs/dev/geometry_mappings.md) for detailed layout rules and examples.

## Documentation

- [CityJSON 2.0](https://www.cityjson.org/specs/2.0.1/)
- [Geometry Boundaries, Semantics, and Appearance](docs/dev/geometry_mappings.md)

todo: link to docs.rs

## Library Layout

| Module      | Contents                                                                                                                            |
|-------------|-------------------------------------------------------------------------------------------------------------------------------------|
| `v2_0`      | Domain types: `CityModel`, `CityObject`, `Geometry`, `GeometryDraft`, `Metadata`, `Transform`, `Semantic`, `Material`, `Texture`, … |
| `resources` | Typed handles, resource pools, and string storage strategies                                                                        |
| `raw`       | Zero-copy read views for use in downstream serializers                                                                              |

## API Stability

This crate follows semantic versioning (`MAJOR.MINOR.PATCH`):

- `MAJOR`: incompatible API changes
- `MINOR`: backwards-compatible feature additions
- `PATCH`: backwards-compatible fixes

## Minimum Rust Version

The minimum supported rustc version is `1.93.0`.

## Contributing

todo: add contributing guidelines

## License

Licensed under either:

- Apache License, Version 2.0 (`LICENSE-APACHE`)
- MIT license (`LICENSE-MIT`)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
cityjson-rs by you, as defined in the Apache-2.0 license, shall be dual licensed as above,
without additional terms or conditions.

## Use of AI in this project

This crate was originally developed without the use of AI.
Since then, it underwent multiple significant refactors and various LLM models (Claude, `ChatGPT`) were used for experimenting with alternative designs, in particular for the resource pool and attribute storage strategies.
LLM generated code is also used for improving the test coverage and documentation and mechanical improvements.
Code correctness and performance are verified by carefully curated test cases and benchmarks that cover the entire `CityJSON` 2.0 specification.