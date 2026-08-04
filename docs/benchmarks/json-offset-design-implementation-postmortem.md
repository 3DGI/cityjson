# JSON-offset vertex-store implementation postmortem

## Purpose and scope

This report explains why the compact JSON-offset vertex-store experiment at
candidate commit `1f33165e5481455074ff007f7dff4b8d948e4287` should not be treated
as a successful implementation of a production-quality JSON-offset design.
The implementation established useful correctness and storage properties, but
it did not establish that its read path was architecturally appropriate or
efficient.

This is a technical postmortem, not an attribution of intent. It distinguishes
three different sources of trouble:

1. mistakes in the JSON-offset store itself;
2. inefficiencies in the shared experimental reconstruction path; and
3. defects in the benchmark comparison used to evaluate the result.

That distinction matters. The candidate did integrate with the experimental
`CityIndex::read_packages_with_vertex_store` interface and reused common
package reconstruction. It did **not** completely bypass `cityjson-index`.
However, below that interface it introduced a parallel JSON-access subsystem
with its own scanning, validation, file-opening, buffering, grouping, and
lookup behavior. Much of that work duplicated responsibilities that should
have been designed as part of the index read path.

## Executive assessment

The implementing work optimized first for compact persistent representation
and local correctness. It did not design the complete hot path from package
request to reconstructed model as one system.

Consequently, the implementation is superficially successful:

- it stores approximately four bytes of offset data per source vertex;
- it validates its sidecar thoroughly;
- it reconstructs correct models in the tested cases;
- it reports zero retained decoded vertex bytes; and
- it coalesces many adjacent source reads effectively.

But those properties were achieved with an expensive composition of repeated
validation, duplicate JSON scans, tree-based intermediate collections,
per-batch metadata queries, repeated file opens, and allocation-heavy
reconstruction. The implementation proves feasibility and correctness of the
stored representation. It does not prove that the resulting design is a good
read architecture.

The reported 2.05x--2.28x slowdown cannot be assigned entirely to JSON
offsets because the benchmark performed materially different work from the
baseline. Even so, direct code inspection and profiling identify genuine
design defects that would remain after benchmark parity is repaired.

## What the implementing agent got wrong

### 1. It treated the storage representation as the design

The implementation centered the design on 16,384-vertex chunks containing
32-bit relative offsets. That is a reasonable encoding, but an encoding is
only one part of a vertex-access design.

The agent did not begin with an end-to-end cost model covering:

- how packages discover their required vertex indices;
- how requirements are grouped across packages and sources;
- how sidecar metadata is validated and cached;
- how source files are opened and shared;
- how byte ranges are scheduled and coalesced;
- how JSON coordinates are decoded;
- how decoded coordinates reach geometry assembly; and
- which allocations and collections remain live at each stage.

Because these boundaries were not designed together, each layer selected a
locally convenient representation. Their composition is expensive even
though each component appears reasonable in isolation.

### 2. It built a second low-level JSON access path

The candidate directly opens source files, seeks to ranges, allocates temporary
buffers, scans JSON value boundaries with `json_value_end`, and then invokes
Serde on the same bytes. This is effectively a candidate-local JSON range
reader.

That subsystem is disconnected from the established `cityjson-index` resource
and reconstruction lifecycle. It does not benefit from a dataset-level source
handle cache, a shared range reader, persistent source metadata, or a unified
JSON boundary abstraction. It also makes the candidate responsible for file
I/O policy that belongs above a storage encoding.

The problem is not merely duplicated code. A private I/O subsystem prevents
the index from optimizing requests globally. The index cannot readily reuse
open handles, coordinate reads across batches, amortize metadata lookup, or
select a decoder based on the complete workload.

The better boundary would have separated:

- offset resolution: map `(source_id, vertex_index)` to source byte ranges;
- source access: execute grouped ranges through shared index-owned resources;
  and
- coordinate decoding: decode exactly one bounded coordinate value once.

### 3. It validates the entire offset payload on every read-only open

`validate_for_read` walks every source and every offset chunk, reads every
offset BLOB, and verifies all entries are strictly monotonic and in bounds.
This is strong corruption detection, but it is placed on the wrong lifecycle
boundary for a performance-sensitive reader.

The full sidecar contains 111,713,328 vertex offsets and 446,880,960 bytes of
candidate payload. The coordinator validates that complete state once, and
each isolated worker validates it again when opening its own connection. A
24-worker run therefore checks approximately 2.79 billion offsets and reads
about 11.17 GB of offset payload before the materialization timer begins.

