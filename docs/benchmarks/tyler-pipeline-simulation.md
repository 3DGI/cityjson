# Tyler pipeline benchmark simulation

The `cityjson-index` benchmark reproduces Tyler's three relevant stages over
the Groningen corpus: extent construction, spatial-grid work, and parallel
feature materialization. The last stage mirrors Tyler's 2,048-feature chunking
and its thread-local `CityIndex` pattern. Each worker consequently owns a
backend and vertex cache; identical source vertices can be retained once per
worker, which is the memory-multiplication behavior under investigation.

## Workload fidelity

`--workers` creates a dedicated Rayon pool of exactly that size for the Tyler
parallel stages. A profiled invocation accepts exactly one worker count and one
layout, runs in a fresh process, and materializes every selected feature once.
The Groningen sources and prepared SQLite sidecar are reused; preparation is
performed before profiling so indexing does not pollute the measured target.
`CITYJSON_GRONINGEN_CORPUS` names the directory that directly contains the
`.city.json` tiles. A profile fails before measurement if its prepared manifest
or worker-specific sidecar is missing, empty, or belongs to a different corpus
or tile count.

Two targets are available to the profiling supervisor:

- `tyler-pipeline` covers all three stages and is the end-to-end signal.
- `tyler-feature-materialization` isolates the cache-heavy stage for expensive
  attribution tools.

The harness appends start and end events for each stage to JSON Lines and
flushes each event. A killed run therefore retains its last known stage.

## Profiling policy

The authoritative policy and experiment matrix are in
[ADR 011](../../crates/cityjson-index/docs/adr/011-tiered-tyler-profiling.md).
In short:

- native cgroup runs and `perf stat` use all 182 tiles at 1, 4, and 24 workers;
- `perf record` and Heaptrack use 24 tiles and the same worker counts;
- Cachegrind and optional Massif use only one tile and one worker.

Valgrind serializes threads, so its output is a reduced diagnostic and cannot
validate the 24-worker OOM. Heaptrack is the allocation profiler for concurrent
execution. A full-corpus run must always specify a cgroup memory cap.

From `crates/cityjson-index`:

```sh
just profile-index perf-stat 24 182 "baseline" 32G
just profile-index heaptrack 24 24 "allocation baseline"
just profile-index-campaign "baseline" 32G
```

The supervisor writes one immutable directory per invocation below
`target/profiling/cityjson-index/`. It contains run metadata, stdout and stderr,
incremental stage events, 100 ms cgroup memory samples, an outcome summary, and
the selected profiler's raw output.

An OOM outcome is based on the cgroup's `memory.events` `oom_kill` counter, not
on an inferred signal. Timeouts and other failures remain distinct outcomes.
`memory.peak` and process `VmHWM` are lifetime metrics; neither is reported as
an operation-local RSS delta.

## Comparing an optimization

Capture a baseline and candidate with the same corpus, worker count, memory
cap, build profile, and host. Use three native/`perf stat` repetitions for
timing, inspect Heaptrack or `perf record` to choose the optimization, then
confirm the result with the full native 182-tile workload. Profiling results are
diagnostic evidence, not a CI performance threshold.
