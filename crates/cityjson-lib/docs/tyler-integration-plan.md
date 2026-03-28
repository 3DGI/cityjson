# Tyler Integration Plan

This plan assumes we are willing to make breaking changes in `tyler` and
delete legacy CityJSON parsing code if the result is cleaner.

The target is not a compatibility shim and not a Tyler-specific adapter layer.
The target is to make `tyler` consume `cjlib` as its CityJSON boundary and to
let that dogfood exercise drive the next round of `cjlib` API design.

## Objective

Replace `tyler`'s local CityJSON parsing with `cjlib` as far as we can do so
cleanly, using the existing 3DBAG test input at:

- `/home/balazs/Data/3DBAG_3dtiles_test/input/metadata.json`
- `/home/balazs/Data/3DBAG_3dtiles_test/input/features/`

The current test dataset shape is already aligned with `cjlib`'s implemented
path:

- metadata file is `CityJSON` `2.0`
- features are separate `.city.jsonl` files under a directory tree
- there are about `227045` feature files in the test tree

## What Tyler Actually Needs

Today `tyler` does not need a general-purpose CityJSON API.
Its parser in `tyler/src/parser.rs` needs a narrow subset:

- metadata:
  - `transform`
  - `metadata.referenceSystem`
- per-feature:
  - `CityObjects`
  - `vertices`
  - geometry boundaries
  - object types

That data is used to compute:

- dataset extent
- feature filtering by object type
- feature centroid
- feature bbox
- grid-cell assignment
- feature path lists handed off to `geof`

Downstream, `tyler` does not currently need to serialize CityJSON again.
It mostly passes feature file paths to `geof`.

## Coordinate Model Implication

This plan must account for an important model difference:

- current `tyler` parsing keeps feature vertices in quantized coordinates and
  repeatedly applies `transform.scale` and `transform.translate`
- `cjlib` wraps `cityjson-rs`, whose `OwnedCityModel` stores geometry vertices
  as real-world coordinates internally

That means the `tyler` rewrite should not port the current quantized-coordinate
accounting into the new architecture.

It should delete it.

In particular, these current Tyler concepts become suspect once parsing moves to
`cjlib`:

- `centroid_qc`
- `bbox_qc`
- `BboxQc`
- repeated `to_bbox(transform, ...)` conversions
- repeated per-vertex transform application during indexing
- storing `Transform` only to support quantized-to-real-world conversion

`Transform` may still matter as source metadata, but it should no longer be the
operational basis of feature indexing if `cjlib` has already normalized the
model to real-world coordinates.

## Target Architecture

`tyler` should stop owning CityJSON serde structs and parsing logic.

Instead:

- `cjlib` owns all CityJSON file parsing
- `tyler` uses `cjlib::CityModel`, `cjlib::json`, and the re-exported
  `cjlib::cityjson` types directly
- `tyler` owns tiling, grid logic, feature placement, subprocess orchestration,
  and output formats

The desired layering is:

1. `cjlib`
   - parse metadata document
   - parse feature file
   - expose the generic model surface
2. `tyler`
   - consume those views
   - compute extent, indexing, quadtree, and tile inputs
   - remain ignorant of JSON schema details

This is cleaner than keeping duplicate CityJSON structs in `tyler` and cleaner
than adding `tyler`-specific custom views to `cjlib`.

## Tyler Changes

### Phase 1: Remove Local Metadata Parsing

Delete or stop using:

- `CityJSONMetadata`
- `Transform` as a serde-boundary type
- `Metadata`
- `Crs` as a serde-boundary type

Replace `World::new` so metadata loading comes through `cjlib`.

The resulting `tyler` code should depend on `cjlib::CityModel` and/or
`cjlib::cityjson` access, not on `serde_json::from_str`.

### Phase 2: Remove Local Feature Parsing

Delete or stop using:

- `CityJSONFeatureVertices`
- feature-file `from_file` parsing in `tyler/src/parser.rs`
- quantized-coordinate feature bookkeeping tied to the legacy parser

Replace the hot paths in `tyler/src/parser.rs` so they consume `cjlib`
feature models directly:

- `extent_qc`
- `extent_qc_init`
- `extent_qc_file`
- `index_feature_path`
- `count_vertices`
- `feature_to_cells`

At this point `tyler` should no longer parse CityJSON directly.

At the same time, rewrite the indexing math to operate directly on real-world
coordinates from `cjlib`, not on quantized coordinates plus delayed transform
application.

### Phase 3: Keep Tyler Domain Types, Drop Tyler Schema Types

Retain `tyler` domain types that are genuinely Tyler-specific:

- `World`
- `Feature`
- grid and quadtree types
- tile-output types

Do not retain local copies of CityJSON schema concepts when they are only there
to deserialize JSON.

Do not retain quantized-coordinate domain types when their only purpose was to
compensate for the old parsing model.

### Phase 4: Simplify Parser Structure

Once `cjlib` owns the parsing boundary, simplify `parser.rs` around that fact.

Likely outcome:

- rename `parser.rs` to a more accurate module such as `input.rs` or
  `cityjson_index.rs`
- isolate filesystem walking from feature scanning
- move the remaining indexing logic fully into real-world coordinates
- keep only:
  - feature discovery
  - feature filtering
  - extent aggregation
  - grid assignment

### Phase 5: Compare Outputs, Then Delete Legacy Branches

Before deleting legacy code completely, run parity checks on the 3DBAG test set.
After parity is acceptable, remove the old parser types and branches entirely.

## Required `cjlib` Additions

The main gap is not format support. The main gap is making the existing
`CityModel`-centric surface practical enough for real systems.

### 1. `from_feature_file`

