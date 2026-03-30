# ADR 0002: Generated FFI Header Workflow

## Status

Accepted.

## Context

The shared C ABI needs a reproducible header so non-Rust bindings can consume
the same contract that the Rust crate exports. Hand-maintaining a header would
introduce drift between the implementation, the docs, and the generated ABI.

## Decision

`ffi/core/cbindgen.toml` will define the public C header contract, and
`cbindgen` will generate the header as a workflow step rather than through
manual editing.

The developer workflow will expose a dedicated `just ffi-header` recipe that
produces the header from the current `ffi/core` crate state.

## Consequences

Positive:

- the header stays aligned with the Rust ABI surface
- the generated contract can be refreshed on demand
- downstream wrappers have one canonical C declaration source

Tradeoffs:

- contributors need `cbindgen` available locally for the header recipe
- the generated header is a derived artifact, so changes should be reviewed by
  comparing the Rust ABI and the generated output together
