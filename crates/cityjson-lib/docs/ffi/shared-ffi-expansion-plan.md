# Shared FFI Expansion And Binding Plan

This page plans the next implementation phase after the shared C ABI
foundation.

The scope here is:

- higher-level shared FFI coverage beyond probe, parse, and serialize
- C++ and Python bindings over the shared ABI
- a narrow wasm adapter over the same shared semantics

The plan is organized so parallel agents can work in separate worktrees and
merge into trunk without breaking correctness.

## Goal

At the end of this phase, `cjlib` should have:

- a wider shared ABI that covers model inspection, model creation, bulk data
  access, targeted mutation, and write options
- a first usable C++ wrapper over that ABI
- a first usable Python binding over that ABI
- a narrow wasm adapter that proves the shared core can serve browser-facing
  use cases without redefining the semantics

The target is not feature completeness in one jump.
The target is to widen the shared core in slices that each land safely and
improve at least one real binding.

## Execution Strategy

The work should be split into a small number of streams with disjoint file
ownership where possible.

The critical path is:

1. widen the shared ABI in compatible slices
2. land tests and documentation for each slice
3. build C++, Python, and wasm adapters on top of those stable slices

Agents should work in separate worktrees and own specific file sets.
Only the integration agent should merge the streams back into trunk.

## Worktree Streams

### Stream A: Shared ABI Surface

Owner:

- `ffi/core/src/**`
- `ffi/core/tests/**`
- `ffi/core/include/cjlib/cjlib.h`
- `ffi/core/cbindgen.toml`

Responsibilities:

- add new opaque handles and `#[repr(C)]` types when required
- add higher-level exported functions
- preserve ABI naming, ownership, and error rules
- keep the generated C header current
- add Rust-level ABI behavior tests

This stream is the dependency source for the language wrappers.

### Stream B: C++ Wrapper

Owner:

- `ffi/cpp/**`

Responsibilities:

- build RAII wrappers over the generated C header
- hide status-code plumbing behind exceptions or result objects if desired
- expose STL-friendly read and write helpers
- add C++ build and smoke-test coverage

This stream must not redefine the low-level ABI.

### Stream C: Python Binding

Owner:

- `ffi/python/**`

Responsibilities:

- bind the shared ABI into Python classes and views
- map low-level errors into Python exceptions
- expose iterable and object-oriented access patterns
- add Python packaging and smoke-test coverage

This stream should prefer the shared ABI instead of a Rust-specific side path.

### Stream D: Wasm Adapter

Owner:

- `ffi/wasm/**`

Responsibilities:

- expose a narrow JS-facing API over shared semantics
- keep handles and deep mutation internal unless needed
- return summaries, typed arrays, or serialized bytes as appropriate
- add wasm-oriented smoke tests or examples

This stream should stay intentionally smaller than C++ and Python.

### Stream E: Integration And Docs

Owner:

- `docs/ffi/**`
- `docs/adr/**`
- `mkdocs.yml`
- `justfile`
- CI or workflow files if added later

Responsibilities:

- document each ABI slice and wrapper decision
- add or update ADRs for cross-cutting choices
- keep developer workflows aligned with the new build and test matrix
- merge streams in dependency order and resolve conflicts

This stream may also carry small cross-stream glue changes if no other stream
can own them cleanly.

## Shared ABI Implementation Slices

The shared ABI should not be widened as one large patch.
It should land in narrow, testable slices that wrappers can adopt
incrementally.

### Slice 1: Read-only Model Inspection

Add shared exports for:

- model metadata and version queries
- cityobject counts and identifiers
- geometry counts and type queries
- appearance and template presence or counts

Design rules:

- prefer bulk or indexed queries over chatty pointer chasing
- return stable value types, spans, or owned buffers with explicit ownership
- avoid binding-specific convenience semantics

Parallelization:

- Stream A can implement this immediately
- Streams B, C, and D can prepare wrappers and tests against the planned
  signatures in parallel, then switch to the landed header

### Slice 2: Bulk Geometry And Boundary Access

Add shared exports for:

- vertex buffer access
- template vertex access
- boundary extraction
- optional flat mesh-oriented extraction for wasm-facing use cases

Design rules:

- optimize for low crossing count
- make copy and ownership policy explicit
- avoid exposing Rust layout assumptions directly

Parallelization:

- Stream A owns the ABI and Rust tests
- Stream D can start as soon as the extraction shape is frozen
- Streams B and C can add higher-level views over the same buffers

### Slice 3: Model Creation And Targeted Mutation

Add shared exports for:

- create empty model handles
- reserve bulk capacity where useful
- add vertices and geometries
- create and update cityobjects
- set or update selected metadata and appearance resources

Design rules:

- keep mutation explicit and validation-aware
- prefer batched insertion APIs over many single-element crossings
- do not let wrapper ergonomics distort the shared semantic model

Parallelization:

- Stream A implements the shared calls
- Streams B and C can build idiomatic builders and mutating views on top
- Stream D should usually stay out of this slice unless a browser use case
  proves it is necessary

### Slice 4: Import, Extract, Remap, And Cleanup Operations

Add shared exports for:

- import or append a submodel into another model
- extract a submodel by selected object identifiers
- run bulk remap operations
- trigger cleanup or normalization-sensitive workflows

Design rules:

- keep these operations explicit rather than folding them into ordinary edits
- preserve Rust ownership of semantic invariants
- return enough diagnostics for wrapper-level reporting

Parallelization:

- Stream A implements the shared operations and tests
- Stream C can build `cjio`-style workflows over them
- Stream B can add higher-level import and extraction helpers after the ABI
  lands

