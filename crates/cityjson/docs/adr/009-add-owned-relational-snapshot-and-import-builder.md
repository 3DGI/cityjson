# 009. Add Owned Relational Snapshot And Import Builder

Date: 2026-04-16

## Status

Accepted

## Context

`cityjson-rs` already had useful low-level pieces:

- contiguous vertex and resource pools
- flat boundary storage
- raw pool accessors and dense remap helpers

That was enough for bespoke serializers, but not enough for the proposed vNext shape.

The main mismatches were:

- the public low-level API was still handle- and object-oriented
- repeated strings remained ordinary owned strings in the main semantic types
- there was no stable relational contract for codecs and FFI layers
- there was no low-level import path that rebuilt an `OwnedCityModel` from ordered relational data

## Decision

Add a new owned-model-first relational API in `cityjson-rs`:

- `cityjson::relational::RelationalAccess` exposes `OwnedCityModel::relational()`
- `ModelRelationalView` provides dense numeric ids plus explicit owned relational tables
- repeated strings are exported through a symbol table and referenced through `SymbolId`
- geometry, topology, semantic/material/texture assignments, attributes, metadata, defaults, and extensions are all represented in relational form
- `RelationalModelBuilder` rebuilds an `OwnedCityModel` directly from those relational tables
- `cityjson::query::summary()` provides cheap scalar counts over the same contract

The new contract is intentionally owned-model specific. It does not try to preserve the older generic storage abstraction as the primary interop surface.

## Consequences

Positive:

- downstream Arrow/FFI work now has a single explicit low-level contract to target
- hot-path joins move to dense numeric ids instead of strings or typed handles
- repeated strings are dictionary-ready at the relational boundary
- import/export roundtrips no longer need to reconstruct through ad hoc semantic helper paths

Negative:

- `relational()` currently materializes an owned snapshot instead of being a pure zero-copy borrow over every internal structure
- the semantic core still stores ordinary owned strings internally, so symbol interning is guaranteed at the relational boundary before it is guaranteed in the in-memory semantic structs themselves
- the legacy raw API still exists in parallel for now

## Follow-up

- move symbol-backed storage deeper into the owned semantic core so the relational snapshot can become cheaper
- trim or retire older raw/handle-heavy interop entry points once downstream crates adopt `cityjson::relational`
