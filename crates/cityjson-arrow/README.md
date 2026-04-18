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
| `io_basisvoorziening_3d_cityjson` | 611.0 MiB/s | 286.1 MiB/s | 2.11x |
| `io_3dbag_cityjson_cluster_4x` | 538.5 MiB/s | 185.9 MiB/s | 2.87x |
| `io_3dbag_cityjson` | 603.7 MiB/s | 192.8 MiB/s | 3.18x |

**Stress cases**

| Case | cityjson-arrow | `cityjson-json` | Factor |
| --- | --- | --- | --- |
| `stress_attribute_heavy_heterogenous` | 258.3 MiB/s | 152.2 MiB/s | 0.97x |
| `stress_attribute_heavy_homogenous` | 182.7 MiB/s | 166.9 MiB/s | 1.61x |
| `stress_boundary_heavy` | 3449.9 MiB/s | 320.7 MiB/s | 6.72x |
| `stress_geometry_heavy` | 1541.5 MiB/s | 281.0 MiB/s | 3.60x |
| `stress_hierarchy_heavy` | 1149.9 MiB/s | 190.9 MiB/s | 5.20x |
| `stress_resource_heavy` | 795.3 MiB/s | 160.0 MiB/s | 3.82x |
| `stress_vertex_heavy` | 4808.9 MiB/s | 364.6 MiB/s | 7.95x |
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
