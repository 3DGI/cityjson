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

## Verification

```shell
just fmt
just lint
just check
just test    # requires ../cityjson-arrow checked out as a sibling
just rustdoc
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
