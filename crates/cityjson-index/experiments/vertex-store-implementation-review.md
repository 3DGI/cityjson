# ADR 012 vertex-store implementation review

- Review date: 2026-08-03
- Review baseline: `vertex-cache` at `2f66f19`
- ADR: [ADR 012](../docs/adr/012-evaluate-persistent-sparse-vertex-storage.md)
- Experiment plan: [Parallel Vertex-Store Bake-Off](parallel-agent-plan.md)
- Packed candidate: `experiment/vertex-store-packed` at `319cbfd`
- JSON-offset candidate: `experiment/vertex-store-offsets` at `fa7ddf8`
- Frame-of-reference candidate: `experiment/vertex-store-for` at `cca5a67`

## Executive verdict

**No-go. None of the three branches correctly implements the experiment
described by ADR 012, and the branches are not ready for controlled
measurement.**

The shared harness is a placeholder rather than an experiment harness. It has
no command that constructs a candidate sidecar, no shared package
reconstruction path, no call to any candidate `VertexStore`, and no
implementation of the four experiments. The only executable behavior is to
check a two-field marker and optionally write an artifact containing zero
telemetry and `{"status":"validated-sidecar"}`. A database containing only
that marker table is accepted as validated.

This common defect is sufficient to reject all three branches: isolated codec
unit tests cannot establish package correctness, bounded end-to-end memory, or
comparable measurements. The strategy implementations also contain independent
defects:

- **Packed chunks is the closest storage primitive**, but it remains
  unreachable from the harness and its validation accepts incomplete chunk
  sets.
- **JSON offsets performs source-position lookups**, but construction is not
  atomic, the read path bypasses the shared marker check, and every load first
  materializes every offset BLOB in the sidecar while telemetry omits those
  reads.
- **Frame of reference cannot produce a sidecar accepted by the shared reader**
  because its build method never writes the required marker. It also uses SQL
  `substr` rather than incremental BLOB I/O and decodes every value in a
  subblock rather than only requested indices.

No Groningen-182 correctness or performance campaign should be run against
these commits. Any artifacts would measure different or nonexistent paths and
could not support the ADR decision.

## Review standard

The review treated every implementation claim as a hypothesis to falsify.
Code received credit only when the behavior was reachable through the shared
harness and supported by a relevant test or direct reproduction. A successful
command that selected zero tests was treated as no evidence. Missing mandatory
behavior was classified as non-compliant rather than deferred.

Severity levels used below:

- **Critical:** invalidates correctness, bounded-memory guarantees, or
  cross-candidate comparability.
- **High:** contradicts a required format, validation, or I/O behavior, or can
  admit corrupt/stale persistent state.
- **Medium:** makes telemetry, CI, or reproduction evidence unreliable.
- **Low:** documentation or maintainability defect without direct result
  corruption.

Branch-local references use `branch@commit:path:line` because each branch has a
different `candidate.rs` at the same path.

## Findings

### Critical: the shared harness baseline does not validate on its own

The coordinator baseline declares `pub mod candidate` at
`experiments/vertex_store/mod.rs:15-16`, but the baseline contains no
`candidate.rs`. Running `just ci` on `vertex-cache` stops in `fmt-check` with
`failed to resolve mod candidate` before compilation or tests. The parallel plan
requires the shared harness to be completed and validated before candidate
worktrees are created (plan lines 17-42). That prerequisite was not met.

**Impact:** there is no green, candidate-neutral harness commit. Common changes
cannot be validated independently, and every strategy branch must provide a
different file merely to make the shared feature compile.

**Required remediation:** provide a candidate-neutral registry or stub that
compiles and tests on the harness branch, then branch all candidates from that
green commit.

### Critical: the shared harness does not implement the experiment

ADR 012 requires one shared path that loads CityObject fragments, collects and
deduplicates requirements for the whole batch, calls the selected store,
remaps geometry, preserves input order and duplicates, and makes
`read_package` a one-item `read_packages` call (ADR lines 234-254). The parallel
plan repeats this as a prerequisite that must be completed before candidate
work starts (plan lines 17-40).

