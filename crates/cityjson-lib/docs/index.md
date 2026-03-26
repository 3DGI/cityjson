# cjlib

`cjlib` is the integration crate for the CityJSON ecosystem in this repository.

It is not a second CityJSON domain model.
The intended architecture is:

- one semantic model: `cityjson::v2_0::OwnedCityModel`
- one facade wrapper: `cjlib::CityModel`
- many format boundaries: `serde_cityjson`, future Arrow and Parquet crates,
  and explicit raw/staged boundary APIs where needed

The responsibility split is:

- `cityjson-rs`: normalized in-memory model, invariants, extraction, and merge
- `serde_cityjson`: CityJSON JSON and JSONL parsing, feature handling,
  staged/raw JSON boundary work, and serialization
- `cjlib`: user-facing convenience API, explicit format modules, version
  dispatch, and future FFI boundary
- `cjfake`: generator crate above `cjlib`, not part of the facade

For the full cross-crate synthesis, see [Architecture](architecture.md).

## Public API Shape

The future public API is intentionally small.

### Primary entry points

- `cjlib::CityModel`
- `cjlib::CityJSONVersion`
- `cjlib::Error`
- `cjlib::ErrorKind`
- `cjlib::ops`
- `cjlib::cityjson`

### Default JSON path

These stay as the ergonomic default for loading one CityJSON document:

- `CityModel::from_slice`
- `CityModel::from_file`

### Explicit format modules

Formats beyond the default single-document CityJSON path should be explicit:

- `cjlib::json`
- `cjlib::arrow`
- `cjlib::parquet`

The design goal is:

- top-level methods for the common single-document CityJSON path
- module-qualified methods for format-specific behavior, model-stream APIs, and
  future raw/lazy access

For JSON, that explicit boundary module should own:

- probing
- document parsing
- feature parsing
- model-stream reading
- strict stream aggregation when a larger document must be rebuilt
- serialization
- future raw/staged access

For Arrow and Parquet, file-oriented helpers are fine, but the semantic rule
stays the same:

- read or write one `CityModel`
- or read or write streams of `CityModel` values

`cjlib` should not surface Arrow batches or Parquet row groups as semantic
application types.

## Higher-level Operations

Application-level workflows that do not belong in the `cityjson-rs` core model
should live under `cjlib::ops`.

That namespace is the intended home for:

- LoD filtering
- version upgrade helpers
- vertex cleanup
- texture path updates
- geometry measurements such as surface area and volume
- feature-gated CRS helpers

Core submodel extraction and merge semantics should stay owned by
`cityjson-rs`.
`cjlib::ops` may wrap those capabilities, but it should not redefine them.

## Working Model

`cjlib::CityModel` should remain a thin owned wrapper over
`cityjson::v2_0::OwnedCityModel`.
That wrapper may represent a full document or a smaller self-contained package.
The facade should add only:

- constructor convenience
- version classification
- a small error surface
- feature-gated format integration

Everything else should come from `cityjson-rs`, accessed explicitly through
`cjlib::cityjson`.
The wrapper boundary should stay explicit with `as_inner`, `as_inner_mut`,
`into_inner`, `AsRef`, and `AsMut`.
It should not rely on `Deref` to blur the boundary.

## User Experience

For most users, the expected workflow should be:

1. read a CityJSON document with `CityModel::from_*`
2. drop down to `cjlib::json`, `cjlib::arrow`, or `cjlib::parquet` when
   explicit boundary control or model streams are needed
3. access the inner model explicitly, then work with `cjlib::cityjson`
4. use `cjlib::ops` for higher-level reusable workflows

## Documentation Structure

The docs site should work for more than just the Rust crate surface.
`cjlib` is intended to become a multi-language entry point through FFI, so the
documentation should stay split by responsibility:

- overview and guides for the common `cjlib` entry points
- architecture and API design pages for the facade itself
- FFI and bindings pages for language-neutral concepts and future bindings
- engineering notes for implementation plans and internal decisions

That is why the project uses MkDocs for the main site rather than relying on
Rust-only generated docs.

## Non-goals

The future `cjlib` API should not:

- reintroduce a second in-memory CityJSON model
- expose indexed JSON internals as the normal user-facing API
- duplicate parsing or conversion logic that belongs in `serde_cityjson`
- absorb storage invariants that belong in `cityjson-rs`
- absorb `cjfake` into the root facade API
- hide format choice behind a generic registry or extension-sniffing dispatcher
- require callers to match on error strings for normal control flow
