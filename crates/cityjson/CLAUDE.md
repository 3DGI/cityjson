# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development commands

```shell
# Check code
just check

# Build
cargo build

# Run linter
just clippy

# Run tests
cargo test

# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Build documentation
just doc

# Run benchmarks
cargo bench
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
use cityjson::cityjson::core::attributes::{AttributeOwnerType, OwnedAttributePool};

// Create a shared attribute pool
let mut pool = OwnedAttributePool::new();

// Add attributes to the pool
let height_id = pool.add_float(
    "measuredHeight".to_string(),
    true,  // is_named
    22.3,
    AttributeOwnerType::CityObject,
    None,
);

// Link attribute to CityObject
let co_attrs = city_object.attributes_mut();
co_attrs.insert("measuredHeight".to_string(), height_id);

// Retrieve values from the pool
if let Some(height) = pool.get_float(height_id) {
    println!("Height: {}", height);
}
```

**Nested attributes (Maps and Vectors):**

```rust
// Create nested structure: address with location geometry
let mut address_map = HashMap::new();

let country_id = pool.add_string(
    "Country".to_string(), true, "Canada".to_string(),
    AttributeOwnerType::Element, None,
);
address_map.insert("Country".to_string(), country_id);

// Add geometry to address
let location_geometry_ref = geometry_builder.build()?;
let location_id = pool.add_geometry(
    "location".to_string(), true, location_geometry_ref,
    AttributeOwnerType::Element, None,
);
address_map.insert("location".to_string(), location_id);

// Create the address map attribute
let address_id = pool.add_map(
    "address".to_string(), false, address_map,
    AttributeOwnerType::Element, None,
);

// Wrap in vector (CityJSON allows multiple addresses)
let addresses_vec_id = pool.add_vector(
    "address".to_string(), true, vec![address_id],
    AttributeOwnerType::CityObject, None,
);
```

### Usage Patterns

Recommended imports:

```rust
use cityjson::prelude::*;
use cityjson::cityjson::core::attributes::{AttributeOwnerType, OwnedAttributePool};
use cityjson::v1_1::*;  // or v1_0, v2_0
```

**Creating geometries:** Always use `GeometryBuilder` rather than manual boundary construction

**Managing attributes:** Create an `AttributePool` at the start, add attributes to it, and reference them by ID from CityObjects

**Error handling:** All operations return `Result<T>` - handle errors appropriately

## Important Notes

- Minimum Rust version: 1.85.0 (MSRV)
- Platform support: 64-bit platforms only (x86_64, aarch64, etc.)
- Use `just` for common development tasks
- Extensions: Supports CityJSON Extensions through extensible types
- Serialization: Serialization is implemented in a separate, dependent crate

