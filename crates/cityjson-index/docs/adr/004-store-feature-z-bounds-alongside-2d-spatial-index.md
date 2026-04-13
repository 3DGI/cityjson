# Store Feature Z Bounds Alongside the 2D Spatial Index

## Status

Accepted

## Date

2026-03-31

## Context

`cityjson-index` currently persists only `x/y` feature bounds in its SQLite index.
The RTree in [/home/balazs/Development/cityjson-index/src/lib.rs](/home/balazs/Development/cityjson-index/src/lib.rs)
stores:

- `min_x`
- `max_x`
- `min_y`
- `max_y`

That is appropriate for the current spatial query API, because `query()` and
`query_iter()` are 2D window lookups.

However, the new full-scan API is also used as an indexed metadata path by
Tyler:

- unfiltered extent construction in
  [/home/balazs/Development/tyler/src/parser.rs](/home/balazs/Development/tyler/src/parser.rs)
- page-based full scan before feature decoding in
  [/home/balazs/Development/tyler/src/parser.rs](/home/balazs/Development/tyler/src/parser.rs)

Because `cityjson-index` exposes only 2D bounds there, Tyler currently converts them
to a 3D bbox with `z = 0` and later repairs the grid `z` range from decoded
features. That workaround exists in:

- [/home/balazs/Development/tyler/src/parser.rs](/home/balazs/Development/tyler/src/parser.rs)
- [/home/balazs/Development/tyler/src/spatial_structs.rs](/home/balazs/Development/tyler/src/spatial_structs.rs)

This is unnecessary. `cityjson-index` already derives feature bounds from decoded
vertices at import time, so missing `z` in the full-scan APIs is a persistence
and API-shape gap, not a fundamental capability gap.

## Decision

`cityjson-index` will keep its spatial query key 2D, but it will also persist
per-feature `min_z` and `max_z` as normal indexed metadata.

The design is:

- keep `cityjson_index::BBox` as the 2D spatial query type
- add a separate public `FeatureBounds` type containing full 3D feature bounds
- store `min_z` and `max_z` alongside feature metadata in the `features` table
- expose `FeatureBounds` from page-oriented full-scan APIs through
  `IndexedFeatureRef`
- do not change `query()` / `query_iter()` semantics to implicit 3D filtering

## Rationale

This gives callers accurate full-feature bounds without widening the scope of
the current query engine.

That is the right tradeoff because:

- Tyler needs accurate indexed `z`, not 3D spatial predicates
- the existing RTree path is tuned for 2D lookup and should stay stable
- adding 3D query semantics would be a larger contract change with unclear
  caller demand
- `min_z` / `max_z` fit naturally as additive columns in the existing schema

## Implementation

### `cityjson-index`

- Add `FeatureBounds { min_x, max_x, min_y, max_y, min_z, max_z }`
- Extend import-time bounds extraction to compute all three axes
- Add `features.min_z` and `features.max_z` with additive schema migration
- Keep `feature_bbox` as a 2D RTree
- Update `IndexedFeatureRef` and page iterators to expose `FeatureBounds`

### Tyler

- Replace the current `cityjson-index` fast-path `z = 0` conversion with direct use of
  indexed 3D bounds
- Remove the post-index grid `z` repair that existed only because indexed
  bounds were incomplete

## Consequences

### Positive

- Tyler gets correct indexed `z` bounds on the fast path
- `cityjson-index` keeps a clear separation between 2D query keys and full 3D feature
  metadata
- the schema change is additive and can be handled by normal sidecar upgrades

### Negative

- `cityjson-index` gains another public bounds type
- older sidecars need a reindex before they can supply indexed `z` bounds

### Neutral tradeoff

We are explicitly not turning the current spatial API into a 3D query API.
That keeps the query contract focused while still exposing the 3D metadata that
full-scan callers need.
