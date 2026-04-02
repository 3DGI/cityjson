# Separate Live Arrow IPC From Persistent Package IO

## Status

Accepted

## Context

[ADR 1](001-canonical-transport-boundary.md) established
`CityModelArrowParts` as the canonical transport decomposition around
`cityjson::v2_0::OwnedCityModel`.

[ADR 2](002-address-transport-performance-bottlenecks.md) then clarified that
the current benchmark gap should be treated as an implementation problem in
transport conversion and package IO, not as evidence that the shared semantic
boundary is wrong.

The implementation review behind ADR 2 exposed a second architectural problem:
the current transport surface conflates two different concerns.

- live process-to-process transport wants incremental delivery and immediate
  decode start
- persistent package IO wants seekable storage, low file-count overhead, and
  lazy access to on-disk batches

The current Arrow IPC path does not cleanly serve either case.

- package IO uses Arrow IPC files arranged as a directory of canonical tables
  rather than a single seekable package abstraction
- readers eagerly collect and concatenate all batches before reconstruction
- the public API exposes `CityModelArrowParts`, which encourages eager,
  fully-materialized transport handling instead of streaming or bound-column
  decoding
- the transport package shape leaks into the public API even though it is not
  the semantic boundary of the system

Arrow's strengths differ by use case:

- for live inter-process exchange, Arrow IPC stream format matches sequential
  transport over pipes or sockets
- for persistent exchange, a memory-mappable package container matches lazy
  reads and avoids the fixed costs of a directory full of small files

The project wants a clean cut, not a compatibility layer that keeps both the
old and new transport architectures alive.

## Decision

`cityarrow` will separate live Arrow transport from persistent package IO.

The new architecture is:

1. `cityjson::v2_0::OwnedCityModel` remains the semantic source and sink
2. live process-to-process transport uses Arrow IPC stream format as the
   primary wire representation
3. persistent package IO uses a seekable, memory-mappable package container
   with lazy batch access rather than a directory of canonical table files
4. conversion code operates on bound-column table views and batch streams
   rather than on a public `CityModelArrowParts` struct
5. the canonical table decomposition remains an internal transport contract for
   encoding, decoding, validation, and documentation, but it is no longer the
   public API boundary

This is an intentional breaking change.

When this architecture lands:

- the current directory-oriented package layout is removed
- the current public `to_parts` / `from_parts` surface and public
  `CityModelArrowParts` transport struct are removed
- the package schema version is cut to a new major alpha revision
- no transition code, dual readers, compatibility shims, or backwards
  compatibility paths will be kept

The project will not adopt Arrow Flight as the primary transport contract.
Flight is a larger RPC choice than the current problem requires.

The project will also not make shared memory the primary public abstraction.
Shared memory may be used later as an optimization behind the live stream or
seekable package abstractions, but it is not the architectural boundary.

## Consequences

Good:

- live IPC and persistent package exchange are optimized for their real access
  patterns instead of sharing one compromised file-oriented surface
- decoders can begin reconstruction incrementally from batch streams instead of
  waiting for eager whole-table concatenation
- package IO no longer pays the current directory-of-many-files overhead before
  reconstruction starts
- the public API becomes narrower and better aligned with the true semantic
  boundary around `OwnedCityModel`
- the internal canonical schema can still stay explicit and testable without
  being exposed as the user-facing architecture

Trade-offs:

- this is a hard breaking change for both the Rust API and the package format
- old `v1alpha1` packages will not remain readable once the new package
  architecture replaces them
- the implementation becomes more specialized around stream readers, seekable
  containers, and bound-column decoding
- package specification work increases because the project now owns a new
  persistent container contract instead of a directory convention

## Post-Acceptance Note: 2026-04-02

The first post-refactor `cjlib` benchmark run shows that the ADR 3 cut line was
useful but not sufficient on its own.

The native read paths improved materially after the refactor, and both
`cityarrow` and `cityparquet` now read the pinned 3DBAG cases faster than the
current shared-model JSON path. That means the architectural cut did remove
real overhead.

The same run also showed that writes remain far behind JSON and that read
allocation totals did not materially change. The current implementation still
routes the hot path through full canonical-parts materialization and eager
whole-input handling, so the intended stream-first and lazy-package execution
model is not yet fully realized.

That follow-up is recorded in
[ADR 2 and ADR 3 benchmark follow-up](../adr-002-003-benchmark-follow-up.md).

Follow-up work:

- define the new public reader and writer API around streaming decode and
  encode operations
- define the new package container specification and schema version
- rewrite the import path around bound columns, per-table binders, and
  geometry-local span indexes instead of `Vec<Row>` materialization
- rewrite package readers to expose lazy batch iteration rather than eager load
  and concatenate behavior
- add split benchmarks for live Arrow stream IO, package IO, conversion only,
  and end-to-end reconstruction
