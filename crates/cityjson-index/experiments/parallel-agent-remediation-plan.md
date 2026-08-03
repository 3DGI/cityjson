# ADR 012 remediation and re-evaluation plan

## Objective

Repair the shared vertex-store bake-off harness and each of the three candidate
branches so they implement ADR 012 end to end, then independently re-verify
correctness, bounded memory, telemetry, and comparability before collecting
decision-grade Groningen-182 measurements.

The review findings in
[vertex-store-implementation-review.md](vertex-store-implementation-review.md)
are acceptance criteria, not suggestions. A candidate remains rejected until
every critical and high-severity finding that applies to it has a regression
test and passes the common verification sequence.

## Phase 1: coordinator-owned harness repair

Complete this phase on `vertex-cache` before rebasing or editing candidate
branches. Record the resulting commit as `HARNESS_SHA`.

1. Add a checked-in candidate-neutral `candidate.rs` so the harness compiles
   and its common tests run without selecting a strategy. Candidate branches
   replace only the registry implementation after rebasing.
2. Change the `VertexStore` boundary so construction participates in a shared
   SQLite transaction and validation/load use the already-open checked
   connection. Loads receive sorted, unique batch requirements. Separate
   one-time validation/setup work from per-operation telemetry and report
   retained decoded-coordinate memory explicitly.
3. Add common per-source state containing the expected vertex count, unit
   count, and candidate payload bytes. Common validation must cover empty
   sources, every indexed regular-CityJSON source, contiguous ordinals, full
   non-final units, and exact final coverage.
4. Add a feature-gated, read-only `CityIndex` open path. It must never create,
   migrate, or rebuild a sidecar and must reject missing normalized tables,
   schema-v2 reindex requirements, stale source metadata, missing/mismatched
   bake-off state, incomplete source state, and invalid candidate tables.
5. Implement one coordinator-owned regular-CityJSON batch reconstruction path:
   stage unique package fragments, collect all referenced
   `(source_id, vertex_index)` pairs, globally sort/deduplicate them, invoke the
   store once, remap geometry, assemble models, and restore request order and
   duplicates. The singleton path delegates to a one-item batch. No decoded
   source-sized coordinate array or encoded buffer survives the call.
6. Replace the marker-only executable with explicit construction, deterministic
   sample generation, correctness/storage, read-latency/batching, and Tyler
   materialization commands. Every measured command requires an explicit
   prebuilt sidecar and complete provenance and fails closed for unknown values.
7. Construct a temporary sibling sidecar, build the normalized index and
   candidate state, write source state plus marker in the construction
   transaction, validate through the read-only path, and replace the retained
   sidecar atomically. A failed build must leave an existing retained sidecar
   unchanged.
8. Add deterministic, versioned 10,000-reference sampling that covers every
   source, largest sources, and package references touching unit boundaries.
   Persist its identity and reuse the same sample across all candidates.
9. Add a canonical `bakeoff-test` recipe that exercises all feature-gated
   common tests. Do not publish `HARNESS_SHA` until `just bakeoff-test`,
   `just ci`, and `just test-python` pass from `crates/cityjson-index`.

## Phase 2: parallel candidate repair

Rebase all three existing worktrees onto the exact `HARNESS_SHA`. Resolve the
expected candidate-registry add/add conflict by retaining the branch strategy
and conforming it to the shared factory. Give each worktree a distinct target,
work, sidecar, and result directory. Candidate agents must not edit shared
harness files; shared defects are reported to the coordinator and fixed once.

Run these agents concurrently:

| Worktree | Branch | Required repair |
|---|---|---|
| `/tmp/cityjson-vertex-store-packed` | `experiment/vertex-store-packed` | Prove complete per-source chunk coverage, including empty sources and final chunks; use the shared transaction/marker lifecycle; retain incremental BLOB reads and adjacent-index coalescing; make telemetry account for actual operation reads. |
| `/tmp/cityjson-vertex-store-offsets` | `experiment/vertex-store-offsets` | Remove corpus-wide offset-BLOB materialization from validation/load; validate incrementally once at open; group requirements by chunk; read each offset region and coalesced source span once; preserve freshness; use the shared atomic lifecycle; report every persistent/source byte read. |
| `/tmp/cityjson-vertex-store-for` | `experiment/vertex-store-for` | Use the shared marker and transaction; keep deletion inside it; replace SQL `substr` with SQLite incremental BLOB reads; read each touched subblock once and decode only requested positions; validate widths, payload size, padding, bounds, and full signed-`i64` conversions. |

Each implementation agent must add malformed/truncated-storage regression tests,
complete boundary/extrema tests, commit a focused series, and report its branch,
SHA, exact commands, and artifact paths. Each must run `just bakeoff-test`,
`just ci`, and `just test-python` before handoff.

## Phase 3: independent cross-review

Use a ring so no implementation is accepted only on its author's evidence:

- the packed implementer verifies JSON offsets;
- the offsets implementer verifies frame of reference;
- the frame-of-reference implementer verifies packed chunks.

The verifier reruns conformance tests, injects malformed/truncated state,
checks marker and read-only failures, compares reconstructed package digests,
and audits telemetry against the actual BLOB/file reads. Findings return to the
implementer; the verifier must recheck every fix. A candidate is accepted only
after both roles sign off on the final SHA.

## Phase 4: coordinator verification

For each final branch, independently rerun:

1. `just bakeoff-test`;
2. regular-CityJSON and normalized-schema reconstruction tests;
3. `just ci`;
4. `just test-python`;
5. source-by-source first/last, subblock/chunk-boundary, and extrema coordinate
   comparisons;
6. singleton and duplicate/out-of-order batch digest comparisons against the
   authoritative production reconstruction path;
7. failed-build preservation and measured-read immutability tests;
8. retained-heap checks showing no source-sized decoded vertex array remains.

Write the evidence, commands, final SHAs, and any exclusions to
`experiments/vertex-store-remediation-verification.md` on `vertex-cache`.

## Phase 5: controlled Groningen-182 campaign

Only accepted branches enter measurement. Use the same immutable corpus,
sample identity, harness SHA, machine, runtime configuration, and sidecar build
procedure.

- Correctness/storage: one fresh sidecar; full conformance; complete and
  per-table sizes; source vertex/payload/unit counts; observed bytes/vertex.
- Reindex: three fresh-process repetitions, four workers, 28 GiB memory limit,
  swap disabled; report median and range.
- Read latency: the common 10,000-reference sample; singleton and 2,048-package
  batches; first and immediately repeated passes reported without claiming
  controlled cache temperature.
- Tyler: all 707,239 packages at 1, 4, and 24 workers, three fresh native
  repetitions per cell, 100 ms memory sampling.

Serialize performance runs and rotate candidate order between repetitions.
Every artifact records strategy, candidate SHA, harness SHA, corpus identity,
sidecar path, workers, repetition, sample identity, limits, and runtime. Keep
large sidecars and raw profiler data under isolated `target/` directories.

Exclude any candidate with a correctness, precision, completeness,
bounded-memory, telemetry, read-only, or CI failure. Update ADR 012 only with
comparable accepted results and the maintainers' selection rationale. Do not
merge candidate branches; port the selected representation in a separate
production change.

## Coordination rules

- Shared harness work is sequential and coordinator-owned; candidate repair is
  parallel; performance measurement is serialized.
- Preserve the existing review and planning artifacts on `vertex-cache`.
- Reuse the named branches and worktrees; do not create replacement histories.
- Connection pooling and a concurrent public `CityIndex` API remain out of
  scope.
