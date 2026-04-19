# Release Checklist

Use this checklist to cut a release of the publishable surfaces in this repo.

## Rust Crate

1. Run `cargo test --locked --workspace --all-targets --all-features`.
2. Run `cargo publish --dry-run --locked -p cityjson-lib`.
3. Confirm that required sibling crates such as `cityjson-rs` and `cityjson-json` are already on crates.io.
4. Publish `cityjson-lib` to crates.io.
5. Publish `cityjson-lib-ffi-core` to crates.io once `cityjson-lib` is visible in the registry index.
6. Run `just ffi test` and confirm the generated C header is clean.

## Python Package

1. Run `uv build` from `ffi/python`.
2. Run the Python unit tests from `ffi/python`.
3. Verify the wheel and sdist artifacts.
4. Publish `cityjson-lib` to PyPI.
5. Confirm the Python FFI smoke tests passed through `just ffi test`.

## C++ Wrapper

1. Build `cityjson-lib-ffi-core`.
2. Configure, build, and test `ffi/cpp` with CMake.
3. Install the wrapper to a staging prefix.
4. Build a downstream consumer with `find_package(cityjson_lib_cpp CONFIG REQUIRED)`.
5. Confirm the wrapper passes the self-append and bulk-inspection smoke cases.

## Final Steps

1. Review the docs for stale references to old crate names or transport work.
2. Commit the release state.
3. Tag the release.
