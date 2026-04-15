# Benchmark Implementation Plan

## Goal

Keep the `cityjson-json` benchmark suite small, deterministic, and fully
driven by the shared `cityjson-corpus` repository.

The benchmark harness should:

- read the shared benchmark index directly
- prepare input outside the timed closure
- measure read and write paths separately
- consume shared benchmark artifacts only
- avoid maintaining a local benchmark corpus mirror

## Current Shape

The benchmark suite now consumes the shared corpus index at
`../cityjson-corpus/artifacts/benchmark-index.json`.

The shared repo publishes:

- synthetic workload benchmark outputs
- published raw 3DBAG workload artifact paths

This crate benchmarks the shared CityJSON artifacts listed in the benchmark
index and does not keep local bootstrap data for performance inputs.

## Benchmark Cases

The current suite is whatever the shared benchmark index publishes for
workload `cityjson` cases. Conformance fixtures stay in the test suite. The
harness no longer owns a separate case taxonomy.

## Harness

Split the benchmark harness by operation:

- `benches/read.rs`
- `benches/write.rs`

The harness should:

- prepare input outside the timed closure
- use `Criterion` throughput reporting for every benchmark group
  - benchmark `cityjson-json` against `serde_json::Value` on the read side
  - benchmark `cityjson-json` against `serde_json::to_string` on the write side
- avoid measuring fixture generation

## Reporting

The reporting layer should continue to produce plots and README-ready tables.
The only structural change is that the source data now comes from the shared
benchmark index instead of a local `tests/data/generated/` mirror.

## Rollout Notes

- remove the local `cityjson-fake` benchmark dependency
- keep the benchmark harness limited to the shared corpus
- load benchmark inputs from the shared corpus checkout

## Acceptance Criteria

The implementation is done when:

- the benchmark suite no longer depends on `cityjson-fake`
- the suite reads benchmark inputs from the shared corpus repository
- read and write benchmarks remain separate and deterministic
- the repository no longer needs `tests/data/generated/`
