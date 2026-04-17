# FFI and Bindings

This section documents the publishable Rust, Python, and C++ bindings around
the shared low-level FFI core.

## Pages

- [Binding API](api.md): tabbed Rust, Python, and C++ examples for the public surface.
- [Shared low-level core](shared-core.md): shared ABI and ownership rules.
- [FFI implementation plan](ffi-implementation-plan.md): current binding work and next slices.
- [Conventions](conventions.md): documentation rules for the FFI section.
- [Architecture decisions](../adr/0001-shared-c-abi-foundation.md): frozen ABI and header choices.
- [Model-authoritative JSON workflows](../adr/0006-model-authoritative-json-ffi-workflows.md): JSON roundtrip workflows over Rust-owned models.
- [JSON write options and feature streams](../adr/0007-json-write-options-and-feature-stream-bytes.md): write options and feature-stream contracts.
- [Wasm32 portability note and wasm64 path](../adr/0008-wasm32-blocker-and-wasm64-path.md): archived wasm portability note.