The shared module defines a trait and a standalone deduplication helper
(`experiments/vertex_store/mod.rs:72-104`) but contains no package preparation,
geometry collection, remapping, or assembly. The production package path was
not adapted: candidate diffs add only `candidate.rs` and a reproduction note,
and no candidate type is referenced outside its own module and unit tests.

The binary (`experiments/vertex-store-bakeoff.rs:74-100`) only:

1. derives a sidecar path;
2. checks the marker;
3. writes a zeroed telemetry object and a fixed status.

It never imports `VertexStore`, never constructs the branch's candidate type,
and never calls `build`, `validate_for_read`, or `load`. The four experiment
arguments at lines 55-71 are labels only. There is no reindex, correctness and
storage, read-latency, batching, or Tyler materialization implementation.

**Impact:** there is no executable path with which to validate package output,
memory behavior, storage size, reindex cost, or read performance. All candidate
branches fail the central intent of ADR 012 regardless of codec quality.

**Required remediation:** implement and test the coordinator-owned harness
before further strategy work. Candidate storage code must be reachable only
through the same shared reconstruction and measurement paths.

### Critical: a marker-only database is reported as a validated sidecar

`open_matching_read_sidecar` validates only `schema_version` and `strategy`
(`experiments/vertex_store/mod.rs:123-162`). The binary does not call the
candidate's `validate_for_read` method.

An adversarial fixture containing only this table was accepted:

```sql
CREATE TABLE vertex_store_bakeoff_state (
    id INTEGER PRIMARY KEY,
    schema_version INTEGER NOT NULL,
    strategy TEXT NOT NULL
);
INSERT INTO vertex_store_bakeoff_state VALUES (1, 3, 'packed-chunks');
```

The command exited successfully and wrote:

```json
{
  "telemetry": {
    "requested_vertex_count": 0,
    "unique_vertex_count": 0,
    "returned_vertex_count": 0,
    "persistent_bytes_read": 0,
    "source_json_bytes_read": 0,
    "touched_units": 0
  },
  "result": { "status": "validated-sidecar" }
}
```

The sidecar SHA-256 was identical before and after the command, so the
read-only property held, but the validation claim was false: the database had
no normalized schema and no candidate table or data.

**Impact:** a missing, corrupt, incomplete, or fabricated candidate can produce
an apparently successful result artifact. Measurement orchestration cannot use
the binary's exit status as an acceptance gate.

**Required remediation:** instantiate the selected candidate, validate the
normalized schema, marker, freshness, strategy table, complete per-source
coverage, and candidate invariants before emitting any result.

### Critical: candidate construction is not exposed

The `VertexStore::build` contract says construction follows an explicit
reindex (`experiments/vertex_store/mod.rs:72-84`), but the CLI has no build or
reindex command and no other code calls candidate construction. Each
reproduction note instructs the operator to use an “explicit reindex workflow”
or “candidate reindex integration” that is absent:

- packed: `experiment/vertex-store-packed@319cbfd:crates/cityjson-index/experiments/vertex_store/packed_chunks.md:23-26`;
- offsets: `experiment/vertex-store-offsets@fa7ddf8:crates/cityjson-index/experiments/vertex_store/json_offsets.md:18-21`;
- frame of reference: `experiment/vertex-store-for@cca5a67:crates/cityjson-index/experiments/vertex_store/frame_of_reference.md:19-22`.

**Impact:** the retained comparison sidecars required by Experiments 1-4
cannot be built reproducibly from these branches. Reindex timing, worker count,
source-read bytes, and transactional failure behavior cannot be measured.

### Critical: frame-of-reference build cannot create a matching sidecar

`FrameOfReferenceStore::build`
(`experiment/vertex-store-for@cca5a67:crates/cityjson-index/experiments/vertex_store/candidate.rs:42-109`)
creates and fills `source_vertex_superchunks` but never calls
`write_sidecar_marker`. In fact, the candidate does not import that helper.
Its `read_connection` at lines 29-34 also opens SQLite directly rather than
using `open_matching_read_sidecar`.

Even if an external caller invoked `build`, the shared binary would reject the
result for lacking the marker. Conversely, candidate-local `load` would bypass
the marker entirely.

