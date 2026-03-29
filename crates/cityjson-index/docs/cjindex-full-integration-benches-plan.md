# Plan for Full Integration Benches Across All `cjindex` Storage Layouts

## Goal

Take `cjindex` from the current mixed benchmark state to a point where all
three supported storage layouts can be benchmarked through the real
`CityIndex` API in release mode:

- feature-files
- regular `CityJSON`
- `NDJSON` / `CityJSONSeq`

The target is not just "some benches exist". The target is a consistent,
defensible benchmark harness that exercises the same public operations across
all layouts and can be run as the full integration benchmark suite.

## Desired End State

At the end of this work, `cargo bench` should cover the following operations
for each storage layout:

- `reindex`
- `get`
- `query`
- `query_iter`
- `metadata`

Those benches should:

- use the real `CityIndex` public API
- run against prepared fixtures derived from real data
- isolate benchmark setup from measured work
- run cleanly in release mode
- be comparable across layouts

## Current State

The current bench setup is uneven:

- [benches/ndjson.rs](/home/balazs/Development/cjindex/benches/ndjson.rs)
  already benchmarks `CityIndex::{reindex,get,query,query_iter,metadata}`
- [benches/cityjson.rs](/home/balazs/Development/cjindex/benches/cityjson.rs)
  only benchmarks raw JSON parse of one file
- [benches/feature_files.rs](/home/balazs/Development/cjindex/benches/feature_files.rs)
  only benchmarks raw JSON parse of one file

That means the benchmark suite currently mixes two different questions:

- "how expensive is raw JSON parsing for this layout?"
- "how expensive is the indexed public API for this layout?"

The next work should remove that mismatch.

## Design Principles

### Benchmark the public API, not parsing internals

The integration benchmark suite should answer:

- how expensive is indexing this layout?
- how expensive is reading one known feature by ID?
- how expensive is bbox lookup through the index?

It should not primarily answer:

- how expensive is `serde_json::from_slice` on one file?

Raw parse benches can remain as secondary micro-benchmarks later, but they
should not be confused with integration benches.

### Keep fixture shape comparable

Each layout should be benchmarked from a fixture that is:

- derived from the same real-data source family
- small enough for routine local runs
- large enough to exercise realistic indexing and lookup paths

The benchmark harness should avoid comparing:

- one full tile for layout A
- one single-feature file for layout B
- one synthetic two-line file for layout C

That kind of mismatch makes the numbers hard to interpret.

### Share harness code aggressively

The three bench files should differ mostly in layout-specific fixture setup.
The operation timing logic should be shared.

## Phase 1: Define the Benchmark Matrix

Before changing code, lock down the benchmark contract.

For each layout, the suite should measure:

1. `reindex`
2. `get`
3. `query`
4. `query_iter`
5. `metadata`

Recommended naming:

- `feature_files_reindex`
- `feature_files_get`
- `feature_files_query`
- `feature_files_query_iter`
- `feature_files_metadata`
- `cityjson_reindex`
- `cityjson_get`
- `cityjson_query`
- `cityjson_query_iter`
- `cityjson_metadata`
- `ndjson_reindex`
- `ndjson_get`
- `ndjson_query`
- `ndjson_query_iter`
- `ndjson_metadata`

This naming makes `cargo bench` output directly comparable and avoids the
current ambiguity around `*_parse`.

## Phase 2: Introduce Shared Benchmark Helpers

Add a small benchmark support module under [benches/](/home/balazs/Development/cjindex/benches)
to centralize repeated setup.

Suggested responsibilities:

- create a unique temp SQLite index path
- create or reuse a prepared benchmark fixture root
- open `CityIndex` for a given `StorageLayout`
- build a fully indexed `CityIndex` for steady-state benches
- locate one stable feature ID for `get`
- compute one stable bbox that is known to hit at least one result

This should remove the current duplication in
[benches/ndjson.rs](/home/balazs/Development/cjindex/benches/ndjson.rs) and
prevent copy-paste drift when CityJSON and feature-files are upgraded.

### Helper split

Keep the helpers in two layers:

- layout-agnostic benchmark harness helpers
- layout-specific fixture builders

That keeps the Criterion timing code identical across all layouts.

## Phase 3: Standardize Benchmark Fixtures

The current NDJSON bench derives a tiny synthetic fixture from one real source
line. That is useful for quick smoke/perf feedback, but it is too small and too
layout-specific to serve as the full integration benchmark shape by itself.

Add a benchmark fixture preparation path with explicit outputs for all three
layouts.

Recommended options:

1. Reuse the prepared dataset root under
   `/home/balazs/Data/3DBAG_3dtiles_test/cjindex` directly when it exists.
2. Add a "materialize benchmark subset" helper that copies a small,
   representative subset into a temp benchmark root.

The second option is better for consistency and isolation.

### Recommended fixture contract

For each layout, prepare a subset that contains:

- more than one source file
- more than one feature
- at least one bbox query that returns multiple hits
- at least one stable known feature ID

That avoids benches degenerating into trivial single-record lookups.

### Preferred implementation

Reuse the existing support in
[tests/common/data_prep.rs](/home/balazs/Development/cjindex/tests/common/data_prep.rs)
and [tests/common/mod.rs](/home/balazs/Development/cjindex/tests/common/mod.rs)
where practical, but do not couple benchmark code directly to test-only
helpers if that forces awkward imports.

If the overlap becomes large, extract shared fixture-prep utilities into a
small internal module that both benches and tests can use.

