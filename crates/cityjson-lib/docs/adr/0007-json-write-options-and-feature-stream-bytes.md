# ADR 0007: JSON Write Options And Feature-Stream Bytes

## Status

Accepted.

## Context

The remaining FFI slices need write controls and feature-stream support, but
there are several possible shapes:

- expose a large generic serializer configuration surface
- expose callback-driven stream ownership
- keep the initial contract small and explicit

The first two options would broaden the ABI before the wrappers have a stable
need for them.

## Decision

The initial write contract uses small explicit JSON options:

- `pretty`
- `validate_default_themes`

The first feature-stream contract stays bytes-based:

- feature streams are represented as newline-delimited feature bytes
- the Rust side can merge or write the stream from owned model handles
- callback-based stream ownership is deferred until a real use case requires it

Transform control remains a separate model mutation concern, and quantization
is left for a later slice.

## Consequences

Positive:

- wrapper code gets a simple, portable contract
- JSON serializer policy remains explicit rather than hidden behind a generic
  codec registry
- stream ownership stays easy to reason about in C++, Python, and wasm

Tradeoffs:

- the initial contract is not a complete serializer configuration API
- callback-based streaming will need a follow-up ADR if and when it becomes
  necessary
- feature streams stay JSONL-oriented for now instead of becoming a general
  transport abstraction
