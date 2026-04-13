# FFI and Bindings

These pages describe how `cityjson_lib` should serve non-Rust targets.

The key architectural decision is:

- one shared low-level FFI core
- separate target-specific public bindings on top

That means the low-level ownership, parse, serialize, and bulk-operation story
should be shared, while C++, Python, and wasm remain free to expose different
host-language APIs.

## Why This Lives In MkDocs

Rustdoc is useful for Rust APIs, but it is not the right home for the whole
cross-language story.
The documentation site needs one place that can cover:

- Rust usage
- low-level FFI concepts
- target-specific bindings
- shared concepts such as ownership, versioning, transforms, and error mapping

MkDocs is the right top-level tool because it is language-neutral.

## Documentation Split

The FFI section is split into:

- [Shared low-level core](shared-core.md)
  The common substrate used by all non-Rust targets.
- [FFI implementation plan](ffi-implementation-plan.md)
  The single source of truth for the current ABI state, wrapper direction,
  benchmark-driven conclusions, and the next grounded expansion slices.
- [Architecture decisions](../adr/0001-shared-c-abi-foundation.md)
  Decision records that freeze the cross-cutting ABI and header workflow
  choices, including the copied coordinate-buffer and columnar boundary
  layouts.
- [Model-authoritative JSON workflows](../adr/0006-model-authoritative-json-ffi-workflows.md)
  Decision record for append, extract, and cleanup over Rust-owned models with
  JSON roundtrips.
- [JSON write options and feature streams](../adr/0007-json-write-options-and-feature-stream-bytes.md)
  Decision record for the initial pretty/validation write options and the
  bytes-based feature-stream contract.
- [Wasm32 portability note and wasm64 path](../adr/0008-wasm32-blocker-and-wasm64-path.md)
  Historical note on the wasm32 portability issue that was resolved in
  `cityjson-benchmarks`, plus the later wasm64 evaluation path.
- [FFI performance analysis](ffi-performance-analysis.md)
  Analysis of why the current end-to-end wrapper benchmarks are much slower
  than the direct Rust baseline and which costs come from wrapper design rather
  than the raw ABI crossing.
- [Conventions](conventions.md)
  Documentation rules for keeping the shared concepts and target wrappers
  aligned.

The target-specific direction now lives inside the consolidated implementation
plan so the wrapper pages cannot drift away from the shared ABI plan.
