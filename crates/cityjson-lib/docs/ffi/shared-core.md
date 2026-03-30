# Shared Low-level FFI Core

This document defines the common substrate that C++, Python, and wasm should
share.

The main rule is simple:

- share one low-level core
- do not force one identical high-level public API on every target

## Goal

The shared core exists to give every non-Rust binding the same semantic
foundation:

- Rust owns the canonical CityJSON semantics
- ownership and lifecycle are explicit
- bulk operations cross the boundary efficiently
- parse, probe, and serialize behavior is consistent across targets

The bindings then decide how to present that core idiomatically.

## Layering

The intended layering is:

```text
cityjson-rs + serde_cityjson + cjlib
    -> shared low-level FFI core
    -> { C++ wrapper, Python binding, wasm adapter }
```

The shared core may be implemented as a C ABI plus Rust-side helpers, or by a
similar low-level mechanism, but the documentation should stay focused on the
concepts rather than on one transport choice.

## Shared Concepts

Every target should map onto the same low-level concepts:

- model handles and explicit ownership
- explicit parse and probe entry points
- explicit serialize entry points
- stable version and error categories
- collection and resource access
- bulk geometry and boundary access
- bulk remap, import, extract, and cleanup operations
- explicit transform and quantization policy

The shared core should expose the operations that every binding needs, even if
some targets choose not to expose them all publicly.

For the advanced workflow slice, the current direction is model-authoritative:
append, extract, and cleanup are expressed as explicit model operations over
Rust-owned state, with JSON roundtrips used to preserve the canonical serializer
and validator behavior. That keeps the shared contract aligned with the Rust
model instead of exposing a parallel foreign import format.

## Required Low-level Operations

At minimum, the shared core should support:

- probe bytes to detect root kind and version
- parse full documents and feature-sized payloads into Rust-owned models
- read and write feature streams
- serialize whole documents and feature-sized payloads
- inspect model metadata, cityobjects, geometries, appearance resources, and
  templates
- expose bulk vertex and boundary data in boundary-friendly buffers or spans
- remap vertex and resource references in bulk
- import, append, and extract self-contained submodels
- run cleanup and validation-sensitive operations
- write with explicit transform and quantization options

## What Stays Out Of The Shared Core

The shared core should not try to be the public API for every language.
The following stay target-specific:

- C++ RAII wrappers, value builders, and STL-facing views
- Python classes, mapping protocol, iterators, and convenience algorithms
- wasm typed-array packaging, browser memory policy, and one-shot task-oriented
  exports

Those are wrapper concerns, not low-level shared semantics.

## Design Constraints

The shared core should preserve these rules:

- Rust remains the only place that owns the CityJSON semantic invariants
- bulk operations are preferred over chatty per-element crossings
- advanced paths remain explicit rather than distorting the common case
- transport-specific units do not become semantic units

## Acceptance Criteria

The shared core is wide enough when:

- C++ can build, inspect, and write models without special-case Roofer-only
  logic
- Python can express generic model editing and `cjio`-style workflows without
  falling back to dict/list document surgery
- wasm can reuse the same probe, parse, serialize, and extraction semantics
  while exposing only a narrow public JS-facing API
