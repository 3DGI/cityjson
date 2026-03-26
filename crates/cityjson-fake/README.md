# cjfake

Generate fake [CityJSON](https://www.cityjson.org/) data for testing.

## Problem

3D city models are commonly encoded with CityJSON. Applications that process these models need test data, but real-world datasets have several limitations:

- Files are often large and slow to download/process during testing
- Models contain much irrelevant information for specific test cases  
- Certain CityObject types (e.g. Tunnels, CityFurniture) are rare or nonexistent
- Advanced features like Appearances or Geometry-templates are rarely modeled

## What It Does

cjfake aims to:

- Generate valid CityJSON test data quickly and efficiently
- Allow precise control over the model contents and structure
- Support the CityJSON v2.0 generation surface used in tests
- Produce models that pass validation with [cjval](https://github.com/cityjson/cjval)

It can generate:

- CityObjects and parent/child hierarchies
- Geometry types, LoDs, and geometry templates
- Vertices within configurable coordinate ranges
- Metadata, materials, textures, attributes, and semantics

The output is schema-valid CityJSON, but the geometry is random and not intended to represent real-world objects.

## Installation

Add cjfake to your `Cargo.toml`:

```toml
[dependencies]
cjfake = "0.1.0"
```

Or install the CLI tool:

```bash
cargo install cjfake
```

## Usage

### As a Library

```rust
use cjfake::prelude::*;

// Create a basic CityJSON model
let model: CityModel<u32, OwnedStringStorage> = CityModelBuilder::default().build();
assert_eq!(model.cityobjects().len(), 1);

// Generate a serialized CityJSON document
let json = cjfake::generate_string(CJFakeConfig::default(), Some(42)).unwrap();
assert!(json.starts_with('{'));
```

See the [API documentation](https://docs.rs/cjfake) for more details.

### Command Line Interface

The CLI provides fine-grained control over the generated content:

```bash
# Generate a basic CityJSON model
cjfake > output.city.json

# Generate model with specific CityObject types
cjfake --allowed-types-cityobject Building,Bridge > output.city.json

# Write directly to a file
cjfake --output output.city.json

# Generate multiple CityJSON files into a directory
cjfake --count 3 --output out/
```

Key configuration options:

- `--allowed-types-cityobject` - Restrict to specific CityObject types
- `--allowed-types-geometry` - Restrict to specific geometry types  
- `--min/max-cityobjects` - Control number of CityObjects
- `--cityobject-hierarchy` - Enable/disable parent-child relationships
- `--min/max-members-*` - Control geometry component counts
- `--min/max-materials` and `--min/max-textures` - Control appearance resources
- `--use-templates` - Enable geometry templates
- `--texture-allow-none` - Allow null values in texture coordinates
- `--output` - Write to a file or directory instead of stdout
- `--count` - Generate multiple documents

Run `cjfake --help` to see all available options.

## API Shape

The easiest entry points are:

- `cjfake::generate_model(config, seed)` for a `CityModel`
- `cjfake::generate_string(config, seed)` for a serialized CityJSON string
- `cjfake::generate_vec(config, seed)` for UTF-8 encoded bytes
- `CityModelBuilder` when you need fine-grained control over generation

## License

This project is licensed under [Apache-2.0].

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
