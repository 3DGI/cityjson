# FFI Implementation Plan

This page is the single implementation plan for `cityjson_lib`'s non-Rust surface.
It replaces the earlier split between:

- shared C ABI foundation
- shared FFI expansion
- C++ wrapper direction
- Python wrapper direction
- wasm adapter direction

Those documents overlapped too much and had started to drift from the current
code and benchmark state.

## Current State

The shared C ABI is no longer just a plan. It already exposes:

- lifecycle and error reporting
- probe, parse, and serialize entry points
- aggregate model summary
- selected metadata reads
- indexed CityObject and geometry inspection
- copied vertex, UV, and boundary extraction
- minimal model creation and bulk-capacity reservation
- vertex, template-vertex, and UV insertion
- targeted model mutation and model-authoritative workflows

The generated header is [cityjson_lib.h](/home/balazs/Development/cityjson-lib/ffi/core/include/cityjson_lib/cityjson_lib.h).

The wrapper layers already exist:

- C++ wrapper in [ffi/cpp](/home/balazs/Development/cityjson-lib/ffi/cpp)
- Python wrapper in [ffi/python](/home/balazs/Development/cityjson-lib/ffi/python)
- wasm adapter in [ffi/wasm](/home/balazs/Development/cityjson-lib/ffi/wasm)

The benchmark work also produced useful reality checks:

- native FFI performance is close to direct Rust when the bindings load release
  artifacts
- the benchmark work now exercises a real `wasm32-unknown-unknown` module in
  `cityjson-benchmarks`
- the remaining large overhead is wasm, not C++ or Python, and it now reflects
  the real JS/Wasm boundary rather than a fallback path

## What This Plan Covers

This plan covers:

- the shared C ABI as the common contract
- C++, Python, and wasm as target-specific projections of that contract
- benchmark-driven follow-up work that materially affects FFI design

This plan does not invent new public use cases or low-level exports that are
not already backed by `cityjson_lib` or `cityjson-rs`.

## Target Shape

The architecture remains:

- one shared low-level C ABI
- one generated C header
- separate public bindings for C++, Python, and wasm

That means:

- ownership, status codes, error mapping, and bulk-transfer rules stay shared
- wrapper ergonomics stay target-specific
- wasm stays narrower than C++ and Python unless real evidence forces
  expansion

## Ground Rules

- Prefer one-shot or bulk operations over chatty pointer chasing.
- Keep ownership explicit and easy to free correctly.
- Avoid exposing Rust layout or borrowing assumptions directly.
- Do not add C ABI functions for speculative future workflows.
- If the Rust facade does not expose a capability cleanly enough to describe,
  do not freeze it into the ABI yet.

## Completed Foundation

The original foundation slice is complete enough to treat as established:

- opaque model handles
- owned byte and copied buffer return types
- stable status and error categories
- panic shielding
- bytes-based document and feature parsing
- bytes-based document and feature serialization
- generated header workflow

That work is tracked by:

- [ADR 0001](../adr/0001-shared-c-abi-foundation.md)
- [ADR 0002](../adr/0002-ffi-header-workflow.md)
- [ADR 0003](../adr/0003-shared-ffi-inspection-and-coordinate-buffers.md)
- [ADR 0005](../adr/0005-columnar-geometry-boundary-abi.md)

The plan below therefore starts from the current implementation, not from an
empty ABI.

## Current ABI Coverage

The current shared ABI covers these concrete capabilities:

### Parse And Serialize

- `cj_probe_bytes`
- `cj_model_parse_document_bytes`
- `cj_model_parse_feature_bytes`
- `cj_model_parse_feature_with_base_bytes`
- `cj_model_serialize_document`
- `cj_model_serialize_feature`
- `cj_model_serialize_document_with_options`
- `cj_model_serialize_feature_with_options`
- `cj_model_parse_feature_stream_merge_bytes`
- `cj_model_serialize_feature_stream`

### Inspection And Extraction

- `cj_model_get_summary`
- `cj_model_get_metadata_title`
- `cj_model_get_metadata_identifier`
- `cj_model_get_cityobject_id`
- `cj_model_get_geometry_type`
- `cj_model_copy_geometry_boundary`
- `cj_model_copy_geometry_boundary_coordinates`
- `cj_model_copy_vertices`
- `cj_model_copy_template_vertices`
- `cj_model_copy_uv_coordinates`

### Targeted Mutation

- `cj_model_create`
- `cj_model_reserve_import`
- `cj_model_add_vertex`
- `cj_model_add_template_vertex`
- `cj_model_add_uv_coordinate`
- `cj_model_set_metadata_title`
- `cj_model_set_metadata_identifier`
- `cj_model_set_transform`
- `cj_model_clear_transform`
- `cj_model_add_cityobject`
- `cj_model_remove_cityobject`
- `cj_model_attach_geometry_to_cityobject`
- `cj_model_clear_cityobject_geometry`
- `cj_model_add_geometry_from_boundary`

