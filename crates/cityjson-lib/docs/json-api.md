# JSON API

This document defines the intended public shape of `cjlib::json`.

The role of `cjlib::json` is not to be a second JSON implementation.
It is the explicit boundary layer over `serde_cityjson` for callers that want control over probing, parsing, and serialization.

## Why A `json` Module Exists

`CityModel::from_slice`, `from_file`, and `from_stream` should stay as the ergonomic default path.

But the crate also needs a place for JSON-specific operations that should not clutter `CityModel`, such as:

- probing the root type and version
- explicit JSON parsing functions
- serialization

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
    pub fn from_stream(reader: impl std::io::BufRead) -> crate::Result<crate::CityModel>;

    pub fn to_vec(model: &crate::CityModel) -> crate::Result<Vec<u8>>;
    pub fn to_string(model: &crate::CityModel) -> crate::Result<String>;
    pub fn to_writer(
        writer: &mut impl std::io::Write,
        model: &crate::CityModel,
    ) -> crate::Result<()>;
}
```

## Relationship To `CityModel`

The intended relationship is:

- `CityModel::from_slice` is a convenience alias for `json::from_slice`
- `CityModel::from_file` is a convenience alias for `json::from_file`
- `CityModel::from_stream` is a convenience alias for `json::from_stream`

Serialization does not belong on `CityModel`.
That should remain explicit and format-qualified:

- `json::to_vec`
- `json::to_string`
- `json::to_writer`

The module should stay function-oriented.
It does not need reader or writer builder types, extension-sniffing helpers, or a second public serde model.

## Probe Instead Of Ad Hoc Helpers

The probing API should be one small object instead of several unrelated helper functions.

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

`probe` is simpler for callers and avoids reparsing the same header multiple times.

For the same reason, the probe surface should stay narrow:

- `probe(bytes)` is enough
- avoid `probe_file`
- avoid `probe_reader`
