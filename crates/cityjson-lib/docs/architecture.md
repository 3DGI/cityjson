# Architecture

This document describes the intended split between `cityjson-rs`,
`serde_cityjson`, `cityjson_lib`, and future bindings.

The aim is to keep one semantic core, explicit boundary crates, and one shared
low-level story for foreign languages.

## Core Rules

1. `cityjson-rs` owns the semantic CityJSON model family and its invariants.
2. `cityjson_lib` owns the stable facade and chooses how that model family is exposed
   as an owned default wrapper.
3. JSON, JSONL, Arrow, Parquet, and future transports are explicit format
   boundaries, not separate semantic model families.
4. C++, Python, and wasm share one low-level FFI core, but they are free to
   expose different public APIs on top of it.
5. Raw, staged, lazy, or performance-oriented paths stay explicit. They do not
   distort the default owned facade.

## One Semantic Core

The same semantic model family must be able to represent:

- a full CityJSON document
- a grouped subset or tile
- a single feature-sized self-contained package

Those are scope differences, not separate semantic kinds.

Implementation details such as string storage, vertex index width, or the exact
`cityjson-rs` instantiation can vary where it materially helps a workload, but
that choice should stay behind the crate boundary. The architecture is about
one semantic core, not one frozen internal typedef.

## Self-contained Models

A self-contained model must carry everything needed for independent use:

- cityobjects
- referenced vertices
- templates and template vertices
- appearance resources
- semantics
- metadata and extras

It does not need to inline all shared data into every geometry. Pooled storage
and indexed references remain the right default as long as the model is
self-contained at its own scope.

## Layering

The dependency direction is:

```text
cjfake
  -> cityjson_lib
  -> { serde_cityjson, cityarrow, cityparquet }
  -> cityjson-rs
```

With FFI layered like this:

```text
Rust crates
  -> shared low-level FFI core
  -> { C++ wrapper, Python binding, wasm adapter }
```

Responsibilities split as follows:

- `cityjson-rs`
  - semantic model family
  - invariants, validation, and correctness-critical mutation
  - extraction, localization, remapping, and merge semantics
- `serde_cityjson`
  - CityJSON JSON and JSONL wire format
  - document parsing, feature parsing, stream parsing, and serialization
  - raw or staged JSON boundary work
- `cityjson_lib`
  - stable Rust facade
  - explicit format modules
  - higher-level reusable operations above the semantic model
  - shared low-level FFI surface for bindings
- bindings
  - host-language ergonomics
  - host-native value types, views, iterators, and convenience helpers

## API Consequences For `cityjson_lib`

`cityjson_lib` should expose one owned default wrapper:

- `cityjson_lib::CityModel`

The root remains small:

- `cityjson_lib::json` as the default-on boundary module
- optional transport modules such as `cityjson_lib::arrow` and `cityjson_lib::parquet`
- `cityjson_lib::ops`
- `cityjson_lib::cityjson`

Format modules speak in terms of:

- one `CityModel`
- or streams of `CityModel` values

They should not promote transport-native units such as JSON feature wrappers,
Arrow batches, or Parquet row groups into the semantic API surface.

## JSON Boundary Direction

`cityjson_lib::json` should own the explicit JSON boundary:

- probing
- document parsing
- feature parsing
- feature-stream reading
- document and feature serialization
- feature-stream writing
- future raw or staged JSON access

`CityJSONFeature` remains a wire-format concern. The semantic unit returned to
callers is still `CityModel`.

## Shared FFI Direction

The shared low-level FFI core should be wide enough for all three target
families:

- C++ needs model construction, inspection, and controlled writing.
- Python needs model handles, collection access, bulk geometry access, and bulk
  mutation primitives.
- wasm needs only a subset, but it should reuse the same core concepts for
  probing, parsing, serialization, and bulk extraction.

What should stay shared:

- ownership and lifecycle rules
- parse, probe, and serialize entry points
- collection and resource access
- bulk geometry and boundary access
- bulk remap, import, extract, and cleanup operations
- explicit transform and quantization policy
- version and error concepts

What should stay target-specific:

- C++ RAII wrappers and value builders
- Python classes, mappings, iterators, and convenience algorithms
- JS-friendly wasm exports, typed arrays, and one-shot task-oriented APIs

## What Belongs In `cityjson-rs`

`cityjson-rs` should own semantic operations such as:

- extracting a self-contained submodel
- localizing shared resources
- merging submodels back together
- validating edits that affect model invariants

Those are not JSON concerns and they should not be reimplemented separately in
`cityjson_lib` or in bindings.

## What Belongs In `cityjson_lib::ops`

`cityjson_lib::ops` lives above the semantic core.
It can offer workflow helpers, but it should delegate correctness-critical
semantics to `cityjson-rs`.

Good candidates include:

- selection helpers
- LoD filtering
- cleanup workflows
- geometry measurements
- texture-path rewriting
- version-upgrade workflows

## Non-goals

The architecture should not:

- reintroduce a second semantic model in `cityjson_lib`
- treat storage-generic implementation choices as the main public abstraction
- hide format choice behind a generic registry
- force C++, Python, and wasm to share one identical host-language API
- duplicate JSON parsing or semantic validation logic across crates
