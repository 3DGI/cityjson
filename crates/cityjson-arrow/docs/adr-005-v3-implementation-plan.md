# ADR 005 V3 Implementation Plan

This document defines the implementation sequence for
[ADR 005: cut `v3alpha1` schema for Arrow-native projection and batch-native conversion](adr/005-cut-v3-schema-for-arrow-native-projection-and-batch-native-conversion.md).

It translates the ADR into concrete workstreams, file-level change targets, and
acceptance gates.

## Scope

This plan is about one clean breaking refactor.

Keep:

- the public semantic boundary centered on `OwnedCityModel`
- one live stream surface in `cityjson-arrow`
- one persistent package surface in `cityjson-parquet`
- the canonical table set and ordinal relation rules established by ADR 4

Do not add in this slice:

- dual-read support for `v2alpha2`
- migration helpers
- a legacy `_json` compatibility lane
- JSON string fallback for heterogeneous attributes
- a partial refactor that leaves both row-staged and batch-native conversion
  paths alive

## Goals

The implementation must deliver these end-state properties:

1. generic projected attributes are Arrow-native and recursive
2. `encode_parts` writes arrays through table-local Arrow builders instead of
   whole `Vec<Row>` staging
3. `decode_parts` consumes bound arrays and cheap indices instead of
   reconstructing row vectors
4. the only supported package version is `cityjson-arrow.package.v3alpha1`
5. the old stringified projection model is removed rather than preserved behind
   branches

## Workstream 1: Cut The Format First

Purpose:
make the break explicit before touching conversion internals so the new code is
not forced to carry old assumptions.

Primary targets:

- `src/schema.rs`
- `src/convert/mod.rs`
- `src/stream.rs`
- `cityjson-parquet/src/package/mod.rs`
- docs that name the current schema id

Work:

- replace `cityjson-arrow.package.v2alpha2` in `src/schema.rs` with
  `cityjson-arrow.package.v3alpha1`
- add `V3Alpha1` to `CityArrowPackageVersion`
- make manifest and header construction default to `V3Alpha1`
- remove `V2Alpha2` from `FromStr`, display, and serde-facing mainline paths
- make stream and package readers reject non-`v3alpha1` input immediately
- update spec and package-schema docs after the code-side type change lands

Deliverables:

- one schema id in the codebase: `cityjson-arrow.package.v3alpha1`
- one mainline read/write format

Exit criteria:

- there is no code path that reads or writes `v2alpha2`
- no compatibility adapter or translation layer exists

## Workstream 2: Replace Flat Projection Types With Recursive Specs

Purpose:
make the schema capable of expressing the structure ADR 005 requires.

Primary targets:

- `src/schema.rs`
- `src/convert/mod.rs`

Current problem:

- `ProjectedValueType` is scalar-only
- `ProjectionLayout` stores flat `Vec<ProjectedFieldSpec>` namespaces
- generic attributes are forced into flat `LargeUtf8` columns

Work:

- replace `ProjectedValueType` and the flat field-only layout with a recursive
  projection grammar
- represent `bool`, `i64`, `u64`, `f64`, `utf8`, `geometry_ref`, `list<T>`, and
  `struct{...}` directly in schema types
- make `ProjectedFieldSpec` describe one named field with recursive value shape
- make each dynamic namespace in `ProjectionLayout` optional struct-shaped
  layout instead of `Vec<ProjectedFieldSpec>`
- teach schema generation to emit Arrow `Struct`, `List`, scalar, and
  `UInt64`-backed `geometry_ref` fields directly

Recommended boundary:

- keep material and texture canonical payload fields on their current typed
  explicit path in this slice
- apply the recursive projection model to:
  - metadata root extra
  - metadata extra
  - cityobject attributes
  - cityobject extra
  - geometry extra
  - semantic attributes

Deliverables:

- recursive projection types in `src/schema.rs`
- canonical schema builders that emit nested Arrow fields directly

Exit criteria:

- the schema layer can express every supported generic attribute shape without
  referring to JSON strings
- no `_json` naming convention remains in projection metadata

