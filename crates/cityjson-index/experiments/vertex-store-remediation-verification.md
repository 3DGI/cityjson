# ADR 012 vertex-store remediation verification

- Verification date: 2026-08-03
- ADR: [ADR 012](../docs/adr/012-evaluate-persistent-sparse-vertex-storage.md)
- Adversarial review:
  [vertex-store-implementation-review.md](vertex-store-implementation-review.md)
- Remediation plan:
  [parallel-agent-remediation-plan.md](parallel-agent-remediation-plan.md)
- Final shared harness: `vertex-cache@bc23d7e`

## Status and verdict

The remediation work resolves the correctness, lifecycle, bounded-memory,
validation, telemetry, and experiment-reachability defects reported in the
original review. All three final candidate commits pass their common bake-off
tests, full repository CI, Python tests, candidate-specific regression tests,
and an independent cross-review.

The implementations are therefore **accepted for the controlled ADR 012
campaign**. The controlled reindex, read-latency, and Tyler campaign is now
complete. The results provide preliminary signals but do not automatically
select a storage strategy; ADR 012 remains Proposed pending maintainer
selection.

## Final revisions

| Component | Branch | Final commit |
|---|---|---|
| Shared harness | `vertex-cache` | `bc23d7e` |
| Packed coordinate chunks | `experiment/vertex-store-packed` | `0ae34892d742f6417a72bdb268157cb98f4a8773` |
| Compact JSON offsets | `experiment/vertex-store-offsets` | `1f33165e5481455074ff007f7dff4b8d948e4287` |
| Frame-of-reference bit packing | `experiment/vertex-store-for` | `e76604d82402dbcc5bb50de23f4c3f8739891591` |

The three candidate branches contain the same final shared harness history.
Their only strategy-dependent implementation is the candidate registry/store.

## Original rejection and disposition

The review at baseline `vertex-cache@2f66f19` rejected all three candidates.
The shared executable only checked a two-field marker and emitted zero
telemetry; it could neither construct a candidate sidecar nor reconstruct a
package through a `VertexStore`. A fabricated marker-only database passed.
There was no green candidate-neutral harness, no complete validation against
the normalized index, no atomic construction path, no boundary-aware sample,
and no implementation of the four planned experiments.

Candidate-local defects compounded that common failure:

- packed validation accepted incomplete per-source chunk sets;
- JSON offsets rebuilt non-transactionally, repeatedly materialized all offset
  BLOBs, bypassed shared marker validation, and under-reported I/O;
- frame of reference did not write the shared marker, deleted outside the
  transaction, used SQL `substr`, decoded whole subblocks, incompletely
  validated storage, and under-reported reads;
- all candidates lacked sufficient malformed-storage and integrated package
  reconstruction evidence, and none passed the required CI gate.

Each finding is now addressed by the shared remediation or the candidate work
described below. No critical or high-severity finding from the original review
remains open.

## Shared harness remediation

The coordinator-owned harness now provides one candidate-neutral experiment
boundary and one reconstruction path for all strategies:

- a checked-in neutral candidate makes the harness independently buildable;
- construction runs inside a shared SQLite transaction and publishes source
  state and the strategy marker only with complete candidate state;
- a temporary sibling sidecar is built and validated before atomic
  replacement, preserving an existing sidecar after failed construction;
- strict read-only open rejects missing normalized tables, schema-v2 reindex
  requirements, stale sources, mismatched markers, incomplete source state,
  and invalid candidate tables without modifying the sidecar;
- authoritative vertex recount and per-source state prove empty-source
  handling, contiguous unit ordinals, full non-final units, and exact final
  coverage independently of candidate metadata;
- regular-CityJSON package reconstruction stages fragments, globally sorts and
  deduplicates `(source_id, vertex_index)` requirements, calls the selected
  store once, remaps geometry, restores request order and duplicates, and
  drops batch-local coordinate/encoded buffers before returning;
- singleton reads delegate to the same batch implementation;
- telemetry separates requested, unique, and returned vertices; persistent
  and source JSON bytes read; touched units; and retained decoded bytes;
- explicit build, sample, correctness/storage, read-latency, and Tyler commands
  fail closed on incomplete provenance and require a prebuilt matching
  sidecar;
- artifacts are written atomically and include the strategy, candidate and
  harness revisions, corpus and sample identities, sidecar, worker count,
  repetition, limits, and runtime configuration;
- model digests use canonical structural serialization with sorted object keys.

Three additional comparability failures were found adversarially while running
the preliminary Groningen checks and repaired in the shared harness:

1. numeric package IDs were sidecar-local and mapped 7,590 of 10,000 sample
   entries to different package identities across candidates; sample schema v2
   replaced them with stable source/model/package identities;
