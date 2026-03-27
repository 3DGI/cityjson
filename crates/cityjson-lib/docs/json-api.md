# JSON API

This document defines the intended public shape of `cjlib::json`.

The role of `cjlib::json` is not to be a second JSON implementation.
It is the explicit boundary layer over `serde_cityjson` for callers that want
control over probing, parsing, and serialization.

## Why A `json` Module Exists

`CityModel::from_slice` and `from_file` should stay as the ergonomic default
single-document path.

But the crate also needs a place for JSON-specific operations that should not
clutter `CityModel`, such as:

- probing the root type and version
- explicit document parsing
- explicit feature parsing
- model-stream reading
- model-stream writing
- serialization
- future raw or staged read paths

That place should be `cjlib::json`.

## Intended Surface

The target API is:

```rust
pub mod json {
    pub enum RootKind {
        CityJSON,
        CityJSONFeature,
    }

    pub struct Probe {
        /* private fields */
    }

    impl Probe {
        pub fn kind(&self) -> RootKind;
        pub fn version(&self) -> Option<crate::CityJSONVersion>;
    }

    pub fn probe(bytes: &[u8]) -> crate::Result<Probe>;

    pub fn from_slice(bytes: &[u8]) -> crate::Result<crate::CityModel>;
    pub fn from_file(path: impl AsRef<std::path::Path>) -> crate::Result<crate::CityModel>;
    pub fn from_feature_slice(bytes: &[u8]) -> crate::Result<crate::CityModel>;
    pub fn read_feature_stream<R>(
        reader: R,
    ) -> crate::Result<impl Iterator<Item = crate::Result<crate::CityModel>>>
    where
        R: std::io::BufRead;
    pub fn write_feature_stream<I, W>(writer: W, models: I) -> crate::Result<()>
    where
        I: IntoIterator<Item = crate::CityModel>,
        W: std::io::Write;

    pub fn to_vec(model: &crate::CityModel) -> crate::Result<Vec<u8>>;
    pub fn to_string(model: &crate::CityModel) -> crate::Result<String>;
    pub fn to_writer(
        writer: &mut impl std::io::Write,
        model: &crate::CityModel,
    ) -> crate::Result<()>;
    pub fn to_feature_string(model: &crate::CityModel) -> crate::Result<String>;
}
```

## Relationship To `CityModel`

The intended relationship is:

- `CityModel::from_slice` is a convenience alias for `json::from_slice`
- `CityModel::from_file` is a convenience alias for `json::from_file`

Serialization does not belong on `CityModel`.
That should remain explicit and format-qualified:

- `json::to_vec`
- `json::to_string`
- `json::to_writer`
- `json::to_feature_string`
- `json::write_feature_stream`

The module should stay function-oriented.
It does not need reader or writer builder types, extension-sniffing helpers, or
a second public serde model.

There is no `CityModel::from_stream` compatibility alias in the current API.
Callers should use `read_feature_stream` explicitly when they want a JSONL
feature stream.

## Probe Instead Of Ad Hoc Helpers

The probing API should be one small object instead of several unrelated helper
functions.

Preferred:

```rust
let probe = cjlib::json::probe(bytes) ?;
let kind = probe.kind();
let version = probe.version();
```

Not preferred:

```rust
let probe = cjlib::json::probe(bytes) ?;
let kind = probe.kind();
let version = probe.version();
```

`probe` is simpler for callers and avoids reparsing the same header multiple
times.

For the same reason, the probe surface should stay narrow:

- `probe(bytes)` is enough
- avoid `probe_file`
- avoid `probe_reader`

## `CityJSONFeature` Is A Boundary Form, Not A Semantic Type

The JSON module needs to deal with `CityJSONFeature`, but only as a wire-format
concern.

The returned semantic unit should still be `crate::CityModel`.
That keeps the semantic architecture aligned with the rest of the ecosystem:

- one model
- or a stream of models
- format differences only at the boundary

## Leave Room For Raw And Staged APIs

If the ecosystem later exposes lower-level JSON access, that should happen
explicitly in this module.
Examples include:

- `RawDocument`
- `from_slice_raw`
- staged section readers

Those are valuable advanced tools, but they should not distort the default
owned `CityModel` path.
