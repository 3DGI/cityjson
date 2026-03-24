# cityjson-rs

`cityjson-rs` provides types and accessors for the `CityJSON` data model in Rust.
The crate focuses on:

- efficient geometry storage (flattened boundary containers),
- typed resource handles (semantics/materials/textures/geometry),
- owned and borrowed string storage strategies,
- a stable public API centered on `CityJSON` v2.0 types.

JSON de/serialization and legacy version upgrades are handled in a separate crate (`serde_cityjson`).

## Documentation

- API docs: <https://docs.rs/cityjson>
- Bench and profiling guide: `BENCHMARK_GUIDE.md`

## Installation

```shell
cargo add cityjson
```

## Quick Start

```rust
use cityjson::v2_0::{CityJSONVersion, CityModel, CityModelType};

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
```

Note: for the common owned `CityJSON` v2.0 path, import from `cityjson::v2_0::*` directly. The `prelude` is intentionally narrow and only reexports crate-wide types, errors, storage strategies, and resource handles.

## Library Organization

- `v2_0`: the primary public `CityJSON` v2.0 API, including model types, builders, and reusable value types such as `Transform`, `Extension`, `Boundary`, and `VertexIndex`
- `resources`: typed handle + mapping + storage utilities
- `raw`: low-level read views for efficient downstream processing

Internal shared layers such as the old `cityjson::core` storage/domain split are implementation details and are not part of the public API surface. Downstream code should import `CityJSON` domain types from `cityjson::v2_0::*`; the prelude is only for crate-wide types, errors, storage strategies, and resource handles.

## Benchmarking

Run the full benchmark + profiling suite:

```sh
just perf "my run description"
```

Quick mode:

```sh
just perf "quick check" mode=fast
```

Analyze results from `bench_results/history.csv`:

```sh
just perf-analyze description="my run description" --plot
just perf-analyze --backend-overview --backend default --mode all
just perf-analyze --series --plot bench="builder/build_with_geometry" metric="time_ms"
```

## API Stability

This crate follows semantic versioning (`MAJOR.MINOR.PATCH`):

- `MAJOR`: incompatible API changes
- `MINOR`: backwards-compatible feature additions
- `PATCH`: backwards-compatible fixes

## Minimum Rust Version

The minimum supported rustc version is `1.93.0`.

## License

Licensed under either:

- Apache License, Version 2.0 (`LICENSE-APACHE`)
- MIT license (`LICENSE-MIT`)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in cityjson-rs by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without additional terms or conditions.
