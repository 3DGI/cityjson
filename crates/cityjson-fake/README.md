# cjfake

Generate fake [CityJSON](https://www.cityjson.org/) data for testing purposes.

## Problem

3D city models are commonly encoded with CityJSON. Applications that process these models need test data, but real-world datasets have several limitations:

- Files are often large and slow to download/process during testing
- Models contain much irrelevant information for specific test cases  
- Certain CityObject types (e.g. Tunnels, CityFurniture) are rare or nonexistent
- Advanced features like Appearances or Geometry-templates are rarely modeled

## Goals

cjfake aims to:

- Generate valid CityJSON test data quickly and efficiently
- Allow precise control over the model contents and structure
- Support the complete CityJSON specification
- Produce models that pass validation with [cjval](https://github.com/cityjson/cjval)

Note: While the generated CityJSON is schema-valid, the geometric values are random and do not represent valid real-world objects.

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
use cjfake::{CityModelBuilder, CJFakeConfig};

// Create a basic CityJSON model
let model = CityModelBuilder::default().build();

// Customize the generation
let config = CJFakeConfig::default();
let model = CityModelBuilder::new(config, None)
    .metadata(None)
    .vertices()
    .materials(None)
    .textures(None)
    .attributes()
    .cityobjects()
    .build();
```

See the [API documentation](https://docs.rs/cjfake) for more details.

### Command Line Interface

The CLI provides fine-grained control over the generated content:

```bash
# Generate a basic CityJSON model
cjfake > output.city.json

# Generate model with specific CityObject types
cjfake --allowed-types-cityobject Building,Bridge > output.city.json

# Control number of objects and vertices
cjfake --min-cityobjects 5 --max-cityobjects 10 --min-vertices 4 --max-vertices 20 > output.city.json
```

Key configuration options:

- `--allowed-types-cityobject` - Restrict to specific CityObject types
- `--allowed-types-geometry` - Restrict to specific geometry types  
- `--min/max-cityobjects` - Control number of CityObjects
- `--cityobject-hierarchy` - Enable/disable parent-child relationships
- `--min/max-vertices` - Control number of vertices
- `--min/max-members-*` - Control number of geometry components
- `--min/max-materials` - Control number of materials
- `--min/max-textures` - Control number of textures
- `--use-templates` - Enable geometry templates
- `--texture-allow-none` - Allow null values in texture coordinates

Run `cjfake --help` to see all available options.

## License

This project is licensed under [Apache-2.0].

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
