# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development commands

```shell
# Check code
just check

# Build
cargo build

# Run linter
just lint

# Run tests and examples
just test

# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Build documentation
just doc

# Run benchmarks
cargo bench
```

### Advanced Development Commands

```shell
# Memory profiling with massif (heap allocations)
just profile-massif

# Cache profiling with cachegrind (cache misses, branch prediction)
just profile-cachegrind

# Memory leak detection with memcheck
just profile-memcheck

# Run all profiling tools
just profile-all

# Run benchmarks and save baseline for comparison
just bench-baseline my-baseline-name

# Compare current performance against a baseline
just bench-compare my-baseline-name

# Track benchmark results with description
just bench-track "description of changes"

# View recent benchmark history
just bench-history

# Open Criterion HTML report in browser
just bench-view

# Show latest benchmark results summary
just bench-summary

# Record comprehensive benchmark and profiling results
just bench-record "description"

# Export all benchmark results for archival
just bench-export

# Compare performance between two git commits
just bench-compare-commits commit1 commit2
```

## Architecture Overview

This crate represents CityJSON the data model in Rust and provides setter and getter methods for it.
_cityjson-rs_ is a foundational layer for libraries and software that implement specialized operations, such as serialization, indexing, domain-specific logic, etc.
There are three guiding principles:

1. Performance: Flattened geometry representation and global resource pools for cache-locality
2. Flexibility: Support for multiple vertex index types (`u16`, `u32`, `u64`) and owned or borrowed strings
3. Multiple versions: Multiple CityJSON versions are supported and the crate is extensible for future versions

### Module Structure

**Core Architecture:**
- **`cityjson`** module: Version-agnostic traits and types forming the stable API
  - `core/`: Concrete implementations (geometry, boundaries, coordinates, attributes, metadata, etc.)
  - `traits/`: Interfaces for all major components
- **Version modules** (`v1_0`, `v1_1`, `v2_0`): Implement cityjson traits for specific CityJSON versions
- **`resources`** module: Resource management utilities
  - `pool/`: Resource pool interface and default implementation
  - `mapping/`: Geometry-to-resource mappings (semantics, materials, textures)
  - `storage/`: String storage (owned and borrowed)

**Key Design Patterns:**
- Generic over vertex references (`VR: VertexRef`), resource references (`RR: ResourceRef`), and string storage (`SS: StringStorage`)
- Flattened vs nested boundary representations - flattened for performance, nested for JSON compatibility
- Builder pattern via `GeometryBuilder` for constructing geometries and adding them to the CityModel and handling the resource references
- Trait-based design allowing multiple CityJSON version implementations
- **Flattened attribute system**: Structure of Arrays (SoA) design for attributes, optimized for columnar storage formats like Parquet

### Flattened Attributes API

Attributes (properties of CityObjects, Semantic surfaces, etc.) use a flattened Structure of Arrays design:

- **`AttributePool`**: Global pool storing all attributes in separate type-specific arrays
- Each attribute type (bool, integer, float, string, vector, map, geometry) is stored in its own array
- Attributes are referenced by ID (`AttributeId32`) rather than stored inline
- This design enables efficient Parquet serialization and reduces memory fragmentation

**Creating and using attributes:**

```rust
use cityjson::cityjson::core::attributes::AttributeOwnerType;
use cityjson::prelude::*;
use cityjson::v1_1::*;

// The attribute pool is now a member of CityModel
let mut city_model: CityModel = CityModel::new(CityModelType::CityJSON);

// Add attributes to the global pool
let height_id = city_model.attributes_mut().add_float(
    "measuredHeight".to_string(),
    true,  // is_named
    22.3,
    AttributeOwnerType::CityObject,
    None,
);

// Link attribute to CityObject
let mut city_object = CityObject::new("building-1".to_string(), CityObjectType::Building);
city_object.attributes_mut().insert("measuredHeight".to_string(), height_id);
city_model.cityobjects_mut().add(city_object);

// Retrieve values from the pool
if let Some(height) = city_model.attributes().get_float(height_id) {
    println!("Height: {}", height);
}
```

**Nested attributes (Maps and Vectors):**

