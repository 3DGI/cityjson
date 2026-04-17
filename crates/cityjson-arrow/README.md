# cityjson-arrow

`cityjson-arrow` is the Arrow codec crate for `cityjson-rs`.

It exposes a batch-first and stream-oriented transport surface over
`cityjson::v2_0::OwnedCityModel`:

- `write_stream` / `read_stream` for live Arrow IPC transport
- `export_reader` for ordered canonical table batches
- `ModelBatchDecoder` / `import_batches` for batch-native import
- shared schema and manifest types used by `cityjson-parquet`

## Current Architecture

- the semantic model stays in `cityjson-rs`
- canonical Arrow tables are an internal transport contract, not the public
  user model
- the live stream path writes batches directly without rebuilding a public
  parts aggregate
- doc-hidden parts bridges remain only for the sibling `cityjson-parquet`
  package crate

## Current Limits

- `cityjson-rs` currently exposes raw pool views and dense remaps, not the
  full proposed relational import/view API
- export still derives `ProjectionLayout` from the semantic model
- import still reconstructs the semantic model through the existing
  `OwnedCityModel` mutation path

## Benchmarks

Read and write throughput compared to `cityjson-json` on the same models.
Factor < 1.0 means Arrow IPC is faster than JSON; > 1.0 means slower.
Full results and plots: `benches/results/`.

<!-- benchmark-summary:start -->
<!-- benchmark-summary:end -->

## Verification

The repository keeps:

- local roundtrip tests for the live stream and batch surfaces
- split benchmarks for conversion-only, transport-only, and end-to-end paths
- `just fmt`
- `just lint`
- `just check`
- `just test`
- `just bench-check`

## Repository Map

- `src/codec.rs`: public batch and stream codec surface
- `src/convert/`: canonical export/import implementation
- `src/stream.rs`: live Arrow IPC framing
- `src/schema.rs`: shared schema and manifest definitions
- `src/internal.rs`: doc-hidden bridges kept for sibling crates and benchmarks
- `docs/`: ADRs, design notes, and format docs
