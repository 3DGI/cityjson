# cityjson-rs

The crate defines the types and methods for representing the complete CityJSON data model in Rust.
*cityjson-rs* is meant to be a core dependency in Rust-based CityJSON software, so that the dependent applications can extend the types with their specific functionality.
Therefore, *citjson-rs* is designed with performance, flexibility, and ease-of-use in mind.
The three criteria are implemented in the following features:

- The Geometry representation is flattened into densely packed containers to minimize allocations, improve cache-locality. This is very different to the nested arrays defined by the CityJSON schema. However, the implementation details are hidden from the API.
- Vertex indices, and consequently boundaries, semantics, and appearances can be specialized with either `u16`, `u32` or `u64` types to enable various use cases and memory optimizations.
- Supports both borrowed and owned values.
- Getter and setter methods are implemented for each CityJSON object and their members to provide a stable API and hide implementation details.
- The API is thoroughly documented, including usage examples.
- Supports CityJSON Extensions.
- Supports multiple CityJSON versions, such as v1.0, v1.1, v2.0, and it is extensible for future versions.

## Documentation

The [cityjson-rs]() documentation include a comprehensive description of the library, including usage examples.

## Installation

Add the `cityjson-rs` crate as a dependency to your project with cargo.

```shell
cargo add cityjson-rs
```

## Library organisation

### Core Structure

- **`cityjson`** module: Contains version-agnostic traits and types forming the stable API

  - Defines interfaces like CityModelTrait, CityObjectTrait, GeometryTrait
  - Contains version-independent types and functionality like coordinate representations, boundary models and attributes

- Version-specific modules (**`v1_0`**, **`v1_1`**, **`v2_0`**)

    - Implement the traits defined in the `cityjson` module
    - Provide concrete types for each CityJSON version

- **`resources`** module: Utilities for resource management

    - `pool`: Defines a resource pool pattern for efficient memory management   
    - `mapping`: Provides mapping between geometries and resources (semantics, materials, textures)
    - `storage`: Implements flexible string storage strategies (owned vs. borrowed)

### Prelude

The prelude re-exports the types and traits from the `cityjson` and `resources` modules.
The recommended way to use `cityjson-rs` is to use its prelude and one of the implemented CityJSON versions, for example v2.0.

```rust
use cityjson::prelude::*;
use cityjson::v2_0::*;
```

### Errors

The library defines custom errors in the `errors` module and uses Result types throughout for fallible operations.

## Performance

example: [https://github.com/rust-lang/regex?tab=readme-ov-file#performance](https://github.com/rust-lang/regex?tab=readme-ov-file#performance)

### Benchmarking

Run the full benchmark + profiling suite:

```sh
just perf "my run description"
```

Quick/fast mode:

```sh
just perf "quick check" mode=fast
```

Analyze results from `bench_results/history.csv`:

```sh
just perf-analyze description="my run description" plot=1
just perf-analyze series=1 plot=1 bench="builder/build_with_geometry" metric="time_ms"
```

## API Stability

This crate follows the semantic versioning system, such as `MAJOR.MINOR.PATCH`.

- `MAJOR` version is increased when there are incompatible API changes,
- `MINOR` version is increased when new functionality is added in a backwards-compatible manner
- `PATCH` version is increased when backwards-compatible bug fixes are made

Migration documentation is provided between major versions.

### Minimum Rust version policy

This crate's minimum supported rustc version is `1.65.0`.

The policy is that the minimum Rust version required to use this crate can be increased in minor version updates. For example, if cityjson-rs `1.0` requires Rust `1.20.0`, then cityjson-rs `1.0.z` for all values of `z` will also require Rust `1.20.0` or newer. However, regex `1.y` for `y > 0` may require a newer minimum version of Rust.
