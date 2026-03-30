# ADR 0004: Wrapper Milestone One Shape

## Status

Accepted.

## Context

The expansion plan calls for C++, Python, and wasm bindings over one shared
low-level ABI, but the targets do not need the same public surface.

The first usable wrapper milestone needed to answer two questions:

- how wide should the initial C++ and Python public layers be
- how narrow should the wasm-facing layer remain

## Decision

Milestone one is defined as:

- C++: an RAII wrapper over the shared C header with parse, create, serialize,
  summary, metadata, indexed inspection, and copied coordinate helpers
- Python: a pure-`ctypes` object wrapper over the shared ABI with the same
  semantic coverage as C++
- wasm: a task-oriented adapter that exposes probe, parsed summaries, and
  coordinate extraction rather than public handle ownership

The wrappers must consume the generated shared C header and exported ABI rather
than adding target-local native side channels.

## Consequences

Positive:

- all targets now exercise the same low-level semantics
- C++ and Python get immediately usable parse or inspect or serialize stories
- wasm stays aligned with browser-facing tasks instead of inheriting the full
  editable model API too early

Tradeoffs:

- the C++ and Python layers are intentionally incomplete relative to the full
  expansion plan
- the wasm public API remains narrower than the shared core and the other
  bindings
- later mutation, import, extract, and stream support will widen the wrappers
  in additional slices instead of landing all at once
