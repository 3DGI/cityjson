# Benchmarking

The benchmark suite on this branch is JSON-focused.

## What It Covers

- document read and write through `cityjson_lib`
- document read and write through `cityjson_lib::json`
- baseline `serde_json::Value` comparisons
- real 3DBAG-backed workloads prepared under `target/bench-data/`

Arrow and Parquet benchmark work is intentionally out of the current release
line.

## Common Commands

Prepare the pinned workloads:

```bash
just perf prepare
```

Run the throughput suite:

```bash
just perf "baseline" fast
just perf "baseline"
```

Profile a single workload:

```bash
just perf profile time cityjson_lib-read io_3dbag_cityjson
just perf profile dhat cityjson_lib-read io_3dbag_cityjson
just perf profile cachegrind cityjson_lib-read io_3dbag_cityjson
```

## Outputs

The benchmark tooling writes:

- per-run profile data under `target/bench-profile/`
- recorded history under `benches/results/history.csv`
- generated plots under `benches/results/plots/`

Use the recorded history helpers when you want to compare runs over time:

```bash
just perf analyze --list
just perf plot --description "baseline"
```
