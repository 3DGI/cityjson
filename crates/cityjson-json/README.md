# serde_cityjson

`serde_cityjson` is a CityJSON v2.0 serde adapter around the [`cityjson`](https://crates.io/crates/cityjson) crate. It provides efficient serialization and deserialization of CityJSON documents with both owned and borrowed string storage options.

## Features

- **v2.0 Support**: Full support for CityJSON v2.0 specification
- **Flexible Memory Models**: Choose between owned deserialization (`from_str_owned`) for simplicity or borrowed deserialization (`from_str_borrowed`) for performance
- **Efficient Serialization**: Convert models back to JSON with optional validation
- **Zero-Copy Parsing**: Borrowed deserialization maintains references to the original input

## Quick Start

Add `serde_cityjson` to your `Cargo.toml`:

```toml
[dependencies]
serde_cityjson = "0.4"
```

### Owned Deserialization

For simple use cases, deserialize into an owned model:

```rust
use serde_cityjson::from_str_owned;

let json_str = r#"{
  "type": "CityJSON",
  "version": "2.0",
  "CityObjects": {},
  "vertices": []
}"#;

let model = from_str_owned(json_str)?;
```

### Borrowed Deserialization

For performance-critical applications, use borrowed deserialization to avoid allocations:

```rust
use serde_cityjson::from_str_borrowed;

let json_str = r#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#;
let model = from_str_borrowed(json_str)?;
// model holds references to json_str
```

### Serialization

Serialize models back to JSON:

```rust
use serde_cityjson::to_string;

let json_output = to_string(&model)?;
```

For validated serialization (checks default theme references):

```rust
use serde_cityjson::to_string_validated;

let json_output = to_string_validated(&model)?;
```

## API Overview

### Core Functions

- **`from_str_owned(input: &str) -> Result<OwnedCityModel>`**: Parse JSON into an owned model. Use this when you need simple, self-contained data structures.

- **`from_str_borrowed(input: &str) -> Result<BorrowedCityModel>`**: Parse JSON into a borrowed model. Use this for performance-critical code where the model lifetime doesn't exceed the input string.

- **`to_string(model: &SerializableCityModel) -> Result<String>`**: Serialize to JSON. Fast path that doesn't validate theme references.

- **`to_string_validated(model: &SerializableCityModel) -> Result<String>`**: Serialize to JSON with validation. Ensures all default theme names reference existing themes.

### Model Types

- **`OwnedCityModel`**: A CityJSON model with owned String storage. Self-contained and doesn't depend on external lifetimes.

- **`BorrowedCityModel`**: A CityJSON model with borrowed string references. More memory efficient but requires careful lifetime management.

Both types implement the same interface through the underlying `cityjson::v2_0::CityModel`.

## Validation Policy

The library provides two serialization paths to balance performance and safety:

- **`to_string()`**: The fast path. Does not validate that default theme names (for materials and textures) actually reference existing themes in the appearance section.

- **`to_string_validated()`**: The strict path. Validates default theme references before serialization to ensure document consistency.

Use `to_string_validated()` when you need guaranteed valid CityJSON output, especially when serializing user-provided models.

## Development

### Running Tests

```bash
cargo test
cargo test --test v2_0
```

### Running Benchmarks

The benchmark corpus is migrating to the shared `cityjson-benchmarks`
repository. Until the shared release index is available, this crate keeps a
local bootstrap copy of the current benchmark inputs.

Download the local test data first:

```bash
just download
just bench-read
just bench-write
just bench-report
```

The benchmarks use Criterion. Read throughput is based on input bytes and write
throughput is based on output bytes. Synthetic cases are generated
deterministically from the profiles in `tests/data/generated/`, which mirrors
the current shared-corpus profile catalog.

### Read Benchmarks

| Case | Description | serde_cityjson | serde_json::Value | Factor |
| --- | --- | --- | --- | --- |
| 3D Basisvoorziening | Large real-world dataset dominated by geometry flattening and vertex import | owned 821.944 ms (453.9 MiB/s) | 1.242 s (300.5 MiB/s) | 0.66x |
| 3DBAG | Real-world medium-size dataset with two geometries per object and parent-child links | owned 35.435 ms (205.1 MiB/s); borrowed 34.607 ms (210.0 MiB/s) | 23.360 ms (311.2 MiB/s) | 1.52x |
| attribute_tree_worst_case | Deep nested attributes with minimal geometry work | owned 27.007 ms (206.2 MiB/s); borrowed 21.911 ms (254.2 MiB/s) | 19.336 ms (288.0 MiB/s) | 1.40x |
| composite_value_favorable_worst_case | Mixed geometry and normalization workload that is smaller but denser | owned 14.500 ms (239.4 MiB/s); borrowed 13.192 ms (263.2 MiB/s) | 13.333 ms (260.4 MiB/s) | 1.09x |
| deep_boundary_stress | Solid-heavy geometry that exercises nested boundary flattening | owned 8.339 ms (317.1 MiB/s); borrowed 8.336 ms (317.2 MiB/s) | 10.190 ms (259.5 MiB/s) | 0.82x |
| geometry_flattening_best_case | Large MultiSurface payload with no relation graph or attribute tree | owned 40.135 ms (331.1 MiB/s); borrowed 39.853 ms (333.5 MiB/s) | 51.264 ms (259.2 MiB/s) | 0.78x |
| relation_graph_worst_case | Dense parent-child graph with small geometry payloads | owned 7.399 ms (293.6 MiB/s); borrowed 7.301 ms (297.6 MiB/s) | 6.823 ms (318.4 MiB/s) | 1.08x |
| vertex_transform_stress | Large vertex pool with very little object-level normalization | owned 2.413 ms (296.1 MiB/s); borrowed 2.385 ms (299.5 MiB/s) | 2.216 ms (322.4 MiB/s) | 1.09x |

### Write Benchmarks

| Case | Description | serde_cityjson | serde_json::Value | Factor |
| --- | --- | --- | --- | --- |
| 3D Basisvoorziening | Large real-world dataset dominated by geometry flattening and vertex import | to_string 2.056 s (181.3 MiB/s); to_string_validated 2.058 s (181.2 MiB/s) | 505.770 ms (737.1 MiB/s) | 4.07x |
| 3DBAG | Real-world medium-size dataset with two geometries per object and parent-child links | to_string 45.950 ms (152.6 MiB/s); to_string_validated 48.937 ms (143.3 MiB/s) | 10.997 ms (637.8 MiB/s) | 4.18x |
| appearance_and_validation_stress | Serializer-heavy case with materials, textures, templates, and semantics | to_string 7.187 ms (219.5 MiB/s); to_string_validated 7.451 ms (211.7 MiB/s) | 1.840 ms (857.2 MiB/s) | 3.91x |
| attribute_tree_worst_case | Deep nested attributes with minimal geometry work | to_string 57.078 ms (97.6 MiB/s); to_string_validated 56.207 ms (99.1 MiB/s) | 10.384 ms (536.3 MiB/s) | 5.50x |
| composite_value_favorable_worst_case | Mixed geometry and normalization workload that is smaller but denser | to_string 25.851 ms (134.3 MiB/s); to_string_validated 25.928 ms (133.9 MiB/s) | 7.169 ms (484.3 MiB/s) | 3.61x |
| deep_boundary_stress | Solid-heavy geometry that exercises nested boundary flattening | to_string 14.376 ms (183.9 MiB/s); to_string_validated 14.462 ms (182.9 MiB/s) | 4.257 ms (621.2 MiB/s) | 3.38x |
| geometry_flattening_best_case | Large MultiSurface payload with no relation graph or attribute tree | to_string 78.413 ms (169.5 MiB/s); to_string_validated 98.400 ms (135.1 MiB/s) | 25.328 ms (524.7 MiB/s) | 3.10x |
| relation_graph_worst_case | Dense parent-child graph with small geometry payloads | to_string 11.547 ms (188.2 MiB/s); to_string_validated 10.369 ms (209.5 MiB/s) | 2.686 ms (808.8 MiB/s) | 4.30x |
| vertex_transform_stress | Large vertex pool with very little object-level normalization | to_string 3.268 ms (218.6 MiB/s); to_string_validated 3.150 ms (226.8 MiB/s) | 773.753 us (923.3 MiB/s) | 4.22x |

### Code Quality

```bash
cargo fmt
cargo check --all-features
```

## Dependencies

- **cityjson**: Core CityJSON v2.0 data structures and validation
- **serde**: Serialization framework
- **serde_json**: JSON parsing and generation
- **serde_json_borrow**: Zero-copy JSON parsing for borrowed deserialization

## License

This crate is part of the serde-cityjson project.

## See Also

- [CityJSON Specification](https://www.cityjson.org/)
- [cityjson-rs crate documentation](https://docs.rs/cityjson/)
- [Shared corpus migration plan](docs/shared-corpus-migration-plan.md)
