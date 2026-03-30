# ADR 0001: Shared C ABI Foundation

## Status

Accepted.

## Context

`cjlib` needs one stable low-level foreign-function surface for C++, Python,
and wasm. The shared core must survive across host languages without exposing
Rust-specific ownership, error, or panic behavior.

The first implementation slice is intentionally small:

- opaque `cj_model_t` handles
- explicit ownership and release functions
- stable status and error categories
- bytes-in, bytes-out probe/parse/serialize functions

## Decision

The shared ABI will use these rules:

- `CityModel` remains opaque to foreign callers.
- Functions return `cj_status_t` and write results through out-pointers.
- Successful calls clear the thread-local last-error state.
- Errors are reported through stable status categories plus a copyable message.
- Ownership is explicit for both model handles and returned byte buffers.
- The ABI surface is format-explicit rather than codec-generic.

This keeps the first contract small and predictable while leaving room for
target-specific wrappers above it.

## Consequences

Positive:

- all bindings start from the same semantic substrate
- ownership is visible at the boundary
- the ABI is easier to document and generate headers for
- panic behavior stays inside Rust

Tradeoffs:

- callers must follow explicit free rules
- later format families such as Arrow and Parquet will be added as separate
  transport-specific entry points rather than as one generic dispatcher
- the first C layer is narrow by design, so some higher-level workflows stay out
  until the shared substrate is proven
