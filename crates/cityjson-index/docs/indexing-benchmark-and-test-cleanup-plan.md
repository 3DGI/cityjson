# Indexing Benchmark And Test Cleanup Plan

## Goal

Use real Basisvoorziening 3D geometry for benchmarking, add Linux-only CLI profiling, and keep the fast test suite focused on small tracked fixtures.

## Benchmark Data

- Treat `/home/balazs/Development/cityjson-corpus` as the corpus source of truth.
- Require the pinned Basisvoorziening 3D artifact at `artifacts/acquired/basisvoorziening-3d/2022/3d_volledig_84000_450000.city.json`.
- If that artifact is missing, fail with the acquisition command from the corpus repository.
- Prefer deterministic single-tile subsets before introducing any multi-tile benchmark expansion.

## Benchmark Harness

- Use a custom JSON-emitting benchmark runner.
- Benchmark only real-geometry Basisvoorziening inputs.
- Measure dataset open, indexing, full-scan reference iteration, representative `get()` calls, bbox queries, and `read_feature` reconstruction.
- Record dataset label, source artifact, subset size, byte size, worker count, operation, elapsed time, RSS, counts, and query hit counts.
- Keep benchmark execution out of `just ci`.

## Linux Profiling

- Add per-command `--profile <PATH>` support on Linux.
- Capture stage timings, RSS snapshots, platform, CPU count, timestamps, and success/error status.
- Reject `--profile` on unsupported platforms.

## Tests

- Keep fast correctness tests on tracked fixtures in `tests/data`.
- Add Linux-only profile-output tests for valid JSON, stage names, durations, and RSS fields.
- Gate corpus-backed tests behind `CITYJSON_CORPUS`.
- Consolidate common fixture helpers in `tests/common/mod.rs`.

