# Realistic Read Benchmark Results

## Run Contract

Date: 2026-03-29

Correctness command:

```bash
cargo test --release --tests
```

Benchmark commands:

```bash
cargo bench --bench cityjson
cargo bench --bench feature_files --bench ndjson
```

The intended one-shot command remains:

```bash
cargo bench --bench feature_files --bench cityjson --bench ndjson
```

The actual run was split only because an earlier combined run was interrupted
after the `cityjson` bench had already completed. The code and build products
were unchanged between the two benchmark commands above.

## Corpus Shape

The harness still benchmarks the canonical prepared corpus rooted at:

- `/home/balazs/Data/3DBAG_3dtiles_test/cjindex`

This is the same full prepared dataset used by the earlier full-integration
benchmarks:

- 227,045 features total
- feature-files: 227,045 individual feature files
- CityJSON: 191 tile files
- NDJSON: 191 sequence files

## Workload Contract

The benchmark contract changed for steady-state reads:

- `get`: 1,000 deterministic pseudo-random `CityIndex::get(...)` calls per
  measured iteration
- `query`: 10 deterministic real bbox queries per measured iteration
- `query_iter`: the same 10 deterministic bbox queries, fully drained
- `metadata`: unchanged
- `reindex`: unchanged

The workload vectors are derived once from the canonical feature-files corpus
and validated against feature-files, regular `CityJSON`, and `NDJSON`.

That means the Criterion `change:` percentages on `get`, `query`, and
`query_iter` are not apples-to-apples versus the previous benchmark reports.
Those percentages reflect both implementation changes and a heavier benchmark
contract.

## Results

| Benchmark | Time |
| --- | --- |
| `feature_files_reindex` | `9.5421 s` to `9.6083 s` |
| `feature_files_get` | `90.186 ms` to `90.727 ms` |
| `feature_files_query` | `1.2144 s` to `1.2205 s` |
| `feature_files_query_iter` | `1.2118 s` to `1.2134 s` |
| `feature_files_metadata` | `2.3162 us` to `2.3254 us` |
| `cityjson_reindex` | `25.275 s` to `25.385 s` |
| `cityjson_get` | `31.171 ms` to `31.260 ms` |
| `cityjson_query` | `1.3540 s` to `1.3569 s` |
| `cityjson_query_iter` | `1.3622 s` to `1.3679 s` |
| `cityjson_metadata` | `11.140 us` to `11.143 us` |
| `ndjson_reindex` | `7.8827 s` to `7.9028 s` |
| `ndjson_get` | `86.853 ms` to `87.380 ms` |
| `ndjson_query` | `1.1799 s` to `1.1837 s` |
| `ndjson_query_iter` | `1.1751 s` to `1.1807 s` |
| `ndjson_metadata` | `10.994 us` to `11.037 us` |

## Normalized Read View

Because `get` now measures 1,000 lookups per benchmark iteration and
`query`/`query_iter` now measure 10 bbox queries per iteration, the most useful
cross-layout comparison is the normalized per-operation cost:

| Layout | `get` per lookup | `query` per bbox | `query_iter` per bbox |
| --- | --- | --- | --- |
| feature-files | about `90 us` | about `122 ms` | about `121 ms` |
| CityJSON | about `31 us` | about `136 ms` | about `136 ms` |
| NDJSON | about `87 us` | about `118 ms` | about `118 ms` |

## Interpretation

The most important change is not subtle:

- regular `CityJSON` is no longer a catastrophic read outlier
- indexed byte-range reads cut `CityJSON` dense read batches from tens of
  seconds to roughly `1.36 s`
- `CityJSON` `get` is now the fastest normalized lookup path in this suite
- `query` and `query_iter` are now in the same general performance band across
  all three layouts

The new ranking is:

- `reindex`: `NDJSON` first, feature-files second, `CityJSON` last
- `get` normalized per lookup: `CityJSON` first, `NDJSON` and feature-files
  very close behind
- `query`/`query_iter` normalized per bbox: `NDJSON` first, feature-files
  second, `CityJSON` close behind rather than orders of magnitude worse

The old benchmark suite conclusion is now outdated. The earlier result said:

- feature-files was the clear steady-state read winner
- `CityJSON` was the dominant read-path problem

After implementing deterministic realistic read batches and indexed byte-range
reads, the current result is different:

- `CityJSON` read performance is now competitive on the realistic batch
  workloads
- the remaining stable outlier is `CityJSON` rebuild cost, not `CityJSON`
  reads
- the benchmark harness now measures many-ID and many-window steady-state reads
  instead of one hot object and one repeated bbox

## Caveats

- `reindex` and `metadata` are directly comparable with the previous report.
- `get`, `query`, and `query_iter` are not directly comparable as raw benchmark
  names, because their measured work changed materially.
- The workload remains deterministic and cache-friendly enough for repeatable
  engineering use; it is not a cache-cold or production-trace simulation.
