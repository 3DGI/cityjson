# Release Roadmap

This document turns the current release-readiness audit into a pragmatic plan
for getting `cityjson-lib` to a state that can stand up in public.

Current assessment: the implementation is further along than the release story.
The Rust JSON layer is real and now the default-on `json` feature, `ops` is
implemented, Arrow support works behind an opt-in feature, and the FFI core,
Python, C++, and wasm lanes all pass local validation. The remaining blockers
are packaging, public API honesty, CI ownership, and documentation discipline.

## Current Verdict

Do not publish the crate as-is.

The codebase is credible as a serious pre-1.0 internal project, but not yet as
a public crate that should be expected to install cleanly, match its docs, and
carry its stated support surface.

## Release Blockers

### 1. Packaging Must Work

The crate is not publishable in its current form.

- `cargo package --allow-dirty --no-verify` currently fails.
- The root crate still depends on sibling path crates.
- Cargo now tries to resolve `cityjson-arrow` from crates.io during packaging
  and fails because it is not published there.
- The include list is no longer the main packaging problem.

Release bar:

- `cargo package --dry-run` succeeds for the root crate.
- Dependency strategy is explicit and publishable.
- Packaged contents match what public users should actually receive.
- Optional transport dependencies do not make packaging fail when they are not
  part of the release contract.

### 2. Public Docs Must Match the Code

The documentation currently overstates or misstates the public Rust API.

Examples of drift:

- The public docs correctly use `cityjson_lib::json` as the main entry point in
  most places now.
- The README still describes Parquet as if it owns persistent package-file I/O,
  while the actual module is a deliberate stub that returns
  `UnsupportedFeature`.
- The README still says higher-level workflows like `ops::merge` are
  intentionally unimplemented, but `cleanup`, `extract`, `append`, and `merge`
  are now implemented.

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

- There is a `src/parquet.rs`, but it is intentionally a stubbed boundary.
- The manifest intentionally keeps `parquet` as a stubbed opt-in feature.
- The public-facing docs still describe Parquet too confidently for the current
  implementation.

Release bar:

- `cityjson-arrow` is an explicit opt-in feature, not part of the default
  install path.
- `cityjson-parquet` exists as a stubbed opt-in feature and is not presented as
  a finished transport layer.
- The first public release is honest that the stable, primary boundary is the
  core JSON-facing crate surface.

### 4. CI Must Cover the Advertised Surface

Current CI is improved locally, but the hosted CI story is still narrower than
the project claims.

- `just ci` now runs the real native lane locally.
- `just ffi ci` now runs the in-repo FFI lane locally.
- GitHub Actions still only has Rust test, clippy, and docs workflows.
- There is still no repo-owned GitHub Actions workflow for the FFI lane.
- Python validation is still delegated to another repository workflow.
- There is still no package dry-run gate in GitHub Actions.
- There is still no coverage gate.
- There is still no multi-OS matrix for the FFI-heavy support story.

Release bar:

- CI runs the real in-repo release checks.
- CI includes strict docs build.
- CI includes package dry-run.
- CI includes the binding tests that this repository publicly claims to support.
- CI should at least make it obvious which targets are tested and which are not.

### 5. Documentation Publishing Needs A Real Tooling Pass

The documentation set is substantial, and the site tooling migration has
happened, but the published shape still needs cleanup before release.

- Proper Docs is now the active site workflow.
- Strict docs build remains on.
- Some docs are still omitted from nav.
- Several pages still contain absolute local filesystem links.
- That is acceptable for internal notes, but not for public docs.

Release bar:

- Proper Docs remains the public site workflow.
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
- `cityjson-export` exists as a separate workspace crate with the `cjexport`
  binary name retained.

What is still thin:

- The crate root is small, but the surrounding docs often pretend it is larger.
- Parquet is intentionally not implemented for the first public release.
- CLI coverage exists now, but it is still thin.
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
- The local FFI workflow is real and passes.
- It is not yet fair to claim fully release-hardened multi-language support in
  hosted CI.

## Test and Coverage Status

### Validation That Already Passed

The following checks pass locally:

- `just ci`
- `just ffi ci`

That is meaningful. The project is not in a fake-green state where only a small
core passes.

### Coverage Readout

Coverage is still not part of the live release gate.
There are historical measurements in this roadmap, but they should now be
treated as stale until rerun.

Pragmatic reading:

- The project has meaningful tests across Rust, FFI core, wasm, Python, and
  C++ smoke paths.
- The current weakness is not total lack of tests, but lack of current
  coverage measurement and gating.
- Error paths, documentation examples, and edge-case behavior still deserve
  more targeted coverage before release.

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
2. Resolve path-dependency publication strategy for `cityjson`, `cityjson-json`,
   and `cityjson-arrow`.
3. Decide whether optional transport crates must already be published for the
   first crates.io release of `cityjson-lib`.
4. Keep `cityjson-arrow` and `cityjson-parquet` default-off in the public
   release plan.

### Phase 2: Make the Public API Honest

1. Rewrite README and guides to use `cityjson_lib::json::*` where that is the
   real API.
2. Remove or clearly mark future/intended surfaces.
3. Make the optional Arrow and Parquet story explicit in code, docs, and
   examples.
4. State clearly that the first public release does not ship with fully
   implemented transport crates.

### Phase 3: Restructure The CLI Boundary

Status: largely done.

1. `cjexport` has been moved into the separate workspace crate
   `cityjson-export`.
2. The installed CLI binary name remains `cjexport`.
3. Keep treating the CLI as a separate deliverable and tighten its tests as
   needed.

### Phase 4: Tighten CI

1. Add package dry-run to CI.
2. Keep strict Proper Docs build in CI.
3. Add a repo-owned GitHub Actions workflow for the in-repo FFI lane.
4. Stop relying on cross-repo Python validation as the primary signal.
5. Make tested platforms and unsupported platforms explicit.

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
