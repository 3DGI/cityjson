# cjindex

`cjindex` provides random-access retrieval of CityJSON features by identifier or bounding box, backed by a persistent SQLite index.

It supports three on-disk storage layouts behind one API:

- CityJSONSeq, also referred to here as `NDJSON`
- regular CityJSON
- individual CityJSONFeature files

The library returns `cjlib::CityModel` values for reads. For `get` and `query`, the intended CLI output is a line-oriented CityJSON stream: the first record is a `CityJSON` document containing metadata and an empty `CityObjects` object, and every following record is a `CityJSONFeature`. File output uses the same record shape, newline-delimited.

## CLI

The CLI now has a dataset-oriented mode as the default workflow:

- `inspect DATASET_DIR`
- `index DATASET_DIR`
- `reindex DATASET_DIR`
- `validate DATASET_DIR`
- `get DATASET_DIR --id ...`
- `query DATASET_DIR --min-x ... --max-x ... --min-y ... --max-y ...`
- `metadata DATASET_DIR`

Dataset mode auto-detects one of the supported layouts under `DATASET_DIR` and
uses `<DATASET_DIR>/.cjindex.sqlite` as the default index sidecar. `inspect`
reports layout, counts, index presence, freshness, and coverage. `validate`
performs the same checks but exits non-zero when the index is missing, stale,
or no longer matches the dataset.

`get` and `query` are read operations. They emit the line-oriented CityJSON
stream described above. `query_iter()` is the streaming library path behind
bbox reads, so query execution can stay lazy instead of buffering every match
up front.

The explicit low-level mode still exists as an escape hatch:

```text
cjindex get \
  --layout feature-files \
  --root /data/feature-files \
  --index /data/feature-files/.cjindex.sqlite \
  --id NL.IMBAG.Pand.0503100000012869-0
```

In explicit mode, `--layout` and `--index` are required. `feature-files` also
requires `--root`; `ndjson` and `cityjson` use `--paths`.

## Storage Layouts

### NDJSON / CityJSONSeq

```text
tiles/
├── 0566.city.jsonl
├── 0599.city.jsonl
└── 0637.city.jsonl
```

Each `.city.jsonl` file begins with the metadata object. Every later line is a self-contained `CityJSONFeature` with one feature package and its own local vertices.

Indexing records the byte offsets and lengths for each feature line and computes a bounding box for each package. Reading seeks directly to the feature bytes and deserializes the line into a `CityModel`.

### CityJSON

```text
tiles/
├── 0566.city.json
├── 0599.city.json
└── 0637.city.json
```

Regular CityJSON files keep a shared `vertices` array and a `CityObjects` dictionary.

Indexing records the metadata, the shared vertices range, and the byte ranges of the indexed feature packages. The current semantics are root-plus-descendants: a top-level CityObject and its children are treated as one feature package.

Reading pulls the indexed member object fragments and the shared vertices range, then reconstructs the full feature package as a `CityModel`. The shared vertices are cached per source file, so repeated reads from the same tile avoid rereading that block.

### Feature Files

```text
features/
├── metadata.json
├── 0566/
│   ├── metadata.json
│   ├── NL.IMBAG.Pand.0566100000032571.city.jsonl
│   └── NL.IMBAG.Pand.0566100000032572.city.jsonl
└── 0599/
    ├── NL.IMBAG.Pand.0599100000023396.city.jsonl
    └── NL.IMBAG.Pand.0599100000023397.city.jsonl
```

Each feature lives in its own file. Metadata is stored in separate CityJSON files discovered via glob patterns and resolved by nearest ancestor.

Reading is the simplest here: the full feature file is read and deserialized, and the cached metadata from the matched source is attached.

## Index Structure

The index is a single SQLite file with these tables:

- `sources` stores source paths, cached metadata JSON, and for CityJSON the shared vertices span
- `features` maps feature IDs to source locations and byte ranges
- `feature_bbox` is an R*Tree virtual table for spatial lookups
- `bbox_map` joins R*Tree rows back to feature IDs

`reindex()` drops and rebuilds the derived index. There is no incremental update path yet.

## Metadata

Each layout stores metadata differently:

- NDJSON: the first line is the metadata object
- CityJSON: metadata is part of the top-level JSON object
- feature files: metadata lives in separate ancestor-discovered `.city.json` files

During indexing, metadata is parsed once and cached in SQLite. Read paths attach that cached metadata to returned `CityModel` values.

## Benchmark Data

The benchmark corpus is produced by `just prep-test-data` against the pinned
`v20250903` 3DBAG tile index. The output root defaults to `./tests/data`
(relative to the repository root) and can be overridden via `CJINDEX_BENCH_ROOT`.
The prep tool writes a manifest under the output root that records the exact
tile list, counts, and checksums for the prepared corpus.

This prep flow is also the reusable bootstrap path for the shared
`cityjson-benchmarks` corpus. The shared corpus repo should own the release
contract, but it can reuse this pipeline for the first 3DBAG-derived artifact
set.

The concrete prep implementation lives in
[tests/common/data_prep.rs](/home/balazs/Development/cjindex/tests/common/data_prep.rs)
and the `prep-test-data` recipe in [justfile](/home/balazs/Development/cjindex/justfile).

The prep pipeline downloads `tile_index.fgb` from `https://data.3dbag.nl/v20250903/`
and parses every tile entry for the `cj_download` URL. It downloads tiles in
lexicographic order, validates each CityJSON with `cjval`, and converts it via
`cjseq cat` into NDJSON/CityJSONSeq plus the per-tile feature files that power
the `feature-files` layout. Once the cumulative CityObject count is between
265k and 275k (target ~270k), it writes checksums, counts, and tool versions
into `manifest.json` and swaps the staging tree into `tests/data` atomically.

Validation with `cjval` takes a long time; pass `--skip-cjval` to `just
prep-test-data` if you want to rebuild the corpus faster and are confident the
downloads are sound.

Target size: about 270,000 `CityObject`s in total.

The benchmark binaries (`just bench-release`, `cargo run --bin investigate-read-performance`, the Criterion benches) always point at the prepared tree under `tests/data` (or whatever path `CJINDEX_BENCH_ROOT` overrides), so they never rebuild from scratch as long as the manifest declares the pinned tile index.

The current steady-state read benchmarks use hot, repeated workloads rather than cold one-off reads:

- `get` uses 1,000 deterministic lookups per measured iteration
- `query` and `query_iter` use 10 bbox reads per measured iteration
- the `get` workload covers all 191 tiles
- the bbox workload rotates through a deterministic 191-bbox ring
- the current canonical `get` batch returns 2,003 `CityObject`s
- the canonical 10-bbox query batch returns 15,886 `CityObject`s

See the benchmark ADRs for the current read-path contract and the historical
benchmark harness:

- [docs/adr/001-full-integration-benchmark-harness.md](/home/balazs/Development/cjindex/docs/adr/001-full-integration-benchmark-harness.md)
- [docs/adr/002-realistic-read-batches-byte-range-reads.md](/home/balazs/Development/cjindex/docs/adr/002-realistic-read-batches-byte-range-reads.md)

Current read ranking in the benchmark ADRs:

- `NDJSON` is the fastest steady-state read backend overall
- feature-files is close behind
- regular CityJSON is no longer broken on reads, but it remains slower than the other layouts

Shared corpus migration:

- [docs/shared-corpus-migration-plan.md](docs/shared-corpus-migration-plan.md)

## Status

`cjindex` now has a working indexing and read library plus a dataset-first CLI
with inspect/validate support. The remaining work is mostly incremental index
maintenance and further read-path performance work rather than missing core
functionality.
