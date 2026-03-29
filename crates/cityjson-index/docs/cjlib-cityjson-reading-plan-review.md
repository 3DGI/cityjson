# Review of `cjlib` Integration Plan for `cjindex` CityJSON Reads

## Findings

### 1. High: `root_state` is not backed by `cjindex`'s current read/index contract

The proposed API says `cjindex` should mostly pass fragments into `cjlib`, but
the new subset input requires a richer base payload than the repo currently
tracks.

The plan proposes:

- `root_state`
- one or more `CityObject` fragments
- the shared root `vertices` slice

That `root_state` is defined as the top-level CityJSON object without
`CityObjects` and without `vertices`, while still carrying `type`, `version`,
`transform`, `metadata`, `extensions`, `appearance`, templates, and any other
root members that apply to extracted objects.

Relevant references:

- [cjlib-cityjson-reading-plan.md](/home/balazs/Development/cjindex/docs/cjlib-cityjson-reading-plan.md#L70)
- [cjlib-cityjson-reading-plan.md](/home/balazs/Development/cjindex/docs/cjlib-cityjson-reading-plan.md#L84)
- [cjlib-cityjson-reading-plan.md](/home/balazs/Development/cjindex/docs/cjlib-cityjson-reading-plan.md#L100)

But the current `cjindex` contract and scaffold only describe:

- cached metadata
- the shared `vertices` range
- per-object byte ranges

They do not define a persisted or cached "root minus `CityObjects` and
`vertices`" payload.

Relevant references:

- [README.md](/home/balazs/Development/cjindex/README.md#L81)
- [README.md](/home/balazs/Development/cjindex/README.md#L117)
- [README.md](/home/balazs/Development/cjindex/README.md#L127)
- [lib.rs](/home/balazs/Development/cjindex/src/lib.rs#L175)

If this API shape is kept, the plan needs an explicit schema and cache change:

- either persist full root-state JSON per source
- or define read-time extraction of root state without pushing JSON surgery back
  into `cjindex`

### 2. Medium: the feature identity contract is still unresolved

`cjindex` currently claims that queries return a model with exactly one
`CityObject`, and its planned index shape is one row per `CityObject`.

Relevant references:

- [README.md](/home/balazs/Development/cjindex/README.md#L5)
- [README.md](/home/balazs/Development/cjindex/README.md#L128)

But the proposed subset API is intentionally multi-object, and the
parent/child section still treats the core semantic choice as open:

- strip dangling references outside the extracted subset
- or include transitive closure automatically

Relevant references:

- [cjlib-cityjson-reading-plan.md](/home/balazs/Development/cjindex/docs/cjlib-cityjson-reading-plan.md#L104)
- [cjlib-cityjson-reading-plan.md](/home/balazs/Development/cjindex/docs/cjlib-cityjson-reading-plan.md#L167)
- [cjlib-cityjson-reading-plan.md](/home/balazs/Development/cjindex/docs/cjlib-cityjson-reading-plan.md#L179)

This needs a hard decision before implementation:

- either `cjindex` is a one-object index and relations are filtered
- or it becomes a grouped-feature/subgraph index

Without that decision, "layout parity" is underspecified for datasets with
parent/child structure.

### 3. Medium: the plan puts too much long-term semantic localization/remap logic in `cjlib`

The plan says `cjlib` should own:

- referenced-vertex collection
- remapping
- rebuilding valid subset models

Relevant references:

- [cjlib-cityjson-reading-plan.md](/home/balazs/Development/cjindex/docs/cjlib-cityjson-reading-plan.md#L58)
- [cjlib-cityjson-reading-plan.md](/home/balazs/Development/cjindex/docs/cjlib-cityjson-reading-plan.md#L224)

That is reasonable as a first dogfood implementation, but it does not match the
current ecosystem architecture as the final destination.

The current architecture says:

- `cityjson-rs` should eventually own extraction, localization, remapping, and
  merge of self-contained submodels
- `serde_cityjson` should own staged and raw JSON boundary work
- `cjlib` should stay a thin facade

Relevant references:

- [architecture.md](/home/balazs/Development/cjlib/docs/architecture.md#L80)
- [architecture.md](/home/balazs/Development/cjlib/docs/architecture.md#L160)
- [architecture.md](/home/balazs/Development/cjlib/docs/architecture.md#L168)

The plan should say explicitly:

- phase 1 may implement subset extraction in `cjlib::json`
- repeated semantic subset/localization logic should migrate into
  `cityjson-rs` once the dogfooded behavior is proven

## Overall Assessment

The direction is sound. Compared with the Roofer integration, this is a much
better fit for `cjlib` because it is a read-boundary problem and belongs under
explicit JSON APIs.

Relevant references:

- [json-api.md](/home/balazs/Development/cjlib/docs/json-api.md#L14)
- [json-api.md](/home/balazs/Development/cjlib/docs/json-api.md#L140)

## Recommended Plan Changes

1. Replace "metadata cache" with a clearly named cached base-root or root-state
   payload for regular CityJSON sources.
2. Decide now whether `cjindex` indexes single objects or grouped feature
   subgraphs.
3. State explicitly that phase 1 may implement subset extraction in
   `cjlib::json`, but repeated semantic subset/localization logic should migrate
   into `cityjson-rs` later.
