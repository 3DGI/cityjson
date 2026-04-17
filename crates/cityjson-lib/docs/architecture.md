# Architecture

This repository is organized around one semantic model and one explicit JSON
boundary.

## Layering

```text
cityjson-rs
  <- cityjson-json
  <- cityjson-lib
       <- ffi/core
            <- ffi/python
            <- ffi/cpp
            <- ffi/wasm
```

## Responsibilities

- `cityjson-rs`
  Owns the semantic CityJSON model, invariants, and correctness-sensitive
  mutation.
- `cityjson-json`
  Owns probing, parsing, staged reconstruction, feature-stream handling, and
  serialization for CityJSON and CityJSONSeq.
- `cityjson-lib`
  Owns the stable Rust facade, error/version translation, `query`, and `ops`.
- `ffi/core`
  Owns the shared low-level C ABI.
- `ffi/python` and `ffi/cpp`
  Own the host-language wrappers over that ABI.
- `ffi/wasm`
  Remains a narrower work-in-progress adapter.

## Core Rules

1. `cityjson-lib` does not define a second semantic model.
2. JSON-aware implementation lives in `cityjson-json`, not in `cityjson-lib`.
3. The public Rust surface stays explicit: `json`, `ops`, `query`, and
   `cityjson`.
4. The low-level ABI is shared across bindings; the high-level APIs are
   target-specific.
5. Transport experiments are not part of the current publishable crate line.

## API Consequences

The published Rust crate is intentionally small:

- `CityModel`
- `CityJSONVersion`
- `Error` and `ErrorKind`
- `json`
- `ops`
- `query`
- `cityjson`

The current format story is equally small:

- `json` is the only publishable format module on this branch
- feature streams remain explicit through `json`
- Arrow and Parquet are not part of the release surface

## FFI Consequences

The shared ABI covers:

- probe, parse, and serialize entry points
- summary and metadata reads
- cityobject and geometry inspection
- copied boundary and coordinate extraction
- targeted mutation
- cleanup, append, and extract workflows

The wrappers then shape that ABI for each host language:

- Python exposes classes and dataclasses
- C++ exposes RAII wrappers and STL-friendly types
- wasm will expose a narrower task-oriented surface when it is ready
