# Benchmark Results

`benches/results/history.csv` is the persistent benchmark history for `cityjson_lib`.

Schema:

```
timestamp,commit,description,mode,backend,bench,metric,value,unit,seed,bench_version,rustc
```

- `time_ms` and `throughput_bytes_s` come from Criterion.
- `logical_throughput_bytes_s` is derived from Criterion wall-clock time using a
  common logical dataset size per benchmark case for fair cross-format speed
  comparisons.
- `heap_*` metrics come from dhat.
- `cache_*_miss_rate` and `branch_miss_rate` come from cachegrind.

Use `just perf "description"` to append a new run. It also refreshes the
baseline-relative speed plots for that snapshot.

Use `just perf-analyze` to inspect the recorded history.

Use `just perf-plot --description "description"` to render baseline-relative
speed plots and a markdown summary into `benches/results/plots/`.
