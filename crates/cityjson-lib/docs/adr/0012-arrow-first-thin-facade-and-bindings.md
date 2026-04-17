# ADR 0012: Arrow-First Thin Facade And Bindings

## Status

Accepted

## Historical Note

This ADR captures the transport-first direction that lived on the Arrow/Parquet
development line.
The current publishable core branch does not ship that transport surface, so
this document should be read as archived design history.

## Context

The vNext design for `cityjson-lib` narrows the repository role:

- `cityjson-rs` stays the only semantic model
- `cityjson-arrow` owns Arrow transport mechanics
- `cityjson-lib` should stay a thin facade over those sibling crates
- non-Rust bindings should use Arrow-first bulk interchange instead of
  wrapper-owned projected object graphs

The previous `cityjson-lib` FFI surface had started to drift away from that
direction by exposing copied wrapper projections such as
`cj_model_copy_projected_cityobjects` and binding helpers like
`CityModel.projected_cityobjects()` and `Model::projected_cityobjects()`.

Those APIs were expensive, obscured full-model traversal behind convenience
names, and duplicated information that already exists in the Arrow transport.

## Decision

`cityjson-lib` now keeps its Rust and non-Rust public surfaces thin:

- the Rust root re-exports `cityjson::v2_0::OwnedCityModel` directly as
  `CityModel` instead of wrapping it in a facade-owned struct
- the Rust `arrow` module exposes explicit Arrow IPC byte helpers plus explicit
  batch export/import helpers
- the shared C ABI no longer exposes projected cityobject wrapper buffers
- the Python and C++ bindings no longer expose wrapper-wide projected
  cityobject or vertex-copy convenience APIs
- bindings keep Arrow bytes as the primary bulk interchange surface and reserve
  scalar or single-item helpers for inspection and editing

## Consequences

Positive:

- the binding surface aligns more closely with the planned architecture
- expensive whole-model conversion is now explicit in the API surface
- Rust callers no longer have to cross wrapper-specific `as_inner` /
  `into_inner` boundaries to reach the underlying semantic model
- wrapper code and tests no longer encode bespoke projection semantics that
  compete with Arrow transport

Negative:

- this is a breaking binding change
- Python and C++ are not yet on Arrow C Data / Arrow C Stream exports because
  that support does not exist in this repository alone
- some low-level copied coordinate helpers remain in the shared ABI for wasm and
  builder-oriented workflows

## Follow-Up

The remaining step toward the full plan is to replace Arrow-byte bulk transport
in Python and C++ with direct Arrow C stream interop once the sibling Arrow
layer exposes the required primitives.
