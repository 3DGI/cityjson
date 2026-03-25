# Serde CityJSON -> cityjson-rs Migration Plan

## Goal

Refactor `serde_cityjson` from a standalone CityJSON data model crate into a
v2.0-only serde adapter around `cityjson`.

The target architecture is:

- `cityjson` owns the in-memory data model and validation
- `serde_cityjson` owns JSON de/serialization
- legacy CityJSON versions are out of scope for the refactor

## Target Public API

Expose adapter-style functions and wrappers instead of versioned model structs.

Recommended API:

```rust
pub fn from_str_owned(input: &str) -> Result<cityjson::v2_0::OwnedCityModel>;

pub fn from_str_borrowed<'a>(
    input: &'a str,
) -> Result<cityjson::v2_0::BorrowedCityModel<'a>>;

pub fn to_string<VR, SS>(
    model: &cityjson::v2_0::CityModel<VR, SS>,
) -> Result<String>;

pub fn to_string_validated<VR, SS>(
    model: &cityjson::v2_0::CityModel<VR, SS>,
) -> Result<String>;

pub fn as_json<'a, VR, SS>(
    model: &'a cityjson::v2_0::CityModel<VR, SS>,
) -> SerializableCityModel<'a, VR, SS>;
```

## Architectural Constraints

1. `serde_cityjson` cannot directly implement `serde::Serialize` or
   `serde::Deserialize` for `cityjson` types because both the trait and the
   target type are foreign.
2. `cityjson` is deliberately `v2_0`-only.
3. `cityjson` stores:
   - real-world `f64` coordinates internally
   - flat boundaries
   - semantics/materials/textures in model-level pools with handles
4. `serde_cityjson` should reuse the optimized parsing and nested-array
   reconstruction ideas from the current codebase, but not keep the old public
   data model.

## File-by-File Plan

### Keep and Refactor

#### `Cargo.toml`

- Add `cityjson` as a dependency.
- Keep `serde`, `serde_json`, and likely `serde_json_borrow`.
- Remove old dependencies that become unused after the refactor.
- Reevaluate `datasize`, `derive_more`, and `ahash` after the port.

#### `src/lib.rs`

- Replace the current version-dispatch public API with the adapter API.
- Re-export selected `cityjson` types if useful for callers.
- Remove the old `CityJSON` enum and legacy `from_str` behavior.
- Keep only v2.0-oriented entry points.

#### `src/errors.rs`

- Reduce the error surface to adapter/import/export concerns.
- Add conversions from:
  - `serde_json::Error`
  - `cityjson::error::Error`
- Add import-specific variants:
  - unresolved `CityObject` ID references
  - unsupported `type`
  - unsupported `version`
  - malformed root object

#### `src/v2_0.rs`

- Repurpose as the public v2.0 adapter surface.
- Define top-level serializer wrapper types such as
  `SerializableCityModel<'a, VR, SS>`.
- Do not define a second standalone `CityModel` type here.

### Replace With Shared Conversion Helpers

#### `src/attributes.rs`

- Rewrite as helpers for converting JSON values to and from
  `cityjson::v2_0::{AttributeValue, Attributes}`.
- Support both owned and borrowed storage strategies.
- Handle the special geometry-valued attribute case used for
  `address.location`.
- Remove the old public `Attributes<'cm>` enum from the public API.

### Delete After Porting Logic Out

These files are implementation donors, not long-term architecture.

#### `src/v1_1.rs`

- Mine for optimized manual geometry parsing logic.
- Extract only the raw-value driven import patterns that remain useful.
- Remove from the public build after the new importer is working.

#### `src/boundary.rs`

- Do not preserve as a separate boundary implementation.
- Use `cityjson` boundary types and conversions instead.

#### `src/labels.rs`

- Reuse the nested semantic/material/texture reconstruction logic as input to
  the new serializer.
- Do not keep the old label-index types as public API.

#### `src/indices.rs`

- Remove once the crate exclusively uses `cityjson` index types.

## New Private Module Layout

### Deserialization

#### `src/de/mod.rs`

- Public-facing import orchestration.
- Owned vs borrowed dispatch.

#### `src/de/header.rs`

- Tiny borrowed parse for root `type` and `version`.
- Reject unsupported versions early.

#### `src/de/storage.rs`

- Traits/helpers abstracting over:
  - `OwnedStringStorage`
  - `BorrowedStringStorage<'de>`

#### `src/de/citymodel.rs`

- Root object import orchestration.
- Capacity planning from parsed counts where possible.
- Import order:
  1. header
  2. transform / metadata / extensions / extra
  3. appearance root resources
  4. vertices
  5. template vertices / template geometries
  6. city objects
  7. fixups

