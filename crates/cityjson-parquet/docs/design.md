# Design

This document describes the design decisions behind `cityjson-parquet`.

## Why a separate crate

`cityjson-parquet` was created by [ADR 3](https://github.com/). Live
process-to-process transport and persistent file storage have different access
patterns and should not share a single implementation.

The durable storage layer owns:

- a seekable single-file container format
- a native Parquet dataset format for ecosystem interoperability
- memory-mapped lazy payload access
- a reader that can inspect a file's manifest without decoding any geometry

The live stream layer stays in `cityjson-arrow`.

## Format surfaces

`cityjson-parquet` exposes two durable representations of the same canonical
CityJSON Arrow tables:

| Format | Physical layout | Use case |
|---|---|---|
| `.cityjson-parquet` package | One seekable file containing Arrow IPC table payloads and a footer manifest | Compact package distribution, package manifest inspection, and viewer-oriented access patterns |
| Native Parquet dataset | `manifest.json` plus `tables/{canonical_table}.parquet` | Cross-library Parquet interoperability, column projection, and predicate pushdown |

These formats intentionally do not try to produce identical bytes. The stable
contract is the canonical table schema plus semantic CityJSON equivalence after
decode.

## Package container format

The `.cityjson-parquet` package is not a Parquet columnar file despite the crate
name. It is a custom seekable container that stores Arrow IPC file payloads:

- The writer appends payloads sequentially. No seek-back pass is required.
- The manifest is written last. The reader finds it by reading the fixed-size
  footer at the end of the file.
- Memory-mapped access lets the reader slice individual payloads without
  loading the full file into memory.

This design prioritises write simplicity and read efficiency for the
random-access patterns typical of 3D city model viewers (load by viewport,
load by object type).

## Native Parquet dataset format

The native Parquet dataset writes each canonical table as a standalone Parquet
file and records table order, row counts, projection layout, and model metadata
in `manifest.json`.

This design prioritises independent implementation and ecosystem validation:
PyArrow, DuckDB, Polars, and other Parquet-native tools can project and filter
the table files directly. Nullable canonical `FixedSizeList` fields are encoded
as nullable Parquet lists with fixed-length validation at the reader boundary,
because this shape has better cross-library behavior than nullable Parquet
fixed-size list columns.

## Shared canonical tables

Both `cityjson-arrow` and `cityjson-parquet` use the same canonical table
schema. `cityjson-parquet` depends on internal bridges from `cityjson-arrow`
for this. A clean stable API boundary between the two crates is planned but not
yet implemented.

## Data model

The public data model is `cityjson_types::v2_0::OwnedCityModel`. Callers interact
with model values only through `PackageWriter` and `PackageReader`. The
canonical Arrow tables are a transport detail and are not part of the public API.

## Spatial index

`SpatialIndex` is built on a Hilbert space-filling curve. City objects are
ranked by their 2D centroid on a 2^16 × 2^16 grid and sorted by Hilbert index.
This layout clusters spatially nearby objects in the index array, making
viewport-based queries cache-friendly.

The index is computed at query time and is not stored in the file. It adds no
write cost and no file-format version constraints.

## Build dependency

`cityjson-parquet` requires `cityjson-arrow` to be checked out as a sibling
directory (`../cityjson-arrow`). There is no published crates.io release.

The shared corpus test suite lives in `../cityjson-arrow/tests/support/`. Both
repos must be present as siblings for the tests to compile.