## Phase 4: Upgrade Feature-Files Benches

Replace the raw parse benchmark in
[benches/feature_files.rs](/home/balazs/Development/cjindex/benches/feature_files.rs)
with the full public-API benchmark matrix.

Implementation steps:

- choose or prepare a representative feature-files subset
- open `CityIndex` with `StorageLayout::FeatureFiles`
- benchmark `reindex`
- benchmark `get` against a known feature ID
- benchmark `query` using a known-hit bbox
- benchmark `query_iter`
- benchmark `metadata`

This is the simplest layout and should be the first one converted.

## Phase 5: Upgrade CityJSON Benches

Replace the raw parse benchmark in
[benches/cityjson.rs](/home/balazs/Development/cjindex/benches/cityjson.rs)
with the same public-API matrix used elsewhere.

Implementation steps:

- choose or prepare a representative regular-`CityJSON` subset
- open `CityIndex` with `StorageLayout::CityJson`
- benchmark the same five operations

Specific risks to watch:

- `reindex` may be dominated by byte-range scanning and bbox derivation
- `get` may reflect one-object extraction and feature/base reconstruction cost
- `query` may amplify repeated per-result reads if the bbox is too broad

That is acceptable. The point is to measure the real layout-specific behavior
through one common API.

## Phase 6: Bring NDJSON Onto the Same Fixture Standard

[benches/ndjson.rs](/home/balazs/Development/cjindex/benches/ndjson.rs) already
benchmarks the right operations, but its fixture strategy should be brought in
line with the other layouts.

Recommended changes:

- stop relying only on the tiny synthetic one-feature file for the main suite
- use the same benchmark-subset strategy as the other layouts
- keep the small synthetic fixture only if it remains useful as a dedicated
  micro-benchmark

If both are kept, name them differently so the full integration bench is not
confused with the micro-bench.

## Phase 7: Make the Bench Inputs Stable and Comparable

Benchmark comparability depends heavily on choosing the same logical workload.

For each layout, define:

- one stable feature ID used by `get`
- one stable bbox used by `query`
- expected minimum hit counts for `query` and `query_iter`

Store those choices in code close to the fixture builder, not as scattered
magic constants in each bench file.

### Selection rules

Prefer:

- a bbox that returns a small handful of features
- a feature ID that exists in every prepared subset instance
- a subset with deterministic file membership

Avoid:

- a bbox covering the whole dataset
- selecting "the first file found on disk" if ordering could drift
- depending on external mutable fixture roots without a stable subset step

## Phase 8: Separate Setup Cost From Measured Work

Criterion benches should avoid mixing one-time fixture creation into the timed
section.

### `reindex`

Measure:

- opening a fresh index path
- performing `reindex`

Do not measure:

- expensive one-time fixture generation from raw upstream data

Use `iter_batched_ref` or equivalent so each iteration gets a fresh index path
against a stable prepared fixture root.

### `get`, `query`, `query_iter`, `metadata`

Measure:

- operations against an already indexed `CityIndex`

Do not measure:

- rebuilding the fixture root every iteration
- repeated selection/discovery of the target feature ID or bbox

The current NDJSON bench already follows this pattern and should be the model
for the other two layouts.

## Phase 9: Add a Full Release-Mode Bench Run Contract

Define the expected command for the full integration benchmark suite:

```bash
cargo bench --bench feature_files --bench cityjson --bench ndjson
```

Use explicit Criterion options when needed for faster iteration during
development, but the acceptance run should be the full release-mode bench suite.

Recommended developer smoke command:

```bash
cargo bench --bench feature_files --bench cityjson --bench ndjson -- --sample-size 10
```

The plan should treat that as a convenience, not as the semantic definition of
completion.

## Phase 10: Document How to Read the Results

Once all three layouts benchmark the same operations, the docs should make the
comparison rules explicit.

Interpretation guidance should include:

- `reindex` compares indexing cost across layouts
- `get` compares one-feature materialization through the indexed path
- `query` compares eager bbox lookup plus eager reads
- `query_iter` compares bbox lookup plus lazy iteration overhead
- `metadata` compares cached source-metadata access

This matters because some layout differences are expected:

- feature-files trade more filesystem fan-out for simpler per-feature reads
- regular `CityJSON` trades fewer files for more extraction work per feature
- NDJSON trades line-oriented scanning for per-feature record boundaries

Without that framing, users will overread single benchmark numbers.

## Recommended Implementation Order

1. Add shared benchmark helpers and fixture-selection rules.
2. Convert feature-files from parse bench to public-API bench.
3. Convert CityJSON from parse bench to public-API bench.
4. Upgrade NDJSON to use the same fixture standard.
5. Run the full release-mode benchmark suite.
6. Adjust helper code or fixture size only if results expose obvious benchmark
   distortion.

This order keeps the changes incremental while converging on a consistent suite
quickly.

## Acceptance Criteria

The work is complete when:

- all three bench files measure the same five `CityIndex` operations
- the main benches no longer benchmark raw `serde_json::Value` parsing only
- fixtures are derived from real data and chosen consistently across layouts
- the full suite runs in release mode through `cargo bench`
- the resulting numbers are comparable enough to support layout-level analysis

## Non-Goals

This plan does not require:

- making the three layouts equally fast
- eliminating all synthetic fixtures
- building a full benchmark-reporting pipeline
- optimizing `cjindex` before the comparable suite exists

The immediate goal is a trustworthy integration benchmark baseline. Optimization
comes after that baseline is in place.
