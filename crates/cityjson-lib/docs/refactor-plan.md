# cjlib Refactoring Plan

## Summary

Rewrite `cjlib` as a thin, owned, user-friendly facade over `cityjson-rs`.

`cjlib` should stop being a second CityJSON domain model. Its job should be:

- convenience constructors: `CityModel::from_slice`, `CityModel::from_file`, `CityModel::from_stream`
- CityJSON version classification and dispatch
- a small error facade around I/O and format-boundary failures
- a narrow amount of ergonomics for end users and future FFI crates

Everything else should come from `cityjson-rs` or the format-serialization crate.

## Why This Should Be A Rewrite

The current crate shape is structurally misaligned with the current project direction:

- `cjlib` still contains its own wrappers for attributes, geometry, city objects, metadata, transform, extensions, and resource pools.
- Several of the constructors that matter most now (`from_slice`, `from_file`, `from_stream`) are `todo!()`.
- The docs and tests describe historical behavior more than the current implementation.
- `cityjson-rs` has explicitly moved to a single in-memory target: `CityJSON` v2.0.
- `serde_cityjson` still owns multi-version JSON classification and parsing.
- `cjlib` currently targets Rust 2021 / 1.64, while `cityjson-rs` already requires Rust 2024 / 1.93.

Trying to incrementally preserve the old `cjlib` model would reintroduce the exact duplication and version spread that `cityjson-rs` already removed on purpose.

## Architectural Decisions

### 1. `cityjson-rs` is the only in-memory source of truth

Use `cityjson::v2_0::OwnedCityModel` internally.

Do not keep local `cjlib` copies of:

- city object types
- geometry types
- metadata
- transform
- extensions
- attribute containers
- resource pools

If a type already exists in `cityjson-rs`, `cjlib` should re-export it rather than wrap or mirror it.

### 2. `cjlib` should expose one owned default model

The user-friendly default should be a thin newtype around the owned `cityjson-rs` model:

```rust
pub struct CityModel(cityjson::v2_0::OwnedCityModel);
```

That wrapper should provide:

- `new(type_model: cityjson::CityModelType)`
- `from_slice`
- `from_file`
- `from_stream`
- `into_inner`
- `as_inner`
- `as_inner_mut`

Current API direction: prefer explicit conversion and access methods over `Deref` and `DerefMut`.
That keeps the facade boundary clearer and avoids making `cjlib` look like it owns the entire `cityjson-rs` surface.

### 3. Keep version classification in `cjlib`

`cjlib` should keep its own boundary-facing version enum:

```rust
pub enum CityJSONVersion {
    V1_0,
    V1_1,
    V2_0,
}
```

This enum should preserve the current string normalization behavior:

- `V1_0`: `1.0`, `1.0.0`, `1.0.1`, `1.0.2`, `1.0.3`
- `V1_1`: `1.1`, `1.1.0`, `1.1.1`, `1.1.2`, `1.1.3`
- `V2_0`: `2.0`, `2.0.0`, `2.0.1`

This enum exists only to classify incoming data and dispatch the correct import path.

Do not add legacy in-memory model types back into `cjlib`.

Per the current requirement, legacy branches should stay explicit `todo!()` for now:

- `CityJSON` v1.0 import path: `todo!()`
- `CityJSON` v1.1 import path: `todo!()`
- legacy streaming import paths: `todo!()`

### 4. Parsing belongs at the format boundary, not in the model crate

`cityjson-rs` intentionally does not own JSON de/serialization.

So the clean dependency direction is:

- `cityjson-rs`: in-memory model and correctness-critical model operations
- format crate (`serde_cityjson` today, or its successor): JSON parsing / serialization / version-aware boundary logic
- `cjlib`: facade that combines the two

Important constraint: `cjlib` should not grow a second tree of JSON-specific model structs just to bridge the crates.

If the current format crate cannot yet produce `cityjson::v2_0::OwnedCityModel` directly, the preferred solutions are:

1. add the conversion in the format crate, or
2. add a tiny dedicated adapter layer whose only job is converting parsed format models into `cityjson-rs` models

