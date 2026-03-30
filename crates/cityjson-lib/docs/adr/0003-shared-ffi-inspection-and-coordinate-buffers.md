# ADR 0003: Shared FFI Inspection And Coordinate Buffer Shape

## Status

Accepted.

## Context

After the shared C ABI foundation landed, the next expansion slice needed to
cover:

- read-only model inspection
- geometry-type queries
- wrapper-friendly bulk coordinate access

The main open design question was whether the shared ABI should expose borrowed
views immediately or copy data into explicit owned buffers first.

## Decision

The shared ABI will widen with these rules:

- aggregate model state is returned through `cj_model_summary_t`
- cityobject identifiers and geometry types are exposed through dense indexed
  queries over occupied items
- root vertices, template vertices, and UV coordinates are returned through
  explicit owned copy buffers
- buffer ownership is released with dedicated free functions

The first higher-level slice does not expose borrowed spans yet.

## Consequences

Positive:

- wrappers can consume the shared ABI without borrowing-lifetime traps
- Python and C++ can build their first usable inspection layers immediately
- wasm can reuse the same copied coordinate buffers for browser-facing tasks

Tradeoffs:

- indexed cityobject access is more chatty than a future bulk ID export
- copied coordinate buffers may allocate more than a borrowed-span design
- later zero-copy or borrowed-view work will need an additional ADR and API
  slice rather than silently reshaping this contract
