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

Results are appended to `benches/results/history.csv`.

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

The CSV schema is documented in `benches/results/README.md`.

Key metrics:

- `time_ms` from Criterion mean estimates.
- `throughput_elem_s` when throughput is configured.
- `heap_max_bytes` and `heap_total_bytes` from dhat.

Benchmark coverage:

- `builder/build_minimal_geometry` builds cityobjects with a minimal solid geometry.
- `builder/build_full_feature` builds cityobjects with attributes, semantics, materials, and textures.
- `memory` builds a **full‑feature** model (materials, textures, semantics, cityobject attributes). Set `BENCH_STREAMING=1` to switch from the bulk path (one model holding N features) to the streaming path (build one-feature model → consume → drop, N times). `just perf` records both as separate rows — `memory/build_model/<size>` and `memory/build_model_streaming/<size>` — so you can compare `heap_max_bytes` (streaming should collapse to ~O(1 feature)) and `heap_total_bytes` (if streaming stays close to bulk, the allocator is reusing freed slabs under build/drop churn).
- `processor/compute_mean_coordinates` is geometry traversal.
- `processor/compute_full_feature_stats` walks attributes, semantics, materials, and textures.
- Geometry-typed attributes are intentionally excluded to avoid backend-specific representation costs.

## Analyzing Results

`just perf-analyze` has four modes. Run `just perf-analyze --help` for the full reference.

### Run history (default)

Shows run-by-run deltas for the default backend — useful for a quick sanity check after `just perf`.

```
just perf-analyze
just perf-analyze --mode all
just perf-analyze --backend-overview-extremes
```

`--mode all` includes both `fast` and `full` runs. `--backend-overview-extremes` adds per-run best/worst movers.

### Compare against a named baseline

Compare the latest run (or a specific run via `--description`) against a previously-stored description.
The typical workflow is to tag a reference run with a memorable description and then compare future runs against it.

```
just perf "v0.7.0 baseline"                       # store the reference run

# ... later, after changes ...
just perf "my feature"
just perf-analyze baseline="v0.7.0 baseline"

# Or compare a specific current run explicitly:
just perf-analyze description="my feature" baseline="v0.7.0 baseline"
just perf-analyze baseline="v0.7.0 baseline" --percent --threshold 1
```

`--percent` shows ratio metrics (cache miss rates) as percentages. `--threshold N` hides changes below N%.

### Compare two commits

```
just perf-analyze --compare abc1234,def5678
just perf-analyze --compare abc1234,def5678 --backend-overview-extremes
```

Commit selectors are prefix-matched, so a 4–7 character prefix is usually enough.

### List available data

```
just perf-analyze --list
```

Shows all descriptions, bench IDs, and metrics in the CSV.

### Time series for one metric

```
just perf-analyze --series bench="builder/build_full_feature" metric="time_ms"
just perf-analyze --series --plot bench="builder/build_full_feature" metric="time_ms"
just perf-analyze --series --series-raw bench="processor/compute_full_feature_stats" metric="time_ms"
```

`--plot` adds a sparkline. `--series-raw` shows individual rows instead of per-timestamp averages.

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
