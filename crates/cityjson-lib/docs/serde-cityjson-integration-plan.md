# serde_cityjson Integration Plan

## Goal

Move the CityJSON JSON-to-model conversion path out of `cjlib` and into the intended pipeline:

`json <--> serde_cityjson <--> cityjson-rs <--> cjlib`

The target end state is:

- `serde_cityjson` owns JSON parsing, JSON serialization, and indexed-wire-format handling
- `cityjson-rs` owns the normalized in-memory model and all correctness-critical model invariants
- `cjlib` is a thin facade over `cityjson-rs` with version dispatch and convenience constructors

`cjlib` should not own a second importer stack and should not know about indexed CityJSON internals.

## Architectural Direction

### 1. Keep `cjlib` thin

`cjlib` should only keep:

- `CityModel::from_slice`
- `CityModel::from_file`
- `CityModel::from_stream`
- `CityJSONVersion`
- a small `Error` facade
- the `cityjson` crate re-export for advanced model access

`cjlib` should not:

- parse JSON into local schema structs
- reconstruct geometry from indexed JSON
- own template / instance conversion rules
- duplicate topology or appearance import logic already needed by `serde_cityjson`

The current `src/import.rs` should be treated as temporary scaffolding and removed once the lower-layer integration is ready.

### 2. Make `serde_cityjson` the JSON boundary

`serde_cityjson` should be the only crate that understands the CityJSON JSON wire format in detail, including:

- root `vertices`
- `geometry-templates.vertices-templates`
- indexed `boundaries`
- `semantics.values`
- `material`
- `texture`
- `vertices-texture`
- `GeometryInstance.template`
- `GeometryInstance.transformationMatrix`
- stream semantics for `CityJSON` + `CityJSONFeature`

This is where indexed CityJSON belongs.

### 3. Keep `cityjson-rs` as the model source of truth

`cityjson-rs` should continue to define:

- the owned in-memory model
- validation rules
- storage invariants
- template geometry invariants
- geometry instance invariants
- resource pool consistency

If JSON import needs more support than the current public API provides, the missing pieces should be added to `cityjson-rs` as import-oriented helpers, not reimplemented in `cjlib`.

## Proposed Responsibilities By Crate

### `serde_cityjson`

Should provide:

- parse full document JSON into boundary structs
- parse `CityJSONFeature` stream items into boundary structs
- convert boundary structs into `cityjson::v2_0::OwnedCityModel`
- convert `cityjson::v2_0::OwnedCityModel` back into JSON boundary structs

Preferred public surface:

```rust
pub fn from_slice(bytes: &[u8]) -> Result<cityjson::v2_0::OwnedCityModel>;
pub fn from_reader<R: std::io::Read>(reader: R) -> Result<cityjson::v2_0::OwnedCityModel>;
pub fn merge_feature_slice(
    model: &mut cityjson::v2_0::OwnedCityModel,
    bytes: &[u8],
) -> Result<()>;
```

Exact names are flexible; the important part is that `cjlib` can call `serde_cityjson` without rebuilding JSON import itself.

### `cityjson-rs`

Should expose only the minimal extra hooks required for `serde_cityjson` to build valid models, especially for:

- template geometry import
- geometry instance import
- exact topology-preserving appearance import

These hooks should be:

- correctness-oriented
- validated
- narrowly scoped
- designed for import code, not end-user authoring ergonomics

Avoid exposing raw internal constructors unless there is no cleaner option.

### `cjlib`

Should:

- sniff version and top-level type
- dispatch to the right `serde_cityjson` conversion path
- wrap the returned `OwnedCityModel`
- preserve the legacy version branches as explicit `todo!()`

It should not contain JSON-boundary data structures beyond trivial header sniffing.

## Key Design Question

The main unresolved technical question is:

How should `serde_cityjson` construct template geometries and geometry instances in `cityjson-rs` without duplicating storage invariants?

The preferred answer is:

- add small import-oriented APIs in `cityjson-rs`
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

### Option B: crate-private import module used via feature or friend API

If you want to keep the public API very tight, expose a narrow import module intended for `serde_cityjson`.

This can still be public, but documented as low-level import support rather than normal user API.

### Option C: expose raw geometry constructors publicly

This is the fallback only if the cleaner approaches prove too costly.

It is less desirable because it increases the chance of invalid external construction and weakens the abstraction boundary in `cityjson-rs`.

## Execution Phases

### Phase 1: Define the boundary interface

Decide the exact API between `cjlib` and `serde_cityjson`.

Deliverable:

- one documented `serde_cityjson` entry point for full-document conversion
- one documented `serde_cityjson` entry point for feature merge / stream conversion

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

- `serde_cityjson` can produce a correct `OwnedCityModel` without `cjlib` importer logic

### Phase 3: Add minimal import hooks to `cityjson-rs`

Only if needed, add the smallest validated APIs required by Phase 2.

Deliverable:

- no template / instance correctness logic duplicated in `cjlib`

### Phase 4: Delete `cjlib` importer scaffolding

Remove:

- `src/import.rs`
- local JSON-boundary import structs
- geometry reconstruction logic from `cjlib`

Replace with:

- direct calls from `cjlib::io` into `serde_cityjson`

Deliverable:

- `cjlib` becomes small again

### Phase 5: Rebuild stream handling around the lower layer

Keep the `cjlib::CityModel::from_stream` API, but implement it as:

- line-oriented reading in `cjlib`
- per-item dispatch into `serde_cityjson`
- model merge into `cityjson-rs`

The strictness rules should remain in force:

- first non-empty item must be `CityJSON`
- remaining items must be `CityJSONFeature`
- versions must match
- duplicate IDs must error
- no lossy merge behavior

Deliverable:

- same facade behavior, thinner implementation

### Phase 6: Tighten tests by crate boundary

Tests should be redistributed:

- `serde_cityjson`: JSON boundary correctness, templates, instances, topology, appearance
- `cityjson-rs`: model invariants and import-helper validation
- `cjlib`: facade behavior only

`cjlib` tests should primarily cover:

- version dispatch
- `from_file` extension dispatch
- `from_stream` strict sequencing
- legacy branch locking
- wrapping / re-export behavior

Deliverable:

- each crate tests what it actually owns

## Migration Guidance For Current `cjlib`

Short-term:

- keep the current temporary importer only as a stopgap
- do not expand it further
- do not implement templates / instances there

Medium-term:

- swap `cjlib::io` over to `serde_cityjson`
- delete the temporary importer

Long-term:

- keep `cjlib` small enough that opening `src/` makes the architecture obvious

## Done Criteria

This refactor is done when:

- `cjlib` no longer contains JSON-to-model conversion logic beyond trivial dispatch
- `serde_cityjson` can construct `cityjson::v2_0::OwnedCityModel` correctly for v2.0
- template geometry and geometry instance handling live below `cjlib`
- `cityjson-rs` remains the only in-memory model source of truth
- `cjlib` preserves its convenience constructors and version facade
- tests are aligned with crate responsibilities

## Recommendation

Do not invest further in the temporary `cjlib` importer.

The cleanest end state is:

- `serde_cityjson` handles indexed JSON
- `cityjson-rs` provides the validated model target and any minimal import hooks
- `cjlib` becomes a very small facade again
