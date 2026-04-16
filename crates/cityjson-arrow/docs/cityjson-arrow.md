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

- stream writes emit canonical table batches directly to Arrow IPC frames
- stream reads decode ordered frames into the incremental model decoder
- batch export exposes ordered canonical tables without rebuilding a public
  `CityModelArrowParts` API surface
- doc-hidden parts helpers remain only for the sibling package crate and local
  diagnostics

## Current Design Gap Versus The VNext Plan

The proposed rewrite plan assumes a richer `cityjson-rs::relational` API than
is available today.

`cityjson-rs` currently provides:

- raw pool views
- dense remaps
- raw boundary accessors

It does not yet provide:

- a full stable relational snapshot API
- symbol-table-backed relational string dictionaries
- a relational import builder

That means `cityjson-arrow` can already present a thinner batch/stream surface,
but it still owns projection discovery and some semantic reconstruction glue.
