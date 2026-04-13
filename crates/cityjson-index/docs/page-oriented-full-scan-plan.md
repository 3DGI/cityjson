# Plan: Page-oriented full-scan APIs

Date: 2026-03-30

## Goal

Add page-oriented non-spatial full-scan APIs to `cityjson-index`, then switch Tyler to
use them for:

- bbox-only extent construction when no feature-type filtering is active
- page-level parallel feature decode for cityjson-index-backed world indexing

## Constraints

- preserve deterministic feature order
- keep SQLite paging serial
- do not require `rayon` inside `cityjson-index`
- preserve Tyler correctness when `cityobject_types` filtering is active
- keep the existing item-at-a-time APIs working

## API shape

Add lightweight public scan descriptor types:

```rust
pub struct IndexedFeatureRef {
    pub feature_id: String,
    pub source_id: i64,
    pub source_path: PathBuf,
    pub offset: u64,
    pub length: u64,
    pub vertices_offset: Option<u64>,
    pub vertices_length: Option<u64>,
    pub member_ranges_json: Option<String>,
    pub bbox: BBox,
}
```

Add page-oriented `CityIndex` methods:

- `iter_all_feature_ref_pages(page_size: usize) -> Result<impl Iterator<Item = Result<Vec<IndexedFeatureRef>>> + '_>`
- `iter_all_bbox_pages(page_size: usize) -> Result<impl Iterator<Item = Result<Vec<IndexedFeatureRef>>> + '_>`
- `read_feature(&self, feature: &IndexedFeatureRef) -> Result<CityModel>`

The first two methods can share the same underlying page query in the initial
implementation. Separate names keep the intended use explicit at the Tyler call
site.

## Implementation steps

### 1. Expand the indexed row shape

Add bbox data to the internal full-scan row type returned from the ordered
`features.id` page query.

That means:

- add a dedicated row mapper for the page-oriented ref query
- join `feature_bbox` in the `lookup_all_ref_page(...)` SQL
- keep paging ordered by `f.id`

### 2. Add public page iterators

Expose a public page iterator that:

- fetches one ordered page of indexed rows
- materializes that page as `Vec<IndexedFeatureRef>`
- returns the page without decoding any `CityModel`

Use a configurable `page_size`, with validation that rejects zero.

### 3. Add direct reconstruction helper

Add `CityIndex::read_feature(&IndexedFeatureRef)` that:

- fetches metadata for the referenced source
- calls the existing backend reconstruction path

This avoids forcing callers to reconstruct low-level arguments from the page
item.

### 4. Keep the existing iterators

Leave `iter_all()`, `iter_all_with_ids()`, and `iter_all_with_metadata()` in
place.

If convenient, refactor them to reuse the same internal row mapper, but do not
change their observable order or semantics.

### 5. Add `cityjson-index` tests

Add focused tests that verify:

- page iteration returns the entire corpus exactly once
- page iteration order matches `iter_all_with_ids()`
- page sizes are honored across multi-page scans
- `read_feature()` reconstructs the same feature id as `get()`
- all supported storage layouts still work

### 6. Update Tyler

Change Tyler's cityjson-index path to:

- compute extent from indexed bbox pages when no `cityobject_types` filter is
  active
- fall back to decoded page processing when a type filter is active
- decode each grid-indexing page in parallel with `rayon`
- integrate the decoded page back into the world serially

### 7. Validate and benchmark

Run in both repos:

- `cargo fmt`
- `cargo clippy --locked --all-targets --all-features -- -D warnings`
- `cargo test --locked`

Then rerun Tyler's release benchmark on
`/home/balazs/Data/3DBAG_3dtiles_test/input` and append the new result to the
performance note.

## Expected result

This should improve Tyler in two ways:

1. remove the first full decode pass entirely for the common unfiltered case
2. recover multicore throughput during the remaining full-feature scan by
   parallelizing decode and geometry work at page granularity

## Non-goals

- replacing Tyler's two-pass architecture in this patch
- adding `rayon` as a dependency of `cityjson-index`
- changing spatial bbox query APIs
- changing the SQLite schema beyond what is needed to expose already indexed
  bbox data through the full-scan query
