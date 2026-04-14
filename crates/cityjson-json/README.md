# cityjson-json

`cityjson-json` is a CityJSON 2.0 serde adapter around the [`cityjson`](https://crates.io/crates/cityjson) crate. It provides efficient serialization and deserialization of CityJSON documents with both owned and borrowed string storage options.

## Installation

```shell
cargo add cityjson-json
```

## Getting Started

### Imports

```rust
use cityjson_json::{from_str_owned, from_str_borrowed, to_string, to_string_validated};
use cityjson_json::{OwnedCityModel, BorrowedCityModel, SerializableCityModel};
```

### Owned Deserialization

For simple use cases, deserialize into an owned model:

```rust
use cityjson_json::from_str_owned;

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
use cityjson_json::from_str_borrowed;

let json_str = r#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#;
let model = from_str_borrowed(json_str)?;
// model holds references to json_str
```

### Serialization

Serialize models back to JSON:

```rust
use cityjson_json::to_string;

let json_output = to_string(&model)?;
```

For validated serialization (checks default theme references):

```rust
use cityjson_json::to_string_validated;

let json_output = to_string_validated(&model)?;
```

## Validation Policy

The library provides two serialization paths to balance performance and safety:

- **`to_string()`**: Fast path. Does not validate that default theme names (for materials and textures) actually reference existing themes in the appearance section.
- **`to_string_validated()`**: Strict path. Validates default theme references before serialization to ensure document consistency.

Use `to_string_validated()` when you need guaranteed valid CityJSON output, especially when serializing user-provided models.

## Documentation

- [CityJSON 2.0](https://www.cityjson.org/specs/2.0.1/)
- [`cityjson` crate documentation](https://docs.rs/cityjson/)

todo: link to docs.rs

## Library Layout

| Module   | Contents                                                                                       |
|----------|------------------------------------------------------------------------------------------------|
| `v2_0`   | CityJSON 2.0 (de)serialization entry points, feature-stream helpers, and `SerializableCityModel` |
| `errors` | `Error` and `Result` types surfaced by the adapter                                             |
| (root)   | Convenience re-exports: `from_str_*`, `to_string*`, `to_vec*`, `to_writer*`, model types       |

Core types re-exported from `cityjson`:

- **`OwnedCityModel`**: A CityJSON model with owned `String` storage. Self-contained and doesn't depend on external lifetimes.
- **`BorrowedCityModel`**: A CityJSON model with borrowed string references. More memory efficient but requires careful lifetime management.

## Design

### Deserialization

Parsing a CityJSON document is split into two sequential phases.

**Phase 1 — root preparation (`parse_root`).**
The document is read once by a handwritten `serde` visitor that fills a
`PreparedRoot<'de>` struct. Well-known sections with bounded size (transform,
metadata, appearance, geometry-templates, extensions) are deserialized eagerly.
The `CityObjects` map, which may be arbitrarily large, is kept as a borrowed
`&RawValue` slice pointing into the original input bytes. Nothing is allocated
for it yet.

**Phase 2 — model construction (`build_model`).**
The prepared root is used to initialize the `CityModel`. Appearance, geometry
templates, and vertices are imported first, establishing handles that the
`CityObjects` import can reference. The `CityObjects` slice is then
deserialized once more, but streamed entry by entry directly into the model
instead of materializing a full intermediate object graph. Parent and child
relations are resolved in a follow-up pass after all objects have been inserted.

**Geometry.**
Each geometry object is parsed by a streaming visitor that reads the `type`,
`lod`, and `boundaries` fields manually. Boundaries are parsed by a specialized
flat parser that scans the raw bytes and writes vertex indices and offset vectors
directly into the shapes the `cityjson` backend expects (`Boundary<u32>`). There
is no intermediate nested boundary tree. Finished geometry parts are inserted
through the backend's trusted raw API (`add_geometry_unchecked`) which skips
the authoring-time validation that `GeometryDraft::insert_into` performs.

**Attributes.**
Attributes and extra properties are deserialized directly into the backend
`AttributeValue<SS>` and `Attributes<SS>` types via
`AttributeValueSeed` / `AttributesSeed` / `OptionalAttributesSeed`. There is no
temporary `RawAttribute` tree: the `CityObject` visitor produces final values in
a single pass.

**Owned and borrowed storage.**
The single `ParseStringStorage<'de>` trait controls whether string values are
heap-allocated (`OwnedStringStorage`) or zero-copy borrowed from the input
(`BorrowedStringStorage`). Borrowed mode fails on strings that contain JSON
escape sequences because those cannot be represented without allocation.

### Serialization

**Direct streaming.**
The serializer writes the `CityModel` directly through `serde::Serialize`
without first constructing an intermediate `serde_json::Value` DOM. Each
section of the document is a dedicated serializer struct that borrows from the
model and emits JSON fields on demand.

**Shared write context.**
Before any field is written, a `WriteContext` is built once for the entire
serialization. It precomputes four lookup maps:

- city object handle → JSON id string
- geometry template handle → dense array index
- material handle → dense array index
- texture handle → dense array index

All nested serializers borrow the same context, so handle-to-index lookups are
O(1) hash-map reads with no repeated work.

**Transform-aware vertex quantization.**
When a transform is present, vertex coordinates are quantized by applying the
inverse transform `(x - translate) / scale` before serialization and then
rounded to the nearest integer. Without a transform, coordinates are written as
floating-point values. The same quantization applies when writing
`CityJSONSeq` streams.

**Material compaction.**
When all surfaces of a geometry in a given material theme share the same
non-null material index, the serializer writes the compact `{"value": N}` form
instead of an explicit `{"values": [...]}` array.

**Validation policy.**
`to_string`, `to_vec`, and `to_writer` serialize without pre-flight checks.
Their `_validated` counterparts call `validate_default_themes` before
serializing to confirm that the default material and texture theme names
reference themes that actually exist in the appearance section.

**CityJSONSeq stream writing.**
`write_cityjsonseq_with_transform_refs` and
`write_cityjsonseq_auto_transform_refs` write a compliant newline-delimited
stream. The first line is a `CityJSON` header serialized with
`CityModelSerializeOptions` that suppresses city objects and vertices; each
subsequent line is a `CityJSONFeature` serialized with options that suppress
metadata, extensions, appearance, and geometry templates. The auto-transform
variant computes the translate from the bounding box of all feature vertices
and writes it as part of the header.

## API Stability

This crate follows semantic versioning (`MAJOR.MINOR.PATCH`):

- `MAJOR`: incompatible API changes
- `MINOR`: backwards-compatible feature additions
- `PATCH`: backwards-compatible fixes

## Minimum Rust Version

The minimum supported rustc version is `1.93.0`.

## Development

### Running Tests

```bash
cargo test
cargo test --test v2_0
```

The corpus-backed correctness tests read fixture IDs from the shared
`cityjson-benchmarks` checkout at
`../cityjson-benchmarks/artifacts/correctness-index.json` by default. Override
the shared root with `CITYJSON_JSON_SHARED_CORPUS_ROOT` or the index path with
`CITYJSON_JSON_CORRECTNESS_INDEX` if your checkout lives elsewhere.

### Running Benchmarks

The benchmark corpus lives in the shared `cityjson-benchmarks` repository.
`cityjson-json` benchmarks the CityJSON artifacts listed in each workload's
`artifacts[]` array and reads the shared benchmark index from
`../cityjson-benchmarks/artifacts/benchmark-index.json` by default. Override
the shared root with `CITYJSON_JSON_SHARED_CORPUS_ROOT` or the index path with
`CITYJSON_JSON_BENCHMARK_INDEX` if your checkout lives elsewhere.

```bash
just bench-read
just bench-write
just bench-report
```

The benchmarks use Criterion. Read throughput is based on input bytes; write
throughput is based on output bytes. README benchmark tables are generated
from the shared corpus and should be refreshed from current benchmark output,
not edited by hand.

## Contributing

Contributions are welcome in all forms.
Please open an issue to discuss any potential changes before working on a patch.
You can submit LLM-generated PRs for bug fixes and documentation improvements.
Regardless of handwritten or LLM-generated code, the PR should follow these guidelines:

- relatively small, focused changes, otherwise I won't be able to review it,
- follow the existing style and conventions,
- include unit tests and documentation for new features and bug fixes,
- the patched code should pass:
  - `just check / lint / fmt / test / docs / bench-check`
- if you remove or merge tests or examples or benchmarks, please explain why and update the documentation accordingly.

## License

Licensed under either:

- Apache License, Version 2.0 (`LICENSE-APACHE`)
- MIT license (`LICENSE-MIT`)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
cityjson-json by you, as defined in the Apache-2.0 license, shall be dual licensed as above,
without additional terms or conditions.

## Use of AI in this project

This crate was originally developed without the use of AI.
Since then, it underwent multiple significant refactors and various LLM models (Claude, `ChatGPT`) were used for experimenting with alternative designs, in particular for the (de)serialization strategies and borrowed-parsing paths.
LLM generated code is also used for improving the test coverage and documentation and mechanical improvements.
Code correctness and performance are verified by carefully curated test cases and benchmarks that cover the CityJSON 2.0 specification.

## Roadmap

There are no major features planned for the near future, beyond bug fixes, test coverage, documentation improvements.