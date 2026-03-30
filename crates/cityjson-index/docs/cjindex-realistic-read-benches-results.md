# Realistic Read Benchmark Results

## Run Contract

Date: 2026-03-30

Correctness commands:

```bash
cargo test --all-features
cargo test --release --tests
cargo check --benches
```

Benchmark commands:

```bash
cargo bench --bench cityjson
cargo bench --bench feature_files --bench ndjson
```

The benchmark run was intentionally split only so the long `cityjson` bench
could complete independently. The code and build outputs were unchanged between
the two commands.

## Corpus Shape

The harness still benchmarks the canonical prepared corpus rooted at:

- `/home/balazs/Data/3DBAG_3dtiles_test/cjindex`

Prepared corpus shape:

- 227,044 indexed feature packages
- feature-files: 227,044 individual feature files under 191 tile directories
- `CityJSON`: 191 tile files
- `NDJSON`: 191 sequence files

## Workload Contract

The steady-state benchmark contract is now:

- `get`: 1,000 deterministic lookups per measured iteration
- `query`: 10 bbox queries per measured iteration
- `query_iter`: the same 10 bbox queries, fully drained
- `metadata`: unchanged
- `reindex`: unchanged

Two details matter for interpreting the current numbers:

1. `get` now guarantees corpus spread.
   The 1,000-ID batch includes at least one feature from every tile in the
   191-tile corpus before filling the remaining IDs from a deterministic
   pseudo-random remainder.
2. `query` and `query_iter` now rotate through a full-corpus bbox ring.
   The harness builds one deterministic tile-local bbox per tile, then each
   measured iteration executes the next 10 bboxes in that ring. Over repeated
   Criterion samples, the suite sweeps the whole 191-tile corpus instead of
   hammering one fixed spatial window.

This means Criterion's historical `change:` percentages on `get`, `query`, and
`query_iter` are not apples-to-apples versus the previous report. Those
percentages now reflect:

- the rotating full-corpus query contract
- the corrected `CityJSON` feature-package semantics
- the current implementation changes

## Semantic Correction

The previous regular `CityJSON` implementation was benchmarking a different
unit of work from the other backends.

It indexed every individual `CityObject`, including children, while
feature-files and `NDJSON` index feature packages. That made `CityJSON query`
return about twice as many hits for the same bbox workload and made
`CityJSON get` artificially cheap because it often reconstructed only one
object instead of a full feature package.

That mismatch is now fixed:

- regular `CityJSON` indexing now groups top-level root objects with their
  descendants into one indexed feature package
- `CityJSON` now indexes the same number of feature packages as feature-files
  and `NDJSON`: 227,044
- the benchmark comparison is now semantically aligned again

## Results

| Benchmark | Time |
| --- | --- |
| `feature_files_reindex` | `9.3700 s` to `9.3988 s` |
| `feature_files_get` | `88.732 ms` to `88.870 ms` |
| `feature_files_query` | `738.46 ms` to `742.19 ms` |
| `feature_files_query_iter` | `742.00 ms` to `744.91 ms` |
| `feature_files_metadata` | `2.3329 us` to `2.3448 us` |
| `cityjson_reindex` | `17.859 s` to `17.928 s` |
| `cityjson_get` | `91.345 ms` to `91.497 ms` |
| `cityjson_query` | `767.98 ms` to `769.88 ms` |
| `cityjson_query_iter` | `769.75 ms` to `771.41 ms` |
| `cityjson_metadata` | `11.230 us` to `11.244 us` |
| `ndjson_reindex` | `7.8998 s` to `7.9170 s` |
| `ndjson_get` | `86.807 ms` to `87.165 ms` |
| `ndjson_query` | `712.94 ms` to `715.09 ms` |
| `ndjson_query_iter` | `712.00 ms` to `713.31 ms` |
| `ndjson_metadata` | `11.027 us` to `11.038 us` |

## Normalized Read View

`get` still measures 1,000 lookups per iteration. `query` and `query_iter`
still measure 10 bboxes per iteration. Normalized per-operation cost therefore
looks like this:

| Layout | `get` per lookup | `query` per bbox | `query_iter` per bbox |
| --- | --- | --- | --- |
| feature-files | about `88.8 us` | about `74.0 ms` | about `74.3 ms` |
| `CityJSON` | about `91.4 us` | about `76.9 ms` | about `77.0 ms` |
| `NDJSON` | about `87.0 us` | about `71.4 ms` | about `71.3 ms` |

## Interpretation

The corrected benchmark story is much cleaner than the previous one.

- `NDJSON` is now the fastest backend on every steady-state read metric in the
  suite.
- feature-files is consistently close behind `NDJSON`.
- regular `CityJSON` is no longer catastrophically slow, but it is also no
  longer the surprising `get` winner once full feature-package semantics are
  restored.
- `CityJSON reindex` improved materially from the earlier report because the
  feature count is no longer inflated by indexing every child object.

The current ranking is:

- `reindex`: `NDJSON` first, feature-files second, `CityJSON` last
- `get`: `NDJSON` first, feature-files second, `CityJSON` third
- `query` / `query_iter`: `NDJSON` first, feature-files second, `CityJSON`
  third

What changed relative to the previous report:

- the query workload now rotates across the full 191-tile corpus instead of
  reusing the same 10 windows forever
- the `get` workload now guarantees tile-level corpus coverage
- the old "`CityJSON get` is far faster than `NDJSON get`" conclusion is gone,
  because that result was largely an artifact of comparing mismatched units of
  work

## Caveats

- `reindex` and `metadata` remain directly comparable with the previous report.
- `get`, `query`, and `query_iter` do not have clean historical continuity,
  because both the workload shape and the `CityJSON` feature semantics changed.
- These are still hot steady-state numbers. They do not include `reindex`, and
  they are still helped by OS page cache and repeated process-local reuse.