## Workstream 3: Replace Projection Discovery With Strict Structural Inference

Purpose:
infer `v3alpha1` projection layouts directly from CityJSON attribute values.

Primary targets:

- `src/convert/mod.rs`

Current problem:

- generic attribute discovery emits flat `*_json` columns
- nested structure is lost at projection-discovery time

Work:

- delete the `_json` projection convention and related prefix/suffix rules
- replace flat attribute discovery with recursive structural inference
- infer one stable shape per attribute path across the whole model
- treat `null` as nullability only
- union object keys recursively into struct fields
- unify list item shapes recursively
- represent geometry handles as a first-class logical `geometry_ref`
- fail fast on incompatible shapes instead of normalizing or stringifying them

Required failure cases:

- scalar kind mismatch such as `u64` versus `utf8`
- scalar versus container mismatch
- incompatible list item shape
- `geometry_ref` mixed with scalar numeric values

Deliverables:

- recursive projection-discovery helpers
- deterministic schema errors for incompatible attribute shapes

Exit criteria:

- generic projection discovery does not emit flat JSON string columns
- incompatible models fail during export before any table data is written

## Workstream 4: Rewrite `encode_parts` Around Table Encoders

Purpose:
move export from row staging to direct Arrow-native table construction.

Primary targets:

- `src/convert/mod.rs`

Current hotspots:

- `emit_tables`
- `cityobject_rows`
- `cityobjects_batch`
- geometry, boundary, semantic, and appearance row builders and batch builders

Work:

- refactor `emit_tables` into orchestration only
- introduce dedicated encoder types such as:
  - `CityObjectsEncoder`
  - `GeometriesEncoder`
  - `BoundariesEncoder`
  - `SemanticsEncoder`
  - `AppearanceEncoder`
- make each encoder own Arrow builders and expose `push_*` plus `finish()`
- keep one prepass for id-map discovery where needed
- append exported values directly into builders while traversing the
  `OwnedCityModel`
- remove whole-model `Vec<Row>` staging from the hot path

Recommended sequence:

1. cityobjects
2. geometries and geometry boundaries
3. semantics
4. materials and textures
5. template geometry tables

Rationale:

- cityobjects are the narrowest proving ground and include the generic
  attribute-path redesign
- geometry and boundary export are the largest remaining row-staging cost center

Deliverables:

- table-local encoder structs with Arrow builders
- `emit_tables` as a table-order coordinator rather than a row aggregator

Exit criteria:

- `encode_parts` performs one prepass for ids and then direct table population
- no whole-table row vectors remain on the export hot path

## Workstream 5: Make Generic Attribute Export Fully Arrow-Native

Purpose:
remove the JSON conversion path from export completely.

Primary targets:

- `src/convert/mod.rs`

Current problem:

- `project_one_attribute` converts
  `OwnedAttributeValue -> serde_json::Value -> String`
- nested generic attributes are serialized rather than appended structurally

Work:

- delete `project_one_attribute`
- delete generic `attribute_to_json` export helpers used only for projected
  attributes
- replace them with recursive append helpers that write directly into the Arrow
  builder tree described by the projection layout
- map `geometry_ref` to its logical typed `u64` representation
- keep null handling explicit and structural

Deliverables:

- recursive `append_attribute(...)` style export helpers
- zero JSON stringify steps in generic attribute export

Exit criteria:

- generic projected attribute export does not use `serde_json`
- nested structs and lists are appended directly into Arrow builders

## Workstream 6: Rewrite `decode_parts` Around Batch Views And Ordered Indices

Purpose:
move import from row materialization to array-backed decoding.

Primary targets:

- `src/convert/mod.rs`

Current hotspots:

- `IncrementalDecoder::dispatch_table`
- `read_*_rows` helpers
- grouped row staging in `PartRowGroups`
- generic projected attribute import through temporary string vectors

Work:

- bind arrays once per batch and operate on typed views
- retain `RecordBatch` references or typed batch-view wrappers where later
  reconstruction depends on earlier table data
- replace grouped `HashMap<u64, Vec<Row>>` staging with cheap ordered indices
  such as `id -> Range<usize>`
