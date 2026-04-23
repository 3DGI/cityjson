# cityjson-parquet

`cityjson-parquet` stores `cityjson-rs` city models as seekable single-file packages
and native Parquet canonical-table datasets.

`PackageWriter` and `PackageReader` encode and decode `cityjson::v2_0::OwnedCityModel`
into a container backed by Arrow IPC table payloads.
`ParquetDatasetWriter` and `ParquetDatasetReader` encode the same canonical tables as
one native Parquet file per table.

- `PackageWriter` — encode a model into a `.cityjson-parquet` file
- `PackageReader` — decode a file into a model, or read its manifest without loading geometry
- `ParquetDatasetWriter` — encode a model into a native Parquet dataset directory
- `ParquetDatasetReader` — decode a native Parquet dataset directory back into a model
- `spatial::SpatialIndex` — Hilbert-curve index over city object bounding boxes for viewport queries

## Formats

`cityjson-parquet` now has two durable formats:

| Format | API | Layout | Primary use |
|---|---|---|---|
| `.cityjson-parquet` package | `PackageWriter`, `PackageReader` | One seekable file containing Arrow IPC table payloads plus a footer manifest | Compact package IO and fast manifest inspection |
| Native Parquet dataset | `ParquetDatasetWriter`, `ParquetDatasetReader` | Directory with `manifest.json` and `tables/{canonical_table}.parquet` | Cross-library Parquet interoperability, column projection, and predicate pushdown |

Both formats use the same CityJSON Arrow canonical table schema. They are not
binary-equivalent encodings, and tests should compare semantic CityJSON equality
rather than byte-for-byte output.

## How it works

- The package format is a seekable single-file container: `PACKAGE_MAGIC`, ordered Arrow IPC
  table payloads, manifest JSON, and `PACKAGE_FOOTER_MAGIC`.
- The manifest is written last, so the writer never seeks back.
- Table payloads are accessed via memory-mapped I/O. The reader decodes only the slices
  referenced by the manifest.
- The native Parquet dataset format writes `manifest.json` plus `tables/{name}.parquet`
  files for PyArrow, DuckDB, Polars, and similar tools.
- Native Parquet encodes canonical nullable `FixedSizeList` fields as variable
  Parquet lists with reader-side fixed-length validation, because PyArrow cannot
  reliably full-scan nullable fixed-size list columns.
- Schema and manifest types are shared with `cityjson-arrow` and re-exported from
  `cityjson_arrow::schema`.
- `SpatialIndex` is built at read time and is not stored in the file.

## Current limits

- There is no streaming writer. The full model is materialised before writing.
- `SpatialIndex::query` performs a linear scan. It does not exploit the Hilbert
  ordering for range pruning.
- `cityjson-parquet` requires `cityjson-arrow` to be checked out as a sibling directory.

## Benchmarks

Package read and write throughput compared to `cityjson-arrow` stream and `cityjson-json`.
Factor < 1.0 means the package format is faster than JSON; > 1.0 means slower.
Full results and plots: `benches/results/`.

<!-- benchmark-summary:start -->
**Acquired data**

| Case | cityjson-parquet | `cityjson-arrow` | `cityjson-json` | Factor |
| --- | --- | --- | --- | --- |
| `io_basisvoorziening_3d_cityjson` | 601.8 MiB/s | 611.9 MiB/s | 282.7 MiB/s | 2.10x |
| `io_3dbag_cityjson_cluster_4x` | 548.0 MiB/s | 530.2 MiB/s | 183.2 MiB/s | 2.96x |
| `io_3dbag_cityjson` | 596.7 MiB/s | 598.3 MiB/s | 190.6 MiB/s | 3.17x |

**Stress cases**

| Case | cityjson-parquet | `cityjson-arrow` | `cityjson-json` | Factor |
| --- | --- | --- | --- | --- |
| `stress_attribute_heavy_heterogenous` | 252.3 MiB/s | 243.7 MiB/s | 151.8 MiB/s | 0.94x |
| `stress_attribute_heavy_homogenous` | 179.9 MiB/s | 174.4 MiB/s | 162.3 MiB/s | 1.60x |
| `stress_boundary_heavy` | 3065.3 MiB/s | 3388.4 MiB/s | 317.7 MiB/s | 6.01x |
| `stress_geometry_heavy` | 1517.5 MiB/s | 1522.6 MiB/s | 271.8 MiB/s | 3.66x |
| `stress_hierarchy_heavy` | 1099.5 MiB/s | 1110.3 MiB/s | 188.9 MiB/s | 5.00x |
| `stress_resource_heavy` | 763.3 MiB/s | 769.3 MiB/s | 154.5 MiB/s | 3.77x |
| `stress_vertex_heavy` | 4374.5 MiB/s | 4435.5 MiB/s | 357.6 MiB/s | 7.37x |
<!-- benchmark-summary:end -->

## Verification

```shell
just fmt
just lint
just check
just test    # requires ../cityjson-arrow checked out as a sibling
just rustdoc
just site-build
just bench-check
```

## Use of AI in this project

This crate was written with AI assistance after the schema and specs were defined by hand.
Development used an iterative process of testing, benchmarking, and optimization controlled and verified by me.

## Repository map

- `src/lib.rs` — public exports; re-exports schema types from `cityjson-arrow`
- `src/dataset.rs` — `ParquetDatasetWriter`, `ParquetDatasetReader`, and native Parquet dataset I/O
- `src/package/mod.rs` — `PackageWriter`, `PackageReader`, and package I/O helpers
- `src/spatial.rs` — `SpatialIndex`, `SpatialEntry`, `BBox2D`, and Hilbert curve
- `examples/viewer.rs` — three.js web viewer served over TCP; uses `SpatialIndex` for frustum culling
- `tests/package_shared_corpus_roundtrip.rs` — conformance roundtrip tests over the shared corpus
- `tests/native_parquet_dataset_roundtrip.rs` — native Parquet dataset roundtrip tests over the shared corpus
