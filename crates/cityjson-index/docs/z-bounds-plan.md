# Z Bounds Plan

## Goal

Persist per-feature `min_z` and `max_z` in `cityjson-index` and expose them through
the page-oriented full-scan APIs used by Tyler, while keeping the current 2D
spatial query contract unchanged.

## Scope

In scope:

- `cityjson-index` schema and API changes for stored 3D feature bounds
- Tyler changes to consume indexed 3D bounds directly
- tests covering reindex, full-scan pages, and Tyler extent/grid behavior

Out of scope:

- 3D spatial query predicates
- replacing the existing 2D RTree with a 3D index
- changing public `query()` behavior

## Plan

1. Add a dedicated 3D bounds type in `cityjson-index`.
   - Keep `BBox` as the 2D query type.
   - Introduce `FeatureBounds` for persisted per-feature bounds.

2. Extend import-time bounds extraction to compute `z`.
   - Update feature-files, NDJSON, and CityJSON scan paths.
   - Ensure shared helpers compute `min_z` and `max_z` from transformed
     vertices.

3. Persist `z` in the sidecar schema.
   - Add `min_z` and `max_z` columns to `features`.
   - Add additive migration checks in `Index::open`.
   - Populate the new columns during `reindex()`.

4. Expose 3D bounds through full-scan APIs.
   - Update `IndexedFeatureRef` to carry `FeatureBounds`.
   - Update ordered page scans to select `min_z` / `max_z`.
   - Keep bbox query methods on the existing 2D type.

5. Update Tyler to consume indexed 3D bounds.
   - Replace the current `z = 0` conversion in the unfiltered `cityjson-index`
     extent path.
   - Remove the grid `z` repair step that was compensating for missing indexed
     `z`.

6. Verify correctness.
   - `cargo fmt`
   - `cargo clippy --locked --all-targets --all-features -- -D warnings`
   - `cargo test --locked`
   - run those checks in both `cityjson-index` and `tyler`

## Expected Result

Tyler keeps the fast bbox-page extent path but now gets correct world/grid
vertical bounds directly from `cityjson-index`, with no decode-only `z` repair step.