2. ordinary JSON serialization made model digests depend on randomized map
   iteration; canonical structural hashing removed this nondeterminism;
3. Tyler scans ordered packages by sidecar-local record ID, changing batch
   boundaries and deduplication; stable identity order now gives every
   candidate the same requests and batch boundaries.

The final harness also executes Tyler materialization with isolated worker
connections for the required 1, 4, and 24 worker cells.

## Candidate remediation and independent cross-review

### Packed coordinate chunks

Final candidate: `0ae34892d742f6417a72bdb268157cb98f4a8773`.

The implementation uses the ADR's 16,384-vertex chunks and exact 24-byte
little-endian signed-coordinate records. Construction participates in the
shared transaction. Validation proves complete source coverage, including
empty sources and the exact final chunk. Loads use SQLite incremental BLOB I/O,
coalesce adjacent requested indices, and account for bytes actually read.

The frame-of-reference implementer independently reviewed packed storage and
found that a chunk with data beyond the authoritative source end could be
accepted. The packed implementer added an explicit trailing-data rejection and
regression test in `0ae3489`; the verifier rechecked and signed off.

### Compact JSON offsets

Final candidate: `1f33165e5481455074ff007f7dff4b8d948e4287`.

Construction is transactional and streams bounded offset chunks. Open-time
validation uses bounded per-row incremental reads and proves monotonic
sentinels, source bounds, complete coverage, and freshness. Loads query only
metadata for requested sources, group requirements by chunk, read each needed
offset region once, coalesce source spans, and include both SQLite and source
file reads in telemetry.

The packed implementer independently found four remaining problems: a
whole-sidecar source-path scan per load, approximately 16,000 tiny validation
BLOB reads, acceptance of a trailing range, and missing cross-chunk overlap
validation. Commit `1f33165` repairs all four and adds regression coverage; the
verifier rechecked and signed off.

### Frame-of-reference bit packing

Final candidate: `e76604d82402dbcc5bb50de23f4c3f8739891591`.

The implementation uses 16,384-vertex superchunks, independently addressable
128-vertex subblocks, exact 27-byte descriptors, little-endian fields, and
least-significant-bit-first streams. Signed-domain arithmetic uses checked
`i128` subtraction/addition over the complete `i64` range. Construction and
deletion share the common transaction. Reads use incremental BLOB I/O, fetch
each touched subblock once, and decode only requested positions. Validation
checks descriptor widths, computed payload length, padding, bounds,
contiguity, and exact source coverage.

The JSON-offsets implementer independently requested stronger fixed-format and
failure evidence and found one unchecked direct addition. Commit `e76604d`
adds fixed 27-byte/golden payload tests (including `[0x34, 0x4e]`), 16,384 and
16,385 boundary cases, corrupt ordinal and short-final-data cases, duplicate
ordering coverage, and checked addition. The verifier rechecked and signed off.

## Final verification gates

The coordinator and cross-reviewers reran the common conformance suite on each
final revision. The bake-off test counts are:

| Candidate | Library tests | Bake-off CLI tests | Result |
|---|---:|---:|---|
| Packed chunks | 32 | 14 | Pass |
| JSON offsets | 28 | 14 | Pass |
| Frame of reference | 31 | 14 | Pass |

For every final revision:

- `just bakeoff-test` passed;
- `just ci` passed, including formatting, Clippy, all-target/all-feature
  checking, workspace tests, and documentation;
- `just test-python` passed;
- candidate and common malformed/truncated-storage regression tests passed;
- read-only failure, failed-build preservation, freshness, complete-coverage,
  boundary/extrema, duplicate/out-of-order batch, and canonical digest checks
  passed;
- measured reads reported `retained_decoded_bytes = 0`.

No candidate implementation contains unresolved review findings at its final
SHA.

## Audited Groningen sample

The controlled read campaign uses
`target/bakeoff/groningen-sample-v3.json` with:

- schema version: 2;
- corpus identity: `groningen-182-local-2026-08-03`;
- sample identity:
  `sha256:4494c7425c2e32d764cbb3fdaeed5dc8e76e5d4084fe4ae7891d2e3aec95d95d`;
- package references: 10,000 stable identities;
- represented sources: all 182 Groningen sources.

Generation deterministically forces baseline per-source coverage, source-first
and source-last vertex packages where available, packages touching the
128-vertex and 16,384-vertex unit boundaries where available, and early
representatives from the largest source files. It then fills in stable
source-round-robin order without duplicates. The sample was audited against
the independent per-package coverage summaries and authoritative per-source
vertex counts. The exact same persisted identity list must be used for every
candidate and repetition.

## Valid Groningen storage evidence

