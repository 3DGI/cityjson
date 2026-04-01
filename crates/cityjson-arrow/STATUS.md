# Status Report

## Current Status

`cityarrow` and `cityparquet` are both implemented and currently share one
canonical transport contract.

- `cityarrow` owns the canonical `CityModelArrowParts` decomposition, schema
  and manifest types, conversion entry points, and Arrow IPC package I/O
- `cityparquet` is a sibling workspace crate that reuses the same canonical
  transport contract and exposes Parquet package I/O
- the current package contract is `cityarrow.package.v1alpha1`
- the repository `HEAD` is `a3dcf15` on `master`, aligned with `origin/master`
- the working tree is clean except for an untracked `.dockerignore`

## What Works Today

The implemented transport surface is strong on semantic exactness.

- `convert::to_parts` and `convert::from_parts` are implemented
- Arrow IPC package write/read is implemented in `cityarrow`
- Parquet package write/read is implemented in `cityparquet`
- both encodings roundtrip through the same canonical table layout and manifest
- the canonical package covers the current `OwnedCityModel` surface used by
  `cityjson-rs`, including templates, geometry instances, semantics,
  materials, textures, metadata, and projected attributes

## Verification Snapshot

The current tree passes the full Rust test suite with `cargo test`.

Coverage currently includes:

1. in-memory `to_parts` and `from_parts` roundtrips for synthetic fixtures
2. exact canonical table equality tests for Arrow IPC and Parquet package
   roundtrips
3. fixture tests that verify package I/O preserves canonical parts and still
   reconstructs `cjval`-valid CityJSON
4. shared corpus conformance tests that roundtrip the same CityJSON 2.0
   correctness fixtures through both encodings
5. schema-lock tests for canonical schemas and manifest snapshots

## Strict Readiness Review

The outputs are trustworthy for correctness-oriented internal use, but not yet
for a permanent public interchange guarantee.

### Trust Level

- semantic exactness: high within the currently implemented surface
- schema discipline: high for the current alpha contract because the package
  layout is schema-locked in tests
- backward-compatibility guarantee: low because the on-disk version remains
  `v1alpha1`
- scale readiness: moderate to low for large datasets because conversion and
  package reads are still eager and fully in-memory
- ecosystem readiness: moderate for controlled producers and consumers, low for
  broad external adoption without a stabilization pass

### Strengths

- one canonical transport shape is shared across Arrow IPC and Parquet
- roundtrip testing covers both synthetic fixtures and a shared correctness
  corpus
- the package contract avoids Arrow union and map types, which reduces
  cross-format ambiguity
- reconstruction is driven by explicit ids and ordinals rather than row-order
  accidents
- the crate surface is now clearer after separating Parquet package I/O into
  `cityparquet`

### Risks And Gaps

- the package format is explicitly alpha, so consumers should expect deliberate
  format changes before stabilization
- eager read and conversion paths increase peak memory pressure on large models
- the current test story is strong on exact roundtrips but does not yet prove
  long-term compatibility across future schema versions
- the manifest `views` field is treated as optional non-canonical metadata and
  is not part of the exact transport guarantee
- template geometry pools cannot themselves contain geometry instances
- texture mappings are only supported on surface-backed geometry types
- robustness against malformed or adversarial third-party packages has less
  evidence here than correctness against known-good corpus inputs

### Release Recommendation

- safe for internal development, validation, and controlled pipelines where you
  own both producer and consumer
- acceptable for early adopters who can tolerate alpha contract changes
- not yet ready to present as a stable archival or third-party interchange
  format promise

### Before Calling It Stable

The highest-value remaining work is:

1. freeze a non-alpha package version and define compatibility policy
2. add versioned compatibility tests across historical package snapshots
3. implement lower-memory package I/O for large-model operation
4. expand negative testing around malformed packages and invariant violations
5. document any remaining unsupported CityJSON edge cases as explicit contract
   limits
