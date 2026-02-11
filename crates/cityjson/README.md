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
use cityjson::prelude::*;
use cityjson::v2_0::*;

let model: CityModel<u32, OwnedStringStorage> = CityModel::new(CityModelType::CityJSON);
assert_eq!(model.version(), Some(CityJSONVersion::V2_0));
```

## Library Organization

- `cityjson`: version-agnostic core types (`attributes`, `boundary`, `coordinate`, `geometry`, `vertex`, etc.)
- `v2_0`: concrete `CityJSON` v2.0 model types and builders
- `resources`: typed handle + mapping + storage utilities
- `raw`: low-level read views for efficient downstream processing

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
