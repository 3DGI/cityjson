# Benchmark Results

This directory stores the single source of truth for performance history.

## Data Store

`bench_results/history.csv` contains one metric per row with the schema:

```
timestamp,commit,description,mode,backend,bench,metric,value,unit,seed,bench_version,rustc
```

- `timestamp`: ISO 8601 UTC timestamp of the run.
- `commit`: short git hash of the code under test.
- `description`: user-provided description.
- `mode`: `fast` or `full` input sizing.
- `backend`: `default` or `nested`.
- `bench`: stable benchmark ID (e.g. `builder/build_with_geometry`).
- `metric`: `time_ms`, `throughput_elem_s`, `heap_max_bytes`, `heap_total_bytes`,
  `heap_max_blocks`, `heap_total_blocks`, `cache_d1_miss_rate`,
  `cache_ll_miss_rate`, `branch_miss_rate`, `stream_total_s`,
  `stream_producer_s`, `stream_consumer_s`, `stream_throughput_buildings_s`.
- `value`: numeric metric value.
- `unit`: unit for `value` (e.g. `ms`, `elem_s`, `bytes`, `blocks`, `ratio`, `s`, `buildings_s`).
- `seed`: RNG seed used for deterministic data.
- `bench_version`: version tag for benchmark definitions (e.g. `v1`).
- `rustc`: full `rustc --version` output.

Notes:
- `tools/perf.sh` defaults to `BACKEND_SPLIT=1`, so nested runs use `--no-default-features` to avoid duplicate benchmarks.

## Running

Use the single entrypoint:

```
just perf "description of changes"
```

All outputs append to `bench_results/history.csv`.
