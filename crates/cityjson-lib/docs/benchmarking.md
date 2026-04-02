# Benchmarking

`cjlib` now carries a consolidated benchmark slice for real 3DBAG data, with a
persisted reporting flow modeled after `cityjson-rs`.

## Pinned Workloads

- `io_3dbag_cityjson`
  One real 3DBAG CityJSON tile (`10-758-50`) from release `v20250903`.
- `io_3dbag_cityjson_cluster_4x`
  A merged four-tile real 3DBAG workload built from:
  `10-758-50`, `10-756-48`, `10-756-50`, and `10-758-48`.

Prepared inputs live under `target/bench-data/3dbag/v20250903/`.

The shared corpus is the source of truth. If the sibling
`../cityjson-benchmarks` checkout already contains the acquired CityJSON,
cityarrow, and cityparquet artifacts, `cjlib` reuses them directly.

## Prepare Inputs

```bash
just bench-prepare
```

This will:

- reuse the shared-corpus tile and native format files if they are already available
- reuse the shared-corpus `cluster_4x.*` artifacts if they are already available
- validate reused `.cjarrow` and `.cjparquet` files against the current decoders
- download the missing pinned 3DBAG tiles
- otherwise merge the four-tile stress case with `cjio`
- otherwise re-export incompatible or missing cityarrow stream files and
  cityparquet package files with `bench_export_formats`

## Throughput Benchmarks

```bash
just bench -- --quick
just bench
```

The Criterion suite benchmarks:

- `serde_json::Value` JSON read/write
- `serde_cityjson` JSON read/write
- `cjlib::json` JSON read/write
- `cityarrow` live Arrow IPC stream read/write
- `cityparquet` persistent package read/write

The Arrow and Parquet timings are end-to-end `format <-> CityModel`
benchmarks. The read path consumes prebuilt native-format files from the
corpus; it does not convert from CityJSON inside the timed loop. These timings
therefore include both native IO and reconstruction of the heap `CityModel`.

## Diagnostic Benchmarks

Use the split diagnostic target when you need the narrower ADR 0010 view:

```bash
just bench-diagnostic -- --quick
just bench-diagnostic
```

This target keeps the same pinned 3DBAG cases but measures the internal native
layers separately from the headline end-to-end path:

- `cityarrow/encode_parts`
- `cityarrow/decode_parts`
- `cityarrow/write_parts`
- `cityarrow/read_parts`
- `cityparquet/write_parts`
- `cityparquet/read_parts`
- `cityparquet/read_manifest`

Use `throughput` for product-facing `format <-> CityModel` numbers and
`diagnostic` to decide whether the remaining cost sits in shared-model
conversion, live-stream transport, or package/container work.

## Profiling

Run a single workload on a single case:

```bash
just bench-profile time serde_cityjson-read io_3dbag_cityjson
just bench-profile dhat serde_cityjson-read io_3dbag_cityjson
just bench-profile cachegrind serde_cityjson-read io_3dbag_cityjson
just bench-profile massif serde_cityjson-read io_3dbag_cityjson
```

Outputs are written under `target/bench-profile/<tool>/<case>/<workload>/`.

The profile harness now prepares only the requested workload before timing or
starting `dhat`. That keeps Arrow and Parquet read profiles focused on native
package-to-memory work instead of carrying preloaded JSON or `CityModel` state.

## Recorded Perf Runs

Use the consolidated runner to append one benchmark campaign to
`bench_results/history.csv`:

```bash
just perf "baseline before refactor" fast
just perf "baseline before refactor"
```

This runs:

- `just bench-prepare`
- the throughput Criterion suite
- dhat profiling for the real-data read workloads on the base and cluster cases
- cachegrind profiling for the same read workloads
- baseline-relative throughput plot generation for the recorded snapshot

Criterion still records the raw per-benchmark byte throughput that it was given.
That encoded-byte throughput is useful within a single format, but it is not a
fair cross-format speed comparison because JSON, Arrow IPC, and Parquet have
different on-disk sizes. The plotter therefore prefers:

- `logical_throughput_bytes_s`
  common-denominator throughput using the pinned CityJSON case size for the
  dataset
- otherwise `time_ms`
  inverse wall-clock time for older snapshots that predate the logical metric

This keeps the scatter plots comparable across formats.

The default profiled workloads are:

- `serde_json-read`
- `serde_cityjson-read`
- `cjlib-json-read`
- `cityarrow-read`
- `cityparquet-read`

Massif remains available as an explicit deep-dive tool:

```bash
just bench-profile massif serde_cityjson-read io_3dbag_cityjson_cluster_4x
PERF_RUN_MASSIF=1 just perf "cluster massif capture" fast
```

Analyze the recorded history with:

```bash
just perf-analyze --list
just perf-analyze --description "baseline before refactor"
just perf-analyze --description "baseline before refactor" --series --bench "deserialize/io_3dbag_cityjson/serde_cityjson/read" --metric heap_max_bytes
```

Generate baseline-relative throughput plots from the recorded history with:

```bash
just perf-plot --description "baseline before refactor"
just perf-plot --description "baseline before refactor" --timestamp 2026-04-01T20:15:06Z
```

Each plotted snapshot writes:

- `bench_results/plots/<description>/<timestamp>/throughput_relative_read.png`
- `bench_results/plots/<description>/<timestamp>/throughput_relative_write.png`
- `bench_results/plots/<description>/<timestamp>/benchmark_summary.md`

and refreshes a stable copy under `bench_results/plots/<description>/latest/`.

Current summaries:

- `summary.json`
  elapsed wall-clock timing for the exact profiled invocation
- `dhat-summary.json`
  peak and total heap usage
- `cachegrind-summary.json`
  D1 miss rate, LL miss rate, and branch miss rate
- `massif.txt`
  textual heap-growth trace from `ms_print`
- `bench_results/plots/...`
  baseline-relative speed plots and a compact markdown summary
