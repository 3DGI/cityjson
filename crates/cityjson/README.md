# cityjson-rs

`cityjson-rs` implements the [CityJSON 2.0](https://www.cityjson.org/specs/2.0.1/) data model in Rust.
This crate provides types and accessor methods for working with a flattened, columnar representation of the `CityJSON` data model.
`cityjson-rs` is meant to be a core library for downstream specialized libraries that implement serialization, indexing, geometry processing, and other features.

## Overview of downstream crates in the cityjson ecosystem

Serialization is implemented by:

- json: [cityjson-json](https://github.com/3DGI/cityjson-json)
- arrow: [cityjson-arrow](https://github.com/3DGI/cityjson-arrow)
- parquet: [cityjson-parquet](https://github.com/3DGI/cityjson-arrow/tree/master/cityjson-parquet)

For a higher-level library that integrates serialization, implements geometry processing, and other features into a single crate, see [cityjson-lib](https://github.com/3DGI/cityjson-lib).

For generating fake, schema-valid data for any combination of the `CityJSON` specs, see [cityjson-fake](https://github.com/3DGI/cityjson-fake).

For efficient indexing and querying individual `CityObjects` across multiple files, see [cityjson-index](https://github.com/3DGI/cityjson-index).

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

This crate follows the workspace contract. See
[`CONTRIBUTING.md`](../../CONTRIBUTING.md) for PR guidelines and
[`docs/development.md`](../../docs/development.md) for tooling, lints,
and release flow.

## License

Dual-licensed under MIT or Apache-2.0, at your option. See
[`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).

## Roadmap

There are no major features planned for the near future, beyond bug fixes, test coverage, documentation improvements.