### Model-Authoritative Workflows

- `cj_model_cleanup`
- `cj_model_append_model`
- `cj_model_extract_cityobjects`

This is enough to support:

- end-to-end parsing and serialization
- explicit write options and feature-stream workflows
- benchmarkable wrapper layers
- read-only bulk geometry extraction
- targeted model mutation and model-authoritative append/extract/cleanup paths

It still does not expose every possible upstream workflow, but it now covers
the shipped wrapper paths and the benchmarked wasm32 path.

## Benchmark-Driven Conclusions

The benchmark work changed the plan in a useful way.

### 1. Native FFI Is Not The Main Performance Problem

With release-mode native artifacts, the C++ and Python end-to-end roundtrip
costs are close to the Rust baseline. That means:

- the C ABI itself is not the main bottleneck
- future ABI work should optimize for completeness and correctness first
- native wrappers do not need a large redesign just to avoid the boundary

### 2. Wasm Is The Main Remaining Cost Center

The current wasm path still shows materially higher latency and memory use.
That means:

- wasm-specific copy behavior and result shaping matter
- the next performance investigation should focus on the JS/Wasm boundary
- native wrapper work should not be blocked on wasm work

### 3. The Benchmark Repo Should Become A Guardrail

The benchmark suite is now part of the design feedback loop, not just a one-off
experiment. It should keep answering:

- whether non-Rust bindings stay close to Rust on native targets
- whether roundtripped output stays valid under `cjval`
- where wasm regressions show up in time and memory

## Implemented Slices

The JSON surface and model-authoritative workflow slices are already
implemented in the shared ABI and wrappers. They stay documented here because
they define the contract shape future work should preserve.

### JSON Surface

These additions are already backed by `cityjson_lib::json` and are implemented in the
shared C ABI.

#### Slice B1: Write Options

Rust support already exists in:

- [json.rs](/home/balazs/Development/cityjson-lib/src/json.rs): `to_vec_with_options`
- [json.rs](/home/balazs/Development/cityjson-lib/src/json.rs): `to_feature_vec_with_options`

The shared C ABI exposes:

- document serialization with `pretty`
- document serialization with `validate_default_themes`
- feature serialization with the same explicit options

This is a direct expansion of existing behavior. It does not invent new write
semantics.

#### Slice B2: Feature-Stream I/O

Rust support already exists in:

- [json.rs](/home/balazs/Development/cityjson-lib/src/json.rs): `read_feature_stream`
- [json.rs](/home/balazs/Development/cityjson-lib/src/json.rs): `write_feature_stream`
- [json.rs](/home/balazs/Development/cityjson-lib/src/json.rs): `write_feature_stream_refs`
- [json.rs](/home/balazs/Development/cityjson-lib/src/json.rs): `merge_feature_stream_slice`

The ABI stays conservative:

- bytes-in stream merge is in scope
- bytes-out feature-stream writing is in scope
- callback-based streaming should only land if the ownership and reentrancy
  rules are fully specified

The first cut should prefer bytes-based APIs over callback-heavy ones.

### Model-Authoritative Operations

These additions are already backed by `cityjson_lib::ops` and are implemented in the
shared C ABI.

Rust support already exists in:

- [ops.rs](/home/balazs/Development/cityjson-lib/src/ops.rs): `cleanup`
- [ops.rs](/home/balazs/Development/cityjson-lib/src/ops.rs): `extract`
- [ops.rs](/home/balazs/Development/cityjson-lib/src/ops.rs): `append`
- [ops.rs](/home/balazs/Development/cityjson-lib/src/ops.rs): `merge`

The shared C ABI exposes these as explicit model-authoritative workflows:

- cleanup of a model into a new normalized model
- extract of a selected submodel by CityObject identifiers
- append of one model into another with the current conservative rules
- merge of multiple models where the Rust implementation already defines the
  behavior

These operations preserve the same constraints the Rust layer already has. In
particular:

- append remains conservative about root-kind compatibility and transforms
- extract operates on explicit CityObject identifier selection
- cleanup remains a serialization-and-parse normalization pass

The ABI should not pretend these are low-level mutations. They are higher-level
operations executed by Rust-owned semantics.

## Open Work, In Order

The remaining work splits into infrastructure and any future ABI expansion.

### Track A: Keep The Benchmarks Honest

This is not an ABI expansion track, but it protects all of the others.

1. Add CI coverage in `cityjson-benchmarks` for a small release-mode slice.
2. Add a dedicated low-level C ABI microbenchmark to separate:
   - raw boundary cost
   - wrapper cost
   - parse and serialize cost
3. Keep `cjval` validation in the benchmark pipeline for produced outputs.

