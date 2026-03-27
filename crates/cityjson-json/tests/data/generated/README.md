# Synthetic Benchmark Dataset Specs

This directory holds specification-only benchmark fixtures for `serde_cityjson`.
It does not contain a generator or committed generated datasets.

The goal is to define two deliberately "pure" dataset shapes so it is obvious
which parts of the current deserializer architecture help and which parts hurt
when compared to `serde_json::Value`.

The machine-readable definitions live in
[`manifest.json`](/home/balazs/Development/serde_cityjson/tests/data/generated/manifest.json).

## Best Case

`best_case_geometry_stream`

This dataset is designed to favor the current `serde_cityjson` import path as
strongly as possible.

It isolates:

- very large numeric boundary payloads
- one geometry per object
- mostly `MultiSurface`
- almost no per-object normalization work outside geometry

It intentionally avoids:

- parent and child relation resolution
- dense attribute trees
- object extra properties
- materials, textures, semantics, and templates
- multiple geometries per object

Why this should be fast:

- the boundary parser in `src/de/geometry.rs` can flatten large nested arrays
  directly into `Boundary<u32>`
- geometry construction goes straight into stored geometry parts instead of a
  generic JSON DOM
- `serde_json::Value` still has to allocate the entire nested JSON tree

## Worst Case

`worst_case_object_normalization`

This dataset is designed to expose the parts of `serde_cityjson` that still
perform semantic normalization work that `serde_json::Value` does not.

It isolates:

- small-to-medium total file size
- many objects relative to the total geometry payload
- multiple geometries per object
- mostly `Solid`
- dense nested attributes
- dense parent and child relations

It intentionally minimizes:

- large contiguous numeric boundary payloads that would favor boundary
  flattening
- any geometry simplicity that would allow the backend layout to dominate

Why this should be slow:

- `serde_cityjson` still builds backend `Attributes`
- relations still need a second resolution step
- vertices still become backend coordinates
- `Solid` boundaries are deeper than `MultiSurface`
- `serde_json::Value` can stop after building a generic DOM

## How To Use The Specs

Read the manifest as the contract for a future generator or hand-built fixture.

The key comparison is not exact byte size. It is whether the dataset keeps the
runtime concentrated in:

- boundary flattening and stored geometry construction, or
- object normalization, attributes, and relation resolution

## Intended Outcome

If the current architectural understanding is correct, the likely result is:

- `best_case_geometry_stream` should beat `serde_json::Value`
- `worst_case_object_normalization` should trail `serde_json::Value`

That makes the benchmark pair useful as a regression guard for future
optimization work.
