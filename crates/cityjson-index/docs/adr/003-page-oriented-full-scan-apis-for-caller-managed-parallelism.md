# Page-Oriented Full-Scan APIs for Caller-Managed Parallelism

## Status

Accepted

## Date

2026-03-30

## Context

`cjindex` now has an efficient non-spatial full-scan iterator based on ordered
`features.id` paging. That removed the worst `DISTINCT` and bbox-query costs
from Tyler's full-corpus scans.

The next performance bottleneck is no longer lookup mechanics alone. Tyler
still reconstructs the full corpus twice:

1. once to compute extent in
   [/home/balazs/Development/tyler/src/parser.rs](/home/balazs/Development/tyler/src/parser.rs)
2. once to assign features to grid cells in
   [/home/balazs/Development/tyler/src/parser.rs](/home/balazs/Development/tyler/src/parser.rs)

On the current full-scan path, `cjindex` exposes one-item-at-a-time iterators:

- `iter_all()`
- `iter_all_with_ids()`
- `iter_all_with_metadata()`

Those are intentionally simple, but they are also inherently serial:

- one SQLite connection
- one ordered page fetch at a time
- one decoded `CityModel` delivered at a time

Tyler still uses `rayon` in other parts of the pipeline, but not in the
`cjindex` full-scan hot path. The benchmark on
`/home/balazs/Data/3DBAG_3dtiles_test/input` showed the current head running at
roughly one core, while the historical baseline used more than one core.

At the same time, adding a `rayon::ParallelIterator` directly to `cjindex`
would be the wrong abstraction:

- SQLite paging itself should remain serial and deterministic
- callers may want different concurrency models, not necessarily `rayon`
- a `ParallelIterator` API would make ordering and error behavior harder to
  reason about
- `cjindex` should stay focused on index lookup and reconstruction, not act as
  a scheduling framework

The relevant Tyler call sites are:

- extent construction in
  [/home/balazs/Development/tyler/src/parser.rs](/home/balazs/Development/tyler/src/parser.rs)
- grid indexing in
  [/home/balazs/Development/tyler/src/parser.rs](/home/balazs/Development/tyler/src/parser.rs)

## Decision

We will add page-oriented, non-spatial full-scan APIs to `cjindex` and keep
parallelism in the caller.

The new API family will expose materialized pages of indexed feature
descriptors, not a first-class `ParallelIterator`.

The page shape must support two Tyler needs:

1. a lightweight full-corpus bbox pass without decoding `CityModel` when no
   feature-type filtering is required
2. batched decode of full features so Tyler can use `rayon` safely after a page
   of feature descriptors has already been materialized from SQLite

The first implementation will therefore add:

- a page iterator over full feature references with stable ordered paging
- a page iterator over bbox-bearing feature references
- a direct `read_feature` helper that reconstructs a `CityModel` from a page
  item

The current item-at-a-time APIs remain in place for compatibility and simple
callers. They can be implemented on top of the new lower-level page iterators
or remain as wrappers around the same lookup logic.

## Implementation

### `cjindex`

The core `Index` paging query stays on ordered `features.id`, but the result row
shape expands to include the indexed bbox columns already stored in
`feature_bbox`.

The public page-oriented API will expose lightweight indexed references that
contain:

- feature identifier
- source identifier
- source path
- byte-range location data
- indexed bbox

That lets callers choose between:

- bbox-only processing
- full reconstruction by calling back into `cjindex`

### Tyler

Tyler will switch from one-item `iter_all*()` loops to page-based processing.

The intended usage is:

- if no `cityobject_types` filter is active, compute extent directly from
  indexed bboxes without decoding features
- otherwise decode features page-by-page and parallelize the geometry work with
  `rayon`
- for grid indexing, fetch a page of feature references from `cjindex`, decode
  them in parallel, then integrate the results serially into the world grid

This keeps correctness intact for filtered workloads while avoiding unnecessary
decode for the common unfiltered case.

## Consequences

### Positive

- `cjindex` remains storage- and lookup-focused instead of becoming tied to
  `rayon`
- callers can recover multicore decode performance without giving up ordered
  deterministic index scans
- Tyler can skip the first full decode pass when no type filtering is active
- the new API is reusable by non-Tyler callers that want batching but not
  parallelism

### Negative

- `cjindex` gains another layer of public API surface
- Tyler has to do slightly more orchestration work per page
- there are now two ways to consume full scans: item-by-item and page-based

### Neutral tradeoff

We are choosing explicit batch boundaries over a more magical iterator API.
That makes the call sites a little more verbose, but keeps concurrency, memory,
ordering, and failure behavior visible and controllable.
