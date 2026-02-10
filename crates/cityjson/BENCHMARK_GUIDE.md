# Benchmark Guide

This project uses a single deterministic workflow for performance tracking.

## Run Full Suite

```
just perf "short description"
```

This runs:
- Criterion benches for each backend (`builder`, `processor`), and
- the memory benchmark (`memory`) with dhat heap profiling, and
- the streaming benchmark (`streaming`) as a supporting workload, and
- the Valgrind profiling suite (massif, cachegrind, memcheck).

Results are appended to `bench_results/history.csv`.

## Optional Knobs

```
just perf "desc" backend=default   # default|nested|both
just perf "desc" mode=fast         # fast|full
just perf "desc" seed=12345         # deterministic RNG seed
just perf "desc" size=2000          # override workload size
BACKEND_SPLIT=0 just perf "desc" backend=both    # opt back into mixed backends
```

Notes:
- `mode=fast` uses smaller inputs and Criterion `--quick`.
- `size` overrides the default workload sizes across suites.
- `mode` is recorded in the CSV so fast/full runs can be mixed safely.
- `BACKEND_SPLIT` defaults to `1` (nested uses `--no-default-features` to avoid duplicate benches).

## Results

The CSV schema is documented in `bench_results/README.md`.

Key metrics:
- `time_ms` from Criterion mean estimates.
- `throughput_elem_s` when throughput is configured.
- `heap_max_bytes` and `heap_total_bytes` from dhat.

Benchmark coverage:
- `builder/build_minimal_geometry` builds cityobjects with a minimal solid geometry.
- `builder/build_full_feature` builds cityobjects with attributes, semantics, materials, and textures.
- `memory` builds a **full‑feature** model (materials, textures, semantics, cityobject attributes).
- `processor/compute_mean_coordinates` is geometry traversal.
- `processor/compute_full_feature_stats` walks attributes, semantics, materials, and textures.
- `streaming/e2e` measures end-to-end ingestion as a supporting workload.
- Geometry-typed attributes are intentionally excluded to avoid backend-specific representation costs.

Analysis examples:
- `just perf-analyze description="my change" --plot`
- `just perf-analyze --series --plot bench="builder/build_full_feature" metric="time_ms"`
- `just perf-analyze --backend-overview --backend default --mode all`

## Profiling Targets

Valgrind runs the benchmark workload `processor/compute_full_feature_stats` per backend. You can override the defaults:

```
PROFILE_BACKEND=nested \
PROFILE_BENCH=processor \
PROFILE_BENCH_ID=compute_full_feature_stats \
PROFILE_MODE=fast \
just perf "desc"
```

Optional overrides:
- `PROFILE_SEED` / `PROFILE_SIZE` to match a specific bench run.
- `BACKEND_SPLIT=0` if you want mixed-backend builds.
