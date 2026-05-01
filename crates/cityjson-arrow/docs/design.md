# Transport design

This document describes how `cityjson-arrow` and `cityjson-parquet` move city
model data across process and storage boundaries.

## Semantic boundary

The data model is `cityjson_types::v2_0::OwnedCityModel`. Both crates read and write
this type. The Arrow tables used for transport are an internal detail; they are
not part of the public API.

## Public API

`cityjson-arrow` exposes:

- `write_stream` / `read_stream` — live Arrow IPC stream transport
- `export_reader` — ordered canonical table batches for consumers such as `cityjson-parquet`
- `ModelBatchDecoder` / `import_batches` — reconstruct a model from ordered batches

`cityjson-parquet` exposes:

- `PackageWriter` / `PackageReader` — write and read a seekable single-file package

## How export works

Export reads the model through `cityjson_types::relational::ModelRelationalView`, which
provides a dense, ordinal-indexed view over all model data. The exporter derives
the attribute projection layout from this view and emits canonical Arrow table
batches in a fixed order.

## How import works

Import consumes ordered canonical table batches and reconstructs an
`OwnedCityModel` incrementally, one table at a time. The `ProjectionLayout`
carried in the stream prelude or package manifest tells the decoder how to
interpret typed attribute columns.

## Live stream vs persistent package

The live stream format (`cityjson-arrow`) is designed for process-to-process
transfer: it writes self-delimiting Arrow IPC frames that can be streamed
without knowing payload lengths up front.

The persistent package format (`cityjson-parquet`) is designed for seekable
file access: it writes Arrow IPC file payloads sequentially with a manifest at
the end, so readers can locate and decode individual tables without reading the
whole file.

Both formats use the same canonical table schema and reconstruction rules.

## Current limits

- The attribute projection layout is derived at export time from the model.
  There is no pre-declared schema registry.
- Import reconstructs `OwnedCityModel` through direct mutation. A future version
  may use a dedicated import builder if one becomes available in `cityjson-rs`.
