# ADR 0010: Shared-Model Benchmark Semantics And Baseline

## Status

Accepted.

## Context

The benchmark campaign now compares four paths against the same real 3DBAG
workloads:

- `serde_json::Value`
- `serde_cityjson`
- `cityarrow`
- `cityparquet`

The benchmark corpus uses two pinned cases from the shared corpus:

- `io_3dbag_cityjson`
- `io_3dbag_cityjson_cluster_4x`

The design goal of this ecosystem is not only to read and write individual
formats quickly, but to make a single core `CityModel` usable across crates.
That means the benchmark that matters most is not raw format IO in isolation,
but end-to-end `format <-> CityModel`.

This needed to be stated explicitly because the first plot interpretation was
too loose: Arrow and Parquet package throughput had been compared using each
format's own encoded byte size. That is not a fair cross-format denominator.

## Decision

The benchmark suite is interpreted in three layers:

1. Primary benchmark: end-to-end `format <-> CityModel`
2. Secondary benchmark: package IO only
3. Secondary benchmark: `parts <-> CityModel` conversion only

The primary benchmark remains the headline result because the shared
`CityModel` is the ecosystem boundary.

Cross-format plots and summaries must use either:

- wall-clock time, or
- a common logical dataset-size denominator

They must not use each format's own encoded byte size as the cross-format
throughput denominator.

## What The Arrow And Parquet Layers Actually Do

For Arrow and Parquet there are two distinct operations.

### `to_parts` and `from_parts`

These convert between the heap `CityModel` and a canonical in-memory columnar
representation, `CityModelArrowParts`.

`CityModelArrowParts` is a set of Arrow `RecordBatch` tables such as:

- `metadata`
- `vertices`
- `cityobjects`
- `geometries`
- `geometry_boundaries`
- semantic tables
- appearance tables

`to_parts` does not write files. It walks the heap model, builds ID maps,
discovers projection layout, and exports the model into canonical tables.

`from_parts` does not read files. It validates the table set, allocates a new
heap model, imports metadata, vertices, semantics, materials, textures,
geometries, and cityobjects, and reconstructs the shared `CityModel`.

So:

- `to_parts`: `CityModel -> CityModelArrowParts`
- `from_parts`: `CityModelArrowParts -> CityModel`

### `write_package_*` and `read_package_*`

These convert between `CityModelArrowParts` and an on-disk package directory.

A package is not one monolithic file. It is a directory with:

- `manifest.json`
- one file per canonical table

For Arrow, the table files are Arrow IPC files. For Parquet, the table files
are Parquet files.

So:

- `write_package_*`: `CityModelArrowParts -> package directory on disk`
- `read_package_*`: `package directory on disk -> CityModelArrowParts`

### End-To-End Composition

The current Arrow and Parquet benchmarks are intentionally end-to-end shared
model benchmarks:

- Arrow write: `CityModel -> to_parts -> write_package_ipc_dir`
- Arrow read: `read_package_ipc_dir -> from_parts -> CityModel`
- Parquet write: `CityModel -> to_parts -> write_package_dir`
- Parquet read: `read_package_dir -> from_parts -> CityModel`

This means the measured cost includes both:

- package IO, and
- reconstruction or flattening of the shared model

That is the correct benchmark for the ecosystem promise, but it is not the same
thing as a raw Arrow or raw Parquet scan benchmark.

## Baseline Results

The current accepted baseline snapshot is the full run at:

- timestamp: `2026-04-01T21:21:02Z`
- description: `baseline`
- corpus version: `real-3dbag-v20250903`

The generated summary is in `bench_results/plots/baseline/latest`, and the raw
metrics are in `bench_results/history.csv`.

### Read Throughput

Using a common logical dataset-size denominator:

| Case | `serde_json::Value` | `serde_cityjson` | `cjlib::json` | `cityarrow` | `cityparquet` |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 225.0 MiB/s | 171.2 MiB/s | 151.0 MiB/s | 153.0 MiB/s | 140.5 MiB/s |
| 3DBAG cluster 4x | 186.7 MiB/s | 151.2 MiB/s | 138.0 MiB/s | 131.3 MiB/s | 125.0 MiB/s |

