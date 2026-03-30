# FFI and Bindings

These pages describe how `cjlib` should serve non-Rust targets.

The key architectural decision is:

- one shared low-level FFI core
- separate target-specific public bindings on top

That means the low-level ownership, parse, serialize, and bulk-operation story
should be shared, while C++, Python, and wasm remain free to expose different
host-language APIs.

## Why This Lives In MkDocs

Rustdoc is useful for Rust APIs, but it is not the right home for the whole
cross-language story.
The documentation site needs one place that can cover:

- Rust usage
- low-level FFI concepts
- target-specific bindings
- shared concepts such as ownership, versioning, transforms, and error mapping

MkDocs is the right top-level tool because it is language-neutral.

## Documentation Split

The FFI section is split into:

- [Shared low-level core](shared-core.md)
  The common substrate used by all non-Rust targets.
- [Shared C ABI foundation plan](shared-c-abi-foundation-plan.md)
  The first implementation slice for the shared core: lifecycle, errors, and
  probe/parse/serialize exports.
- [Architecture decisions](../adr/0001-shared-c-abi-foundation.md)
  Decision records that freeze the cross-cutting ABI and header workflow
  choices.
- [Conventions](conventions.md)
  Documentation rules for keeping the shared concepts and target wrappers
  aligned.
- [C++ plan](cpp-ffi-plan.md)
  C++ wrapper direction over the shared core.
- [Python plan](python-ffi-plan.md)
  Python binding direction over the shared core.
- [Wasm plan](wasm-ffi-plan.md)
  Narrow browser-facing adapter over the shared core.

The target plan pages should not redefine the shared core. They should explain
how each target projects that core into an idiomatic public API.
