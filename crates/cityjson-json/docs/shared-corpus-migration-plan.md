# Shared Corpus Migration Plan

`serde_cityjson` should treat benchmark fixtures and correctness cases as
shared corpus inputs, not as crate-owned data definitions.

## What Stays Local

The crate still owns:

- parser and serializer implementation code
- crate-specific regression tests
- temporary bootstrap copies of fixtures when the shared corpus is not yet
  available in a pinned release

## What Moves To The Shared Corpus

The following should come from `cityjson-benchmarks` over time:

- correctness fixture ids
- invalid fixture ids
- synthetic benchmark profiles
- real-data provenance metadata
- pinned benchmark release artifacts

## Current Bridge

`serde_cityjson/tests/data/v2_0` already contains the handcrafted fixture set
used for correctness testing. Those files should be treated as the current
local mirror of the shared conformance corpus until the shared repo publishes
the same ids.

`tests/data/generated/` is the same story for benchmarks: it is a local
bootstrap for the current synthetic cases, but the canonical profile catalog
belongs in the shared corpus repo.

## Migration Steps

1. keep using the existing handcrafted fixtures for correctness tests
2. align the local fixture ids with the shared corpus ids
3. move benchmark inputs to the shared corpus release index once it exists
4. treat the 3DBAG download workflow as historical bootstrap code, not as the
   long-term source of truth

