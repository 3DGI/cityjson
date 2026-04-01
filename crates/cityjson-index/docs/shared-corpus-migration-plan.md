# Shared Corpus Migration Plan

`cjindex` is the reusable real-data preparation path for the shared CityJSON
corpus.

## Role In The Ecosystem

This repository owns:

- the indexed storage layouts
- the dataset-first CLI
- the 3DBAG preparation pipeline used to reshape raw inputs locally

It does not own the canonical benchmark catalog or the release contract for
the shared corpus itself.

## Why This Matters

The shared corpus repo needs a reproducible real-data acquisition path for
3DBAG-derived cases. `cityjson-benchmarks` now owns the acquisition contract,
and `cjindex` remains the layout builder:

- `tests/common/data_prep.rs`
- `justfile`'s `prep-test-data` recipe
- pinned tile index source
- deterministic tile selection
- one prep pass that produces CityJSON, NDJSON, and feature-files layouts
- checksum and count recording in the prep manifest

That makes this repository the natural bootstrap source for the first shared
real-data release.

## Migration Boundary

The shared corpus should absorb the acquisition contract and the published
artifacts. `cjindex` should remain the implementation behind the layout
conversion, not the release contract.

In practical terms:

1. keep the existing `prep-test-data` workflow working locally
2. reuse the same tile selection and manifest data for the shared corpus
3. publish checksums and provenance in the corpus repo rather than in ad hoc
   notes
4. let downstream consumers pin the shared corpus release instead of rerunning
   prep every time

## Consumer Expectation

Other crates should use `cjindex` as a reusable preparation source during the
transition, not as the owner of a separate benchmark corpus.