- replace unique row maps with `id -> row_index` or direct row-position access
- keep table-order validation and fail immediately on out-of-order or invalid
  data

Recommended sequence:

1. cityobjects batch import
2. geometry boundary and geometry table import
3. semantic/material/texture attachment tables
4. template geometry tables

Deliverables:

- typed batch-view helpers for core canonical tables
- ordered span indices for grouped attachment tables

Exit criteria:

- `decode_parts` does not rebuild grouped `Vec<Row>` staging structures
- reconstruction is driven by arrays and ordered indices instead of row clones

## Workstream 7: Decode Generic Attributes Recursively From Arrays

Purpose:
remove JSON parsing and per-row temporary string vectors from import.

Primary targets:

- `src/convert/mod.rs`

Current problem:

- `import_cityobjects_batch` builds temporary `Vec<Option<String>>`
- `apply_projected_attributes` calls `serde_json::from_str`

Work:

- bind struct and scalar projected columns once
- read nested values directly from arrays while iterating rows
- rebuild `OwnedAttributeValue` recursively from Arrow arrays
- map `u64` geometry references back to geometry handles through the import
  state id map
- delete generic attribute import helpers that assume flat JSON-string columns

Deliverables:

- recursive attribute decode helpers
- direct cityobject and semantic projected-attribute reconstruction from arrays

Exit criteria:

- no `serde_json::from_str` remains in generic projected attribute import
- no temporary per-row `Vec<Option<String>>` remains in the hot path

## Workstream 8: Keep The Pass Structure Simple

Purpose:
avoid an over-engineered pipeline with multiple incompatible execution models.

Implementation rules:

- `encode_parts` does:
  - one prepass for id maps and projection discovery
  - direct table population through encoders
  - final `RecordBatch` emission in canonical table order
- `decode_parts` does:
  - batch ingestion and validation
  - ordered index construction
  - semantic model materialization
- no compatibility branches exist inside encode or decode hot paths
- no alternate legacy projection model exists beside the recursive one

Exit criteria:

- the high-level flow can be explained without mentioning legacy branches or
  temporary compatibility shims

## Workstream 9: Delete The Old Model Completely

Purpose:
finish the break cleanly so the old design cannot drift back in.

Primary targets:

- `src/convert/mod.rs`
- `src/schema.rs`

Delete:

- row structs that only existed to stage canonical export/import
- `read_*_rows` helpers on the conversion hot path
- `project_attribute_columns`
- `apply_projected_attributes`
- `_json` suffix and prefix decoding helpers used only for the old layout
- schema helpers that only exist for stringified projection columns
- old package-version branches

Deliverables:

- one conversion model
- one projection model
- one package version

Exit criteria:

- the old stringified projection design is not present in dead code or dormant
  helpers

## Validation And Acceptance

Correctness gates:

- extend round-trip tests to cover:
  - nested struct attributes
  - list attributes
  - null-heavy attributes
  - geometry-reference attributes
  - incompatible-shape failure cases
- update any package/schema assertions that pin the old version or old flat
  projection layout

Performance gates:

- use the split benchmark surface in `benches/split.rs`
- compare `convert_encode_parts` and `convert_decode_parts` before and after
  each major workstream
- do not treat stream/package transport numbers as success criteria for this
  slice, because ADR 4 already showed those are not the bottleneck

Acceptance:

1. there is no generic attribute JSON stringify or parse path anywhere in
   encode or decode
2. there are no `Vec<Row>` staging structures in the conversion pipeline
3. the only supported package version is `cityjson-arrow.package.v3alpha1`
4. split benchmarks in `benches/split.rs` show material improvement in
   `convert_encode_parts` and `convert_decode_parts`

## Recommended Delivery Sequence

To keep the branch reviewable, land the work in this order:

1. schema version cut plus recursive schema types
2. structural projection inference
3. cityobject export/import on the new recursive projection model
4. geometry and boundary export/import on direct builder and batch-view paths
5. semantic and appearance table migration
6. deletion of old row-staging and JSON helpers
7. benchmark pass and documentation updates