```rust
use std::collections::HashMap;

// Create nested structure: address with location geometry
let mut address_map = HashMap::new();

let country_id = city_model.attributes_mut().add_string(
    "Country".to_string(), true, "Canada".to_string(),
    AttributeOwnerType::Element, None,
);
address_map.insert("Country".to_string(), country_id);

// Add geometry to address using GeometryBuilder
let location_geometry_ref = geometry_builder.build()?;
let location_id = city_model.attributes_mut().add_geometry(
    "location".to_string(), true, location_geometry_ref,
    AttributeOwnerType::Element, None,
);
address_map.insert("location".to_string(), location_id);

// Create the address map attribute
let address_id = city_model.attributes_mut().add_map(
    "address".to_string(), false, address_map,
    AttributeOwnerType::Element, None,
);

// Wrap in vector (CityJSON allows multiple addresses)
let addresses_vec_id = city_model.attributes_mut().add_vector(
    "address".to_string(), true, vec![address_id],
    AttributeOwnerType::CityObject, None,
);

// Link to CityObject
city_object.attributes_mut().insert("address".to_string(), addresses_vec_id);
```

### Using Attributes During Geometry Construction

When building geometries with `GeometryBuilder`, you have access to the global attribute pool through the model reference:

```rust
use cityjson::prelude::*;
use cityjson::v1_1::*;
use cityjson::cityjson::core::attributes::AttributeOwnerType;

let mut city_model: CityModel = CityModel::new(CityModelType::CityJSON);

// Add semantic attributes BEFORE creating GeometryBuilder
let material_id = city_model.attributes_mut().add_string(
    "material".to_string(),
    true,
    "concrete".to_string(),
    AttributeOwnerType::Semantic,
    None,
);

let year_id = city_model.attributes_mut().add_integer(
    "year_constructed".to_string(),
    true,
    1985,
    AttributeOwnerType::Semantic,
    None,
);

// Create semantic and attach attributes
let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
roof_semantic.attributes_mut().insert("material".to_string(), material_id);
roof_semantic.attributes_mut().insert("year_constructed".to_string(), year_id);

// Use semantic in geometry (through GeometryBuilder or other means)
```

### Important Usage Pattern: Ordering Matters

Because Rust enforces exclusive mutable borrows, follow this pattern when working with attributes:

**✅ CORRECT - Add to pool first, then use references:**
```rust
// 1. Add attribute to pool (borrows city_model.attributes)
let height_id = city_model.attributes_mut().add_float(
    "height".to_string(), true, 42.0,
    AttributeOwnerType::CityObject, None,
);
// Borrow ends here

// 2. Get object and use attribute ID (new borrow of city_model.cityobjects)
let city_object = city_model.cityobjects_mut().get_mut(co_ref).unwrap();
city_object.attributes_mut().insert("height".to_string(), height_id);
```

**❌ INCORRECT - Borrowing conflicts:**
```rust
// This won't compile:
let city_object = city_model.cityobjects_mut().get_mut(co_ref).unwrap(); // borrows city_model
let height_id = city_model.attributes_mut().add_float(...); // ERROR: already borrowed!
city_object.attributes_mut().insert("height".to_string(), height_id);
```

**For Multiple Attributes:**
```rust
// Add all attributes first
let height_id = city_model.attributes_mut().add_float(...);
let width_id = city_model.attributes_mut().add_float(...);
let name_id = city_model.attributes_mut().add_string(...);

// Then attach to objects
let city_object = city_model.cityobjects_mut().get_mut(co_ref).unwrap();
let attrs = city_object.attributes_mut();
attrs.insert("height".to_string(), height_id);
attrs.insert("width".to_string(), width_id);
attrs.insert("name".to_string(), name_id);
```

### Usage Patterns

Recommended imports:

```rust
use cityjson::prelude::*;
use cityjson::cityjson::core::attributes::AttributeOwnerType;
use cityjson::v1_1::*;  // or v1_0, v2_0
```

**Creating geometries:** Always use `GeometryBuilder` rather than manual boundary construction

**Managing attributes:** The `AttributePool` is a member of `CityModel`. Add attributes using `city_model.attributes_mut()`, and reference them by ID from CityObjects, Semantics, etc.

**Ordering pattern:** Always add attributes to the pool before attaching them to objects to avoid borrow checker conflicts

**Error handling:** All operations return `Result<T>` - handle errors appropriately

## Important Notes

- Minimum Rust version: 1.85.0 (MSRV)
- Platform support: 64-bit platforms only (x86_64, aarch64, etc.)
- Use `just` for common development tasks
- Extensions: Supports CityJSON Extensions through extensible types
- Serialization: Serialization is implemented in a separate, dependent crate

## Backend Features

The crate supports two backend implementations via Cargo features:

- `backend-default` (default): Flattened representation optimized for performance and memory efficiency
- `backend-nested`: JSON-like nested structure for 1:1 JSON compatibility
- `backend-both`: Both backends compiled together for benchmarking and comparison

For detailed performance characteristics and when to use each backend, see the benchmarking results in `target/criterion/` after running `just bench`.

