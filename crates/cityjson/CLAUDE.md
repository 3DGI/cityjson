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
  - `core/`: Concrete implementations (geometry, boundaries, coordinates, metadata, etc.)
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


### Usage Patterns

Recommended imports:

```rust
use cityjson::prelude::*;
use cityjson::v1_1::*;  // or v1_0, v2_0
```

Creating geometries: Always use `GeometryBuilder` rather than manual boundary construction

Error handling: All operations return `Result<T>` - handle errors appropriately

## Important Notes

- Minimum Rust version: 1.85.0 (MSRV)
- Platform support: 64-bit platforms only (x86_64, aarch64, etc.)
- Use `just` for common development tasks
- Extensions: Supports CityJSON Extensions through extensible types
- Serialization: Serialization is implemented in a separate, dependent crate

