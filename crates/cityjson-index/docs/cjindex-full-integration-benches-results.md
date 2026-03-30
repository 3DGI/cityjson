# Full Integration Benchmark Results

> Historical note: this report captures the earlier full-corpus integration
> benchmark shape. The current read-path analysis and benchmark contract are in
> [docs/cjindex-realistic-read-benches-results.md](/home/balazs/Development/cjindex/docs/cjindex-realistic-read-benches-results.md)
> and [docs/cjindex-backend-perf-investigation-results.md](/home/balazs/Development/cjindex/docs/cjindex-backend-perf-investigation-results.md).

## Run Contract

Date: 2026-03-29

Commands:

```bash
cargo test --release --tests
cargo bench --bench feature_files --bench cityjson --bench ndjson
```

The benchmark entry points now carry an explicit Criterion configuration with
`sample_size(10)`. That is necessary because full-corpus `reindex` and
full-corpus spatial reads are too expensive for Criterion's implicit
100-sample default.

## Corpus Shape

The benchmark harness now uses the prepared full dataset under the bench root:

- `CJINDEX_BENCH_ROOT` or the default root `/home/balazs/Data/3DBAG_3dtiles_test/cjindex`

The prepared corpus is produced by the reproducible 3DBAG prep pipeline against
the pinned `v20250903` tile index. The prep manifest under the output root
records the exact tile list, counts, and checksums. The figures below reflect
the manifest present on the run date above.

The measured corpus in this environment contains:

- 227,045 features total
- feature-files: 227,045 individual feature files
- CityJSON: 191 tile files
- NDJSON: 191 sequence files

This is the real prepared corpus, not a synthetic subset.

## Workload Shape

The benchmarked operations are still identical across layouts:

- `reindex`
- `get`
- `query`
- `query_iter`
- `metadata`

The read workload is now defined like this:

- `get` uses one stable feature ID from the full prepared corpus
- `query` and `query_iter` use a deterministic spatial workload built from
  1,000 selected features
- those 1,000 features are taken from the first lexicographically selected tile
  that has at least 1,000 feature files
- in the current corpus, that tile is `10/256/588`
- that tile contains 1,027 feature files total

The query bbox is built from the union of the 1,000 selected feature models, so
the spatial workload stays localized to one real tile-scale region instead of
accidentally spanning large parts of the corpus.

## Results

| Benchmark | Time |
| --- | --- |
| `feature_files_reindex` | 9.2951 s to 9.3231 s |
| `feature_files_get` | 73.094 us to 73.394 us |
| `feature_files_query` | 88.030 ms to 88.460 ms |
| `feature_files_query_iter` | 87.634 ms to 87.786 ms |
| `feature_files_metadata` | 2.3212 us to 2.3312 us |
| `cityjson_reindex` | 23.056 s to 23.136 s |
| `cityjson_get` | 27.105 ms to 27.191 ms |
| `cityjson_query` | 73.774 s to 73.925 s |
| `cityjson_query_iter` | 73.859 s to 74.234 s |
| `cityjson_metadata` | 11.014 us to 11.026 us |
| `ndjson_reindex` | 7.7962 s to 7.8153 s |
| `ndjson_get` | 163.97 us to 164.21 us |
| `ndjson_query` | 188.56 ms to 189.30 ms |
| `ndjson_query_iter` | 193.11 ms to 195.09 ms |
| `ndjson_metadata` | 10.893 us to 10.949 us |

## Interpretation

The earlier 9-feature subset understated the real differences between layouts.
On the full corpus, the storage formats no longer look close.

Observed behavior on the full benchmark set:

- `reindex` is no longer "basically the same" across layouts.
- NDJSON is fastest at roughly `7.8 s`.
- feature-files follows at roughly `9.3 s`.
- CityJSON is slowest at roughly `23.1 s`.
- `get` is strongest for feature-files at roughly `73 us`.
- NDJSON `get` is about `164 us`, around `2.2x` slower than feature-files.
- CityJSON `get` is about `27.1 ms`, roughly `165x` slower than NDJSON and
  roughly `370x` slower than feature-files.
- `query` and `query_iter` on the 1,000-feature tile-local workload are
  roughly `88 ms` for feature-files.
- NDJSON is roughly `189 ms` to `195 ms`, about `2.1x` to `2.2x` slower than
  feature-files.
- CityJSON is roughly `74 s`, which is about `390x` slower than NDJSON and
  about `840x` slower than feature-files on this workload.

The relative shape is now clear:

- feature-files is the best steady-state read layout
- NDJSON is the best reindex layout and a reasonable second-place read layout
- CityJSON is the dominant outlier on steady-state reads and also the slowest
  layout to rebuild

## Likely Causes

The full-corpus results line up with the backend designs:

- feature-files reads are cheap because each lookup starts from a
  single-feature payload that is already isolated on disk
- NDJSON reads still work with feature-shaped payloads, but they pay extra
  sequence-file handling and parsing cost
- CityJSON reads are much more expensive because each returned feature must be
  extracted from a multi-feature tile and rebuilt as a one-object package

The CityJSON query numbers are especially severe because the query workload is
tile-local and feature-dense. That means `cjindex` is repeatedly pulling many
single-object packages back out of the same larger CityJSON tile. The current
read path does not yet make that access pattern cheap enough.

## Conclusions

- The benchmark suite now reflects the real prepared corpus rather than a tiny
  subset.
- The read-path ranking on realistic data is now clear:
  feature-files first, NDJSON second, CityJSON far behind.
- The earlier subset benchmark understated how expensive the CityJSON read path
  is under dense tile-local workloads.
- The next performance target is unambiguous: regular `CityJSON` read/query
  behavior.
