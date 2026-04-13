# Plan: Non-spatial full-scan iterator

Date: 2026-03-30

## Problem

Tyler currently builds its world extent by calling `CityIndex::query_iter()` with an
all-covering bbox. That drives `cityjson-index` through the spatial lookup path even though the
caller is asking for a full corpus scan.

The investigation on `/home/balazs/Data/3DBAG_3dtiles_test/input` showed:

- `world_from_cityjson_index`: about `111s`
- Tyler geometry work inside that phase: about `0.4s`
- `cityjson-index` fetch before Tyler sees the model: about `111s`
- inside `cityjson-index` fetch:
  - `location_lookup`: about `93.8s`
  - `metadata_lookup`: about `0.0s`
  - `read_one`: about `17.1s`

So the main regression is not feature decoding. It is the use of the paginated bbox query for
"iterate every feature in the dataset".

## Goal

Add a non-spatial full-scan iterator to `cityjson-index` that can enumerate all indexed features
without using the RTree lookup path, `DISTINCT`, or `ORDER BY` over the bbox result set.

The iterator should support the current three read shapes:

- locations only
- `(feature_id, CityModel)`
- `(metadata, CityModel)`

## Constraints

- Keep the current spatial query API unchanged for real bbox lookups.
- Do not regress the existing `get()` and `query()` paths.
- Work for all three backends: feature-files, CityJSON, and NDJSON.
- Preserve deterministic iteration order.
- Keep the implementation incremental; this should not require an index schema rewrite for the
  first version.

## Proposed API

Add a parallel set of full-scan methods on `CityIndex`:

- `iter_all() -> Result<impl Iterator<Item = Result<CityModel>> + '_>`
- `iter_all_with_ids() -> Result<impl Iterator<Item = Result<(String, CityModel)>> + '_>`
- `iter_all_with_metadata() -> Result<impl Iterator<Item = Result<(Arc<Meta>, CityModel)>> + '_>`

If Tyler needs bbox-only access later, add that as a separate API rather than overloading this
iterator.

## Implementation approach

### 1. Add a non-spatial location iterator in `Index`

Add an `AllLocationIter` that pages directly over indexed features instead of the RTree tables.

The lookup query should page on a stable integer key, not `feature_id` text ordering. The
natural candidate is the `features.id` primary key.

The page query should return the same fields currently needed by `FeatureLocation`:

- `feature_id`
- `source_id`
- `path`
- `offset`
- `length`
- `vertices_offset`
- `vertices_length`
- `member_ranges`

That means the full-scan path can read from `features` plus `sources` directly and bypass:

- `feature_bbox`
- `bbox_map`
- `DISTINCT`
- `ORDER BY bm.feature_id`

Target SQL shape:

```sql
SELECT
    f.id,
    f.feature_id,
    s.id,
    f.path,
    f.offset,
    f.length,
    s.vertices_offset,
    s.vertices_length,
    f.member_ranges
FROM features AS f
JOIN sources AS s ON s.id = f.source_id
WHERE (?1 IS NULL OR f.id > ?1)
ORDER BY f.id
LIMIT ?2
```

`AllLocationIter` should remember the last seen numeric feature row id and request the next page
until exhaustion.

### 2. Build `CityIndex` full-scan iterators on top of it

Mirror the current `query_iter*()` structure:

- `iter_all_with_metadata()` uses `AllLocationIter`, `get_metadata()`, and `backend.read_one()`
- `iter_all()` maps away metadata
- `iter_all_with_ids()` keeps feature ids

This keeps backend reconstruction unchanged and limits the first patch to lookup mechanics.

### 3. Add focused tests

Add tests that verify:

- `iter_all()` returns every indexed feature exactly once
- `iter_all_with_ids()` order is deterministic and complete
- `iter_all_with_metadata()` returns metadata compatible with reconstruction
- all three storage layouts behave the same way

Add at least one regression test with more than one page of results so the paging logic is
actually exercised.

### 4. Add a micro-benchmark or investigation command

Extend the existing `perf-test` or investigation tooling to measure:

- `iter_all` location lookup only
- `iter_all` full decode
- existing bbox query for comparison

The point is to validate that the new iterator removes the lookup bottleneck rather than just
moving it around.

## Tyler integration plan

After the `cityjson-index` API exists, switch Tyler's full dataset passes away from:

- `query_iter(all_features_bbox())`

to:

- `iter_all()` or `iter_all_with_ids()`

This applies at least to:

- world extent construction
- grid indexing pass

That change should be done in a follow-up patch so the `cityjson-index` improvement can be measured on
its own first.

## Expected outcome

For Tyler's current workload, this should remove the dominant `location_lookup` cost from the
full scan path.

The measured `read_one` cost is already only about `75us` per feature, so the expected win is
not from decode. It is from replacing the current `~94s` paginated spatial lookup with a cheap
sequential index walk.

## Non-goals for the first patch

- changing the spatial bbox query behavior
- folding Tyler's two full passes into one
- adding bbox-only or metadata-only scan APIs
- changing index schema unless the direct table scan turns out to need it

## Follow-up opportunities

- Add a bbox-and-location full-scan iterator so Tyler can compute extent from indexed bbox data
  without decoding every feature in the first pass.
- Consider exposing raw `FeatureLocation` iteration publicly if more callers need low-level scan
  access.
- If needed, add a dedicated covering index for the full-scan query, but only after measuring the
  direct `features.id` paging path.