This is an example of correctness logic becoming pathological through
placement rather than through intent. Full validation belongs at construction,
explicit validation, schema migration, or a once-per-sidecar trust boundary.
A normal read-only open should use cheap structural and provenance checks, or
reuse a validated immutable state.

The agent should have specified validation tiers before implementation:

- constant- or small-bounded-cost checks on every open;
- complete validation on explicit request and immediately after construction;
- corruption detection local to every range actually read; and
- a durable generation, checksum, or validated-state mechanism where the
  threat model requires it.

### 4. It scans every selected coordinate twice

For each requested coordinate, the candidate first calls `json_value_end` to
find the value boundary. It then calls `serde_json::from_slice`, which scans
and parses those bytes again.

The sidecar already stores a start and an end offset for every vertex. If that
end offset is trusted after validation, parsing should use the exact bounded
slice. If local defensive validation is required, the decoder should both
parse and verify consumption in one pass. Maintaining a custom boundary
scanner followed by a general parser pays twice for the same lexical work.

This duplicated scan is a direct consequence of layering validation and
decoding independently without measuring their combined cost.

### 5. It used ordered trees as bulk transport structures

The hot path repeatedly converts linear, already sortable data into
`BTreeSet` and `BTreeMap` structures:

- a `BTreeSet` collects referenced vertex indices per package;
- another `BTreeSet` globally deduplicates `VertexRequirement`s;
- the candidate groups resolved vertices in a `BTreeMap`;
- returned coordinates are inserted into a
  `BTreeMap<VertexRequirement, [i64; 3]>`; and
- model assembly performs repeated tree lookups for individual coordinates.

The candidate also sorts per-source resolved vertices even though the input
requirements are already required to be globally sorted and deduplicated.

Ordered trees are useful when incremental ordered mutation is essential. Here
the workload is primarily bulk collection, sort, merge, sequential traversal,
and indexed assembly. Sorted `Vec`s, compact per-source spans, merge joins, or
batch-local dense lookup tables would reduce allocations, pointer chasing,
comparisons, and memory fragmentation.

The one-tile Cachegrind run attributed 607 million candidate instructions to
vertex-set insertion, compared with 138 million in the baseline's comparable
insertion symbol. It also showed substantial collection and remapping work.
This is evidence that intermediate data structures were not treated as part
of the performance design.

### 6. It repeated batch-invariant resource work per batch

Each batch repeats work that should be owned by a longer-lived dataset or
worker context:

- query the paths of requested sources;
- construct source-id maps;
- open source files;
- allocate range buffers;
- load chunk metadata; and
- rebuild intermediate grouping structures.

Some of this work is necessary once, and some varies with a batch. The
implementation did not separate the two. With batches of 2,048 packages, the
same source metadata and files are rediscovered and reopened many times.

An efficient design would introduce a worker-scoped read context containing
validated source metadata, reusable SQLite statements, source handles or an
explicit handle-cache policy, scratch buffers, and reusable requirement/range
vectors. Ownership and memory bounds should be explicit rather than recreated
implicitly by every call to `load`.

### 7. It optimized coalescing too late in the pipeline

The candidate does successfully coalesce consecutive coordinate ranges. The
full run reads 3.10 GB of source JSON and touches 15,476 offset chunks, which
is positive evidence for the basic offset representation.

However, coalescing happens only after requirements have passed through
multiple tree collections, SQLite lookups, offset-buffer reads, per-source
maps, and sorting. Good source-I/O coalescing cannot compensate for excessive
CPU and allocation overhead before and after the read.

The one-tile syscall audit reinforces this point: only about 0.143 seconds of
a 0.787-second candidate materialization run was spent in syscalls. Raw
storage access was not the dominant cost. The agent should have profiled the
complete CPU and allocation path before concluding that source-range
coalescing was sufficient.

### 8. It did not challenge the shared reconstruction contract

The shared experimental interface requires sorted, deduplicated requirements
and returns a `BTreeMap` of coordinates. The common reconstruction path also
collects vertex indices into trees more than once and reconstructs batches of
up to 2,048 models.

These costs are not solely errors in the JSON-offset candidate. They are
shared harness and API design problems. Nevertheless, a high-complexity
implementer should not treat a supplied interface as automatically suitable
for the intended workload. The agent should have produced an early complexity
review showing that the contract forced allocation-heavy transport and
point-lookups, then requested or proposed a candidate-neutral interface
revision before optimizing inside the wrong boundary.

