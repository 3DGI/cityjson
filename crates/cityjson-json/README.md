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

Download test data first:

```bash
just download
cargo bench --no-run
```

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
