# ADR 011: Use tiered profiling for the Tyler benchmark

- Status: Accepted
- Date: 2026-08-03

## Context

The Tyler pipeline benchmark reproduces the `cityjson-index` memory failure on
the 182-tile Groningen corpus with 24 workers. Optimizing it requires evidence
about both peak memory and throughput. No single profiler can provide that
evidence faithfully: instrumentation changes allocation and scheduling,
Valgrind serializes thread execution, and the production-size case can exhaust
the host before a profiler writes its result.

The benchmark also previously selected an indexing worker count without
constraining the Rayon pool used by Tyler's materialization stage. It measured
the materialization operation a second time to estimate an RSS delta. That
delta was neither an operation-local peak nor safe for an OOM reproducer.

## Decision

Use one benchmark implementation with two explicit profiling targets and a
tiered experiment matrix.

The harness owns a fixed-size Rayon pool. Each profiled worker count runs in a
fresh process, and feature materialization executes exactly once. Stage events
are appended and flushed as JSON Lines, so the last completed stage survives an
OOM. Process-lifetime `VmHWM` remains available only as a process metric; it is
not interpreted as an operation-local delta.

Preparation (format conversion and index construction) runs before the measured
process. The profiled process requires the matching prepared dataset and
worker-specific sidecar. It fails rather than creating, deleting, or rebuilding
that sidecar under the profiler.

Every experiment is launched in a transient user systemd cgroup v2 service.
The supervisor sets `MemoryMax`, disables swap with `MemorySwapMax=0`, samples
`memory.current`, `memory.peak`, `memory.events`, `memory.stat`, and
`memory.pressure` every 100 ms, and retains stdout, stderr, metadata, stage
events, samples, and tool output. A run is classified as an OOM only when the
cgroup reports an increment to `oom_kill`; timeouts and other signals are
reported separately. The full 182-tile run is rejected without an explicit
memory limit.

Use these tiers:

| Purpose | Target | Corpus | Workers | Tool |
|---|---|---:|---:|---|
| End-to-end time and memory | Full Tyler pipeline | 182 tiles | 1, 4, 24 | native cgroup run plus `perf stat`, three repetitions |
| CPU attribution | Feature materialization | 24 tiles | 1, 4, 24 | `perf record` |
| Allocation attribution | Feature materialization | 182 tiles | 1, 4, 24 | staged Heaptrack runs, one repetition |
| Cache and branch diagnosis | Feature materialization | 1 tile | 1 | Cachegrind |
| Optional allocation timeline | Feature materialization | 1 tile | 1 | Massif |

Heaptrack is the primary allocation profiler because it preserves concurrent
execution. Full-corpus Heaptrack runs execute sequentially under a 28 GiB
cgroup limit. The 4-worker peak is attempted only when four times the observed
1-worker peak remains below 90% of the limit. The 24-worker projection uses the
observed 1-to-4-worker duplicated-memory slope and the same threshold. A failed
gate or cgroup OOM stops the full-corpus sequence and triggers a comparable
24-tile Heaptrack fallback for 1, 4, and 24 workers.

The benchmark reports the retained vertex count and allocated vertex-buffer
capacity for every worker immediately after materialization. It then clears
the thread-local `CityIndex` values while the Rayon pool remains alive and
records a second RSS checkpoint. Heaptrack's live requested heap, these exact
cache capacities, and the before/after RSS checkpoints are interpreted
together. RSS minus requested heap is an upper bound that also includes
allocator metadata and arenas, stacks, anonymous mappings, and profiler
overhead; it is not labelled as pure fragmentation.

Cachegrind and Massif are diagnostic only and are mechanically restricted to
one worker and one tile because Valgrind serializes threads and adds substantial
memory and time overhead.

Profiling is diagnostic. This decision introduces no automated performance
threshold; comparisons record the commit, command, platform, corpus size,
worker count, memory cap, outcome, and raw artifacts for review.

## Consequences

- Worker-count comparisons describe the materialization pool that actually ran.
- OOM experiments cannot take down the host and leave useful partial evidence.
- Full-pipeline measurements remain representative, while Heaptrack either
  measures the full corpus safely or records why the reduced fallback was used.
- Running the complete campaign requires Linux, cgroup v2, a systemd user
  manager, `perf`, Heaptrack, and Valgrind.
- Reduced profiler runs diagnose causes but do not replace the native 182-tile
  validation after an optimization.

## Alternatives considered

- Extract the `cityjson-types` presentation suite first. Rejected for this
  change: presentation does not correct workload validity or contain OOMs. The
  shared result system remains a separate workspace concern in
  [ADR 004](../../../../docs/adr/004-shared-benchmark-results-presentation.md).
- Run the full corpus under every profiler. Rejected because instrumentation
  changes the workload and can prevent useful results from being written.
- Use Valgrind for multithread scaling. Rejected because its serialized thread
  execution cannot represent the production failure mode.
- Treat a signal or non-zero exit as an OOM. Rejected because it conflates
  timeouts, profiler failures, manual termination, and memory enforcement.

## Usage

Run one experiment from `crates/cityjson-index`:

```sh
just profile-index perf-stat 24 182 "baseline before vertex-cache change" 28G
just profile-index heaptrack 1 182 "allocation baseline" 28G
just profile-index cachegrind 1 1 "cache diagnostic"
```

Run the defined matrix with:

```sh
just profile-index-campaign "baseline before vertex-cache change"
```

Artifacts are written below `target/profiling/cityjson-index/`.
