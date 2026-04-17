# cityjson-parquet Design

This document records the design decisions behind `cityjson-parquet`.

## Origin

`cityjson-parquet` was created by ADR 3 ("Separate Live Arrow IPC From
Persistent Package IO"). ADR 3 established that live process-to-process
transport and persistent file storage have different access patterns and should
not share a single implementation.

The persistent package layer therefore owns:

- a seekable single-file container format
- memory-mapped lazy payload access
- a manifest-first reader that can inspect a file without decoding geometry

The live stream layer remains in `cityjson-arrow`.

## Package Container

The format is intentionally not a Parquet columnar file despite the crate name.
It is a bespoke seekable container that embeds Arrow IPC file payloads:

- the writer appends payloads sequentially — no seek-back pass is required
- the manifest is written last; the reader finds it by reading the fixed-size
  footer at the end of the file
- memory-mapped access lets the reader slice individual payloads without
  allocating or deserialising the full file

The container design prioritises write simplicity and read efficiency for the
random-access patterns typical of 3D city model viewers (load by viewport,
load by object type).

## Canonical Table Sharing

Both `cityjson-arrow` and `cityjson-parquet` use the same canonical table
schema, `IncrementalDecoder`, and `CanonicalTableSink`. The package crate
depends on doc-hidden bridges from `cityjson-arrow` for this.

A clean public API boundary between the two crates is planned but not yet
implemented.

## Semantic Boundary

The semantic unit remains `cityjson::v2_0::OwnedCityModel`. The package format
is a transport-layer detail; callers interact only with `OwnedCityModel` values
via `PackageWriter` and `PackageReader`.

## Spatial Index

`SpatialIndex` is built on a Hilbert space-filling curve. Objects are ranked by
their 2D centroid on a 2^16 × 2^16 grid, then sorted by Hilbert index. This
layout clusters spatially nearby objects in the index array, making
viewport-based queries cache-friendly.

The index is computed at read time and is not stored in the file, so it adds no
write cost and no file-format version constraints.

## Upstream Dependency

`cityjson-parquet` requires `cityjson-arrow` to be checked out as a sibling
directory. There is no published crates.io release of `cityjson-arrow` that
this crate consumes; the dependency is `path = "../cityjson-arrow"`.

The shared corpus test suite also lives in `../cityjson-arrow/tests/support/`
and is included directly via a Rust path include. Both repos must be present as
siblings for the tests to compile.
