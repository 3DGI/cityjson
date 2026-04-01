# Benchmarking

`cjlib` now carries a first consolidated benchmark slice for real 3DBAG data.

## Pinned Workloads

- `io_3dbag_cityjson`
  One real 3DBAG CityJSON tile (`10-758-50`) from release `v20250903`.
- `io_3dbag_cityjson_cluster_4x`
  A merged four-tile real 3DBAG workload built from:
  `10-758-50`, `10-756-48`, `10-756-50`, and `10-758-48`.

Prepared inputs live under `target/bench-data/3dbag/v20250903/`.

## Prepare Inputs

```bash
just bench-prepare
```

This will:

- reuse the shared-corpus tile if it is already available
- download the missing pinned 3DBAG tiles
- merge the four-tile stress case with `cjio`

## Throughput Benchmarks

```bash
just bench -- --quick
just bench
```

The Criterion suite benchmarks:

- `serde_json::Value` JSON read/write
- `serde_cityjson` JSON read/write
- `cjlib::json` JSON read/write
- `cityarrow` Arrow IPC package read/write
- `cityparquet` Parquet package read/write

## Profiling

Run a single workload on a single case:

```bash
just bench-profile time serde_cityjson-read io_3dbag_cityjson
just bench-profile dhat serde_cityjson-read io_3dbag_cityjson
just bench-profile cachegrind serde_cityjson-read io_3dbag_cityjson
just bench-profile massif serde_cityjson-read io_3dbag_cityjson
```

Outputs are written under `target/bench-profile/<tool>/<case>/<workload>/`.

Current summaries:

- `summary.json`
  elapsed wall-clock timing for the exact profiled invocation
- `dhat-summary.json`
  peak and total heap usage
- `cachegrind-summary.json`
  D1 miss rate, LL miss rate, and branch miss rate
- `massif.txt`
  textual heap-growth trace from `ms_print`
