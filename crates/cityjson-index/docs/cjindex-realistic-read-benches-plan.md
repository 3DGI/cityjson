# Plan for Realistic Read Benchmarks in `cjindex`

## Goal

Replace the current cache-friendly single-ID `get` benchmark and single-bbox
query benchmark with a more representative steady-state read workload on the
full prepared corpus.

The target benchmark contract is:

- 1,000 deterministic "random" `get` lookups per measured run
- 10 alternating bbox queries per measured run
- the same contract for:
  - feature-files
  - regular `CityJSON`
  - `NDJSON` / `CityJSONSeq`

The point is not to simulate production perfectly. The point is to stop
benchmarking one hot object and one repeated bbox while still keeping the suite
repeatable and comparable across layouts.

## Why This Is Needed

The current full-corpus harness answers only part of the read question:

- `get` repeats one stable feature ID, which is intentionally friendly to OS
  and process caches
- `query` repeats one stable bbox, which is useful, but too narrow as the only
  spatial-read workload

That makes the current read numbers valid as hot steady-state baselines, but
too optimistic and too shape-specific for broader interpretation.

We need the next harness revision to answer:

- how does repeated lookup behave when we touch many different objects?
- how do bbox queries behave across several real spatial windows rather than
  one fixed tile-local union?
- does the relative ordering between storage layouts hold under a more varied
  read workload?

## Non-Goals

- true runtime randomness during the benchmark
- cache-cold benchmarking
- statistically perfect workload modeling
- mixing benchmark generation cost into measured work

The harness should stay deterministic and engineering-friendly.

## Benchmark Contract

### `get`

Each measured benchmark iteration for `get` should execute:

- 1,000 `CityIndex::get(...)` calls

Those 1,000 IDs should come from a deterministic pseudo-random sample of the
full corpus.

Requirements:

- the sample must be identical across layouts
- the sample must be stable across repeated local runs
- the sample should span many source files and many tiles
- the sample must avoid pathological concentration in one tile or one file

Recommended shape:

- derive the ID pool from the canonical feature-files corpus
- sort all IDs lexicographically
- use a fixed seeded shuffle
- take the first 1,000 IDs after shuffling

This gives "random enough" coverage without introducing run-to-run drift.

### `query`

Each measured benchmark iteration for `query` and `query_iter` should execute:

- 10 bbox queries

Those 10 bbox queries should alternate across a deterministic set of 10 known
real bounding boxes.

Requirements:

- all 10 bboxes must be derived from the same underlying real corpus
- all 10 bboxes must return non-empty results in every layout
- the set should contain a mix of lighter and denser windows
- the set should not collapse into 10 nearly identical queries over the same
  exact object cluster

Recommended shape:

- group features by tile
- choose 10 stable qualifying tiles
- for each tile, derive one bbox from a deterministic subset of features within
  that tile
- keep result sizes in a bounded target range so one outlier tile does not
  dominate the whole benchmark

Suggested target:

- each bbox should hit roughly 100 to 1,000 features

That keeps the workload realistic while keeping total benchmark time
manageable.

## Design Principles

### Deterministic pseudo-randomness

The harness should not call a random generator inside the measured closure.

Instead:

- select the ID sample and bbox set once during setup
- use a fixed seed
- store the final ordered workload vectors
- cycle through them in the measured closures

This preserves repeatability and avoids contaminating timing with workload
generation.

### Batch the measured work explicitly

The Criterion unit of measurement should be one workload batch, not one API
call.

That means:

- one `get` iteration = 1,000 `get` calls
- one `query` iteration = 10 bbox queries
- one `query_iter` iteration = 10 bbox iterator walks

This makes the benchmark names honest and prevents Criterion from overfitting
to extremely small per-call timings.

### Keep `metadata` and `reindex` unchanged

This plan only changes the read workloads.

`reindex` and `metadata` can keep the current contract:

- `reindex`: one full rebuild per iteration
- `metadata`: one metadata call per iteration

## Implementation Plan

### Phase 1: Add workload descriptors to the shared harness

Extend [benches/support.rs](/home/balazs/Development/cjindex/benches/support.rs)
with typed benchmark inputs:

- `get_ids: Vec<String>`
- `query_bboxes: Vec<[f64; 4]>` or the project-equivalent bbox type

These should be derived once during setup and then reused for all layouts.

### Phase 2: Build the deterministic 1,000-ID sample

Implementation steps:

1. enumerate all feature IDs from the canonical feature-files corpus
2. sort them lexicographically
3. apply a fixed seeded shuffle
4. take 1,000 IDs
5. validate that every selected ID resolves in all three layouts

If any ID is missing from one layout, fail setup loudly rather than silently
dropping it.

### Phase 3: Build the deterministic 10-bbox set

Implementation steps:

1. enumerate candidate tiles from the feature-files corpus
2. keep tiles with enough features to support a meaningful bbox
3. choose 10 stable tiles after lexicographic sort
4. for each tile, select a deterministic feature subset
5. compute one union bbox per tile
6. validate that each bbox returns results in all three layouts

If some tiles are too sparse or too dense, adjust the tile-selection heuristic
before shipping the benchmark.

### Phase 4: Refactor bench execution to batch the operations

Update the harness so:

- `get` loops over the 1,000 IDs inside one measured iteration
- `query` loops over the 10 bboxes and materializes all results
- `query_iter` loops over the same 10 bboxes and fully drains the iterators

Use `black_box(...)` on both inputs and outputs so the compiler cannot discard
the work.

### Phase 5: Revisit Criterion sizing

The new workload is materially heavier.

After implementation:

- rerun the suite in release mode
- inspect total runtime
- tune `sample_size` if needed

It may make sense to keep:

- one conservative sample size for CityJSON
- one slightly larger sample size for feature-files and NDJSON

But use a single policy first unless runtime becomes unreasonable.

## Verification Plan

Before trusting the numbers:

1. verify that the same 1,000 IDs resolve successfully in all three layouts
2. verify that all 10 bboxes are non-empty in all three layouts
3. verify that `query` and `query_iter` return matching total counts for the
   same bbox set
4. run `cargo test --release --tests`
5. run the full bench suite in release mode

## Expected Outcome

At the end of this work:

- `get` will no longer be benchmarking one hot object repeatedly
- `query` will no longer depend on one single bbox shape
- the benchmark results will better reflect warm multi-object lookup behavior
- the ADR and benchmark results doc can describe the read workload precisely

## Follow-up

After this lands, the next useful split would be:

- hot `get` benchmark
- many-ID warm `get` benchmark
- dense tile-local bbox benchmark
- broader multi-window bbox benchmark

But the immediate priority is to replace the misleading single-ID `get`
contract with the 1,000-ID batch contract.
