# Shared C ABI Foundation Plan

This page breaks the first implementation slice of the shared low-level FFI
core into a concrete sequence.

The scope here is intentionally narrow:

- handle lifecycle
- status and error reporting
- probe, parse, and serialize exports

That is the smallest useful substrate for the future C++, Python, and wasm
layers.

## Goal

The first shared ABI should:

- keep `CityModel` opaque across the FFI boundary
- expose only bytes-in, bytes-out operations
- make ownership explicit
- return stable status and error categories
- shield foreign callers from Rust panics

This keeps the first surface small while still being complete enough for every
wrapper to build on.

## Proposed File Layout

The `ffi/core` crate should expand into:

- `src/lib.rs`
  Module wiring and public re-exports.
- `src/abi.rs`
  `#[repr(C)]` enums and structs shared by every export.
- `src/error.rs`
  Status mapping, thread-local last-error storage, and error-message copy
  helpers.
- `src/handle.rs`
  Opaque model and byte-buffer ownership helpers.
- `src/exports.rs`
  `extern "C"` functions only.
- `tests/*.rs`
  ABI-facing tests.
- `cbindgen.toml`
  Generated public header configuration.

## Implementation Order

### 1. Harden the Rust Boundary

Before exporting symbols, add a small internal adapter layer in `ffi/core`:

- isolate calls into `cjlib::json`
- avoid exposing `todo!()` paths for unsupported versions
- wrap every exported entry point in `std::panic::catch_unwind`
- map panics to a dedicated internal failure status

This prevents accidental Rust implementation details from leaking into the ABI.

### 2. Define the Shared C ABI Types

Add stable `#[repr(C)]` definitions for:

- `cj_status_t`
- `cj_error_kind_t`
- `cj_root_kind_t`
- `cj_version_t`
- `cj_probe_t`
- `cj_bytes_t`
- opaque `cj_model_t`

All exported functions should return `cj_status_t` and write results through
out-pointers.

The first status set should distinguish at least:

- success
- invalid argument
- io
- syntax
- version
- shape
- unsupported
- model
- internal panic or internal error

### 3. Implement Handle Lifecycle

Add explicit ownership functions first:

- `cj_model_free(model)`
- `cj_bytes_free(bytes)`

Rules for the first ABI slice:

- freeing `NULL` is always allowed
- only handles allocated by the library are valid
- arbitrary foreign pointers are undefined behavior

Do not introduce a pointer registry unless there is a real need for invalid
handle detection.

### 4. Implement Error and Status Reporting

Map `cjlib::ErrorKind` into stable ABI error categories and add thread-local
last-error state.

Export a small retrieval API:

- `cj_last_error_kind()`
- `cj_last_error_message_len()`
- `cj_last_error_message_copy(buffer, capacity, out_len)`
- `cj_clear_error()`

Successful calls should clear the last error so wrappers do not read stale
state.

### 5. Implement Probe Export

Add the first allocation-free read-only export:

- `cj_probe_bytes(data, len, out_probe)`

`cj_probe_t` should carry:

- root kind
- version
- whether a version was present

This gives wrappers a cheap way to branch before parsing.

### 6. Implement Parse Exports

Add bytes-based parse entry points:

- `cj_model_parse_document_bytes(data, len, out_model)`
- `cj_model_parse_feature_bytes(data, len, out_model)`
- `cj_model_parse_feature_with_base_bytes(feature_data, feature_len, base_data, base_len, out_model)`

Each export should:

- validate pointer and length pairs before parsing
- convert Rust `CityModel` values into boxed opaque handles
- report unsupported or malformed inputs through the shared status and
  last-error path

### 7. Implement Serialize Exports

Add bytes-based serialize entry points:

- `cj_model_serialize_document(model, out_bytes)`
- `cj_model_serialize_feature(model, out_bytes)`

`cj_bytes_t` should return an owned pointer plus length, and callers must
release that buffer with `cj_bytes_free`.

The first ABI slice should stay bytes-only. File and stream I/O can come later.

### 8. Generate the Public Header

Add `cbindgen.toml` and generate one C header for the shared core.

The generated header should be treated as part of the ABI contract and included
in the normal FFI developer workflow.

### 9. Verify the ABI in Rust Tests

Add `ffi/core` tests for:

- null pointer argument handling
- probe success and failure
- parse and serialize round-trips
- last-error message retrieval and clearing
- freeing null handles and null byte buffers
- panic shielding
- unsupported version behavior

The first ABI should be considered ready only after these behavior-level tests
pass.

## Rationale For The Narrow First Slice

The first public C surface should stop at bytes and opaque handles.

That gives every target:

- a stable ownership model
- a consistent error model
- a shared parse and serialize story

It avoids early commitments on:

- collection views
- spans and borrowing rules
- mutation APIs
- bulk geometry access
- remap, import, extract, and cleanup operations

Those can be layered on once the basic ABI contract is stable.
