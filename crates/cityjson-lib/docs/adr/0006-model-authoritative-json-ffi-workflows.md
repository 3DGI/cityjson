# ADR 0006: Model-Authoritative JSON FFI Workflows

## Status

Accepted.

## Context

The remaining FFI slices need append, extract, remap, and cleanup behavior.
There were two plausible implementation models:

- define a separate foreign import/remap ABI and let wrappers manage it
- keep `cityjson_lib` authoritative and express the workflows as explicit model
  operations over Rust-owned state

The latter better matches the existing Rust model and keeps serialization and
validation behavior consistent across C++, Python, and wasm.

## Decision

Append, extract, and cleanup are implemented as model-authoritative JSON
workflows.

- the Rust model is serialized to JSON as the interchange form
- selection, merge, or cleanup logic runs over that JSON representation
- the result is parsed back into a Rust-owned `CityModel`

Append is intentionally conservative in the first cut:

- source and target must have the same root kind
- source and target must have matching root transforms
- appearance resources and geometry templates are not merged by this slice
- appended geometry remaps vertex references by source vertex count
- extract prunes parent and child links that point outside the selected set

## Consequences

Positive:

- the shared semantics stay aligned with the Rust serializer and validator
- wrappers do not need to duplicate import or cleanup logic
- the model-authoritative path can be widened later without changing the
  wrapper contract

Tradeoffs:

- append is not a full lossless model merger yet
- advanced workflows pay a serialize/parse cost
- some future low-level remap optimizations remain deferred until a concrete
  ABI need justifies them