The method also deletes old rows before starting its transaction (lines 58 and
77). A failed rebuild therefore commits the deletion while rolling back only
new inserts. If a marker had been added externally, it could remain next to an
empty candidate table.

**Impact:** Option C is internally inconsistent with the shared lifecycle and
cannot participate in the bake-off.

### High: JSON-offset validation performs a corpus-wide BLOB load per call

`JsonOffsetsVertexStore::load` calls `validate_for_read` for every batch
(`experiment/vertex-store-offsets@fa7ddf8:crates/cityjson-index/experiments/vertex_store/candidate.rs:140-142`). Validation
selects the `offsets` BLOB for every chunk and collects every row into a
`Vec<OffsetChunk>` before iterating it (lines 76-102).

For Groningen-182 this can materialize roughly the entire estimated 426 MiB
offset payload on every singleton or batch call. The actual requested chunks
are then queried again—once per requested vertex—at lines 145-179. The
`touched` map deduplicates only the telemetry count, not the SQLite reads.

`persistent_bytes_read` at lines 180-186 reports each touched offsets BLOB once
and omits the corpus-wide validation read and repeated per-requirement reads.

**Impact:** singleton and batch latency, memory, and persistent-byte telemetry
would be grossly misleading. The implementation defeats the sparse-read intent
even though it does not decode a complete coordinate array.

**Required remediation:** validate persistent structure once when opening a
prepared sidecar, without retaining all BLOBs; group requirements by chunk;
load each touched offset pair/chunk once; and account for every measured read
using one cross-candidate telemetry definition.

### High: JSON-offset rebuild is non-transactional and can preserve a stale marker

`JsonOffsetsVertexStore::build`
(`experiment/vertex-store-offsets@fa7ddf8:crates/cityjson-index/experiments/vertex_store/candidate.rs:44-69`) deletes all
offset rows, builds sources through independent connection operations, and
writes the marker only after success. It does not start a transaction.

On a first build, failure leaves a partial table without a marker. On a rebuild,
the previous valid marker remains while rows are deleted and partially
reinserted. `validate_for_read` does not prove that every non-empty source is
represented or that the final chunk reaches the source's true vertex count, so
a partial rebuild can remain marker-valid.

**Impact:** failed construction can destroy the previous usable store or expose
partial data as prepared. This conflicts with fail-closed reconstruction and
the repository's transactional indexing convention.

### High: JSON-offset and frame-of-reference loads bypass strategy markers

Offsets opens its path directly with `SQLITE_OPEN_READ_ONLY`
(`experiment/vertex-store-offsets@fa7ddf8:crates/cityjson-index/experiments/vertex_store/candidate.rs:31-36`) and frame of
reference does the same (`experiment/vertex-store-for@cca5a67:crates/cityjson-index/experiments/vertex_store/candidate.rs:29-34`).
Neither candidate-local load verifies bake-off schema or strategy. Packed uses
the shared checked open at
`experiment/vertex-store-packed@319cbfd:crates/cityjson-index/experiments/vertex_store/candidate.rs:181-183`.

**Impact:** direct or future integrated use of offsets/FOR can read a stale or
strategy-mismatched database despite the trait's stated marker contract.

### High: frame-of-reference I/O and decoding contradict ADR 012

ADR 012 requires SQLite incremental BLOB I/O for only the touched encoded
subblock and decoding only requested indices (ADR lines 205-209).

The implementation fetches payload bytes using SQL `substr(payload, ...)`
(`experiment/vertex-store-for@cca5a67:crates/cityjson-index/experiments/vertex_store/candidate.rs:558-576`), not
`rusqlite::Connection::blob_open`. Its decoder allocates one `[u64; 3]` for
every vertex in the subblock and decodes all axis streams before selecting
requested entries (lines 457-494).

The allocation is bounded to 128 vertices and is dropped, so this is not an
unbounded-cache defect. It is nevertheless a direct deviation from the format's
specified read path and biases comparison with the other candidates.

### High: incomplete candidate tables pass structural validation

All three validators check rows that exist but do not prove complete source
coverage:

- Packed checks adjacency only after a previous row exists and does not require
  each source's first ordinal to be zero
  (`experiment/vertex-store-packed@319cbfd:crates/cityjson-index/experiments/vertex_store/candidate.rs:134-174`).
- Offsets requires ordinal zero for represented sources but does not require a
  row for every non-empty source or prove the last sentinel is the actual end
  of the source vertex array
  (`experiment/vertex-store-offsets@fa7ddf8:crates/cityjson-index/experiments/vertex_store/candidate.rs:71-120`).
- Frame of reference validates rows independently and never checks ordinal
  contiguity or represented sources
  (`experiment/vertex-store-for@cca5a67:crates/cityjson-index/experiments/vertex_store/candidate.rs:111-151`).

An empty candidate table therefore passes each candidate's validation when the
table and source metadata exist. Missing requested coordinates later produce
an error, but a prepared sidecar must be rejected before measurement, not
mid-campaign.

### Medium: frame-of-reference telemetry omits significant persistent reads

Every `load` invokes `validate_for_read`, which reads every superchunk header
and computes every payload length (FOR lines 111-150 and 294-300). Loading a
touched subblock then reads its full superchunk header again (lines 182-189 and
236-249). Telemetry counts only the selected payload returned by SQL `substr`
(lines 190-193).

**Impact:** reported persistent bytes are not comparable to packed or offsets
and cannot explain measured latency.

### Medium: the deterministic sample does not satisfy ADR 012

`deterministic_stratified_sample`
(`experiments/vertex_store/mod.rs:165-201`) round-robins sorted package record
IDs by source. It has no inputs describing source size or vertex chunk
boundaries, so it cannot guarantee packages from the largest files or packages
touching chunk starts and ends as required by ADR lines 299-303. The function
is also never called by the binary.

### Medium: result provenance permits placeholders and omits runtime configuration

Candidate commit, harness commit, and corpus identity default to `"unknown"`
(`experiments/vertex-store-bakeoff.rs:28-33`), while the plan requires every
artifact to identify them. `runtime_configuration` is always empty (line 90).
Worker count, repetition, and experiment selection change labels only and do
not alter execution.

### Medium: every documented candidate test command runs zero tests

The exact commands in the three reproduction notes all exit successfully with
zero tests:

| Candidate | Documented filter | Observed result |
|---|---|---|
| Packed | `packed_chunks::tests` | 0 run, 20 filtered out |
| JSON offsets | `vertex_store_bakeoff::json_offsets` | 0 run, 17 filtered out |
| Frame of reference | `frame_of_reference` | 0 run, 18 filtered out |

All strategy implementations live under the module name `candidate`, so the
filters no longer match after the final module-alignment commits.

### Medium: no candidate passes the required CI gate

`just ci` was run at each candidate tip:

- Packed reached Clippy and failed on six shared-harness
  `missing_errors_doc` errors.
- JSON offsets stopped at `cargo fmt --all --check`; a separate `just lint`
  failed on the shared six errors plus `too_many_lines` in candidate `load`.
- Frame of reference failed on the shared six errors plus five
  `cast_possible_truncation` errors in candidate code.

Because CI stopped early, later `check`, `test`, and `doc` stages in the recipe
did not run as part of `just ci`. The separately executed all-features tests are
reported below.

## Candidate implementation assessment

### Packed coordinate chunks

**Verdict: storage primitive partially compliant; branch rejected.**

What is implemented correctly in isolation:

- DDL matches Option A, including the 16,384 limit and 24-byte length check
  (packed lines 36-50).
- Construction streams `[i64; 3]` values into bounded chunks and writes
  little-endian coordinates inside one transaction (lines 68-114 and 219-294).
- Reads use incremental SQLite BLOB I/O and coalesce adjacent requested indices
  (lines 298-389).
- The focused tests cover signed little-endian decoding, one chunk boundary,
  malformed row lengths, coalescing, and duplicate requirement rejection.

Why it is rejected:

- It is not connected to construction, package reconstruction, or experiments.
- Validation does not prove complete per-source chunk coverage.
- No package digest, relationship/membership, freshness lifecycle, malformed
  sidecar, empty-source, or retained-memory conformance test exists.