`cjlib::json::from_file` is document-oriented and rejects `.jsonl` files.
That is reasonable for the public document API, but `tyler` needs an explicit
feature-file helper:

```rust
pub fn from_feature_file(path: impl AsRef<Path>) -> Result<CityModel>;
```

This should be a thin convenience wrapper over `from_feature_slice`.

### 2. Generic Model Ergonomics

`tyler` should use the actual `cjlib` model surface, not a second API.
If that is awkward in practice, the fix should be a generic improvement to
`CityModel` or to the re-exported `cityjson-rs` surface.

The kinds of additions that fit this rule are:

- generic accessors on `CityModel`
- generic traversal helpers
- generic convenience constructors
- generic file helpers

The kinds of additions that do not fit this rule are:

- Tyler-specific metadata views
- Tyler-specific scan objects
- parallel APIs that duplicate model access in a different shape

### 3. Geometry And Metadata Access

The dogfood pass should answer this directly:

- Is `cjlib::CityModel` plus `as_inner()` enough?
- Are the re-exported `cjlib::cityjson` types ergonomic enough?
- Do we need a few generic methods on `CityModel` to make common reads obvious?
- Does `tyler` need any transform access at all once feature indexing runs on
  real-world coordinates?

If we add helpers, they should be generic and model-oriented, for example:

- `CityModel::transform()`
- `CityModel::reference_system()`
- `CityModel::vertices()`
- `CityModel::city_objects()`

Those are still one API, not a second projection layer.

The key design rule is that `cjlib` should expose the actual model cleanly
enough that `tyler` can delete its quantized-coordinate shadow model.

### 4. Benchmarks For Tyler Workloads

`cjlib` currently lacks a benchmark harness for the parser workloads that matter
to `tyler`.

Add benchmarks for:

- metadata load
- single feature parse
- bbox/type scan of a feature
- directory-tree extent scan

Without these, we will be arguing about performance from anecdotes.

## Recommended `cjlib` API Shape

The cleanest production-facing direction is to keep one real API:

- `cjlib::CityModel`
- `cjlib::json`
- `cjlib::cityjson`

Dogfooding with `tyler` should validate whether that surface is already enough.
If it is not enough, improve that same surface rather than adding a second one.

## Performance Plan

Do not start by assuming that full `CityModel` parse is the right benchmark.
`tyler`'s hot path is bulk feature scanning over a directory tree.

Measure both the legacy and `cjlib` versions for:

1. metadata load time
2. extent scan over `/home/balazs/Data/3DBAG_3dtiles_test/input/features/`
3. `index_with_grid`
4. total wall-clock time for a representative `tyler` run
5. peak RSS

Acceptance criterion for the first integration round:

- equal results
- operationally acceptable performance

Not:

- instant full replacement of every low-level optimization

If `cjlib` loses badly on extent scan or grid indexing, that is a signal to
improve the core model/file API, not a reason to keep duplicate parsing logic
in `tyler`.

The benchmark comparison should also acknowledge that the rewritten Tyler path
is allowed to have different internal arithmetic if it now operates directly on
real-world coordinates instead of quantized coordinates.

## Execution Order

### Milestone 1: Parser Rewrite Seam

In `tyler`:

- add `cjlib` dependency
- route metadata loading through `cjlib`
- identify and remove transform-dependent indexing code that only exists for
  quantized coordinates

In `cjlib`:

- add `from_feature_file`

### Milestone 2: Feature Scan Integration

In `tyler`:

- replace `CityJSONFeatureVertices` usage across extent and grid indexing
  with `cjlib::CityModel` traversal
- replace quantized centroid/bbox bookkeeping with real-world coordinate
  bookkeeping
- delete local serde structs for feature parsing

### Milestone 3: Parity Validation

Run the 3DBAG test input and compare:

- feature counts
- ignored counts
- dataset extent
- grid statistics
- generated tile input lists

Because the arithmetic path changes, parity should be evaluated at the level of
tiling-relevant outcomes, not by insisting that legacy quantized intermediates
still exist.

### Milestone 4: Performance Validation

Benchmark legacy vs rewritten parser path.

If the rewritten path is acceptable:

- delete legacy parser implementation completely

If the rewritten path is not acceptable:

- improve `cjlib::CityModel` and related generic APIs
- do not restore duplicate parsing code in `tyler`

### Milestone 5: Prepare FFI On Top Of Real Usage

After the `tyler` dogfood pass stabilizes the Rust API:

- add a `cjlib-ffi` crate with a narrow C ABI
- add Python bindings on top of that or on top of a small Rust wrapper crate

The FFI surface should be based on the proven parser and scan APIs, not on
speculation.

## Explicit Non-Goals For This Phase

- Arrow integration
- Parquet integration
- generalized operations API for `tyler`
- CityJSON stream aggregation in `cjlib`
- preserving `tyler`'s existing parser architecture for compatibility's sake

## Success Criteria

This integration phase is successful when:

- `tyler` no longer owns CityJSON parsing structs
- `tyler` reads metadata and features through `cjlib`
- `tyler` no longer depends on quantized-coordinate indexing machinery that
  exists only because of the old parser
- the 3DBAG test dataset runs correctly
- output parity is acceptable
- performance is measured, not guessed
- the resulting `cjlib` additions are improvements to the main API rather than
  a second Tyler-specific API

## Summary

The clean move is to treat `tyler` as the first serious consumer of `cjlib`,
not as a special case to be adapted around.

That means:

- rewrite `tyler`'s CityJSON boundary aggressively
- use `cjlib`'s real API surface and improve that surface where needed
- let the real-world coordinate model simplify Tyler instead of preserving the
  old quantized architecture
- validate on the real 3DBAG test corpus
- only then freeze any FFI surface
