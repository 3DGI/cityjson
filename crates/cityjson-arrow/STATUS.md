# Status Report

## Current State

`cityjson-arrow` is now described and exercised as a batch-first and
stream-oriented Arrow codec over `cityjson-rs`.

- live Arrow transport is `write_stream` / `read_stream`
- ordered batch export is `export_reader`
- ordered batch import is `ModelBatchDecoder` / `import_batches`
- doc-hidden parts bridges remain for the sibling `cityjson-parquet` crate
- the package schema id is `cityjson-arrow.package.v3alpha2`

## What Changed In This Slice

- the root public API no longer centers `ModelEncoder` / `ModelDecoder`
- repo-local tests and benches now use the function-based stream API
- the crate exposes a public ordered-batch reader and decoder surface
- documentation and ADRs now describe `cityjson-arrow` as a thin codec rather
  than a parts-centric transport layer

## Verification Snapshot

The tree is expected to pass:

- `just fmt`
- `just lint`
- `just check`
- `just test`

## Remaining Limits

- `cityjson-rs` does not yet expose the full proposed relational import/view
  API, only raw pool views and dense remaps
- `ProjectionLayout` discovery still happens in `cityjson-arrow`
- some doc-hidden compatibility hooks remain because `cityjson-parquet`
  currently depends on them
