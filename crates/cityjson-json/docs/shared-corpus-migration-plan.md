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

Corpus-backed correctness tests now resolve fixture ids through the shared
correctness index in
`../cityjson-benchmarks/artifacts/correctness-index.json`.

`serde_cityjson/tests/data/v2_0` still contains the historical handcrafted
fixture mirror, but it is no longer the primary correctness catalog.

Benchmark inputs now come from the shared corpus repo directly. The crate no
longer owns a local synthetic benchmark mirror; it only keeps the 3D
Basisvoorziening bootstrap data under `tests/data/downloaded/`.

## Migration Steps

1. consume the shared correctness and benchmark indices directly
2. keep crate-specific regression tests local
3. keep the 3D Basisvoorziening download workflow local to this crate
4. remove the local conformance mirror once a pinned shared release replaces it
