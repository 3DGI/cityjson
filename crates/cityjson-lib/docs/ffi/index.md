# FFI and Bindings

This section documents the publishable Rust, Python, and C++ bindings around
the shared low-level FFI core in `ffi/core`.

## Pages

- [Binding API](api.md)
  Tabbed Rust, Python, and C++ examples for the public surface.
- [FFI Performance Visibility](performance.md)
  Local benchmark runner for wrapper-vs-ABI overhead checks.
- [Writing Data](../guide-writing.md)
  The current typed authoring flow, with the C++ fake-complete example as the full reference.
- [FFI Authoring API Proposal](authoring-api-proposal.md)
  Original review document for the write-side redesign that is now implemented.

The wasm adapter remains work in progress and is not part of the release-facing
binding docs.
