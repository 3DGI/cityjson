# ADR: Deterministic Realistic Read Batches and Indexed Byte-Range Reads

## Status

Accepted

## Date

2026-03-29

## Context

The previous integration benchmark harness established one useful property: all
three storage layouts were being measured through the same `CityIndex` API.

Two important problems remained:

1. the steady-state read workload was too friendly
2. `NDJSON` and regular `CityJSON` still reread whole source files on each hit

The old harness measured:

- one repeated stable `get`
- one repeated stable bbox for `query`
- one repeated stable bbox for `query_iter`

That answered only a hot-object and hot-window question. It did not answer how
the backends behave when the workload touches many objects and several spatial
windows.

At the same time, the backend read paths were still leaving obvious indexed I/O
performance on the table:

- `NDJSON` indexed one feature span, but still did `fs::read(...)` on the whole
  `.jsonl` file before slicing it
- `CityJSON` indexed the `CityObject` span and shared `vertices` span, but
  still did `fs::read(...)` on the full `.city.json` tile before slicing and
  rebuilding one-object feature payloads

That combination made the previous benchmark results directionally useful, but
too optimistic in workload shape and too pessimistic about what the indexed
layouts could do once they actually honored their stored byte ranges.

## Decision

We made two linked changes.

### 1. Replace the read benchmark contract with deterministic realistic batches

The steady-state benchmark contract is now:

- `get`: 1,000 deterministic pseudo-random lookups per measured iteration
- `query`: 10 deterministic real bbox queries per measured iteration
- `query_iter`: the same 10 deterministic bbox queries, fully drained

The workload vectors are built once during fixture setup from the canonical
feature-files corpus and then reused for all layouts.

The setup phase validates that:

- every selected ID resolves in feature-files, `CityJSON`, and `NDJSON`
- every selected bbox produces non-empty `query` and `query_iter` results in
  all three layouts

This keeps the suite deterministic and repeatable while avoiding the earlier
single-hot-object and single-hot-bbox bias.

### 2. Make indexed layouts read indexed byte ranges instead of whole files

We added a shared positioned-read helper in
[/home/balazs/Development/cjindex/src/lib.rs](/home/balazs/Development/cjindex/src/lib.rs)
that:

- opens a file
- seeks to the indexed offset
- reads exactly the indexed length
- fails clearly on oversized allocations or short reads

That helper is now used by:

- `NdjsonBackend::read_one`
- `CityJsonBackend::read_one`
- `FeatureFilesBackend::read_one`

For `CityJSON`, the read path now:

- reads only the indexed `CityObject` fragment
- reads only the indexed shared `vertices` fragment
- uses the already indexed/stored base metadata from SQLite instead of rereading
  the whole tile

We explicitly did not add a new SQLite schema field for base metadata, because
the `sources.metadata` column already contained the necessary CityJSON base
document produced during scanning.

## Implementation

### Benchmark harness

The main harness changes live in
[/home/balazs/Development/cjindex/benches/support.rs](/home/balazs/Development/cjindex/benches/support.rs).

Key implementation points:

- fixture setup now collects canonical feature records from the feature-files
  corpus
- `get` IDs are lexicographically sorted, fixed-seed shuffled, truncated to
  1,000, and reused for all layouts
- bbox workloads are built from 10 deterministic qualifying tiles and reused for
  all layouts
- the setup path validates workload correctness against all three layouts before
  Criterion starts measuring
- measured closures now batch the read work explicitly instead of measuring one
  API call per iteration

During the real run, the harness exposed two path-resolution bugs in workload
setup:

- metadata discovery initially only walked `feature-files/features`
- metadata ancestor resolution initially stopped at `feature-files/features`
  instead of `feature-files`

Both were fixed on trunk before the final release benchmarks were recorded.

### Indexed reads

The hot-path I/O changes live in
[/home/balazs/Development/cjindex/src/lib.rs](/home/balazs/Development/cjindex/src/lib.rs).

Key implementation points:

- `read_exact_range(...)` and `read_exact_range_from_file(...)` now provide
  explicit range reads with clear failure modes
- `NDJSON` no longer rereads the full sequence file for each hit
- `CityJSON` no longer rereads the full tile for each hit
- `CityJSON` shared vertices caching remains in place, but is now populated from
  the indexed vertices span rather than from a whole-file buffer
- tests were added for exact-range reads, short-read failures, and oversized
  length rejection

## Results

The full benchmark report is recorded in
[/home/balazs/Development/cjindex/docs/cjindex-realistic-read-benches-results.md](/home/balazs/Development/cjindex/docs/cjindex-realistic-read-benches-results.md).

The most important release-mode timings are:

- `feature_files_reindex`: `9.5421 s` to `9.6083 s`
- `feature_files_get`: `90.186 ms` to `90.727 ms` per 1,000-lookups batch
- `feature_files_query`: `1.2144 s` to `1.2205 s` per 10-bbox batch
- `cityjson_reindex`: `25.275 s` to `25.385 s`
- `cityjson_get`: `31.171 ms` to `31.260 ms` per 1,000-lookups batch
- `cityjson_query`: `1.3540 s` to `1.3569 s` per 10-bbox batch
- `ndjson_reindex`: `7.8827 s` to `7.9028 s`
- `ndjson_get`: `86.853 ms` to `87.380 ms` per 1,000-lookups batch
- `ndjson_query`: `1.1799 s` to `1.1837 s` per 10-bbox batch

Normalized steady-state read cost now looks like this:

- feature-files `get`: about `90 us` per lookup
- `CityJSON` `get`: about `31 us` per lookup
- `NDJSON` `get`: about `87 us` per lookup
- feature-files `query`: about `122 ms` per bbox
- `CityJSON` `query`: about `136 ms` per bbox
- `NDJSON` `query`: about `118 ms` per bbox

## Consequences

### Positive

- The benchmark harness now measures varied steady-state reads rather than one
  hot object and one repeated spatial window.
- The indexed layouts now actually honor their indexed byte ranges during hot
  reads.
- `CityJSON` is no longer a catastrophic read-path outlier.
- The current benchmark numbers are realistic enough to support backend design
  decisions again.

### Negative

- The benchmark fixture setup is more complex and will now fail loudly if the
  deterministic workload cannot be validated across all layouts.
- Criterion's historical `change:` percentages on read benchmarks are no longer
  semantically clean because the measured work changed.
- `CityJSON` rebuild cost remains materially higher than the other two layouts.

### Neutral tradeoff

We intentionally traded a simpler harness for a more defensible one. The suite
is still deterministic and engineering-friendly, but the setup phase now does
more upfront work so the measured batches are meaningful.