#### `src/de/appearance.rs`

- Parse root `appearance`.
- Build:
  - dense material index -> `MaterialHandle`
  - dense texture index -> `TextureHandle`
  - dense `vertices-texture` index -> UV vertex index
- Set default material/texture theme names via
  `set_default_material_theme` / `set_default_texture_theme` using
  `ThemeName<SS>`.

#### `src/de/metadata.rs`

- Import metadata and point-of-contact structures.
- Convert generic JSON address objects into typed attribute maps.

#### `src/de/cityobjects.rs`

- Parse each `CityObjects` entry once.
- Build object body immediately.
- Insert object into `cityjson` and record `json_id -> CityObjectHandle`.
- Defer only string-ID relation fixups (`parents` and `children`).
- After all objects are inserted, resolve fixups using the handle map.

Notes:

- This should not be a second JSON pass.
- It should be a single parse plus a final linear fixup pass over deferred
  references.

#### `src/de/geometry.rs`

- Import regular geometry and template geometry.
- Port the current manual raw `boundaries` / `semantics` / `material` /
  `texture` parsing strategy.
- Convert nested JSON arrays into:
  - `cityjson` boundaries
  - dense semantic/material assignment arrays
  - dense ring-anchored texture mappings
- Create geometry via validated `cityjson` insertion paths.

### Serialization

#### `src/ser/mod.rs`

- Public-facing serializer orchestration.

#### `src/ser/citymodel.rs`

- Serialize root object members:
  - `type`
  - `version`
  - `transform`
  - `metadata`
  - `extensions`
  - `CityObjects`
  - `vertices`
  - `appearance`
  - `geometry-templates`
  - extra root properties

#### `src/ser/appearance.rs`

- Use `cityjson::raw` and `DenseIndexRemap` to export sparse internal pools as
  dense JSON arrays.
- Emit:
  - `materials`
  - `textures`
  - `vertices-texture`
  - `default-theme-material`
  - `default-theme-texture`
- Read default themes directly from
  `default_material_theme()` / `default_texture_theme()`.
- Keep normal serialization fast and validation-free.
- Add a separate strict serializer entry point that calls
  `validate_default_themes()` before writing JSON.

#### `src/ser/boundary.rs`

- Emit nested `boundaries` arrays from `cityjson` boundaries via:
  - `to_nested_multi_point`
  - `to_nested_multi_linestring`
  - `to_nested_multi_or_composite_surface`
  - `to_nested_solid`
  - `to_nested_multi_or_composite_solid`

#### `src/ser/mappings.rs`

- Rebuild nested `semantics.values` from dense primitive assignment arrays plus
  the matching boundary.
- Rebuild:
  - `material[theme].value`
  - `material[theme].values`
  - `texture[theme].values`
- Reuse the current optimized nested reconstruction ideas from `src/labels.rs`.

#### `src/ser/attributes.rs`

- Serialize `cityjson` typed attributes back to JSON objects and arrays.
- Preserve extension and extra-property output behavior.

## Import Strategy Details

### Root Parsing

- Parse a small root header first to validate:
  - `type == CityJSON || CityJSONFeature`
  - `version == 2.0 / 2.0.0 / 2.0.1`
- Reject everything else early.

### `CityObjects` Handling

Use a single parse plus deferred fixups:

1. Parse one `CityObject`.
2. Parse and insert its geometries immediately.
3. Insert the object and record its handle.
4. Store deferred relation records for unresolved string IDs.
5. Resolve all deferred relations after object insertion is complete.

This avoids a second full JSON pass while still matching `cityjson`'s
handle-based object graph.

### Geometry Handling

For each geometry:

- Read `type` first.
- Parse `boundaries` using a type-specific raw-value strategy.
- Parse `semantics`, `material`, and `texture` into dense internal arrays.
- Resolve dense JSON indices through the appearance remap tables.
- Insert validated geometry into the model.

### Attribute Handling

Define explicit conversion rules for:

- null
- bool
- integer / unsigned / float
- string
- array
- object
- geometry reference attributes

Do not keep generic `serde_json::Value` as the main in-memory attribute model.

## Serialization Strategy Details

### Root Serialization

- Serialize directly from `cityjson::v2_0::CityModel`.
- Do not construct an intermediate legacy-style Rust model.

### Resource Remapping

Because `cityjson` pools can be sparse internally:

- build dense export remaps for semantics
- build dense export remaps for materials
- build dense export remaps for textures
- use identity remaps for vertex arrays where already dense

### Nested Mapping Reconstruction

The serializer must rebuild JSON-shaped nested arrays from:

