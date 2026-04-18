# cityjson-parquet

`cityjson-parquet` is the persistent package crate for `cityjson-rs`.

It owns the durable storage boundary in the ADR 3 architecture.
`PackageWriter` and `PackageReader` wrap `cityjson-arrow`'s canonical tables
into a seekable single-file container backed by Arrow IPC payloads.

- `PackageWriter` — encodes an `OwnedCityModel` into a `.cityjson-parquet` file
- `PackageReader` — decodes a file back into an `OwnedCityModel` or a
  `PackageManifest`
- `read_package_manifest()` — fast manifest-only read; does not load geometry
- `spatial::SpatialIndex` — Hilbert-curve index over `CityObject` bounding
  boxes for viewport queries

## Current Architecture

- the persistent format is a seekable single-file container: `PACKAGE_MAGIC`,
  ordered Arrow IPC table payloads, manifest JSON, and `PACKAGE_FOOTER_MAGIC`
- the manifest is written at the end so the writer never needs a seek-back pass
- table payloads are accessed via memory-mapped I/O; only the slices referenced
  by the manifest are decoded
- schema and manifest types are shared with `cityjson-arrow` and imported from
  `cityjson_arrow::schema`
- `SpatialIndex` is a post-load utility that sorts objects by Hilbert curve
  value; it is not stored in the file

## Current Limits

- there is no streaming writer; the current path materialises the full model
  before writing
- `SpatialIndex::query` performs a linear scan; it does not exploit the Hilbert
  ordering for range pruning
- `cityjson-parquet` depends on doc-hidden bridges from `cityjson-arrow` and
  requires both repos to be checked out as siblings

## Benchmarks

Package read and write throughput compared to `cityjson-arrow` stream and `cityjson-json`.
Factor < 1.0 means the package format is faster than JSON; > 1.0 means slower.
Full results and plots: `benches/results/`.

<!-- benchmark-summary:start -->
**Acquired data**

| Case | cityjson-parquet | `cityjson-arrow` | `cityjson-json` | Factor |
| --- | --- | --- | --- | --- |
| `io_basisvoorziening_3d_cityjson` | 601.8 MiB/s | 611.9 MiB/s | 282.7 MiB/s | 2.13x |
| `io_3dbag_cityjson_cluster_4x` | 548.0 MiB/s | 530.2 MiB/s | 183.2 MiB/s | 2.99x |
| `io_3dbag_cityjson` | 596.7 MiB/s | 598.3 MiB/s | 190.6 MiB/s | 3.13x |

**Stress cases**

| Case | cityjson-parquet | `cityjson-arrow` | `cityjson-json` | Factor |
| --- | --- | --- | --- | --- |
| `stress_attribute_heavy_heterogenous` | 252.3 MiB/s | 243.7 MiB/s | 151.8 MiB/s | 1.66x |
| `stress_attribute_heavy_homogenous` | 179.9 MiB/s | 174.4 MiB/s | 162.3 MiB/s | 1.11x |
| `stress_boundary_heavy` | 3065.3 MiB/s | 3388.4 MiB/s | 317.7 MiB/s | 9.65x |
| `stress_geometry_heavy` | 1517.5 MiB/s | 1522.6 MiB/s | 271.8 MiB/s | 5.58x |
| `stress_hierarchy_heavy` | 1099.5 MiB/s | 1110.3 MiB/s | 188.9 MiB/s | 5.82x |
| `stress_resource_heavy` | 763.3 MiB/s | 769.3 MiB/s | 154.5 MiB/s | 4.94x |
| `stress_vertex_heavy` | 4374.5 MiB/s | 4435.5 MiB/s | 357.6 MiB/s | 12.23x |
<!-- benchmark-summary:end -->

## Verification

```shell
just fmt
just lint
just check
just test    # requires ../cityjson-arrow checked out as a sibling
just rustdoc
just bench-check
```

## Repository Map

- `src/lib.rs`: public exports — re-exports schema types from `cityjson-arrow`
  and `PackageReader`, `PackageWriter` from the `package` module
- `src/package/mod.rs`: `PackageWriter`, `PackageReader`, `PackageSink`, and
  all lower-level read/write helpers
- `src/spatial.rs`: `SpatialIndex`, `SpatialEntry`, `BBox2D`, and Hilbert curve
  implementation
- `examples/viewer.rs`: three.js web viewer served over a raw TCP listener;
  uses `SpatialIndex` for frustum culling
- `tests/package_shared_corpus_roundtrip.rs`: conformance roundtrip tests over
  the shared corpus in `../cityjson-arrow/tests/support/`
