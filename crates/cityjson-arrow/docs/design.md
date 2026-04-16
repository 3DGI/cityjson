# cityjson-arrow Design

This document records the current execution model for `cityjson-arrow`.

## Core Boundary

- the semantic model is still `cityjson::v2_0::OwnedCityModel`
- the public Arrow surface is batch-first and stream-oriented
- canonical tables remain an internal transport contract shared with
  `cityjson-parquet`

## Public Surfaces

- `write_stream` / `read_stream` own the live Arrow IPC stream boundary
- `export_reader` exposes ordered canonical table batches
- `ModelBatchDecoder` / `import_batches` own the batch-native import surface
- `cityjson_arrow::internal` keeps doc-hidden compatibility hooks for sibling
  crates and benchmarks

## Current Implementation Shape

- the live stream format is a JSON prelude plus ordered Arrow IPC table frames
- the incremental decoder reconstructs the semantic model table by table
- the public stream path does not route through `CityModelArrowParts`
- batch export currently materializes Arrow batches from the semantic model and
  carries `ProjectionLayout` explicitly

## Upstream Dependency

The proposed long-term design assumes a richer relational core in
`cityjson-rs`.

Until that exists, this crate still needs to:

- discover dynamic attribute projection layouts itself
- map current raw handles and dense remaps into canonical batch ids
- reconstruct `OwnedCityModel` through the existing semantic mutation path