A more appropriate contract might return coordinates in requirement order,
stream per-source spans into assembly, or accept a callback/sink that consumes
resolved coordinates without constructing a tree map.

### 9. It verified correctness without verifying operational suitability

The remediation campaign established valuable invariants:

- transactional construction;
- complete source coverage;
- read-only failure behavior;
- monotonic and bounded offsets;
- stable package identity and model digests;
- malformed-storage regression coverage; and
- zero retained decoded bytes.

Those tests answer whether the implementation is safe and produces correct
results. They do not answer whether it is a viable production read path.

Missing acceptance gates included:

- maximum work performed by read-only open;
- number of full payload scans per process and per worker;
- source opens and metadata queries per batch;
- allocations per reconstructed package or vertex;
- coordinate bytes scanned per returned coordinate;
- asymptotic behavior of intermediate collections;
- peak live packages under equal batching; and
- instruction attribution against a workload-equivalent baseline.

The implementing agent treated green functional tests and zero retained
decoded vertices as sufficient evidence of design success. For a performance
architecture task, they are necessary but not sufficient.

### 10. It accepted an invalid benchmark comparison

The candidate's timed materialization path performs canonical serialization,
reparses each model as `serde_json::Value`, sorts object keys, and computes
SHA-256. The baseline reconstructs models and drops them without equivalent
digest work.

In the one-tile Cachegrind comparison:

- the candidate executed 11.73 billion instructions;
- the baseline executed 4.36 billion instructions; and
- SHA-256 compression alone consumed 2.50 billion candidate instructions.

The candidate also retains batches of up to 2,048 reconstructed models while
the baseline processes one model at a time. Direct peak-memory comparison is
therefore invalid as well.

The implementing and benchmarking work should have rejected these results as
non-comparable before publishing a strategy-level slowdown. A performance
campaign must establish workload parity before interpreting totals. Stable
candidate digests demonstrate candidate consistency; they do not compensate
for absent baseline digest work.

## Root causes in the implementation process

The defects share several process causes.

### Local optimization replaced architecture

The agent focused on making each offset chunk compact, bounded, and valid. It
did not maintain an end-to-end model of data movement, lifetime, and repeated
work. This led to a compact sidecar surrounded by expensive orchestration.

### Interfaces were treated as fixed rather than reviewed

The `VertexStore` trait and shared reconstruction path were accepted without
testing whether their collection and ownership semantics supported the
performance goal. Inefficiency became distributed across layers and therefore
harder to attribute.

### Correctness evidence dominated performance evidence

The review process was effective at finding malformed-state and lifecycle
bugs. It did not impose similarly concrete budgets for opens, scans, queries,
allocations, or instructions.

### Measurement was postponed until after convergence

By the time full profiling exposed the costs, the storage schema, validation
behavior, shared interface, reconstruction path, and benchmark harness had
all solidified. Early one-tile instruction and allocation profiles would have
revealed the double parsing, tree transport, and hashing mismatch much sooner.

### Benchmark parity was assumed from common output

Both paths produced reconstructed models, but they did not perform the same
work or retain the same number of models. Equivalence was judged by the final
artifact rather than by the operations inside the timed and measured regions.

## What should have been done instead

Before implementation, the agent should have written a short executable design
specification containing:

1. the exact hot-path stages and ownership boundaries;
2. a complexity and byte-movement budget for each stage;
3. which work is per dataset, process, worker, batch, package, and vertex;
4. the validation threat model and validation lifecycle;
5. the data structure chosen for every boundary and why;
6. how existing `cityjson-index` source access and reconstruction are reused or
   deliberately extended;
7. a benchmark contract with identical work and residency; and
8. stop/go thresholds from a representative one-tile prototype.

The implementation should then have proceeded vertically rather than by
finishing the storage layer first:

- resolve offsets for a small real package batch;
- read through a shared source-access abstraction;
- decode each coordinate exactly once;
- feed coordinates directly into common assembly using linear structures;
- measure instructions, allocations, opens, and bytes;
- validate parity against the baseline; and only then
- complete construction, corruption validation, concurrency, and full-corpus
  coverage.

The likely production architecture would keep the compact offset sidecar but
change most of the read plumbing:

- cheap open-time checks plus explicit/full validation at controlled
  boundaries;
- worker-scoped prepared statements, source metadata, handles, and scratch
  storage;