### Write Throughput

Using the same logical denominator:

| Case | `serde_json::Value` | `serde_cityjson` | `cjlib::json` | `cityarrow` | `cityparquet` |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 695.2 MiB/s | 623.9 MiB/s | 594.3 MiB/s | 100.9 MiB/s | 85.7 MiB/s |
| 3DBAG cluster 4x | 552.4 MiB/s | 478.7 MiB/s | 473.9 MiB/s | 85.3 MiB/s | 73.8 MiB/s |

### Read Peak Heap

| Case | `serde_json::Value` | `serde_cityjson` | `cjlib::json` | `cityarrow` | `cityparquet` |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 47.5 MB | 26.9 MB | 26.9 MB | 43.1 MB | 43.1 MB |
| 3DBAG cluster 4x | 172.9 MB | 100.7 MB | 100.7 MB | 155.1 MB | 154.8 MB |

### Read Total Allocated Bytes

| Case | `serde_json::Value` | `serde_cityjson` | `cjlib::json` | `cityarrow` | `cityparquet` |
| --- | --- | --- | --- | --- | --- |
| 3DBAG tile | 54.4 MB | 88.9 MB | 88.9 MB | 105.9 MB | 140.0 MB |
| 3DBAG cluster 4x | 198.8 MB | 329.1 MB | 329.1 MB | 391.4 MB | 503.5 MB |

## Interpretation

The current results say:

- `serde_json::Value` is still the fastest end-to-end baseline for both read
  and write.
- `serde_cityjson` is the strongest shared-model JSON path.
- `serde_cityjson` and `cjlib::json` materially reduce peak heap relative to
  `serde_json::Value`.
- Arrow and Parquet do not currently win on end-to-end `format <-> CityModel`.
- Parquet is consistently the heaviest path in both write cost and allocation
  churn.

The memory story is not one-dimensional:

- `serde_json::Value` is fastest, but carries the highest resident read
  footprint.
- `serde_cityjson` and `cjlib::json` minimize peak heap for the shared model,
  but allocate more total bytes than `serde_json::Value`.
- Arrow and Parquet have peak heap closer to `serde_json::Value` than to
  `serde_cityjson`, and they allocate substantially more total bytes.

The cache results do not reverse the overall ranking:

- `serde_cityjson` and `cjlib::json` show the best D1 miss rates on read.
- Arrow and Parquet show lower branch miss rates than the shared-model JSON
  paths.
- `serde_json::Value` remains fastest overall despite not having the best cache
  profile on every metric.

So the dominant cost is not explained by one cache number alone. The main cost
appears to be the total work required to flatten or reconstruct the shared
model.

## Memory Growth Pattern

The `cluster_4x` case is about `3.50x` larger than the base tile by logical
dataset size.

Across all formats:

- read peak heap grows by about `3.6x` to `3.7x`
- read total allocated bytes grow by about `3.6x` to `3.7x`
- read time grows by about `3.8x` to `4.2x`
- write time grows by about `4.1x` to `4.6x`

The current behavior is therefore close to linear in memory growth, with mildly
superlinear time growth at the larger working set.

## Consequences

Positive:

- the benchmark story now matches the actual ecosystem boundary
- cross-format plots use a defensible denominator
- the current baseline is explicit and reproducible
- Arrow and Parquet results can be discussed honestly without confusing package
  IO with full shared-model reconstruction

Tradeoffs:

- Arrow and Parquet do not currently support the intended narrative of being
  faster end-to-end than JSON for `CityModel` materialization
- the current suite still conflates format IO cost with `from_parts` and
  `to_parts` cost inside the end-to-end numbers

## Follow-Up

Keep the current end-to-end benchmark as the headline benchmark and add split
diagnostic benchmarks for:

- `read_package_*` only
- `write_package_*` only
- `from_parts` only
- `to_parts` only

Those secondary benchmarks should be used to answer a narrower question:

- is the format layer itself expensive, or
- is the shared-model conversion the dominant cost?
