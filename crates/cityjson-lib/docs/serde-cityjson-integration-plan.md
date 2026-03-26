# serde_cityjson Integration Plan

## Goal

Move the CityJSON JSON-to-model conversion path out of `cjlib` and into the
intended ecosystem pipeline:

`json <--> serde_cityjson <--> cityjson-rs <--> cjlib`

The target end state is:

- `serde_cityjson` owns JSON parsing, JSON serialization, feature handling, and
  staged/raw JSON boundary work
- `cityjson-rs` owns the normalized semantic model, correctness-critical
  invariants, and submodel extraction and merge semantics
- `cjlib` is a thin facade over `cityjson-rs` with explicit format modules and
  a small set of convenience constructors

`cjlib` should not own a second importer stack and should not know about
indexed CityJSON internals.

## Architectural Direction

### 1. Keep one semantic model

There should be exactly one semantic model:

- `cityjson::v2_0::OwnedCityModel`

And exactly one semantic interchange unit:

- a self-contained `OwnedCityModel`

A full document and a feature-sized package should both materialize to that
same semantic type.
`CityJSONFeature` should remain a JSON wire-format concern, not a second
conceptual model.

### 2. Keep `cjlib` thin

`cjlib` should keep:

- `CityModel::from_slice`
- `CityModel::from_file`
- `CityJSONVersion`
- a small `Error` facade
- explicit format modules such as `cjlib::json`
- the `cityjson` crate re-export for advanced model access

`cjlib` should not:

- parse JSON into local schema structs
- reconstruct indexed CityJSON geometry
- own template or instance import rules
- own format-specific semantic types
- absorb raw/staged parsing complexity into the default `CityModel` path

The current temporary JSON boundary code should be treated as scaffolding and
removed once the lower-layer integration is ready.

### 3. Make `serde_cityjson` the JSON boundary

`serde_cityjson` should be the only crate that understands the CityJSON JSON
wire format in detail, including:

- root `vertices`
- `geometry-templates.vertices-templates`
- indexed `boundaries`
- `semantics.values`
- `material`
- `texture`
- `vertices-texture`
- `GeometryInstance.template`
- `GeometryInstance.transformationMatrix`
- document and feature stream handling
- staged and raw JSON section boundaries

This is where indexed CityJSON belongs.

### 4. Keep `cityjson-rs` as the model source of truth

`cityjson-rs` should continue to define:

- the owned in-memory model
- validation rules
- storage invariants
- template geometry invariants
- geometry instance invariants
- resource pool consistency
- submodel extraction
- resource localization and remapping
- merge and assembly of self-contained models

If JSON import needs more support than the current public API provides, the
missing pieces should be added to `cityjson-rs` as import-oriented or
submodel-oriented helpers, not reimplemented in `cjlib`.

## Proposed Responsibilities By Crate

### `serde_cityjson`

Should provide:

- parse a full document into one `OwnedCityModel`
- parse one `CityJSONFeature` item into one self-contained `OwnedCityModel`
- read a feature stream as a stream of self-contained `OwnedCityModel` values
- merge a strict `CityJSON` plus `CityJSONFeature` stream when a caller wants
  to rebuild one larger model
- serialize one `OwnedCityModel` as either a document or a feature item
- expose raw/staged JSON boundaries explicitly when they become worthwhile

Preferred public surface:

```rust
pub fn from_slice_document(bytes: &[u8]) -> Result<cityjson::v2_0::OwnedCityModel>;
pub fn from_reader_document<R: std::io::Read>(reader: R) -> Result<cityjson::v2_0::OwnedCityModel>;

pub fn from_slice_feature(bytes: &[u8]) -> Result<cityjson::v2_0::OwnedCityModel>;
pub fn read_feature_stream<R: std::io::BufRead>(
    reader: R,
) -> Result<impl Iterator<Item = Result<cityjson::v2_0::OwnedCityModel>>>;

pub fn merge_feature_stream<R: std::io::BufRead>(
    reader: R,
) -> Result<cityjson::v2_0::OwnedCityModel>;

pub fn to_string_document(model: &cityjson::v2_0::OwnedCityModel) -> Result<String>;
pub fn to_string_feature(model: &cityjson::v2_0::OwnedCityModel) -> Result<String>;
```

Exact names are flexible; the important part is that `cjlib` can call
`serde_cityjson` without rebuilding JSON import itself.

### `cityjson-rs`

Should expose the minimal extra hooks required for `serde_cityjson` to build
valid models and for the wider ecosystem to package and merge them, especially
for:

- template geometry import
- geometry instance import
- exact topology-preserving appearance import
- extraction of self-contained submodels
- merge and remapping of self-contained submodels

These hooks should be:

- correctness-oriented
- validated
- narrowly scoped
- designed for import code, not end-user authoring ergonomics

Avoid exposing raw internal constructors unless there is no cleaner option.

### `cjlib`

Should:

- sniff version and top-level type
- dispatch to the right `serde_cityjson` conversion path for the default
  single-document convenience path
- wrap the returned `OwnedCityModel`
- expose explicit JSON boundary helpers for feature parsing, stream reading, and
  stream aggregation
- preserve the legacy version branches as explicit `todo!()` where necessary

It should not contain JSON-boundary data structures beyond trivial header
sniffing and thin delegation.

## Key Design Question

The main unresolved semantic question is no longer "how many model types do we
need".
The answer there is clear:

- one semantic model
- one semantic interchange unit