- sorted vector-based requirement and result transport;
- one-pass bounded coordinate parsing;
- merge-based assembly instead of repeated tree lookup;
- cross-batch or worker-level source-range scheduling where memory limits
  permit; and
- an equal-work correctness sink shared by baseline and candidates.

## General instructions for agents implementing complex systems work

The following guidelines can be given directly to future agents.

### 1. Start with the existing architecture, not with the requested mechanism

Trace the complete current read and write paths before writing code. Identify
the existing abstractions for I/O, caching, validation, batching, ownership,
and reconstruction. Reuse them where they fit. If they do not fit, explain the
mismatch and propose the smallest coherent extension instead of building a
parallel subsystem inside a feature implementation.

### 2. Write an end-to-end cost model before implementation

For every stage, state its time complexity, allocations, bytes read or copied,
data structure, lifetime, and repetition scope: dataset, process, worker,
batch, package, or element. Explicitly flag any full-data scan and multiply it
by the expected worker and repetition counts.

### 3. Separate representation, lifecycle, and execution design

Do not equate a compact schema or correct encoding with a successful design.
Review independently:

- persistent representation;
- construction and migration;
- validation and trust boundaries;
- read execution and resource reuse;
- concurrency and memory residency; and
- observability and benchmarking.

### 4. Treat supplied interfaces as hypotheses

Before implementing behind an interface, test whether its ownership and data
shapes fit the workload. Look for forced maps, duplicate sorting, unnecessary
materialization, point lookups, and inability to reuse resources. Escalate an
interface problem early rather than hiding it with local workarounds.

### 5. Prefer linear bulk processing on hot paths

When inputs can be sorted and processed in batches, prefer sorted vectors,
merge joins, spans, stable indices, and reusable buffers. Use trees and hash
maps only when their dynamic lookup behavior is actually required. Document
why every non-linear hot-path collection is necessary.

### 6. Decode and validate data once

Avoid consecutive scanners or parsers over the same bytes. Design APIs that
parse bounded values and verify full consumption in one pass. Put complete
validation at an explicit lifecycle boundary and keep normal reads locally
defensive without rescanning the entire persistent state.

### 7. Make resource lifetime explicit

List which connections, prepared statements, metadata, file handles, caches,
and scratch buffers are reused per process and per worker. Do not reopen or
reallocate batch-invariant resources accidentally. State the bound on every
cache and reusable buffer.

### 8. Establish benchmark parity before comparing results

Baseline and candidate must perform identical correctness work, use the same
batch boundaries, retain equivalent live data, validate at equivalent
boundaries, and measure the same region. Audit the actual call graph, not just
the command names and output fields. Reject comparison results when parity is
not proven.

### 9. Profile a representative vertical slice early

Before completing the full feature, run a small real workload through the
entire proposed path. Collect instruction attribution, allocations, syscalls,
bytes read, file opens, and peak live objects. Use the result to revise the
architecture while interfaces and schemas are still inexpensive to change.

### 10. Define non-functional acceptance tests

Alongside correctness tests, encode budgets or counters for expensive
behaviors. Examples include no full payload scan on normal open, at most one
source open per worker/source, no duplicate coordinate scan, bounded
allocations per batch, and identical benchmark sink work. A complex task is
not complete until both functional and operational invariants pass.

### 11. Keep attribution honest

When profiling crosses shared and candidate-specific code, report costs by
layer. Do not assign all elapsed time to the new mechanism. Conversely, do not
use shared-path overhead to excuse defects that are clearly inside the
candidate. State what the evidence proves, what it suggests, and what remains
unisolated.

### 12. Stop and redesign when evidence contradicts the architecture

Passing tests is not a reason to preserve a poor boundary. If early profiles
show repeated full scans, dominant allocation overhead, or materially more
work than the baseline, pause feature completion. Present the evidence,
identify the responsible layer, revise the design, and rerun the small
comparison before proceeding to a full campaign.

## Final conclusion

The JSON-offset experiment should be retained as evidence that compact offsets
can reconstruct correct packages without retaining decoded vertices. It should
not be treated as evidence that the implemented read architecture is suitable,
nor should the existing slowdown be treated as a clean measurement of JSON
offsets themselves.

The principal failure was not choosing offsets. It was implementing offsets as
a locally correct storage plugin without redesigning and measuring the complete
index-owned read path around them. Future work should preserve the useful
encoding evidence, replace the duplicated and allocation-heavy execution path,
and require equal-work profiling before making a strategy decision.
