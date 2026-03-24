# Benchmark Guide

This project uses one benchmark-first workflow for performance tracking.

## Run Full Suite

```
just perf "short description"
```

This runs:

- Criterion benches (`builder`, `processor`), and
- the memory benchmark (`memory`) with dhat heap profiling, and
- the Valgrind profiling suite (massif, cachegrind, memcheck).

Results are appended to `bench_results/history.csv`.

## Optional Knobs

```
just perf "desc" mode=fast         # fast|full
just perf "desc" seed=12345        # deterministic RNG seed
just perf "desc" size=2000         # override workload size
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

Benchmark coverage:

- `builder/build_minimal_geometry` builds cityobjects with a minimal solid geometry.
- `builder/build_full_feature` builds cityobjects with attributes, semantics, materials, and textures.
- `memory` builds a **full‑feature** model (materials, textures, semantics, cityobject attributes).
- `processor/compute_mean_coordinates` is geometry traversal.
- `processor/compute_full_feature_stats` walks attributes, semantics, materials, and textures.
- Geometry-typed attributes are intentionally excluded to avoid backend-specific representation costs.

Analysis examples:

- `just perf-analyze description="my change" --plot`
- `just perf-analyze --series --plot bench="builder/build_full_feature" metric="time_ms"`
- `just perf-analyze --mode all`

## Profiling Targets

Use benchmark profiling targets for deep diagnosis of a single benchmark workload.

Default workload is `processor/compute_full_feature_stats`.

```
just profile-bench tool=time
just profile-bench tool=massif
just profile-bench tool=cachegrind
just profile-bench tool=memcheck
```

Override workload and sizing:

```
PROFILE_BENCH=processor \
PROFILE_BENCH_ID=compute_full_feature_stats \
PROFILE_MODE=fast \
PROFILE_SEED=12345 \
PROFILE_SIZE=2000 \
just profile-bench tool=massif
```