The retained sidecars were built from accepted candidate storage logic over
the same 182-source corpus. These storage quantities are structural and remain
valid; they are not timing results.

All candidates contain 111,713,328 vertices in 6,912 candidate units.

| Candidate | Candidate payload bytes | Candidate table bytes (`dbstat`) | Complete sidecar bytes | Payload bytes/vertex |
|---|---:|---:|---:|---:|
| Packed chunks | 2,681,119,872 | 2,684,768,256 | 3,322,355,712 | 24.000 |
| JSON offsets | 446,880,960 | 450,482,176 | 1,087,533,056 | 4.0003 |
| Frame of reference | 808,504,699 | 838,103,040 | 1,475,747,840 | 7.237 |

The complete-sidecar values include the common normalized index and SQLite
overhead. JSON-offset payload size does not include authoritative source JSON
bytes, which remain external and are counted during reads.

## Superseded preliminary artifacts

Files currently under `target/bakeoff/` named `*-correctness.json`,
`*-correctness-v2.json`, `*-correctness-stable.json`,
`*-correctness-final.json`, `*-read-stable.json`, `*-read-final.json`,
`*-tyler-stable.json`, or `*-tyler-1-final.json` are diagnostic artifacts, not
controlled campaign results. Their labels such as `stable` and `final` are
historical and must not be interpreted as acceptance.

Those files predate one or more of the stable-identity, canonical-digest,
stable full-scan-order, boundary-aware v3 sample, isolated-worker, and final
candidate revisions. Some Tyler artifacts visibly have different package
deduplication counts or digests between candidates. Earlier retained sidecar
build timings were single unconstrained runs rather than three fresh
28-GiB/swap-disabled repetitions. These measurements were useful for finding
harness defects but are intentionally excluded from the ADR comparison.

## Controlled Groningen-182 campaign — complete

The controlled campaign used the final revisions listed above, the immutable
Groningen-182 corpus, the audited v3 sample, serialized candidate order, fresh
processes, three repetitions per cell, a 100 ms memory sampler, and isolated
raw artifacts under `target/bakeoff/campaign-final/`.

### Environment and provenance

- Machine: Linux host.
- Memory limit: cgroup `MemoryMax=30064771072` (28 GiB).
- Swap limit: cgroup `MemorySwapMax=0`; every successful run observed zero
  swap.
- Memory sampling: 100 ms interval.
- Raw artifacts: `target/bakeoff/campaign-final/`.
- Command family used for each candidate and repetition under the systemd cgroup
  (with explicit dataset, sidecar, result, candidate commit, harness commit,
  corpus identity, worker/repetition, sample where applicable, and runtime
  provenance arguments):

  ```text
  systemd-run --user --unit=adr012-<experiment> --wait --collect \
    -p MemoryMax=30064771072 -p MemorySwapMax=0 \
    /tmp/adr012-run-limited.sh <raw-prefix> <final-release-binary> \
    <subcommand> --dataset-root <corpus> --sidecar <sidecar> \
    --result <result> --candidate-commit <candidate-commit> \
    --harness-commit bc23d7e --corpus-identity <corpus-id> \
    --workers <workers> --repetition <repetition> [--sample <sample>] \
    --runtime campaign=adr012-remediation --runtime profile=release
  ```

  `<subcommand>` was explicitly one of `build`, `correctness-storage`,
  `read-latency`, or `tyler-materialization`. The final release binaries were:

  - packed: `/tmp/cityjson-vertex-store-packed/crates/cityjson-index/target/release/vertex-store-bakeoff`;
  - offsets: `/tmp/cityjson-vertex-store-offsets/crates/cityjson-index/target/release/vertex-store-bakeoff`;
  - frame of reference: `/tmp/cityjson-vertex-store-for/target/release/vertex-store-bakeoff`.

  The wrapper recorded exact cgroup peaks and 100 ms memory samples for every
  run. The common gate was `just bakeoff-test` in each final worktree.

### Reindex: four workers

Times are wall-clock medians and ranges across three fresh repetitions.

| Candidate | Median wall time | Range | Peak RSS range | Peak cgroup range | Swap | Result |
|---|---:|---:|---:|---:|---:|---|
| Packed chunks | 12:38.10 | 12:37.92–12:38.85 | 4,463,276–4,536,988 KiB | 5.519–5.664 GiB | 0 | Pass |
| JSON offsets | 6:37.27 | 6:34.20–6:37.34 | 4,433,608–4,504,820 KiB | 5.427–6.394 GiB | 0 | Pass |
| Frame of reference | 12:40.93 | 12:40.84–12:44.31 | 4,434,924–4,444,076 KiB | 5.424–6.713 GiB | 0 | Pass |