- flat boundaries
- dense primitive assignment arrays
- dense texture ring maps

This is where most of the current optimized logic should be reused.

## Tests and Fixtures

### Replace Existing Top-Level Tests

#### `tests/common/mod.rs`

- Replace generic serde-struct roundtrip helpers with adapter-aware helpers:
  - parse owned
  - parse borrowed
  - serialize back
  - compare JSON values

#### `tests/v1_1.rs`

- Retire as the primary top-level test file.
- Split reusable cases into new v2.0-oriented suites.

### Add New Test Files

#### `tests/v2_0_roundtrip.rs`

- Full document roundtrip tests against v2.0 fixtures.

#### `tests/v2_0_components.rs`

- Focused tests for:
  - transform
  - metadata
  - extensions
  - materials
  - textures
  - geometry templates
  - semantics
  - geometry instances

#### `tests/v2_0_borrowed.rs`

- Owned vs borrowed import parity tests.

#### `tests/v2_0_invalid.rs`

- Invalid reference tests.
- Invalid mapping shape tests.
- Invalid geometry topology tests.
- Invalid unresolved `CityObject` relation tests.

### Fixture Strategy

#### Keep and Reuse

- Keep `tests/data/v1_1` during migration as a donor source of small component
  fixtures.
- Reuse especially:
  - geometry semantics fixtures
  - material fixtures
  - texture fixtures
  - metadata fixtures
  - transform fixtures

#### Add v2.0 Fixture Set

- Create `tests/data/v2_0`.
- Copy the fake complete v2.0 fixture from `cityjson-rs`.
- Add component fixtures that match the current v2.0 API and spec.

## Benchmarks

#### `benches/speed.rs`

- Retarget to:
  - `from_str_owned`
  - `from_str_borrowed`
  - `to_string`

#### `benches/datasize.rs`

- Delete or redefine.
- The old benchmark no longer measures the final architecture once the old data
  model is removed.

## Documentation

#### `README.md`

- Rewrite examples around `cityjson::v2_0::{OwnedCityModel, BorrowedCityModel}`.
- Document that:
  - `serde_cityjson` is v2.0-only
  - legacy versions are not part of the in-memory model
  - older versions will eventually be handled via a separate
    `serde_json::Value`-based upgrade path

## Implementation Phases

### Phase 1: Public API Skeleton

- Update `Cargo.toml`
- Rewrite `src/errors.rs`
- Rewrite `src/lib.rs`
- Repurpose `src/v2_0.rs`
- Add empty `de` / `ser` module structure

### Phase 2: Owned Deserialization

- Implement root header parsing
- Implement root import orchestration
- Implement appearance/resource import
- Implement geometry import
- Implement object insertion and deferred relation fixups
- Reach a passing owned importer for core v2.0 fixtures

### Phase 3: Serialization

- Implement root serializer wrapper
- Implement boundary serialization
- Implement semantic/material/texture nested reconstruction
- Pass full owned roundtrip tests

### Phase 4: Borrowed Deserialization

- Implement borrowed string-storage path
- Ensure attribute conversion supports borrowed values
- Add owned vs borrowed parity tests

### Phase 5: Cleanup

- Delete legacy public modules and donors no longer needed:
  - `src/v1_1.rs`
  - `src/boundary.rs`
  - `src/labels.rs`
  - `src/indices.rs`
- Update README and benches
- Prune unused dependencies

## Key Risks and Decisions

### 1. Default Appearance Theme Validation

`cityjson` now stores default appearance themes as `ThemeName<SS>` and exposes:

- `default_material_theme()`
- `default_texture_theme()`
- `validate_default_themes()`

Chosen policy:

- `to_string()` stays fast and serializes configured theme names as-is
- `to_string_validated()` performs `validate_default_themes()` first for
  callers that want strict output

### 2. Borrowed Attribute Conversion

Borrowed import is likely the most complex part because attributes are recursive
and typed in `cityjson`.

Recommendation:

- do not block the owned path on perfect borrowed support
- land owned first, then add borrowed carefully

### 3. Serializer Performance

The main performance-sensitive parts are:

- geometry boundary parsing
- dense mapping import/export
- attribute conversion
- pool remapping during serialization

These are the areas where logic from the old implementation is most worth
reusing.

## Recommended First Implementation Slice

The highest-value first slice is:

1. add the new API skeleton
2. implement owned root import
3. implement owned geometry import
4. implement deferred `CityObject` relation fixups
5. wire one full v2.0 fake-complete fixture
6. implement serialization for that same fixture

That gets the crate onto the new architecture quickly and creates a stable base
for further optimization.