The least desirable option is rebuilding the old `cjlib` wrapper model again.

### 5. `from_stream` stays, but becomes stricter

The preserved stream behavior should be:

- line-delimited input
- first non-empty item must be a `CityJSON` object
- remaining non-empty items must be `CityJSONFeature` objects
- the sequence version must stay consistent
- duplicate IDs and conflicting root-level state must be explicit errors

For now:

- v2.0 stream path should be implemented cleanly
- v1.0/v1.1 stream paths should exist as branches and remain `todo!()`

No lossy or "best effort" feature merging should be accepted just to keep the old API name alive.

## Proposed Crate Shape

```text
src/
  lib.rs
  model.rs
  io.rs
  version.rs
  error.rs
```

Suggested responsibilities:

- `lib.rs`: public exports and module wiring
- `model.rs`: `CityModel` newtype and thin ergonomic methods
- `io.rs`: `from_slice`, `from_file`, `from_stream`, header sniffing, dispatch
- `version.rs`: `CityJSONVersion`, parsing, display, classifier helpers
- `error.rs`: small `cjlib::Error`

Everything currently in:

- `src/attributes.rs`
- `src/boundary.rs`
- `src/cityobject.rs`
- `src/extensions.rs`
- `src/geometry.rs`
- `src/metadata.rs`
- `src/resource_pool.rs`
- `src/transform.rs`

should be deleted or reduced to re-exports once the facade is in place.

## Proposed Public API Direction

### Default path for users

Users should primarily interact with:

- `cjlib::CityModel`
- `cjlib::CityJSONVersion`
- re-exported `cityjson` types needed to work with the model

### Advanced path

Re-export the `cityjson` crate, or at minimum re-export the relevant `cityjson::v2_0` types, so advanced users can drop down to the underlying API when they need finer control.

### FFI path

Future FFI crates should bind to `cjlib`'s owned facade, not directly to the generic `cityjson-rs` model with storage/index parameters.

That gives FFI one stable integration layer while keeping the real domain model in one place.

## Execution Phases

### Phase 0: Align the Baseline

- Bump `cjlib` to Rust 2024 and `rust-version` compatible with `cityjson-rs`.
- Replace the current dependency story with the intended one:
  - `cityjson-rs` for the model
  - one format-boundary crate for JSON parsing/serialization
- Decide whether the conversion from parsed JSON to `cityjson-rs` lives in:
  - the existing format crate, or
  - a tiny adapter module/crate
- Treat all existing docs/tests as requirement hints, not as code to preserve.

Deliverable: an empty but compiling facade skeleton that matches the new dependency direction.

### Phase 1: Build the Minimal Facade

- Introduce the new `CityModel` newtype around `cityjson::v2_0::OwnedCityModel`.
- Add `new`, `into_inner`, `as_inner`, `as_inner_mut`.
- Add `AsRef`, `AsMut`, and `From<OwnedCityModel>`.
- Re-export the minimum useful `cityjson` surface.
- Introduce a tiny `Error` type and the `CityJSONVersion` enum.

Deliverable: `cjlib` is structurally small and no longer owns its own domain model.

### Phase 2: Implement Version Detection and `from_slice`

- Implement header sniffing for `type` and `version`.
- Dispatch by `(type, version)`.
- `CityJSON` + `V2_0` -> parse and convert into `cityjson::v2_0::OwnedCityModel`
- `CityJSON` + `V1_0` -> `todo!()`
- `CityJSON` + `V1_1` -> `todo!()`
- `CityJSONFeature` passed to `from_slice` -> explicit error
- missing version -> explicit error

Deliverable: full-document v2.0 import from memory is correct and tested.

### Phase 3: Implement `from_file`

- Keep the historical convenience behavior:
  - `.json`, `.city.json`, `.cityjson` -> full document path
  - `.jsonl`, `.city.jsonl` -> stream path
- Unknown extensions should either:
  - fall back to full-document parsing, or
  - error deterministically

Recommendation: keep the old fallback-to-document behavior only if it is tested and still useful. Otherwise prefer explicitness.

Deliverable: file import works without duplicating parsing logic already present in `from_slice` / `from_stream`.