- Its documented focused-test command runs no tests and `just ci` fails.

### Compact JSON offsets

**Verdict: non-compliant; branch rejected.**

What is implemented correctly in isolation:

- DDL matches Option B and stores little-endian `u32` offsets plus a sentinel
  (offsets lines 44-60 and 331-367).
- Source size/mtime freshness is checked (lines 278-296).
- The scanner is bounded to the indexed vertex range, rejects a chunk span over
  `u32::MAX`, checks strict offset monotonicity, and coalesces consecutive source
  reads (lines 298-329, 404-503, 573-649).
- Focused tests exercise a 16,384 boundary and non-monotonic offsets.

Why it is rejected:

- It is unreachable from the harness.
- Validation reads the complete offset store on every call, then reloads
  touched chunks once per requested vertex, while telemetry hides those reads.
- Rebuild is non-transactional and can leave partial data under an old marker.
- Candidate reads bypass the schema/strategy marker.
- Structural validation cannot establish complete source coverage.
- Tests do not cover stale sources, malformed/truncated JSON ranges, sentinel
  forgery, failed rebuild rollback, package output, or retained memory.
- Formatting, lint, and the documented test command fail their gates.

### Frame-of-reference bit packing

**Verdict: non-compliant; branch rejected.**

What is implemented correctly in isolation:

- DDL, 16,384-vertex superchunks, 128-vertex subblocks, and 27-byte headers
  match Option C (FOR lines 42-56 and 390-427).
- The encoding order is axis-major and LSB-first, subblocks start on a new byte,
  and minima are little-endian `i64` values (lines 390-455 and 497-516).
- Checked `i128` difference/addition supports `i64::MIN` through `i64::MAX`
  (lines 430-437 and 457-494).
- Focused tests cover the full signed range, zero-width axes, a 128 boundary,
  padding, invalid widths, and payload length mismatch.

Why it is rejected:

- Build never writes the required marker and cannot produce a shared-reader
  compatible sidecar.
- Old rows are deleted outside the construction transaction.
- Candidate reads bypass marker validation.
- SQL `substr` replaces required incremental BLOB I/O.
- Every coordinate in a touched subblock is decoded, not only requested ones.
- Validation does not prove chunk contiguity/coverage, and telemetry omits
  global header validation and repeated header reads.
- No integrated build/load boundary, empty-source, failed-build, freshness,
  package-output, or retained-memory test exists.
- `just ci` and the documented test command fail their gates.

## ADR 012 compliance matrix

Legend: **P** proven in the isolated primitive, **F** violated, **U**
unimplemented or unreachable, **I** insufficient end-to-end evidence.

| Requirement | Packed | Offsets | FOR | Notes |
|---|:---:|:---:|:---:|---|
| Exact signed `i64` coordinates | P | P | P | Focused primitive coverage is strongest for packed/FOR; no package proof. |
| Original vertex index is lookup key | P | P | P | Direct arithmetic/grouping uses `vertex_index`. |
| Singleton avoids complete source vertex array | I | I | I | Primitive code is bounded, but no singleton package path exists. |
| Batch deduplicates globally and retains no cache | U | U | U | Helper exists; no package batch invokes it or any store. |
| Malformed/truncated data fails before partial model | U | U | U | No candidate-backed model assembly exists; structural validation is incomplete. |
| Empty source emits no rows | P | P | P | Proven by builder control flow, not focused empty-source tests. |
| Source-position freshness | N/A | P | N/A | Offsets checks size and mtime, but shared CLI does not call it. |
| Schema-v2 requires reindex | I | I | I | Shared marker rejects missing/stale markers, but no integrated candidate lifecycle exists and offsets/FOR loads bypass it. |
| Distinct paths and strategy provenance | I | I | I | Paths/enum exist; emitted provenance can be `unknown` and validates no strategy data. |
| Little-endian binary fields | P | P | P | Unit/code inspection supports this. |
| Shared package preparation/assembly | U | U | U | Critical common omission. |
| `read_package` uses one-item batch path | U | U | U | No experimental package API. |
| Complete sidecar construction command | U | U | U | Documentation refers to nonexistent integration. |
| Read process fails on invalid candidate data | F | F | F | Marker-only database is accepted by the binary. |
| Correctness/storage experiment | U | U | U | Only an enum label exists. |
| Reindex experiment | U | U | U | No construction command or measurements. |
| Singleton/batch latency experiment | U | U | U | No package reads or sample execution. |
| Tyler full-corpus materialization | U | U | U | Existing Tyler path is unchanged. |
| Comparable truthful telemetry | I | F | F | Packed payload accounting is plausible in isolation; offsets/FOR omit major reads. |
| Required CI green | F | F | F | All three `just ci` runs failed. |