The remaining technical question is:

How should `serde_cityjson` construct template geometries, geometry instances,
and self-contained submodels in `cityjson-rs` without duplicating storage
invariants?

The preferred answer is:

- add small import-oriented and submodel-oriented APIs in `cityjson-rs`
- use those APIs from `serde_cityjson`

The least desirable answer is:

- keep a large custom conversion layer in `cjlib`

## Recommended `cityjson-rs` Additions

Add only what `serde_cityjson` needs.

Possible shapes:

### Option A: import-oriented draft API

Expose a public API specifically for importing already-parsed geometry:

```rust
pub enum ImportTarget {
    RegularGeometry,
    TemplateGeometry,
}

pub struct ImportedGeometryDraft { ... }

impl ImportedGeometryDraft {
    pub fn insert_into(
        self,
        model: &mut OwnedCityModel,
        target: ImportTarget,
    ) -> Result<...>;
}
```

This is the preferred direction if it can stay small and validated.

### Option B: low-level import and submodel module

If you want to keep the public API very tight, expose a narrow module intended
for `serde_cityjson` and sibling boundary crates.

This can still be public, but documented as low-level import support rather than
normal user API.

### Option C: expose raw constructors publicly

This is the fallback only if the cleaner approaches prove too costly.

It is less desirable because it increases the chance of invalid external
construction and weakens the abstraction boundary in `cityjson-rs`.

## Execution Phases

### Phase 1: Define the boundary interface

Decide the exact API between `cjlib`, `serde_cityjson`, and `cityjson-rs`.

Deliverable:

- one documented `serde_cityjson` entry point for full-document conversion
- one documented `serde_cityjson` entry point for feature-sized conversion
- one documented `serde_cityjson` entry point for model-stream reading
- one documented `serde_cityjson` entry point for strict stream aggregation

### Phase 2: Finish `serde_cityjson -> cityjson-rs` conversion

Implement full v2.0 conversion in `serde_cityjson`, including:

- regular geometries
- materials
- textures
- UV coordinates
- semantics
- metadata
- extensions
- extra attributes
- geometry templates
- geometry instances

Deliverable:

- `serde_cityjson` can produce correct full-document and feature-sized
  `OwnedCityModel` values without `cjlib` importer logic

### Phase 3: Add minimal semantic hooks to `cityjson-rs`

Only if needed, add the smallest validated APIs required by Phase 2.

Deliverable:

- no template, instance, or submodel correctness logic duplicated in `cjlib`

### Phase 4: Delete `cjlib` importer scaffolding

Remove local JSON-to-model conversion logic from `cjlib`.

Replace it with:

- direct calls from `cjlib::io` into `serde_cityjson`
- a thin explicit `cjlib::json` boundary module

Deliverable:

- `cjlib` becomes small again

### Phase 5: Rebuild explicit JSON boundary handling around the lower layer

Implement `cjlib::json` as a thin facade over `serde_cityjson` for:

- probing
- document parsing
- feature parsing
- model-stream reading
- strict stream aggregation
- document and feature serialization

If a compatibility `CityModel::from_stream` alias survives temporarily, it
should delegate to the explicit JSON boundary helper rather than remain a
first-class architectural concept.

The strict aggregation rules should remain in force where that helper exists:

- first non-empty item must be `CityJSON`
- remaining items must be `CityJSONFeature`
- versions must match
- duplicate IDs must error
- no lossy merge behavior

Deliverable:

- same behavior, thinner implementation, cleaner ownership boundaries

### Phase 6: Tighten tests by crate boundary

Tests should be redistributed:

- `serde_cityjson`: JSON boundary correctness, templates, instances, topology,
  appearance, feature parsing, and staged/raw boundary behavior
- `cityjson-rs`: model invariants, import-helper validation, extraction, and
  merge
- `cjlib`: facade behavior and module wiring only

`cjlib` tests should primarily cover:

- version dispatch
- `from_file` document dispatch
- `cjlib::json` delegation and strict stream aggregation wiring
- legacy branch locking
- wrapping / re-export behavior

Deliverable:

- each crate tests what it actually owns

## Migration Guidance For Current `cjlib`

Short-term:

- keep the current temporary importer only as a stopgap
- do not expand it further
- do not implement templates or instances there
- do not let it become the long-term stream or raw JSON layer

Medium-term:

- swap `cjlib::io` over to `serde_cityjson`
- build out `cjlib::json` as a thin explicit boundary module
- delete the temporary importer

Long-term:

- keep `cjlib` small enough that opening `src/` makes the architecture obvious

## Done Criteria

This refactor is done when:

- `cjlib` no longer contains JSON-to-model conversion logic beyond trivial
  dispatch
- `serde_cityjson` can construct full-document and feature-sized
  `cityjson::v2_0::OwnedCityModel` values correctly for v2.0
- template geometry and geometry instance handling live below `cjlib`
- submodel extraction and merge semantics live in `cityjson-rs`
- `cityjson-rs` remains the only in-memory model source of truth
- `cjlib` preserves a small owned convenience facade plus explicit format
  modules
- tests are aligned with crate responsibilities

## Recommendation

Do not invest further in the temporary `cjlib` importer.

The cleanest end state is:

- `serde_cityjson` handles indexed JSON and explicit feature/model-stream
  boundary work
- `cityjson-rs` provides the validated model target and the semantic submodel
  hooks
- `cjlib` becomes a very small facade again
