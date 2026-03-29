# Plan for Reducing `cjindex` JSON Assembly While Keeping `feature + base`

## Goal

Keep `feature + base` as the logical API shape for regular `CityJSON` reads,
but stop requiring `cjindex` to assemble that feature payload through ad hoc
`serde_json::Value` manipulation.

The desired end state is:

- `cjindex` still extracts one object and localizes its vertices
- `cjindex` still hands off a feature-sized payload plus a base document
- the JSON-boundary crate owns materializing the actual `CityJSONFeature`
- `cjindex` no longer rebuilds nested JSON trees just to cross that boundary

## Why This Change Is Worth Doing

The current dogfood implementation proved that the `feature + base` concept is
workable, but it also exposed avoidable friction at the call site.

The awkward part is not that we use `CityJSONFeature`.

That part is aligned with:

- the wider CityJSON ecosystem
- the CityJSONSeq model
- the current `cjlib::json::from_feature_slice_with_base` boundary

The awkward part is that `cjindex` currently has to:

- parse an extracted `CityObject` fragment into `serde_json::Value`
- walk nested boundary arrays as generic JSON
- mutate those nested arrays in place
- rebuild a `CityJSONFeature` as another `Value`
- serialize that payload back to bytes
- hand those bytes to `cjlib` and `serde_cityjson` for parsing again

That is acceptable for a first dogfood pass, but it is a poor long-term
call-site boundary.

## What We Want To Preserve

This follow-up should preserve the parts that already make sense.

### 1. Preserve the logical `feature + base` shape

The broader ecosystem already uses:

- a base `CityJSON` document
- one or more `CityJSONFeature` payloads

That is a sensible conceptual model unless it becomes clearly suboptimal.

### 2. Preserve one semantic model

`cjlib` and `cityjson-rs` already treat:

- full documents
- grouped subsets
- feature-sized packages

as the same semantic in-memory model.

This work should not introduce:

- a separate package model
- a separate partial semantic type
- a second semantic representation beside `OwnedCityModel`

### 3. Preserve `cjindex` ownership of extraction semantics

For the current one-object read contract, `cjindex` should still decide:

- which object is being returned
- which shared vertices are referenced
- how sparse indices are localized
- how parent/child links are filtered for the one-object subset

That is selection and extraction policy, not JSON-boundary assembly.

## Core Design Direction

The best next step is:

- keep `feature + base` as the public conceptual boundary
- add a helper that accepts structured feature parts
- let the JSON-boundary crate materialize the actual `CityJSONFeature`

The main effect should be that `cjindex` passes typed pieces rather than a
fully assembled `serde_json::Value` tree.

## Recommended Layering

### Phase 1 target: `serde_cityjson`

The first helper should live in `serde_cityjson`, with a thin public wrapper in
`cjlib::json`.

Why:

- the pain is mostly JSON-boundary assembly
- `feature + base` is a format-boundary concern
- `cjlib` should stay thin and ergonomic
- `cityjson-rs` should only absorb logic once we can name a repeated semantic
  operation clearly

Recommended split:

- `cjindex`
  - extract one object
  - localize referenced vertices
  - decide relation filtering policy
- `serde_cityjson`
  - materialize a valid `CityJSONFeature` from structured inputs
  - combine that feature with the base document root
- `cjlib::json`
  - expose the public helper
- `cityjson-rs`
  - later own generalized extraction/localization/merge semantics if they prove
    to be semantic operations rather than JSON-boundary concerns

## Proposed API Direction

Start narrow.

Do not design a fully generic subset system yet.

The first helper should target the current one-object or small-feature package
use case with structured inputs.

Illustrative shape:

```rust
pub struct FeatureParts<'a> {
    pub id: &'a str,
    pub cityobjects: &'a [FeatureObject<'a>],
    pub vertices: &'a [[i64; 3]],
}

pub struct FeatureObject<'a> {
    pub id: &'a str,
    pub object: &'a serde_json::value::RawValue,
}

pub fn from_feature_parts_with_base(
    parts: FeatureParts<'_>,
    base_document_bytes: &[u8],
) -> Result<CityModel>;
```

