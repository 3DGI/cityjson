# Realistic Full-Corpus Read Batches, Indexed Byte-Range Reads, and Correct `CityJSON` Feature Semantics

## Status

Accepted

## Date

2026-03-30

## Context

The earlier benchmark harness had already moved away from a single repeated
`get`, but two material problems remained:

1. the bbox workload still reused a small fixed window set too aggressively
2. `query_iter()` needed to remain a streaming path rather than a buffered
   iterator over all bbox matches
3. regular `CityJSON` was still not being compared on the same semantic unit as
   feature-files and `NDJSON`

The original regular `CityJSON` scan path indexed every `CityObject`, including
children. Feature-files and `NDJSON`, by contrast, index feature packages. That
meant the suite was partly comparing:

- one-object `CityJSON` reads
- full-package feature-files / `NDJSON` reads

That mismatch made the old "`CityJSON get` is the fastest backend" conclusion
look stronger than it really was.

At the same time, the hot-path I/O work from the previous round was still
correct and valuable:

- `NDJSON` had switched from whole-file rereads to exact feature-range reads
- `CityJSON` had switched from whole-tile rereads to indexed object and
  vertices ranges
- `CityJSON` still reused shared vertices through a per-source cache

So the next step was not to undo byte-range reads. It was to make the workload
more corpus-representative and make the `CityJSON` feature semantics correct.

## Decision

We made three linked decisions.

### 1. Keep indexed byte-range reads

The earlier positioned-read work remains in place:

- `read_exact_range(...)` and `read_exact_range_from_file(...)`
- `NdjsonBackend::read_one`
- `FeatureFilesBackend::read_one`
- `CityJsonBackend::read_one`

This remains the correct indexed I/O baseline.

### 2. Make the benchmark read workload sweep the whole corpus over time

The steady-state benchmark contract is now:

- `get`: 1,000 deterministic lookups per measured iteration
- `query`: 10 bbox queries per measured iteration
- `query_iter`: the same 10 bbox queries, fully drained through the streaming path

But the workload construction is now stricter:

- the 1,000-ID `get` batch includes at least one selected feature from every
  tile in the 191-tile corpus before filling the remainder from a deterministic
  pseudo-random pool
- bbox workload construction now builds one deterministic tile-local bbox per
  tile
- each measured `query` / `query_iter` iteration executes the next 10 bboxes
  from that full-corpus ring

This keeps per-iteration work bounded while ensuring the suite no longer
repeats the same narrow spatial slice forever.

### 3. Make regular `CityJSON` index feature packages, not every child object

Regular `CityJSON` now indexes feature packages rooted at top-level objects:

- root objects are discovered from the `CityObjects` graph
- child objects are grouped with their root feature
- each indexed `CityJSON` feature stores the member object ranges needed to
  reconstruct the full package

At read time, regular `CityJSON` now rebuilds a full feature package:

- all indexed member object fragments are read
- only relationships within the feature package are retained
- shared vertices are localized across the full member set
- the resulting `CityModel` matches the feature-package unit exposed by the
  other backends

## Implementation

### Shared workload builder

The deterministic workload builder now lives in
[/home/balazs/Development/cityjson-index/src/realistic_workload.rs](/home/balazs/Development/cityjson-index/src/realistic_workload.rs)
and is shared by:

- the Criterion harness
- the investigation binary

This avoids drift between the reported analysis and the actual benchmark
contract.

### Benchmark harness

The current Criterion harness lives in
[/home/balazs/Development/cityjson-index/benches/support.rs](/home/balazs/Development/cityjson-index/benches/support.rs).

Key points:

- `get` uses the full 1,000-ID deterministic corpus-spread batch every
  iteration
- `query` and `query_iter` rotate through a deterministic 191-bbox ring in
  10-bbox batches
- setup still validates the workload, but bbox validation is sampled so startup
  cost stays bounded

### `CityJSON` feature-package indexing

The hot-path and index-structure changes live in
[/home/balazs/Development/cityjson-index/src/lib.rs](/home/balazs/Development/cityjson-index/src/lib.rs).

Key points:

- `features.member_ranges` was added to the SQLite index schema
- regular `CityJSON` scan now indexes root feature packages instead of every
  child object
- `CityJSON read_one` now reads every member fragment for the selected feature
  package
- bbox iteration now streams ordered SQLite results in pages instead of
  materializing the full hit set up front
- member relationships are filtered to local package references only
- shared vertices are still cached per source file
- tests now cover:
  - exact-range reads
  - root-feature grouping with children
  - local relationship filtering during feature reconstruction

## Results

The benchmark and investigation results are summarized here so the ADR is
self-contained.

The key release-mode benchmark numbers are:

- `feature_files_get`: `87.759 ms` to `88.336 ms` per 1,000-lookups batch
- `cityjson_get`: `92.092 ms` to `92.581 ms` per 1,000-lookups batch
- `ndjson_get`: `87.349 ms` to `87.774 ms` per 1,000-lookups batch
- `feature_files_query`: `751.62 ms` to `755.50 ms` per 10-bbox batch
- `cityjson_query`: `796.16 ms` to `799.79 ms` per 10-bbox batch
- `ndjson_query`: `734.70 ms` to `737.69 ms` per 10-bbox batch
- `feature_files_reindex`: `9.3796 s` to `9.4389 s`
- `cityjson_reindex`: `17.862 s` to `18.045 s`
- `ndjson_reindex`: `7.9518 s` to `7.9843 s`

The most important updated conclusion is simple:

- `CityJSON` is no longer broken on reads
- `CityJSON` is also no longer the surprising `get` winner once full
  feature-package semantics are restored
- `NDJSON` is currently the fastest backend overall

## Benchmark Interpretation Notes

`reindex` remains separate from hot reads.

- `reindex` is an upfront or occasional rebuild cost
- `get`, `query`, and `query_iter` measure steady-state reads against an
  already-built index

These are still hot steady-state numbers:

- the same deterministic IDs and bbox ring are reused across Criterion samples
- OS page cache still helps repeated file access
- `CityJSON` still benefits from its shared-vertices application cache after
  first touch per source file

So a result like "`~91 us` per `CityJSON get`" does not mean "scan the whole
corpus in 91 microseconds". It means "retrieve one already indexed full feature
package from a warmed working set in about 91 microseconds".

The investigation binary showed that:

1. direct lookup time is small for all three backends
2. explicit byte-read time is also small relative to full batch cost
3. the remaining backend gap is dominated by decode / reconstruction work

That explains why `CityJSON` can still read slightly fewer bytes than `NDJSON`
but still lose overall on the corrected full-feature workload.

## Consequences

### Positive

- The benchmark now has explicit corpus spread instead of an accidental hot
  tile bias.
- The backend comparison is semantically aligned again.
- Indexed byte-range reads remain in place and still provide the right I/O
  baseline.
- `CityJSON` is now a viable read backend rather than a catastrophic outlier.

### Negative

- The benchmark fixture and index schema are both more complex.
- Historical `change:` percentages on read benchmarks are no longer cleanly
  comparable across benchmark generations.
- `CityJSON` still carries the highest full-feature reconstruction cost and the
  highest rebuild cost.

### Neutral tradeoff

We traded simpler benchmark mechanics for defensible semantics and broader
corpus coverage. That makes the suite slower to reason about, but far more
useful for backend decisions.
