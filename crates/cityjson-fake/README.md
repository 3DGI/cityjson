# cityjson-fake

Generate fake [CityJSON](https://www.cityjson.org/) data for testing.

## Problem

3D city models are commonly encoded with CityJSON. Applications that process these models need test data, but real-world datasets have several limitations:

- Files are often large and slow to download/process during testing
- Models contain much irrelevant information for specific test cases
- Certain CityObject types (e.g. Tunnels, CityFurniture) are rare or nonexistent
- Advanced features like Appearances or Geometry-templates are rarely modeled

## What It Does

cityjson-fake aims to:

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

Add `cityjson-fake` to your `Cargo.toml`:

```toml
[dependencies]
cityjson-fake = "0.5.0"
```

If you want the JSON serialization helpers, enable the `json` feature:

```toml
[dependencies]
cityjson-fake = { version = "0.5.0", default-features = false, features = ["json"] }
```

Or install the CLI tool:

```bash
cargo install cityjson-fake --features cli
```

## Usage

### As a Library

```rust
// `generate_string` is available with the `json` feature.
# #[cfg(feature = "json")]
# {
use cityjson_fake::prelude::*;

// Create a basic CityJSON model
let model: CityModel<u32, OwnedStringStorage> = CityModelBuilder::default().build();
assert_eq!(model.cityobjects().len(), 1);

// Generate a serialized CityJSON document
let json = cityjson_fake::generate_string(CJFakeConfig::default(), Some(42)).unwrap();
assert!(json.starts_with('{'));
# }
```

See the [API documentation](https://docs.rs/cityjson-fake) for more details.

### Command Line Interface

The CLI exposes the same generation controls as the library API. The top-level command is:

```bash
cjfake [OPTIONS]
```

Common examples:

```bash
# Write a single document to stdout
cjfake --output-format json > output.city.json

# Restrict the generated CityObject types
cjfake --allowed-types-cityobject Building,Bridge --output-format json > output.city.json

# Write a single document to a file
cjfake --output-format json --output output.city.json

# Write multiple documents into a directory
cjfake --count 3 --output-format json --output out/

# Generate from a manifest-driven case catalog
cjfake --manifest manifest.json --output out/

# Validate a manifest without generating output
cjfake --manifest manifest.json --check-manifest
```

The available options are grouped below.

| Group       | Flags                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
|-------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| General     | `--seed`, `--output`, `--count`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| Manifest    | `--manifest`, `--schema`, `--case`, `--check-manifest`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| CityObjects | `--allowed-types-cityobject`, `--min-cityobjects`, `--max-cityobjects`, `--cityobject-hierarchy`, `--min-children`, `--max-children`                                                                                                                                                                                                                                                                                                                                                                                                                       |
| Geometry    | `--allowed-types-geometry`, `--allowed-lods`, `--min-members-multipoint`, `--max-members-multipoint`, `--min-members-multilinestring`, `--max-members-multilinestring`, `--min-members-multisurface`, `--max-members-multisurface`, `--min-members-solid`, `--max-members-solid`, `--min-members-multisolid`, `--max-members-multisolid`, `--min-members-compositesurface`, `--max-members-compositesurface`, `--min-members-compositesolid`, `--max-members-compositesolid`, `--min-members-cityobject-geometries`, `--max-members-cityobject-geometries` |
| Vertices    | `--min-coordinate`, `--max-coordinate`, `--min-vertices`, `--max-vertices`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| Materials   | `--materials-enabled`, `--min-materials`, `--max-materials`, `--nr-themes-materials`, `--generate-ambient-intensity`, `--generate-diffuse-color`, `--generate-emissive-color`, `--generate-specular-color`, `--generate-shininess`, `--generate-transparency`                                                                                                                                                                                                                                                                                              |
| Textures    | `--textures-enabled`, `--min-textures`, `--max-textures`, `--nr-themes-textures`, `--max-vertices-texture`, `--texture-allow-none`                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Templates   | `--use-templates`, `--min-templates`, `--max-templates`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| Metadata    | `--metadata-enabled`, `--metadata-geographical-extent`, `--metadata-identifier`, `--metadata-reference-date`, `--metadata-reference-system`, `--metadata-title`, `--metadata-point-of-contact`                                                                                                                                                                                                                                                                                                                                                             |
| Attributes  | `--attributes-enabled`, `--min-attributes`, `--max-attributes`, `--attributes-max-depth`, `--attributes-random-keys`, `--attributes-value-mode` (`heterogenous` \| `homogenous`), `--attributes-allow-null`                                                                                                                                                                                                                                                                                                                                                |
| Semantics   | `--semantics-enabled`, `--allowed-types-semantic`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |

Manifest mode accepts a JSON file whose cases flatten the normal `CJFakeConfig`
fields at the top level. Manifest handling is CLI-only. The manifest should validate against
the bundled `cityjson-fake-manifest.schema.json` schema:

```json
{
  "version": 1,
  "cases": [
    {
      "id": "spec_complete_omnibus",
      "seed": 42,
      "min_cityobjects": 2,
      "max_cityobjects": 2,
      "allowed_types_geometry": ["MultiSurface"]
    }
  ]
}
```

When `--manifest` is present, the manifest supplies the generation config.
If `--schema` is not provided, `cjfake` uses the bundled schema.
Use `--schema` to validate against a different copy, and `--check-manifest`
to validate and exit without generating output.

The `--output-format` flag currently accepts `json` only.

With multiple cases, `--output` must name a directory and each case is written
as `<id>.city.json` unless the case defines its own output path.

Use `cjfake --help` for the exact defaults and the full clap-generated help text.

## API Shape

The easiest entry points are:

- `cityjson_fake::generate_model(config, seed)` for a `CityModel`
- `cityjson_fake::generate_string(config, seed)` for a serialized CityJSON string with the `json` feature
- `cityjson_fake::generate_vec(config, seed)` for UTF-8 encoded bytes with the `json` feature
- `cityjson_fake::manifest::load_manifest(path)` for raw manifest loading with the `cli` feature
- `cityjson_fake::manifest::load_manifest_validated(path, schema)` when you want
  schema validation before generation with the `cli` feature
- `CityModelBuilder` when you need fine-grained control over generation

## Contributing

This crate follows the workspace contract. See
[`CONTRIBUTING.md`](../../CONTRIBUTING.md) for PR guidelines and
[`docs/development.md`](../../docs/development.md) for tooling, lints,
and release flow.

## License

Dual-licensed under MIT or Apache-2.0, at your option. See
[`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).

## Roadmap

There are no major features planned for the near future, beyond bug fixes, test coverage, performance optimization, and documentation improvements.
