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

Benchmark inputs now come from the shared corpus repo directly. The crate no
longer owns a local synthetic benchmark mirror; it only keeps the 3D
Basisvoorziening bootstrap data under `tests/data/downloaded/`.

## Migration Steps

1. keep using the existing handcrafted fixtures for correctness tests
2. align the local fixture ids with the shared corpus ids
3. consume the shared corpus benchmark index directly
4. keep the 3D Basisvoorziening download workflow local to this crate
