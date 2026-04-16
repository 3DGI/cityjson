# cityjson-arrow

`cityjson-arrow` is the live Arrow codec crate for `cityjson-rs`.

## Public Surface

- `write_stream(writer, model, &ExportOptions)`
- `read_stream(reader, &ImportOptions)`
- `export_reader(model, &ExportOptions)`
- `ModelBatchDecoder`
- `import_batches(header, projection, batches, &ImportOptions)`

The semantic boundary remains `cityjson::v2_0::OwnedCityModel`.

## Execution Model

- batch export and stream writes start from `cityjson::relational::ModelRelationalView`
- stream writes emit canonical table batches directly to Arrow IPC frames
- stream reads decode ordered frames into the incremental model decoder
- batch export exposes ordered canonical tables without rebuilding a public
  `CityModelArrowParts` API surface
- doc-hidden parts helpers remain only for the sibling package crate and local
  diagnostics

## Remaining Gap Versus The VNext Plan

`cityjson-arrow` now consumes the borrowed relational export view exposed by
`cityjson-rs`, and `cityjson-rs` also exposes an owned relational
snapshot/import builder.

The remaining work is narrower:

- `cityjson-arrow` still owns projection discovery
- Arrow schema assembly still lives here
- Arrow-specific import glue still rebuilds the owned model in this crate

That keeps the crate thin, but it is still an Arrow transport layer rather than
pure forwarding around a fully upstreamed relational codec.