### Phase 4: Implement `from_stream`

- Parse line by line.
- Skip leading and interstitial blank lines deliberately.
- First non-empty line must be `CityJSON`.
- Remaining lines must be `CityJSONFeature`.
- Validate version consistency before merging.
- Merge features into the owned v2.0 model using explicit, tested rules.

If this step reveals missing conversion support in the format-boundary crate, add it there first instead of compensating with ad hoc JSON manipulation in `cjlib`.

Deliverable: JSONL / feature-stream import is correct, deterministic, and strict.

### Phase 5: Delete Dead Surface Area

- Remove the old wrapper modules and unused exports.
- Remove stale tests that describe behavior the new crate no longer owns.
- Remove stale docs and replace them with small, accurate examples.
- Remove dependencies that only existed for the old wrapper model.

Deliverable: the crate becomes obviously small when you open `src/`.

### Phase 6: Rebuild Docs and Tests From Behavior

- Rewrite `README.md` around the facade design.
- Rewrite examples to show:
  - constructing a new model
  - reading from bytes
  - reading from file
  - reading a feature stream
  - dropping down to the underlying `cityjson-rs` API
- Rebuild tests as integration tests around actual public behavior.

Deliverable: docs and tests describe the real crate again.

## Test Strategy

Correctness in CityJSON handling is the priority, so the tests should be integration-heavy and version-aware.

### 1. Constructor coverage

- `from_slice` on valid v2.0 root documents
- `from_file` on full-document fixtures
- `from_file` on JSONL fixtures
- `from_stream` on valid feature streams

### 2. Version and type classification

- all accepted version aliases map to the expected enum variant
- missing `version` errors clearly
- unsupported `type` errors clearly
- `CityJSONFeature` passed to the wrong constructor errors clearly

### 3. Legacy branch locking

Until legacy support is implemented, add explicit tests that assert the current temporary behavior for v1.0 and v1.1 branches.

That prevents accidental silent fallback behavior.

### 4. Cross-crate integration tests

For supported v2.0 fixtures:

- parse through the format-boundary layer
- build the `cityjson-rs` model through `cjlib`
- assert semantic equivalence on the resulting model state

### 5. Stream aggregation tests

- first line not `CityJSON` -> error
- duplicate city object IDs -> error
- mixed versions in a stream -> error
- blank lines handling
- stream result matches equivalent fully aggregated fixture

### 6. Documentation tests

Every example in `README.md` and crate docs should compile.

## Documentation Positioning

The new docs should say this plainly:

- `cityjson-rs` is the in-memory model
- `cjlib` is the ergonomic facade
- legacy version recognition exists at the boundary
- only `CityJSON` v2.0 is backed by the real in-memory model today
- legacy import branches are intentionally `todo!()` for now

Do not repeat the old aspiration of mapping the full CityJSON data model separately inside `cjlib`.

## Non-Goals

The rewrite should explicitly avoid:

- preserving the current bespoke wrapper modules
- keeping API compatibility with outdated types
- reintroducing `v1_0` / `v1_1` in-memory model trees into `cjlib`
- adding convenience setters/getters that already exist in `cityjson-rs`
- updating FFI crates before the facade is stable

## Done Criteria

The refactor is done when all of the following are true:

- `cjlib` uses `cityjson-rs` as its only in-memory model
- `cjlib::CityModel` is a thin owned facade, not a second model tree
- `CityModel::from_slice`, `from_file`, and `from_stream` exist and work for v2.0
- `CityJSONVersion` and version branching logic are preserved in `cjlib`
- legacy version branches exist and are explicitly `todo!()`
- docs and tests are rewritten around the new facade
- the remaining codebase is materially smaller and easier to maintain than the current crate

## Recommendation

Do the rewrite in one deliberate pass instead of trying to preserve the current modules.

The clean target is:

- one real model crate: `cityjson-rs`
- one format-boundary crate: `serde_cityjson` or successor
- one user-facing facade: `cjlib`

That is the smallest design that still preserves the constructor conveniences, the version-dispatch boundary, and a stable place for future FFI integration.