This shape is illustrative, not prescriptive.

The important properties are:

- `vertices` stay typed
- object payloads can stay borrowed raw JSON
- the caller does not need to assemble nested `Value` trees
- the helper still preserves the logical `feature + base` model

## Concrete Work Breakdown

### 1. `serde_cityjson`

Add a staged input helper for materializing feature-sized packages from
structured pieces.

Requirements:

- accept typed localized vertices
- accept borrowed raw `CityObject` JSON payloads
- write the `CityJSONFeature` envelope internally
- reuse the current base-root merge behavior
- return the same semantic model as the existing feature-string path

Possible internal responsibilities:

- write `type = "CityJSONFeature"`
- write `id`
- write `CityObjects`
- write localized `vertices`
- combine with the base root the same way
  `from_feature_str_owned_with_base` already does

### 2. `cjlib`

Add a thin forwarding wrapper in `cjlib::json`.

Requirements:

- preserve `cjlib` as the user-facing surface
- keep the API explicit and JSON-module-scoped
- avoid surfacing `serde_cityjson` directly unless needed

### 3. `cjindex`

Refactor the regular `CityJSON` read path to use the new helper.

Keep in `cjindex`:

- object extraction
- referenced-vertex collection
- local vertex localization
- boundary remap
- one-object parent/child filtering policy

Remove from `cjindex`:

- full feature assembly as nested `serde_json::Value`
- general-purpose feature-envelope construction
- as much dynamic JSON mutation as possible

### 4. `cityjson-rs`

Do nothing in the first follow-up unless the refactor exposes a clearly named
semantic operation.

Only move work here if repeated needs emerge such as:

- extract a self-contained submodel
- localize shared resources into a package
- merge a package back into a larger model
- assemble larger models from smaller self-contained packages

Those are semantic-model operations.
They should not be introduced just because the current call site is messy.

## Explicit Non-Goals

This follow-up should not:

- replace `feature + base` with a non-ecosystem-specific model
- introduce a separate semantic package type
- redesign all subset semantics up front
- force `cityjson-rs` to own JSON envelope assembly
- require `cjindex` to construct full semantic models before crossing the JSON
  boundary

## Suggested Implementation Order

1. Add a structured feature-parts helper in `serde_cityjson`.
2. Add a thin wrapper in `cjlib::json`.
3. Refactor `cjindex` regular `CityJSON` reads to use that helper.
4. Compare the resulting `cjindex` code with the current implementation and
   record what JSON-boundary awkwardness remains.
5. Only then decide whether any repeated logic belongs in `cityjson-rs`.

## Testing Plan

### `serde_cityjson`

- verify the new helper produces the same semantic result as the current
  `from_feature_str_owned_with_base` path
- verify base-root members are preserved
- verify one-object and small multi-object packages materialize correctly

### `cjlib`

- add wrapper parity tests against the underlying `serde_cityjson` helper
- verify the helper remains an explicit JSON-boundary API, not a second
  semantic model

### `cjindex`

- keep existing regular-`CityJSON` unit and integration coverage green
- verify sparse vertex localization still behaves the same
- verify one-object relation filtering still behaves the same
- verify the refactor removes or significantly reduces ad hoc `Value`
  assembly at the call site

## What Success Looks Like

This plan is complete when all of the following are true:

- `feature + base` remains the logical public API shape
- `cjindex` no longer assembles full feature payloads through broad
  `serde_json::Value` surgery
- a helper exists in `serde_cityjson` and is surfaced ergonomically through
  `cjlib::json`
- the resulting semantic `CityModel` is unchanged from the current working
  backend behavior
- any decision to move work into `cityjson-rs` is based on proven repeated
  semantic extraction/localization needs, not just discomfort with JSON
  assembly
