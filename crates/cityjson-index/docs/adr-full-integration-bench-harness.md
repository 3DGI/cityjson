# ADR: Full Integration Benchmark Harness Across All Storage Layouts

## Status

Accepted

## Date

2026-03-29

## Context

`cjindex` supports three storage layouts:

- feature-files
- regular `CityJSON`
- `NDJSON` / `CityJSONSeq`

The first integration benchmark harness fixed an earlier apples-to-oranges
problem by benchmarking the same `CityIndex` API across all three layouts.
That part was correct.

What was not correct enough was the dataset shape. The first version still used
a tiny 9-feature subset. That made the benchmark suite easy to run, but it
understated the real differences between layouts so badly that the numbers were
misleading.

For realistic performance work, we need the harness to answer:

- how does each layout behave on the real prepared corpus?
- how does `query` behave when it returns on the order of 1,000 features rather
  than single-digit toy results?
- which layout actually scales better under dense tile-local read workloads?

## Decision

We kept the shared integration benchmark harness, but changed its workload
contract from a tiny subset to the full prepared dataset.

The harness now benchmarks against the canonical prepared roots under:

- `/home/balazs/Data/3DBAG_3dtiles_test/cjindex`

The measured corpus in this environment is:

- 227,045 features total
- feature-files: 227,045 individual feature files
- CityJSON: 191 tile files
- NDJSON: 191 sequence files

The operation contract remains the same:

- `reindex`
- `get`
- `query`
- `query_iter`
- `metadata`

The workload contract changed:

- `get` remains a single-object lookup
- `query` and `query_iter` now use a deterministic 1,000-feature spatial
  workload
- those 1,000 features are selected from one real tile, not scattered across
  the corpus
- the first qualifying tile in the current dataset is `10/256/588`
- that tile contains 1,027 feature files total

We also made the Criterion configuration explicit with `sample_size(10)`,
because full-corpus `reindex` and full-corpus dense spatial queries are far too
expensive for Criterion's implicit 100-sample default.

## Implementation

### 1. Shared harness retained

The main implementation still lives in
[benches/support.rs](/home/balazs/Development/cjindex/benches/support.rs).

It still exposes:

- `LayoutKind`
- `bench_layout(c: &mut Criterion, kind: LayoutKind)`

The per-layout bench files remain thin wrappers:

- [benches/feature_files.rs](/home/balazs/Development/cjindex/benches/feature_files.rs)
- [benches/cityjson.rs](/home/balazs/Development/cjindex/benches/cityjson.rs)
- [benches/ndjson.rs](/home/balazs/Development/cjindex/benches/ndjson.rs)

### 2. Full prepared corpus instead of a subset

The harness no longer materializes a synthetic benchmark subset into `/tmp`.

Instead, it reuses the canonical prepared dataset under
`/home/balazs/Data/3DBAG_3dtiles_test/cjindex`. If the prepared roots are
missing, the harness can still fall back to `prepare_test_sets(...)`, but the
intended steady-state path is to benchmark the already prepared full dataset.

This removes the biggest source of benchmark distortion from the earlier
version.

### 3. Deterministic 1,000-feature spatial workload

The selector in
[benches/support.rs](/home/balazs/Development/cjindex/benches/support.rs)
now groups feature files by tile and picks the first lexicographically ordered
tile that contains at least 1,000 feature files.

For the current corpus, that is:

- `10/256/588`

The harness then:

1. takes the first 1,000 features from that tile
2. uses the first selected ID as the stable `get` target
3. computes one bbox by unioning the selected 1,000 models
4. uses that bbox for both `query` and `query_iter`

This keeps the query workload large enough to be realistic while avoiding the
earlier mistake of spreading one benchmark bbox across multiple tiles.

### 4. Explicit Criterion sizing

Each bench entry point now uses:

- `Criterion::default().sample_size(10)`

This is not a cosmetic change. On the full corpus:

- `reindex` takes seconds to tens of seconds per sample
- CityJSON dense spatial reads take tens of seconds per sample

Without an explicit smaller sample size, the suite becomes effectively
unrunnable.

## Consequences

### Positive

- The suite now measures the real prepared corpus instead of a toy subset.
- `query` and `query_iter` now exercise a dense, tile-local workload of about
  1,000 features.
- The benchmark results are now realistic enough to drive backend priorities.
- The relative differences between layouts are much clearer than before.

### Negative

- The full benchmark suite is much slower than the earlier subset-based suite.
- Criterion had to be configured more conservatively just to keep the suite
  runnable.
- Full-corpus results are now dominated by real backend costs, so regressions
  are more expensive to measure.

### Neutral tradeoff

We intentionally traded convenience for realism. The earlier subset suite was
faster, but it answered the wrong question. The current suite is slower, but it
measures the workload that actually matters.

## Results and Interpretation

The benchmark run and exact numbers are recorded in
[docs/cjindex-full-integration-benches-results.md](/home/balazs/Development/cjindex/docs/cjindex-full-integration-benches-results.md).

The most important result is that the tiny-subset conclusion was wrong.

On the full corpus:

- feature-files is the fastest steady-state read layout
- NDJSON is the fastest reindex layout and a reasonable second-place read
  layout
- CityJSON is the clear outlier on dense read workloads

Observed release-mode timings:

- `feature_files_reindex`: `9.2951 s` to `9.3231 s`
- `feature_files_get`: `73.094 us` to `73.394 us`
- `feature_files_query`: `88.030 ms` to `88.460 ms`
- `feature_files_query_iter`: `87.634 ms` to `87.786 ms`
- `cityjson_reindex`: `23.056 s` to `23.136 s`
- `cityjson_get`: `27.105 ms` to `27.191 ms`
- `cityjson_query`: `73.774 s` to `73.925 s`
- `cityjson_query_iter`: `73.859 s` to `74.234 s`
- `ndjson_reindex`: `7.7962 s` to `7.8153 s`
- `ndjson_get`: `163.97 us` to `164.21 us`
- `ndjson_query`: `188.56 ms` to `189.30 ms`
- `ndjson_query_iter`: `193.11 ms` to `195.09 ms`

The practical conclusion is not subtle:

- CityJSON read/query performance is now the dominant problem
- feature-files is better than NDJSON on steady-state reads
- NDJSON is better than feature-files on rebuild cost

## Follow-up

The next performance work should focus on regular `CityJSON`, especially:

- one-object extraction cost
- repeated reads from the same larger tile
- repeated feature-package reconstruction during dense queries

Benchmark coverage is no longer the main issue. The full-corpus harness now
makes the backend priority obvious.
