# cityjson-lib

`cityjson_lib` is the integration layer for the CityJSON crates in this repository.
It keeps one semantic core in Rust, adds a small user-facing facade, and
provides the place where format modules, higher-level operations, and foreign
language bindings meet.

It is not a second CityJSON model.

## Responsibility Split

- `cityjson-rs`
  Semantic model family, invariants, validated mutation, extraction, and merge.
- `serde_cityjson`
  CityJSON JSON and JSONL parsing, probing, feature handling, and
  serialization through the default-on `json` feature.
- `cityjson_lib`
  Rust facade, explicit format modules, reusable operations above the model,
  and the shared low-level FFI core used by bindings.
- `cjfake`
  Test-data and generator crate above `cityjson_lib`.

## Public Shape

The Rust-facing surface stays intentionally small:

- `cityjson_lib::CityModel` as the owned default wrapper
- `cityjson_lib::CityJSONVersion`, `cityjson_lib::Error`, and `cityjson_lib::ErrorKind`
- `cityjson_lib::json` for explicit JSON and JSONL boundary work, enabled by default
- optional sibling format modules such as `cityjson_lib::arrow` and `cityjson_lib::parquet`
- `cityjson_lib::ops` for higher-level reusable workflows
- `cityjson_lib::cityjson` as the explicit drop-down path to the model crate

The common path is:

1. load one document through `cityjson_lib::json::from_*`
2. switch to `cityjson_lib::json` or another explicit module when boundary control,
   streams, or backend-specific options matter
3. work with the underlying model through `cityjson_lib::cityjson`
4. use `cityjson_lib::ops` for reusable workflow helpers that do not belong in the
   semantic model crate

## FFI Direction

Bindings are organized around one shared low-level core, not around three
independent foreign APIs.

The common concepts live under [FFI and Bindings](ffi/index.md):

- one Rust-owned semantic core
- one shared low-level ownership and bulk-operation story
- target-specific public wrappers for C++, Python, and wasm

That keeps the low-level contract unified while still allowing each binding to
be idiomatic in its host environment.

## Documentation Map

- [Ecosystem Overview](ecosystem-overview.md)
  Plain-language map of the related repositories, how they fit together, and
  where different users should start.
- [CityJSON Ecosystem Naming Map](ecosystem-naming.md)
  Proposed repository naming scheme for the CityJSON family, including the
  current-to-target rename map.
- [Architecture](architecture.md)
  Cross-crate responsibility split and long-term layering rules.
- [Shared Corpus Migration Plan](shared-corpus-migration-plan.md)
  Concrete plan for moving correctness fixtures and benchmark cases into one
  shared, multi-consumer corpus contract.
- [Architecture Decisions](adr/0001-shared-c-abi-foundation.md)
  Decision records for cross-cutting implementation choices, including the
  shared C ABI foundation and header workflow.
- [Guide](guide.md)
  How the Rust facade is meant to be used.
- [Public API](public-api.md)
  Overview of the stable Rust-facing surface.
- [FFI and Bindings](ffi/index.md)
  Shared foreign-language concepts plus target plans.

## Non-goals

`cityjson_lib` should not:

- reintroduce a second in-memory CityJSON model
- make format-specific transport details look like semantic application types
- hide format choice behind one generic dispatcher
- duplicate JSON parsing logic that belongs in `serde_cityjson`
- absorb storage invariants that belong in `cityjson-rs`
- force C++, Python, and wasm to share one identical high-level public API
