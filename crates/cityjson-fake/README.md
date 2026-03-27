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
let model: CityModel<u32, OwnedStringStorage> = CityModelBuilder::default ().build();
assert_eq!(model.cityobjects().len(), 1);

// Generate a serialized CityJSON document
let json = cjfake::generate_string(CJFakeConfig::default (), Some(42)).unwrap();
assert!(json.starts_with('{'));
```

See the [API documentation](https://docs.rs/cjfake) for more details.

### Command Line Interface

The CLI exposes the same generation controls as the library API. The top-level command is:

```bash
cjfake [OPTIONS]
```

Common examples:

```bash
# Write a single document to stdout
cjfake > output.city.json

# Restrict the generated CityObject types
cjfake --allowed-types-cityobject Building,Bridge > output.city.json

# Write a single document to a file
cjfake --output output.city.json

# Write multiple documents into a directory
cjfake --count 3 --output out/
```

The available options are grouped below.

| Group       | Flags                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
|-------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| General     | `--seed`, `--output`, `--count`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| CityObjects | `--allowed-types-cityobject`, `--min-cityobjects`, `--max-cityobjects`, `--cityobject-hierarchy`, `--min-children`, `--max-children`                                                                                                                                                                                                                                                                                                                                                                                                                       |
| Geometry    | `--allowed-types-geometry`, `--allowed-lods`, `--min-members-multipoint`, `--max-members-multipoint`, `--min-members-multilinestring`, `--max-members-multilinestring`, `--min-members-multisurface`, `--max-members-multisurface`, `--min-members-solid`, `--max-members-solid`, `--min-members-multisolid`, `--max-members-multisolid`, `--min-members-compositesurface`, `--max-members-compositesurface`, `--min-members-compositesolid`, `--max-members-compositesolid`, `--min-members-cityobject-geometries`, `--max-members-cityobject-geometries` |
| Vertices    | `--min-coordinate`, `--max-coordinate`, `--min-vertices`, `--max-vertices`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| Materials   | `--materials-enabled`, `--min-materials`, `--max-materials`, `--nr-themes-materials`, `--generate-ambient-intensity`, `--generate-diffuse-color`, `--generate-emissive-color`, `--generate-specular-color`, `--generate-shininess`, `--generate-transparency`                                                                                                                                                                                                                                                                                              |
| Textures    | `--textures-enabled`, `--min-textures`, `--max-textures`, `--nr-themes-textures`, `--max-vertices-texture`, `--texture-allow-none`                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Templates   | `--use-templates`, `--min-templates`, `--max-templates`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| Metadata    | `--metadata-enabled`, `--metadata-geographical-extent`, `--metadata-identifier`, `--metadata-reference-date`, `--metadata-reference-system`, `--metadata-title`, `--metadata-point-of-contact`                                                                                                                                                                                                                                                                                                                                                             |
| Attributes  | `--attributes-enabled`, `--min-attributes`, `--max-attributes`, `--attributes-max-depth`, `--attributes-random-keys`, `--attributes-random-values`                                                                                                                                                                                                                                                                                                                                                                                                         |
| Semantics   | `--semantics-enabled`, `--allowed-types-semantic`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |

Use `cjfake --help` for the exact defaults and the full clap-generated help text.

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
