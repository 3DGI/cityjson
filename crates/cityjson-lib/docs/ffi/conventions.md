# FFI Documentation Conventions

These conventions keep the binding docs small and accurate.

## Document Shared Concepts Once

Shared ownership, error handling, and ABI rules belong in
[Shared low-level core](shared-core.md).
Binding pages should link there instead of repeating the same contract.

## Use Parallel Examples

When a workflow is public in multiple languages, prefer tabbed examples for:

- Rust
- Python
- C++

The wasm adapter is still work in progress and should be called out as such
instead of being documented as a finished peer surface.

## Prefer Release-Facing APIs

The docs should describe:

- published crate and package names
- shipped wrapper methods
- stable error and version concepts

They should not read like an implementation notebook or a future design pitch.
