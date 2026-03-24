# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

```shell
# Check code (all targets, all features)
just check

# Build
cargo build

# Run linter (pedantic, all targets, all features)
just lint

# Run tests and examples
just test

# Run a single test
cargo test --all-features test_name

# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Build documentation (requires nightly)
just doc

# Run benchmarks
cargo bench
```

### Profiling Commands

```shell
# Profile a single benchmark workload (tool: massif|memcheck|cachegrind)
just profile-bench tool=massif

# Override which benchmark and ID to profile
PROFILE_BENCH=processor PROFILE_BENCH_ID=compute_full_feature_stats just profile-bench tool=massif

# Run all profiling tools on the benchmark
just profile-bench-all

# Comprehensive benchmarking and profiling
just perf "description of changes"

# Analyze benchmark and profiling results
just perf-analyze
```

## Architecture Overview

This crate represents the CityJSON data model in Rust and provides setter and getter methods for it.
`cityjson-rs` is a foundational layer for libraries that implement specialized operations such as serialization, indexing, and domain-specific logic.

Three guiding principles:
1. **Performance**: Flattened geometry representation and resource pools for cache locality
2. **Flexibility**: Generic over vertex index types (`u16`, `u32`, `u64`) and owned or borrowed strings
3. **Stable public API**: Centered on CityJSON v2.0 types

### Module Structure

**Public modules:**
- **`v2_0`**: The primary public API. Imports all domain types from here (`CityModel`, `GeometryDraft`, `CityObject`, `Geometry`, `Semantic`, `Boundary`, `Transform`, `Metadata`, etc.)
- **`resources`**: Typed handles, resource pool, mappings, and string storage strategies
  - `handles`: `CityObjectHandle`, `GeometryHandle`, `SemanticHandle`, `MaterialHandle`, `TextureHandle`, `TemplateGeometryHandle`
  - `mapping`: Geometry-to-resource mappings (`SemanticMap`, `MaterialMap`, `TextureMap`)
  - `storage`: `OwnedStringStorage`, `BorrowedStringStorage` — the two `StringStorage` implementations
- **`raw`**: Zero-copy read views over core containers for downstream serializers (e.g. Parquet/Arrow exporters)
- **`prelude`**: Narrow re-export of crate-wide types: `CityJSON`, `CityJSONVersion`, `CityModelType`, `Error`/`Result`, storage strategies, and resource handles — **not** domain types

**Internal (not public API):**
- `backend::default`: Concrete implementations of backend storage and geometry validation logic
- `cityjson::core`: Shared internal modules used to implement the versioned API

### Key Design Patterns

- `CityModel<VR, SS>` is generic over vertex index type (`VR: VertexRef`, e.g. `u32`) and string storage (`SS: StringStorage`, e.g. `OwnedStringStorage`). The shorthand `CityModel<u32>` uses the default `OwnedStringStorage`. Convenience aliases: `OwnedCityModel = CityModel<u32, OwnedStringStorage>`, `BorrowedCityModel<'a> = CityModel<u32, BorrowedStringStorage<'a>>`. Similar `Owned*`/`Borrowed*` aliases exist for `Semantic`, `Material`, `Texture`, `CityObjects`, `AttributeValue`, and `Attributes`.
- **Flattened boundary representation**: Geometries store boundaries in flat `Vec`s with offset counters for cache locality. Nested views are available for JSON compatibility via `boundary::nested`.
- **Direct insertion plus draft authoring**: Final stored geometry can be inserted directly with validation, while the optional `GeometryDraft` API provides nested authoring from raw coordinates and inserts in one shot. `GeometryInstance` authoring goes through `GeometryDraft::instance(...)`.
- **Resource pools**: Semantics, materials, and textures are stored in global resource pools on `CityModel` and referenced by typed handles (`SemanticHandle`, etc.).
- **Inline attributes (AoS)**: Attributes are stored directly on objects (`CityObject`, `Semantic`, etc.) as `HashMap<String, AttributeValue>` — not in a global pool. This avoids borrow-checker conflicts that arise with global pool designs.

### Attribute API

Attributes use an Array of Structures (AoS) design: each object owns its attributes inline.

```rust
use cityjson::v2_0::{CityModel, CityModelType, CityObject, CityObjectIdentifier, CityObjectType, OwnedAttributeValue};

let mut city_model = CityModel::<u32>::new(CityModelType::CityJSON);

let mut building = CityObject::new(
    CityObjectIdentifier::new("building-001".to_string()),
    CityObjectType::Building,
);
building.attributes_mut().insert("measuredHeight".to_string(), OwnedAttributeValue::Float(25.5));
building.attributes_mut().insert("yearOfConstruction".to_string(), OwnedAttributeValue::Integer(1985));
```

`AttributeValue` variants: `Bool`, `Integer`, `Float`, `String`, `Vector(Vec<AttributeValue>)`, `Map(HashMap<String, AttributeValue>)`, `Geometry(GeometryHandle)`.

### Recommended Imports

```rust
// Domain types (CityModel, GeometryDraft, CityObject, Geometry, Semantic, etc.)
use cityjson::v2_0::*;

// Crate-wide utilities (handles, storage strategies, error types)
use cityjson::prelude::*;
```

Do **not** import from `cityjson::cityjson::core::*` — that is a private implementation module.

### Geometry Construction

```rust
use cityjson::v2_0::{GeometryDraft, OwnedCityModel, PointDraft};
use cityjson::CityModelType;

let mut model = OwnedCityModel::new(CityModelType::CityJSON);

let geometry_handle = GeometryDraft::multi_point(
    None,
    [
        PointDraft::new([0.0, 0.0, 0.0]),
        PointDraft::new([1.0, 0.0, 0.0]),
    ],
)
.insert_into(&mut model)?;
```

## Important Notes

- Minimum Rust version: 1.93.0 (MSRV); docs require nightly
- Platform support: 64-bit platforms only (x86_64, aarch64, etc.)
- The `raw` module is intended for downstream crates building custom serializers — use it to access zero-copy views without rebuilding nested structures
- JSON de/serialization is implemented in the separate `serde_cityjson` crate
- Profiling output is saved under `./profiling/<date>_<git-ref>/`
