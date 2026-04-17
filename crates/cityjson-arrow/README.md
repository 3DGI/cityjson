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
**Acquired data**

| Case | cityjson-arrow | `cityjson-json` | Factor |
| --- | --- | --- | --- |
| `io_basisvoorziening_3d_cityjson` | 623.8 MiB/s | 280.7 MiB/s | 2.19x |
| `io_3dbag_cityjson_cluster_4x` | 524.7 MiB/s | 185.3 MiB/s | 2.80x |
| `io_3dbag_cityjson` | 587.9 MiB/s | 191.4 MiB/s | 3.12x |

**Stress cases**

| Case | cityjson-arrow | `cityjson-json` | Factor |
| --- | --- | --- | --- |
| `stress_attribute_heavy` | 202.4 MiB/s | 171.0 MiB/s | 1.76x |
| `stress_boundary_heavy` | 3399.4 MiB/s | 317.4 MiB/s | 6.69x |
| `stress_geometry_heavy` | 1541.7 MiB/s | 276.4 MiB/s | 3.66x |
| `stress_hierarchy_heavy` | 1133.4 MiB/s | 193.0 MiB/s | 5.07x |
| `stress_resource_heavy` | 784.8 MiB/s | 159.2 MiB/s | 3.79x |
| `stress_vertex_heavy` | 4434.6 MiB/s | 360.5 MiB/s | 7.41x |
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