## Validation evidence

### Branch ancestry and isolation

All pairwise merge bases are `2f66f195aa89a4f77f42f8e0624048c1c291c9dd`.
Each candidate adds only its strategy-specific `candidate.rs` and one Markdown
reproduction note. This satisfies the common-base and focused-diff mechanics,
but also proves no branch supplied the missing shared integration.

### Tests

`cargo test -p cityjson-index --all-features` passed on all three branches.
Relevant experimental unit tests executed:

- Packed: five candidate tests plus four shared-harness tests.
- JSON offsets: two candidate tests plus four shared-harness tests.
- Frame of reference: three candidate tests plus four shared-harness tests.

The remaining passing integration tests exercised the unchanged production
index and production `CityIndex::read_packages`; they never instantiate a
candidate store. The experimental binary target itself reported zero tests on
all branches.

No common conformance suite promised by plan lines 39-40 and 111-115 exists.
In particular, no test reconstructs a package through a candidate, compares
digests/counts/relationships, verifies a 2,048-package batch, corrupts a
prepared candidate sidecar end to end, or measures retained heap.

### Groningen-182

The local corpus is present with 182 `.city.json` files and a schema-v2
normalized sidecar. Candidate sidecars were not built because the required
construction integration does not exist. Running a manual, strategy-specific
out-of-band builder would not repair the missing shared package path or yield
comparable ADR experiments, so no corpus measurement was manufactured.

### CI and Python validation

The current `vertex-cache` report branch also fails `just ci`: `fmt-check`
cannot resolve the declared but absent `experiments/vertex_store/candidate.rs`.
This is a pre-existing harness failure, not caused by the Markdown report.

The all-features Rust tests passed independently, but the authoritative
`just ci` command failed on every branch as described above.
`just test-python` passed on all three branches: each ran 11
`cityjson-lib` and 6 `cityjson-index` Python tests. The candidate changes are
unreachable from the default Python API, so those passes do not supply evidence
for ADR 012 and do not override the failed CI verdict.

## Required gates before measurement

The following work is required before any candidate result is accepted:

1. Complete the shared harness with explicit sidecar construction and one
   candidate-backed package reconstruction path used by singleton, batch, and
   Tyler workloads.
2. Make candidate selection instantiate the correct store and enforce marker,
   normalized-schema, freshness, table integrity, and complete source coverage
   before a measured read.
3. Make construction atomic, including deletion/replacement and marker update;
   a failed rebuild must preserve the prior valid sidecar.
4. Define telemetry at the shared boundary and count all validation, header,
   SQLite BLOB, and source reads consistently across strategies.
5. Fix offsets to avoid global BLOB materialization and repeated chunk loads.
6. Fix FOR to write its marker, use incremental BLOB I/O, and decode only
   requested indices.
7. Add the common conformance suite: empty sources, order/duplicates,
   boundaries/extrema, missing/corrupt/truncated storage, freshness, schema-v2,
   failed-build rollback, deterministic package digests/counts/relationships,
   and retained-memory checks.
8. Implement the four experiment entry points and the boundary-aware,
   largest-source-inclusive deterministic sample.
9. Make provenance mandatory and populate runtime configuration; reject
   `unknown` identifiers in recorded campaigns.
10. Correct all reproduction commands and require a non-zero intended test
    count.
11. Pass `just ci` and the applicable Python validation before handoff.

After these common gates pass, repeat this review on new commit SHAs. Only then
should Groningen-182 sidecars be built and controlled measurements begin.
