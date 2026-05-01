# cityjson-arrow

`cityjson-arrow` is the Arrow stream and batch codec for `cityjson-rs`.

## Public API

| Function / type | Purpose |
|---|---|
| `write_stream(writer, model, &ExportOptions)` | Write a model to a live Arrow IPC stream |
| `read_stream(reader, &ImportOptions)` | Read a model from a live Arrow IPC stream |
| `export_reader(model, &ExportOptions)` | Iterate over ordered canonical table batches |
| `ModelBatchDecoder` | Decode canonical table batches incrementally |
| `import_batches(header, projection, batches, &ImportOptions)` | Reconstruct a model from batches |

The input and output type is always `cityjson_types::v2_0::OwnedCityModel`.

## How it works

- Export reads the model through `cityjson_types::relational::ModelRelationalView` and
  writes canonical Arrow table batches in a fixed order.
- Stream write emits each batch directly as an Arrow IPC frame; no intermediate
  aggregate is built.
- Stream read decodes ordered frames one table at a time and reconstructs the
  model incrementally.
- Batch export (via `export_reader`) exposes the same ordered tables without
  writing a stream.

## Current limits

- Attribute projection layout is discovered from the model at export time. There
  is no schema registry or pre-declared projection.
- Import reconstructs `OwnedCityModel` through direct mutation. A future version
  may use a dedicated import builder when one is available in `cityjson-rs`.

## Shared schema types

The following types from `cityjson_arrow::schema` are used by both this crate
and `cityjson-parquet`:

- `CityArrowHeader` — identifies the package version, city model id, and CityJSON version
- `ProjectionLayout` — records the typed attribute layout used during export
- `PackageManifest` — describes the tables written to a persistent package
- `ExportOptions`, `ImportOptions` — control export and import behaviour
