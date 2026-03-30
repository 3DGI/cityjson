# cityarrow

`cityarrow` is the Arrow and Parquet transport layer for the `cityjson-rs`
data model.

It should not define a second semantic model. The semantic unit remains
`cityjson::v2_0::CityModel`; Arrow IPC, Parquet, and stream framing are explicit
format boundaries built around that model.

## Status

On 2026-03-30, `cargo check` against `cityjson-rs v0.4.1` fails. The crate
still contains significant code written against an older `cityjson-rs` API,
including:

- `AttributePool`-based attribute conversion paths that no longer exist
- older `CityModel<u32, ResourceId32, SS>` generics
- `QuantizedCoordinate`-based vertex conversion
- imports through private or removed modules

That means the current crate is best understood as a stale prototype plus a set
of useful schemas and file-format experiments, not as a working transport
crate. The preferred recovery path is a rewrite, not a compatibility migration.

## What Exists Today

The current codebase already has the right high-level decomposition:

- `CityModelArrowParts` splits a `CityModel` into component batches
- Arrow IPC directory writing exists
- framed Arrow IPC stream writing exists
- Parquet directory writing exists
- component schemas exist for metadata, transform, vertices, cityobjects,
  geometries, and semantics

The current implementation shape is still useful. The problem is that several
critical conversion paths are incomplete or stale.

## Current Gap Summary

| Area | Current State | Gap |
| --- | --- | --- |
| Build status | `cargo check` fails | Rebase the whole crate to `cityjson-rs v0.4.1` |
| Attributes and extra | Conversion code assumes removed global attribute pools and Arrow dense unions | Redesign around current inline `AttributeValue` and Parquet-safe projection |
| Vertices and transform | Code assumes older quantized-coordinate model | Rebase to current `RealWorldCoordinate`-based API and preserve transform explicitly |
| Geometries | Batch schema exists, but end-to-end conversion is only partially wired | Reconnect geometry export/import to the current `Boundary`, semantic, material, and texture views |
| Semantics | Schema and partial conversion exist | Rebase to current handles and attribute model |
| Materials, textures, template geometry, UV vertices | Present in `CityModelArrowParts`, mostly not implemented end-to-end | Define canonical batch schemas and round-trip behavior |
| Reading | Arrow IPC directory reader exists | Add Parquet reader and align both readers with the same transport model |
| Tests | Many tests are disabled or reflect removed APIs | Rebuild the suite around current `cityjson-rs` behavior |

## Design Direction

The redesign should follow four rules.

1. `CityModel` stays the semantic unit.
2. Arrow and Parquet stay explicit transport boundaries.
3. `CityModelArrowParts` is an internal transport decomposition, not a new data
   model family.
4. Parquet support must be designed around Parquet's real constraints, not
   around Arrow-only features such as unions.

The implementation stance is intentionally aggressive:

- prefer a clean rewrite over compatibility shims
- delete stale assumptions instead of adapting around them
- accept breaking changes if they produce a cleaner transport boundary
- only preserve code paths that fit the target architecture

Two design consequences follow from the current review.

### 1. Attributes must be projected, not pool-backed

`cityjson-rs v0.4.1` stores attributes inline again through
`AttributeValue::{Null, Bool, Unsigned, Integer, Float, String, Vec, Map, Geometry}`.

The old `cityarrow` approach tried to model attributes as:

- Arrow `Map<Utf8, Union>`
- then Parquet

That is the wrong seam. Parquet still cannot represent Arrow unions natively,
and the in-memory `AttributePool` path used by the current crate no longer
matches `cityjson-rs`.

The replacement direction is:

- discover attribute keys per owner scope
- resolve a Parquet-safe physical type per key
- write typed sibling columns
- encode nested `Vec` and `Map` values as JSON strings
- encode geometry-valued attributes as a handle or optional WKB shadow column

### 2. Geometry export should normalize at the boundary

Current `cityjson-rs` still stores boundaries in an offset-based flattened
structure:

- `vertices`
- `rings`
- `surfaces`
- `shells`
- `solids`

That is appropriate for in-memory semantics, but Arrow and Parquet benefit from
a more serialization-oriented layout. `cityarrow` should therefore normalize the
boundary representation at export time instead of forcing `cityjson-rs` to adopt
a transport-native shape internally.

The intended direction is:

- keep the current `cityjson-rs` boundary as the source of truth
- derive a GeoArrow-friendlier count-based layout at the Arrow/Parquet boundary
- keep CityJSON-specific shell and solid hierarchy as sidecar data

## Fit In The Wider Stack

`cityarrow` should align with the neighboring crate architecture documented in
`cjlib`.

- `cityjson-rs` owns semantic correctness and invariants
- `cityarrow` owns the Arrow transport boundary
- a future `cityparquet` can either be a sibling crate or a narrower facade on
  top of the Parquet parts of `cityarrow`
- `cjlib::arrow` and `cjlib::parquet` should expose explicit format modules that
  read and write `CityModel`, not transport-native fragments

## Rewrite Target

The next implementation should be done as one architectural rewrite, not as a
phased migration.

That rewrite should:

- rebase the entire crate to `cityjson-rs v0.4.1` in one pass
- replace pool- and union-based attribute assumptions with projected,
  Parquet-safe columns
- normalize geometry topology at the transport boundary instead of preserving
  stale wire layouts for compatibility
- define the full component set only once, in the target shape
- rebuild the tests around the new architecture instead of reviving obsolete
  ones
- mirror the manifest-driven fixture setup used by `serde_cityjson` for
  roundtrip acceptance instead of relying only on unit fixtures
- treat final JSON emission through `serde_cityjson` plus `cjval` validation of
  the real-world `3DBAG` and `3D Basisvoorziening` datasets as the end gate

## Repository Map

- `src/lib.rs`
  - top-level `CityModelArrowParts` and conversion entry points
- `src/writer.rs`
  - Arrow IPC, stream, JSON debug, and Parquet writers
- `src/reader.rs`
  - Arrow IPC directory reader
- `src/conversion/*.rs`
  - component-level schema and conversion code
- `docs/design.md`
  - current design document and redesign target
- `docs/package-schema.md`
  - first concrete draft of the canonical package schema

## Design Doc

The current design document is [docs/design.md](docs/design.md).
The current schema draft is [docs/package-schema.md](docs/package-schema.md).
