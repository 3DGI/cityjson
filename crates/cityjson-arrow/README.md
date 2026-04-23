# cityjson-arrow

`cityjson-arrow` is the Arrow stream and batch codec for `cityjson-rs`.

It moves `cityjson::v2_0::OwnedCityModel` across Arrow IPC boundaries:

- `write_stream` / `read_stream` — live Arrow IPC stream transport
- `export_reader` — ordered canonical table batches (used by `cityjson-parquet`)
- `ModelBatchDecoder` / `import_batches` — reconstruct a model from ordered batches
- shared schema and manifest types re-exported by `cityjson-parquet`

## How it works

- Export reads the model through `cityjson::relational::ModelRelationalView` and
  emits canonical Arrow table batches in a fixed order.
- The live stream path writes batches directly as Arrow IPC frames without
  building an intermediate aggregate.
- Import decodes ordered frames one table at a time and reconstructs the model
  incrementally.

## Current limits

- Attribute projection layout is derived at export time. There is no
  pre-declared schema registry.
- Import reconstructs `OwnedCityModel` through direct mutation.

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

```shell
just fmt
just lint
just check
just test
just bench-check
just rustdoc
just site-build
```

## Use of AI in this project

This crate was written with AI assistance after the schema and specs were defined by hand.
Development used an iterative process of testing, benchmarking, and optimization controlled and verified by me.

## Repository map

- `src/codec.rs` — public stream and batch codec surface
- `src/stream.rs` — live Arrow IPC framing
- `src/convert/` — export and import implementation
- `src/schema.rs` — shared schema and manifest types
- `src/internal.rs` — bridges for `cityjson-parquet` and benchmarks
- `docs/` — format specs, design notes, and ADRs
