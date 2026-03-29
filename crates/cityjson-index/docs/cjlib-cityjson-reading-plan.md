# Dogfood-First Plan for `cjindex` CityJSON Reads Through `cjlib`

## Goal

Implement the regular `CityJSON` read path in `cjindex` by using the current
`cjlib` JSON API and the current `cityjson-rs` model as they already exist.

The point of this work is not to design an ideal long-term subset API up
front. The point is to dogfood the current surface honestly and see where it
breaks down.

That means phase 1 should answer these questions with code, not speculation:

- is `cjlib::json::from_feature_slice_with_base` sufficient for this read path?
- where is the real friction: `cjindex`, `cjlib`, `serde_cityjson`, or
  `cityjson-rs`?
- which missing abstraction is actually needed, if any?

## Current Contract to Preserve

`cjindex` already describes a narrow regular-`CityJSON` read contract:

- the index stores one row per `CityObject`
- a lookup returns one `cjlib::CityModel`
- that model contains exactly one `CityObject`

For regular `CityJSON`, the documented read path is:

- read the selected `CityObject`
- read the shared root `vertices`
- collect only the referenced vertex indices
- build local vertices
- remap boundaries
- assemble the one-object result

That contract is already specific enough for the first implementation.
We do not need to generalize it before it exists.

## Existing `cjlib` API to Dogfood

`cjlib` already exposes the relevant public API:

```rust
pub fn from_feature_slice_with_base(
    feature_bytes: &[u8],
    base_document_bytes: &[u8],
) -> Result<CityModel>;
```

That is the first path `cjindex` should use.

For regular `CityJSON`, the simplest correct base input is the full source
document bytes for the file that contains the selected object. If that means
reading or caching the full document in phase 1, that is acceptable.

The first pass should optimize for clarity and signal, not for minimal bytes
read.

## Recommended First Implementation

Keep phase 1 deliberately narrow:

1. Implement `CityJsonBackend::read_one` only for the current one-object read
   contract.
2. In `cjindex`, read whatever source bytes are needed to build one valid
   `CityJSONFeature`:
   - the selected `CityObject`
   - the shared root `vertices`
   - the base document bytes
3. In `cjindex`, perform the currently documented extraction work:
   - collect referenced vertex indices
   - build a dense local vertex array
   - remap geometry boundary indices
4. Materialize a minimal one-object `CityJSONFeature` JSON payload in
   `cjindex`.
5. Pass that payload and the base document bytes into
   `cjlib::json::from_feature_slice_with_base`.
6. Return the resulting `cjlib::CityModel`.

This keeps the dogfood loop honest:

- `cjindex` proves whether the current boundary is workable
- `cjlib` proves whether its existing feature+base loader is the right
  normalization entry point
- follow-up API changes are justified by concrete pain

## Semantics for the First Pass

The first pass should stay aligned with the existing one-object contract.

### Parent/child references

Do not introduce grouped extraction or transitive closure.

For a one-object result:

- keep parent/child links that still point inside the returned subset
- drop links that point to objects not included in the one-object feature

### Appearance and templates

Do not design special subset APIs for these yet.

Phase 1 should rely on the base document passed to
`from_feature_slice_with_base` for root-level context such as:

- metadata
- transform
- appearance
- extensions
- geometry templates

If this turns out to be insufficient or awkward, that is valuable dogfood
feedback.

### Root members

Do not invent a `root_state` payload in phase 1.

If later work shows that passing the full base document is too expensive or too
awkward, that may justify a narrower base representation. That is a follow-up
optimization, not a starting assumption.

## What We Are Explicitly Not Designing Yet

Phase 1 should not introduce any of the following:

- a new `cjlib::json::from_cityjson_subset` API
- a public `root_state` input type
- multi-object extraction as the default path
- grouped-subgraph semantics in `cjindex`
- `cityjson-rs` extraction/localization primitives before we can name the
  missing semantic operation clearly

Those may become reasonable later, but only after the current path has been
implemented and evaluated.

## What We Want to Learn from Dogfooding

The output of this work is not just a working backend. It is also a clearer
understanding of where the real abstraction boundary belongs.

Use the implementation to answer:

- Is building the one-object `CityJSONFeature` in `cjindex` straightforward, or
  is it too much JSON surgery?
- Does `from_feature_slice_with_base` compose cleanly with regular-`CityJSON`
  reads, or does it expose a missing helper in `cjlib` or `serde_cityjson`?
- Are the awkward parts fundamentally JSON-boundary problems, or are they
  really missing semantic operations that belong in `cityjson-rs`?

Expected interpretations:

- if the pain is mostly JSON assembly, improve `serde_cityjson` first
- if the pain is mostly semantic submodel extraction or localization, add
  targeted `cityjson-rs` operations
- if the pain is mostly call-site ergonomics, add a thin `cjlib::json` helper

## Concrete Work Breakdown

### 1. `cjindex`

- implement `CityJsonBackend::read_one`
- read the selected object and shared vertices from the indexed source
- read or cache the full base document bytes for the source file
- build a one-object `CityJSONFeature` JSON payload with local vertices
- call `cjlib::json::from_feature_slice_with_base`
- add backend tests and cross-layout parity tests

### 2. `cjlib`

- no new API required for phase 1
- only change `cjlib` if the implementation uncovers a concrete ergonomic gap

### 3. `serde_cityjson`

- no refactor required up front
- only change it if phase 1 reveals a clearly reusable JSON-boundary helper

### 4. `cityjson-rs`

- no changes in phase 1
- revisit only if we can name a missing semantic operation precisely

## Testing Plan

### `cjindex`

- test `CityJsonBackend::read_one` on a regular `CityJSON` fixture
- verify the returned model contains exactly one `CityObject`
- verify sparse vertex indices are localized and remapped correctly
- verify metadata and transform are preserved through the feature+base path
- verify dangling parent/child links are filtered

### Cross-layout parity

For the same feature ID, compare the semantic result returned from:

- feature-files layout
- NDJSON layout
- regular `CityJSON` layout

The resulting `CityModel` should match semantically across layouts.

## Suggested Implementation Order

1. Implement `cjindex::CityJsonBackend::read_one` with the existing
   `from_feature_slice_with_base` API.
2. Add backend tests and cross-layout parity tests.
3. Record the concrete friction points from that implementation.
4. Only then decide whether to change `cjlib`, `serde_cityjson`, or
   `cityjson-rs`.

## Non-Goals for the First Pass

- zero-copy subset reads
- a generic subset API
- multi-object extraction
- minimal appearance/template pool subsetting
- preemptive refactors to move logic across crates before the pain is proven
- optimizing away full-base-document reads before correctness exists

## Acceptance Criteria

This plan is complete when all of the following are true:

- `cjindex` regular `CityJSON` reads work end to end
- the implementation uses the existing `cjlib` feature+base read path
- the returned model preserves the current one-object contract
- parity tests pass across the supported storage layouts
- any proposed follow-up API changes are based on concrete limitations found
  during this dogfood pass