### Correctness and read latency: audited v3 sample

Each one-off correctness materialization per candidate
produced the same digest
`sha256:c34c7086281c6e0883c09b3395885dd65756353ff3bc2398ce269285a3e7c7ea`.
Each returned 6,261,600 unique and returned vertices, with retained decoded
bytes equal to zero. Persistent/source JSON bytes are the per-candidate
operation totals:

| Candidate | Persistent bytes | Source JSON bytes | Retained decoded bytes | Result |
|---|---:|---:|---:|---|
| Packed chunks | 150,278,400 | 0 | 0 | Pass |
| JSON offsets | 71,593,872 | 164,644,920 | 0 | Pass |
| Frame of reference | 66,290,087 | 0 | 0 | Pass |

Read times below are seconds, shown as median (range) across three
repetitions. All 36 read passes produced the digest above and retained zero
decoded bytes.

| Candidate | Singleton first | Singleton repeat | Batch first (2,048) | Batch repeat (2,048) |
|---|---:|---:|---:|---:|
| Packed chunks | 10.2266 (10.2210–11.2665) | 10.2194 (10.2111–10.2396) | 10.4777 (10.4576–10.4881) | 10.3545 (10.3234–10.3699) |
| JSON offsets | 10.1832 (10.1168–11.9664) | 10.1535 (10.1176–10.1877) | 10.5895 (10.5877–10.6029) | 10.4794 (10.4564–10.4819) |
| Frame of reference | 10.3595 (10.3455–10.3871) | 10.3822 (10.3590–10.4055) | 10.8369 (10.8300–10.8603) | 10.7177 (10.6735–10.7322) |

### Tyler: all 707,239 packages

All 27 successful cells (three candidates × three worker counts × three
repetitions) materialized 707,239 packages, requested 125,352,085 vertices,
and returned 118,325,159 unique/returned vertices with retained decoded bytes
equal to zero. Each strategy produced the common digest
`sha256:7dc19f3033c0c7b8198114f5de9fec177baaba32a14b5aa5265b6d0de11de38c`.

Materialization elapsed times are seconds, median (range):

| Candidate | 1 worker | 4 workers | 24 workers |
|---|---:|---:|---:|
| Packed chunks | 244.398 (243.358–246.482) | 68.265 (68.041–68.544) | 23.765 (23.630–29.084) |
| JSON offsets | 249.796 (249.345–250.512) | 69.448 (69.285–69.734) | 24.287 (24.143–25.083) |
| Frame of reference | 251.144 (250.909–251.621) | 69.746 (69.544–69.944) | 24.316 (23.912–25.547) |

The 100 ms cgroup sampler recorded the following peak memory ranges (bytes)
across the three repetitions. Every range is below the 30,064,771,072-byte
cap, and all swap peaks were zero:

| Candidate | 1 worker | 4 workers | 24 workers |
|---|---:|---:|---:|
| Packed chunks | 4,264,284,160–4,404,146,176 | 10,063,376,384–10,146,791,424 | 21,491,101,696–23,583,150,080 |
| JSON offsets | 4,532,260,864–5,290,123,264 | 9,740,791,808–10,415,194,112 | 21,571,665,920–23,455,432,704 |
| Frame of reference | 4,523,958,272–5,093,785,600 | 10,193,801,216–10,332,680,192 | 22,985,535,488–23,831,994,368 |


Total pipeline elapsed times, including worker setup and orchestration, are
seconds, median (range):

| Candidate | 1 worker | 4 workers | 24 workers |
|---|---:|---:|---:|
| Packed chunks | 252.311 (251.303–254.315) | 76.077 (75.818–76.351) | 38.748 (37.687–41.107) |
| JSON offsets | 259.377 (257.643–260.013) | 78.003 (77.891–78.309) | 39.696 (38.972–41.536) |
| Frame of reference | 269.871 (269.652–270.773) | 89.307 (89.192–89.361) | 60.293 (59.531–61.760) |

The first frame-of-reference one-worker repetition failed closed on a
stable-identity error. A bounded probe passed, and the retry succeeded; the
failed attempt remains in the raw artifacts as anomalous evidence and is
excluded from the three-repetition statistics above. The failure did not
produce a partial result or alter the retained sidecar.

### Final decision

The controlled campaign is complete and all three candidates remain accepted
as reliable experimental implementations. The signals show a JSON-offset
advantage in sidecar size, reindex time, and singleton reads; packed chunks
lead batch reads and Tyler materialization/pipeline time; and frame of
reference has no clear advantage in these cells. Maintainers must still choose
the trade-off among size, source-file dependency, reindex cost, latency, and
concurrency. ADR 012 remains Proposed, and no candidate branch is merged as
the production implementation.
