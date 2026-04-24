#!/usr/bin/env just --justfile
# Root workspace recipes. Crate-specific recipes (bench, profile, ffi) remain
# under crates/<name>/justfile and can be invoked with:
#   just -f crates/<name>/justfile <recipe>

_default:
    just --list

# cargo clean across the workspace
clean:
    cargo clean

# cargo check, all targets, all features, across the workspace
check:
    cargo check --workspace --all-targets --all-features

# cargo build, all targets, all features, across the workspace
build *args:
    cargo build --workspace --all-targets --all-features {{args}}

# Strict clippy across the workspace
lint:
    cargo clippy --workspace --all-targets --all-features -- -Dclippy::all -Dclippy::pedantic

# cargo fmt across the workspace
fmt:
    cargo fmt --all

# Verify formatting
fmt-check:
    cargo fmt --all --check

# cargo test across the workspace
test:
    cargo test --workspace --all-features

# Build docs (nightly, docsrs cfg, deny warnings)
doc:
    RUSTDOCFLAGS="--cfg docsrs -Dwarnings" cargo +nightly doc --workspace --all-features --no-deps

# Miri on the cityjson crate's unsafe-touching test suites
miri:
    MIRIFLAGS="-Zmiri-strict-provenance" cargo +nightly miri test -p cityjson boundary
    MIRIFLAGS="-Zmiri-strict-provenance" cargo +nightly miri test -p cityjson vertex
    MIRIFLAGS="-Zmiri-strict-provenance" cargo +nightly miri test -p cityjson vertices
    MIRIFLAGS="-Zmiri-strict-provenance" cargo +nightly miri test -p cityjson handles
    MIRIFLAGS="-Zmiri-strict-provenance" cargo +nightly miri test -p cityjson raw_access
    MIRIFLAGS="-Zmiri-strict-provenance" cargo +nightly miri test -p cityjson geometry

# Run the Python binding test suites (tox smoke) for both crates
test-python:
    cd crates/cityjson-lib/ffi/python && uv run tox run
    cd crates/cityjson-index/ffi/python && uv run tox run

# Build the Python wheels for both crates
build-python:
    cd crates/cityjson-lib/ffi/python && uv build --wheel
    cd crates/cityjson-index/ffi/python && uv build --wheel

# Delegate to the cityjson-lib ffi helper (build/headers/etc)
ffi *args:
    cd crates/cityjson-lib && ./tools/ffi.sh {{args}}

# Full local CI (fmt + lint + check + test + doc)
ci: fmt-check lint check test doc
