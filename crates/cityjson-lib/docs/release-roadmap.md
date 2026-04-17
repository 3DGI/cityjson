# Release Roadmap

This document turns the current release-readiness audit into a pragmatic plan
for getting `cityjson-lib` to a state that can stand up in public.

Current assessment: the implementation is further along than the release story.
The Rust JSON layer is real and now the default-on `json` feature, `ops` is
small but implemented, Arrow support works behind an opt-in feature, the FFI
core is substantial, and the Python, C++, and wasm layers all have working
smoke or unit coverage. The blockers are packaging, API honesty, CI scope, and
release discipline.

## Current Verdict

Do not publish the crate as-is.

The codebase is credible as a serious pre-1.0 internal project, but not yet as
a public crate that should be expected to install cleanly, match its docs, and
carry its stated support surface.

## Release Blockers

### 1. Packaging Must Work

The crate is not publishable in its current form.

- `Cargo.toml` packages only a narrow include set.
- The package manifest references license files that are not present.
- The crate depends on sibling path crates at versions that are not available
  on crates.io.
- `cargo package --allow-dirty` currently fails.
- Examples, benches, and tests are omitted from the packaged crate by the
  current include list.

Release bar:

- `cargo package --dry-run` succeeds for the root crate.
- Dependency strategy is explicit and publishable.
- Packaged contents match what public users should actually receive.

### 2. Public Docs Must Match the Code

The documentation currently overstates or misstates the public Rust API.

Examples of drift:

- Older docs repeatedly referred to `CityModel::from_file` and
  `CityModel::from_slice`, while the actual public entry points are under
  `cityjson_lib::json`.
- Some pages still describe future-facing surfaces as if they already exist.

Release bar:

- README, guide pages, API pages, and examples reflect the real exported API.
- All public-facing code snippets compile or are covered by doctests/tests.
- The docs stop mixing "intended future shape" with "available now" unless the
  distinction is explicit.

### 3. Transport Scope Must Be Explicit

Arrow and Parquet should not block the first public release of `cityjson-lib`.

For the first public version:

- `cityjson-arrow` and `cityjson-parquet` should be default-off features.
- The core crate should ship without pretending those transport layers are fully
  implemented or fully release-hardened.
- Public docs must describe them as optional and still in-progress where that is
  the truth.

Current problems:

- There is a `src/parquet.rs`.
- The manifest intentionally keeps `parquet` as a stubbed opt-in feature.
- The example uses `#[cfg(feature = "parquet")]`, which now matches a real
  feature flag.

Release bar:

- `cityjson-arrow` is an explicit opt-in feature, not part of the default
  install path.
- `cityjson-parquet` exists as a stubbed opt-in feature and is not presented as
  a finished transport layer.
- The first public release is honest that the stable, primary boundary is the
  core JSON-facing crate surface.

### 4. CI Must Cover the Advertised Surface

Current CI is narrower than the project claims.

- Main Rust CI runs Linux `cargo build` and `cargo test`.
- In-repo Python and C++ binding tests are not part of the main CI path.
- Python validation is delegated to another repository workflow.
- Docs CI is weaker than the local `just docs-build` task.
- There is no package dry-run gate.
- There is no coverage gate.
- There is no multi-OS matrix for the FFI-heavy support story.

Release bar:

- CI runs the real in-repo release checks.
- CI includes strict docs build.
- CI includes package dry-run.
- CI includes the binding tests that this repository publicly claims to support.
- CI should at least make it obvious which targets are tested and which are not.

### 5. Documentation Publishing Needs A Real Tooling Pass

The documentation set is substantial, but the site tooling and publishing
workflow should change before the first public release.

- The repository should switch from the current MkDocs setup to Proper Docs,
  following the pattern already used in `~/Development/cityjson-corpus`
  with `properdocs.yml` and `uv run properdocs build/serve`.
- Strict docs build should remain mandatory after that migration.
- Some docs are omitted from nav.
- Several pages contain absolute local filesystem links.
- That is acceptable for internal notes, but not for public docs.

Release bar:

- Proper Docs replaces the current MkDocs setup as the public site workflow.
- No absolute local-path links in published docs.
- Navigation reflects the intended public information architecture.
- Internal planning notes are either clearly marked or kept out of the public
  site.

## Implementation Status

The implementation itself is in better shape than the release posture.

### Rust API

What is real today:

- `cityjson_lib::json` is the real primary boundary.
- `ops` is implemented, not just sketched.
- Arrow read/write and batch export/import work.
- Error typing exists and is usable.

What is still thin:

- The crate root is small, but the surrounding docs often pretend it is larger.
- CLI coverage is minimal.
- Some error-handling paths are under-tested compared with the main happy path.
- The first public version should stay centered on the core JSON-facing API
  rather than on optional transport crates.

### FFI and Bindings

The FFI story is not vaporware.

What is real today:

- shared C ABI core
- wasm adapter
- Python `ctypes` layer
- C++ RAII wrapper

What that means pragmatically:

- It is fair to say the project has a functioning FFI stack.
- It is not yet fair to claim fully release-hardened multi-language support.

## Test and Coverage Status

### Validation That Already Passed

The following checks pass locally:

- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `just docs-build`
- `just ffi-test`

That is meaningful. The project is not in a fake-green state where only a small
core passes.

### Coverage Readout

Coverage is measurable, which is already better than hand-waving about tests.
The current signal is mixed:

- `src/json.rs`: about 76.6% line coverage
- `src/ops.rs`: about 65.0% line coverage
- `src/arrow.rs`: about 73.9% line coverage
- `ffi/core/src/exports.rs`: about 79.9% line coverage
- `ffi/wasm/src/lib.rs`: about 88.8% line coverage
- `src/error.rs`: about 37.7% line coverage
- `src/bin/cjexport.rs`: 0% line coverage

Pragmatic reading:

- Core paths are exercised.
- FFI is better covered than many early projects.
- Error paths and CLI behavior are not yet where they should be for a public
  release.

## Publishability Decision

### Could It Be Published Today?

No.

Not because the code is unserious, but because the public release contract is
currently weaker than the implementation.

### Would It Stand Up Well In Public?

Not yet.

Public users will judge the crate on:

- whether it installs cleanly
- whether the docs are true
- whether examples compile
- whether advertised modules exist
- whether CI clearly backs the support claims

Right now those are the weak points.

## First Public Version Scope

The first public release should be intentionally narrower than the full
repository surface.

What the first public version should claim confidently:

- the core `cityjson-lib` crate
- the explicit JSON boundary
- the small implemented `ops` surface
- the current error model

What the first public version should not over-claim:

- a fully implemented `cityjson-arrow`
- a fully implemented `cityjson-parquet`
- a broad default feature set for transport backends

Release policy for v1 public debut:

- `cityjson-arrow` should be default off.
- `cityjson-parquet` should be default off.
- Incomplete transport support should not block releasing the core crate.
- The docs and feature matrix must make that boundary obvious.

## Minimum Release Plan

### Phase 1: Make the Release Real

1. Make `cargo package --dry-run` pass.
2. Fix manifest packaging contents and license-file handling.
3. Resolve path-dependency publication strategy.
4. Make `cityjson-arrow` and `cityjson-parquet` default-off features for the
   public release plan.

### Phase 2: Make the Public API Honest

1. Rewrite README and guides to use `cityjson_lib::json::*` where that is the
   real API.
2. Remove or clearly mark future/intended surfaces.
3. Make the optional Arrow and Parquet story explicit in code, docs, and
   examples.
4. State clearly that the first public release does not ship with fully
   implemented transport crates.

### Phase 3: Restructure The CLI Boundary

1. Move `cjexport` out of `cityjson-lib` into a separate workspace crate named
   `cityjson-export`.
2. Keep the installed CLI binary name as `cjexport`.
3. Test and package the CLI as its own deliverable instead of treating it as an
   incidental bin target of the library crate.

### Phase 4: Tighten CI

1. Add package dry-run to CI.
2. Run strict Proper Docs build in CI after the docs migration.
3. Run the in-repo FFI test path in CI.
4. Make tested platforms and unsupported platforms explicit.

### Phase 5: Close the Most Visible Test Gaps

1. Add tests for error-path behavior.
2. Add tests for `cjexport`.
3. Add tests that lock public examples and README snippets to real code.

## Practical Recommendation

Treat the next release as a release-hardening milestone, not as a feature
milestone.

The project does not primarily need more surface area. It needs alignment:

- implementation
- manifest
- docs
- CI
- support claims

That alignment should also include a narrower first release:

- core crate first
- transport crates optional and default off
- CLI split into its own crate
- Proper Docs as the public documentation path

Once those are aligned, publishing a pre-1.0 crate becomes reasonable.
