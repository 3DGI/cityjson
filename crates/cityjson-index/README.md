# cjindex

`cjindex` provides random-access retrieval of CityJSON features by identifier or bounding box, backed by a persistent SQLite index.

It supports three on-disk storage layouts behind one API:

- CityJSONSeq, also referred to here as `NDJSON`
- regular CityJSON
- individual CityJSONFeature files

The library returns `cjlib::CityModel` values for reads. For `get` and `query`, the intended CLI output is a line-oriented CityJSON stream: the first record is a `CityJSON` document containing metadata and an empty `CityObjects` object, and every following record is a `CityJSONFeature`. File output uses the same record shape, newline-delimited.

## CLI

The CLI surface is centered on the same operations as the library:

- `index`
- `reindex`
- `get`
- `query`
- `metadata`

`get` and `query` are read operations. They emit the line-oriented CityJSON stream described above. `query_iter()` is the streaming library path behind bbox reads, so query execution can stay lazy instead of buffering every match up front.

`reindex` rebuilds the SQLite sidecar index from the configured source layout. `metadata` exposes the cached source metadata entries.

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

The current benchmark docs are based on the prepared corpus rooted at:

- `/home/balazs/Data/3DBAG_3dtiles_test/cjindex`

The prepared corpus shape is:

- 227,044 indexed feature packages
- 191 `CityJSON` / `NDJSON` tiles
- 227,044 feature files for the feature-files layout

The current steady-state read benchmarks use hot, repeated workloads rather than cold one-off reads:

- `get` uses 1,000 deterministic lookups per measured iteration
- `query` and `query_iter` use 10 bbox reads per measured iteration
- the `get` workload covers all 191 tiles
- the bbox workload rotates through a deterministic 191-bbox ring
- the current canonical `get` batch returns 2,003 `CityObject`s
- the canonical 10-bbox query batch returns 15,886 `CityObject`s

See the full benchmark write-up in [docs/cjindex-realistic-read-benches-results.md](/home/balazs/Development/cjindex/docs/cjindex-realistic-read-benches-results.md) and the backend investigation in [docs/cjindex-backend-perf-investigation-results.md](/home/balazs/Development/cjindex/docs/cjindex-backend-perf-investigation-results.md).

Current read ranking in the benchmark docs:

- `NDJSON` is the fastest steady-state read backend overall
- feature-files is close behind
- regular CityJSON is no longer broken on reads, but it remains slower than the other layouts

## Status

`cjindex` is currently a working indexing and read library with a matching CLI surface. The remaining work is mostly in polish, CLI ergonomics, and performance tuning rather than in the core retrieval model.
