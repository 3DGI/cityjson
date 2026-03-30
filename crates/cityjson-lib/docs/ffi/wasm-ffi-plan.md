# Wasm FFI Plan

This page describes the wasm direction over the shared low-level FFI core.

Unlike C++ and Python, wasm does not need a broad public model API. It should
reuse the same shared semantics while exposing a much narrower browser-facing
surface.

## Goal

Expose a small wasm adapter that can support tasks such as:

- probing and summarizing CityJSON input
- extracting geometry or mesh buffers
- serializing or roundtripping models when needed

The wasm layer should reuse the shared low-level core where possible, but it is
free to expose only a strict subset of it.

## Public Wasm Shape

The public JS-facing surface should stay task-oriented:

- bytes in
- flat summaries or typed-array buffers out
- explicit one-shot operations where that keeps the browser API simpler

There is no requirement for wasm to expose the same rich editable model surface
that Python or C++ may expose.

## What The Shared Core Must Provide For Wasm

The wasm adapter should reuse shared primitives for:

- probing root kind and version
- parsing documents or feature-sized payloads
- serializing documents or feature-sized payloads
- extracting bulk geometry and boundary data
- stable error and version reporting

The wasm layer can keep handles and deeper mutation APIs internal unless a
browser-facing use case proves they are necessary.

## What Stays Wasm-specific

The following belong in the wasm adapter layer:

- JS-friendly flat result structs
- typed arrays and copy policy decisions
- browser memory and packaging strategy
- narrow task-oriented export functions

Those concerns should not reshape the shared low-level contract for the other
targets.

## Non-goals

This plan should not:

- force wasm to expose the same public API as C++ or Python
- make JS object graphs the primary semantic interface
- broaden the shared core just to satisfy browser ergonomics

## Deliverables

1. Map the probe, parse, serialize, and bulk extraction subset of the shared
   core into wasm-friendly exports.
2. Validate that the narrow wasm API can answer real browser-facing use cases.
3. Keep the richer model-editing surface in the shared core and in the C++ and
   Python bindings, not in the wasm public API.

## Follow-Up ADR

The concrete wasm32 portability blocker and the later wasm64 evaluation path
are tracked in [ADR 0008](../adr/0008-wasm32-blocker-and-wasm64-path.md).