### Slice 5: Write Options And Stream Support

Add shared exports for:

- document write options
- feature write options
- transform and quantization control
- feature-stream read and write callbacks where justified

Design rules:

- keep format-specific policy explicit
- make callback ownership and reentrancy rules precise before exposing streams
- do not mix JSON-specific options into generic wrapper APIs without clear
  naming

Parallelization:

- Stream A owns the ABI and callback rules
- Streams B and C can expose file and stream convenience helpers
- Stream D can selectively expose only bytes-based write options

## Binding Milestones

The wrappers should not wait for the entire shared ABI to be complete.
Each binding should target a minimal usable milestone, then widen as new slices
land.

### C++ Milestone 1

Deliver:

- RAII `Model` wrapper over `cj_model_t`
- parse, probe, serialize helpers
- basic inspection APIs from Slice 1
- initial build integration through CMake

Acceptance:

- a C++ smoke test can parse a document, inspect object counts, and serialize
  it again

### C++ Milestone 2

Deliver:

- builder or mutating APIs over Slice 3
- bulk geometry access from Slice 2
- import and extract helpers from Slice 4

Acceptance:

- a C++ test can build or edit a model and emit a valid document

### Python Milestone 1

Deliver:

- `CityModel` class over `cj_model_t`
- parse, probe, serialize bindings
- basic object and geometry inspection from Slice 1
- packaging and importable module structure

Acceptance:

- a Python smoke test can load a document, inspect objects, and roundtrip it

### Python Milestone 2

Deliver:

- Python views and iterators over bulk geometry data
- mutation and extraction helpers from Slices 3 and 4
- Python exception hierarchy mapped from shared errors

Acceptance:

- a Python test can perform a `cjio`-style edit or extraction workflow without
  document-shaped dict surgery

### Wasm Milestone 1

Deliver:

- probe and parse entry points
- summary queries from Slice 1
- targeted geometry extraction from Slice 2
- serialize or roundtrip support where useful

Acceptance:

- a wasm smoke test or browser example can summarize a document and extract
  geometry buffers

## Merge Order

To keep trunk correct, merge in this order:

1. documentation and ADR scaffolding for the next slices
2. Shared ABI Slice 1 with tests and regenerated header
3. wrapper updates that only depend on Slice 1
4. Shared ABI Slice 2 with tests and regenerated header
5. wasm extraction work and any wrapper geometry views that depend on Slice 2
6. Shared ABI Slice 3 with tests and regenerated header
7. C++ and Python mutation layers that depend on Slice 3
8. Shared ABI Slice 4 with tests and regenerated header
9. wrapper workflows that depend on import, extract, remap, and cleanup
10. Shared ABI Slice 5 with tests and regenerated header
11. wrapper stream and write-option helpers that depend on Slice 5

No wrapper merge should land before the ABI slice it depends on is merged.

## Integration Rules

Every stream should follow these rules before merge:

- rebase or merge trunk into the worktree before final validation
- regenerate `ffi/core/include/cjlib/cjlib.h` whenever ABI changes land
- update docs when public behavior changes
- never hand-edit the generated header

The integration agent should:

- merge one stream at a time
- run formatting, checks, lint, and tests after each merge group
- resolve conflicts in favor of the shared ABI contract rather than wrapper
  convenience
- reject wrapper-local API drift that does not match the landed header

## Validation Matrix

The minimum validation set for each merge group is:

- `just fmt`
- `just check`
- `just lint`
- `just test`

When wrapper-specific tooling lands, extend validation with:

- C++ configure and smoke tests
- Python package build and smoke tests
- wasm build and smoke tests
- docs build for plan and ADR integrity

## Suggested Parallel-Agent Breakdown

This is the recommended first split:

1. Agent 1: Shared ABI Slice 1 and Slice 2
   Owns `ffi/core/src/**`, `ffi/core/tests/**`, and header generation.
2. Agent 2: C++ Milestone 1 wrapper
   Owns `ffi/cpp/**` and tracks the header generated by trunk.
3. Agent 3: Python Milestone 1 binding
   Owns `ffi/python/**` and tracks the header generated by trunk.
4. Agent 4: Wasm Milestone 1 adapter
   Owns `ffi/wasm/**`.
5. Agent 5: docs, ADRs, and workflow integration
   Owns `docs/**`, `mkdocs.yml`, and `justfile`.

After Slice 2 lands, start a second wave:

1. Agent 1: Shared ABI Slice 3
2. Agent 2: C++ Milestone 2
3. Agent 3: Python Milestone 2
4. Agent 4: Shared ABI Slice 4 and Slice 5, unless that is too large and
   needs another split
5. Agent 5: integration and docs refresh

This keeps the highest-conflict files concentrated in one stream while still
letting the wrapper work proceed in parallel.

## ADR Candidates

The following implementation choices are likely large enough to deserve new
ADRs once work starts:

- how bulk buffers and spans are represented in the shared ABI
- whether stream I/O uses callbacks, pull APIs, or wrapper-local file helpers
- how C++ reports errors over the status-based ABI
- how Python owns native handles and exposes live views safely
- what wasm copy policy is acceptable for typed-array exports

## Done Criteria

This phase is complete when:

- the shared ABI covers inspection, bulk geometry access, targeted mutation,
  import or extract workflows, and explicit write options
- C++ can parse, inspect, build or edit, and serialize models through its
  wrapper
- Python can parse, inspect, edit, and perform extraction-style workflows
  through classes and views
- wasm can probe, summarize, extract geometry, and serialize through a narrow
  browser-facing API
- all new public behavior is documented and backed by tests
