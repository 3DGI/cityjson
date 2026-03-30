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

Convenience recipe:

```bash
just bench-release
```

The benchmark run was intentionally split only so the long `cityjson` bench
could complete independently. The code and build outputs were unchanged between
the two commands.

## Corpus Shape

The harness benchmarks the prepared corpus under the bench root:

- `CJINDEX_BENCH_ROOT` or the default root `/home/balazs/Data/3DBAG_3dtiles_test/cjindex`

The prepared corpus is now produced by the reproducible 3DBAG prep pipeline
against the pinned `v20250903` tile index. The prep manifest under the output
root records the exact tile list, counts, and checksums. The figures below are
the counts captured for the run date of this report.

Prepared corpus shape at the time of this run:

- 227,044 indexed feature packages
- feature-files: 227,044 individual feature files under 191 tile directories
- `CityJSON`: 191 tile files
- `NDJSON`: 191 sequence files

## Workload Contract

The steady-state benchmark contract is now:

- `get`: 1,000 deterministic lookups per measured iteration
- `query`: 10 bbox queries per measured iteration
- `query_iter`: the same 10 bbox queries, fully drained through the streaming iterator path
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

On the current canonical workload shape, those measured iterations correspond
to:

- `get`: 1,000 feature-package reads returning 2,003 `CityObject`s
- `query`: 10 bbox reads returning 7,927 feature packages and 15,886
  `CityObject`s in the canonical batch used by the investigation binary

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
| `feature_files_reindex` | `9.3796 s` to `9.4389 s` |
| `feature_files_get` | `87.759 ms` to `88.336 ms` |
| `feature_files_query` | `751.62 ms` to `755.50 ms` |
| `feature_files_query_iter` | `753.62 ms` to `756.84 ms` |
| `feature_files_metadata` | `2.3406 us` to `2.3492 us` |
| `cityjson_reindex` | `17.862 s` to `18.045 s` |
| `cityjson_get` | `92.092 ms` to `92.581 ms` |
| `cityjson_query` | `796.16 ms` to `799.79 ms` |
| `cityjson_query_iter` | `794.37 ms` to `797.59 ms` |
| `cityjson_metadata` | `11.260 us` to `11.317 us` |
| `ndjson_reindex` | `7.9518 s` to `7.9843 s` |
| `ndjson_get` | `87.349 ms` to `87.774 ms` |
| `ndjson_query` | `734.70 ms` to `737.69 ms` |
| `ndjson_query_iter` | `728.16 ms` to `733.10 ms` |
| `ndjson_metadata` | `11.228 us` to `11.269 us` |

## Normalized Read View

`get` still measures 1,000 lookups per iteration. `query` and `query_iter`
still measure 10 bboxes per iteration. Normalized per-operation cost therefore
looks like this:

| Layout | `get` per lookup | `query` per bbox | `query_iter` per bbox |
| --- | --- | --- | --- |
| feature-files | about `88.0 us` | about `75.4 ms` | about `75.5 ms` |
| `CityJSON` | about `92.3 us` | about `79.8 ms` | about `79.6 ms` |
| `NDJSON` | about `87.6 us` | about `73.6 ms` | about `73.1 ms` |

Because the corrected benchmark semantics now return full feature packages
again, it is also useful to normalize by returned `CityObject`. Combining the
Criterion timings above with the measured workload shape from
`investigate-read-performance` gives this approximate view:

| Layout | `get` per returned `CityObject` | `query` per returned `CityObject` |
| --- | --- | --- |
| feature-files | about `44.0 us` | about `47.4 us` |
| `CityJSON` | about `46.1 us` | about `50.2 us` |
| `NDJSON` | about `43.7 us` | about `46.3 us` |

That per-`CityObject` view is the cleanest cross-backend comparison for the
current workload, because the average feature package in the benchmark contains
about two `CityObject`s.

## Interpretation

The corrected benchmark story is much cleaner than the previous one.

- `NDJSON` is now the fastest backend on every steady-state read metric in the
  suite.
- feature-files is consistently close behind `NDJSON`.
- regular `CityJSON` is no longer catastrophically slow, but it is also no
  longer the surprising `get` winner once full feature-package semantics are
  restored.
- The same backend ordering still holds after normalizing by returned
  `CityObject`: `NDJSON` first, feature-files second, `CityJSON` third.
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