### Track D: Complete Read Coverage Before Broad Write Coverage

The next read-side additions should only cover data the Rust layer already
models clearly and that wrappers can use in bulk.

Priority order:

1. any remaining summary or count-style inspection that already exists in Rust
2. additional copied bulk buffers only when they map to stable upstream
   semantics
3. any future selection-style inputs only when a new Rust op needs them in bulk

This track should not invent rich borrowed views or chatty per-field getters.

### Track E: Hold Off On Invented Builder APIs

The current C ABI has the first building blocks for authoring:

- create model
- reserve capacities
- add vertices
- add template vertices
- add UV coordinates

The current C ABI also includes targeted mutation and model-authoritative
helpers:

- metadata setters
- transform setters and clearers
- CityObject add/remove helpers
- geometry attach and clear helpers
- cleanup, append, and extract workflows

That still does not justify freezing a broad foreign authoring API for:

- a fully general editable CityObject graph
- richer geometry construction primitives
- appearance editing
- deeper metadata editing
- transform editing beyond the current explicit setters

Those may be worth adding later, but only after the Rust facade exposes clear,
stable authoring semantics that are strong enough to document. Until then, the
plan is to stop short rather than standardize a weak ABI.

## Binding-Specific Direction

The target wrappers still have different jobs.

### C++

The C++ wrapper should:

- stay RAII-based
- stay container-friendly
- expose the new JSON options and model-authoritative operations first
- avoid inventing a separate semantic model from the shared ABI

### Python

The Python wrapper should:

- stay object-oriented at the public surface
- keep bytes-oriented fast paths where payload size matters
- expose `cleanup`, `extract`, `append`, and feature-stream workflows through
  Pythonic helpers over the shared ABI
- avoid falling back to ad hoc document-shaped dict surgery as the main path

### Wasm

The wasm adapter should:

- stay narrow and task-oriented
- prefer one-shot operations and bulk result buffers
- avoid exposing deep editable handle graphs unless a real browser case proves
  it is necessary
- stay tied to the real wasm32 benchmark path and any future wasm64
  evaluation in [ADR 0008](../adr/0008-wasm32-blocker-and-wasm64-path.md)

## Worktree Split

If this work is parallelized again, the clean split is:

### Stream A: Shared ABI

Own:

- `ffi/core/src/**`
- `ffi/core/tests/**`
- `ffi/core/include/cityjson_lib/cityjson_lib.h`
- `ffi/core/cbindgen.toml`

Deliver:

- write options
- feature-stream bytes support
- model-authoritative ops exports
- ABI tests and header regeneration

### Stream B: C++ Wrapper

Own:

- `ffi/cpp/**`

Deliver:

- C++ wrappers for the new ABI slices
- smoke tests
- wrapper ergonomics over the shared semantics

### Stream C: Python Wrapper

Own:

- `ffi/python/**`

Deliver:

- Python bindings for the new ABI slices
- tests
- bulk and bytes-oriented wrapper improvements where needed

### Stream D: Wasm And Benchmarking

Own:

- `ffi/wasm/**`
- benchmark-related follow-up in the benchmark repo

Deliver:

- keep the real wasm32 path benchmarked
- wasm-focused performance investigation
- ABI microbenchmark and regression thresholds in the benchmark repo

### Stream E: Integration And Docs

Own:

- `docs/ffi/**`
- `docs/adr/**`
- `mkdocs.yml`
- workflow and helper scripts when needed

Deliver:

- integration sequencing
- documentation updates
- merge and release criteria

## Merge Order

The safe merge order is:

1. shared ABI slice
2. regenerated header and tests
3. C++ and Python wrapper adoption
4. wasm adaptation where the slice is relevant
5. benchmark and documentation updates

No wrapper merge should land before the ABI slice it depends on.

## Release Criteria For Each Slice

Each ABI expansion slice is only done when:

- the shared header is regenerated
- Rust ABI tests cover the new behavior
- C++ and Python smoke tests pass if they adopt the slice
- produced CityJSON output remains valid under `cjval` where applicable
- benchmark coverage is updated when the slice can affect performance

## Explicit Non-Goals

This plan does not currently commit to:

- a callback-heavy streaming ABI without stronger rules
- a large foreign authoring API not already anchored in Rust semantics
- speculative browser-first functions that bypass the shared model
- wrapper-specific semantics becoming the definition of the shared ABI

## Immediate Next Steps

In concrete terms, the next recommended sequence is:

1. add benchmark CI and a raw C ABI microbenchmark
2. add write-options exports
3. add conservative bytes-based feature-stream exports
4. add `cleanup`, `extract`, `append`, and `merge` exports
5. let C++ and Python adopt those slices
6. return to wasm profiling once the shared slices are stable

That sequence is narrow enough to execute safely and broad enough to make the
shared C ABI materially more complete without inventing unsupported surface
area.
