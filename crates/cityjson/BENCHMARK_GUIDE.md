# Benchmark Guide

This project uses a single deterministic workflow for performance tracking.

## Run Full Suite

```
just perf "short description"
```

This runs:
- all Criterion benches for each backend, and
- the Valgrind profiling suite (massif, cachegrind, memcheck).

Results are appended to `bench_results/history.csv`.

## Optional Knobs

```
just perf "desc" backend=default   # default|nested|both
just perf "desc" mode=fast         # fast|full
just perf "desc" seed=12345         # deterministic RNG seed
just perf "desc" size=2000          # override workload size
BACKEND_SPLIT=1 just perf "desc" backend=nested  # split-backend branch only
```

Notes:
- `mode=fast` uses smaller inputs and Criterion `--quick`.
- `size` overrides the default workload sizes across suites.
- `mode` is recorded in the CSV so fast/full runs can be mixed safely.

## Results

The CSV schema is documented in `bench_results/README.md`.

Key metrics:
- `time_ms` from Criterion mean estimates.
- `throughput_elem_s` when throughput is configured.
- `heap_max_bytes` and `heap_total_bytes` from dhat.

## Profiling Targets

Valgrind runs via `just profile-all`. You can override the default test target:

```
PROFILE_PKG=cityjson \
PROFILE_TEST=v2_0 \
PROFILE_TEST_FILTER=test_producer_consumer_stream \
just perf "desc"
```
