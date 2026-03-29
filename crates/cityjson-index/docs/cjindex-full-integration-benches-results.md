# Full Integration Benchmark Results

## Run Contract

Date: 2026-03-29

Commands:

```bash
cargo test --release --tests
rm -rf target/criterion
cargo bench --bench feature_files --bench cityjson --bench ndjson
```

The `target/criterion` directory was cleared before the benchmark run so the
results below are absolute timings for the current harness rather than
Criterion change reports against an older, differently shaped benchmark.

## Fixture Shape

The benchmark harness now materializes one shared raw-input subset and derives
all three storage layouts from it. The subset currently contains:

- 3 tiles
- 3 feature files per tile
- 9 features total

Selected raw-input tiles:

- `10/256/588`
- `10/256/590`
- `10/256/596`

This means:

- feature-files benchmarks operate on 9 individual feature files
- CityJSON benchmarks operate on 3 tile files with 3 features each
- NDJSON benchmarks operate on 3 sequence files with 3 features each

The benchmarked operations are identical across layouts:

- `reindex`
- `get`
- `query`
- `query_iter`
- `metadata`

## Results

| Benchmark | Time |
| --- | --- |
| `feature_files_reindex` | 4.3400 ms to 4.4566 ms |
| `feature_files_get` | 72.957 us to 73.153 us |
| `feature_files_query` | 915.04 us to 916.08 us |
| `feature_files_query_iter` | 915.05 us to 915.85 us |
| `feature_files_metadata` | 2.3149 us to 2.3163 us |
| `cityjson_reindex` | 4.5582 ms to 4.6971 ms |
| `cityjson_get` | 95.515 us to 95.737 us |
| `cityjson_query` | 2.8397 ms to 2.8438 ms |
| `cityjson_query_iter` | 2.8060 ms to 2.8110 ms |
| `cityjson_metadata` | 2.4150 us to 2.4172 us |
| `ndjson_reindex` | 4.2319 ms to 4.3517 ms |
| `ndjson_get` | 72.189 us to 72.293 us |
| `ndjson_query` | 904.48 us to 906.02 us |
| `ndjson_query_iter` | 902.55 us to 905.11 us |
| `ndjson_metadata` | 2.4074 us to 2.4091 us |

## Interpretation

The main result is that the three layouts now have directly comparable
integration benchmarks. We are no longer comparing NDJSON API timings against
CityJSON or feature-files parse micro-benchmarks.

Observed behavior on this subset:

- `reindex` is tightly clustered across all three layouts at roughly
  `4.2 ms` to `4.7 ms`.
- `get` is essentially tied for feature-files and NDJSON at roughly `72 us`,
  while CityJSON is slower at roughly `96 us`.
- `query` and `query_iter` are also essentially tied for feature-files and
  NDJSON at roughly `0.9 ms`, while CityJSON is about `2.8 ms`.
- `metadata` is effectively identical across layouts at about `2.3 us` to
  `2.4 us`.

The likely reason CityJSON is slower on `get`, `query`, and `query_iter` is
that it has to extract a single object out of a multi-feature tile and rebuild
the one-object feature payload from the shared root document on each read. The
feature-files and NDJSON layouts both start from feature-shaped payloads, so
their steady-state read path is cheaper.

The gap between feature-files and NDJSON is small on this subset. NDJSON is
slightly faster on `get`, `query`, and `query_iter`, while feature-files is
slightly faster on `metadata`. At this scale, those differences are minor
compared to the CityJSON read-path gap.

## Conclusions

- The full integration benchmark suite is now in place for all three layouts.
- The results are coherent and align with the current backend designs.
- The next performance question is not benchmark coverage anymore. It is
  whether the CityJSON one-object extraction path should be optimized further,
  since that is now the clear steady-state outlier.
