# cjlib

`cjlib` is the user-facing facade for the CityJSON crates in this repository.

The current rewrite is deliberately narrow:

- `cityjson-rs` owns the semantic model
- `serde_cityjson` owns the JSON boundary
- `cjlib` owns the thin facade, explicit JSON helpers, version dispatch, and a
  small public error surface

Unfinished areas are intentionally explicit.
Illustrative modules such as `ops`, `arrow`, and `parquet` expose one
unimplemented function each so the intended boundary is visible without
pretending the implementation exists.

## Core Surface

The implemented core surface is:

- `CityModel`
- `CityJSONVersion`
- `Error`
- `ErrorKind`
- `json`
- `cityjson`

`CityModel` is a thin owned wrapper around
`cityjson::v2_0::OwnedCityModel`.
The wrapper boundary stays explicit and does not use `Deref`.

```rust
use cjlib::CityModel;

let from_file = CityModel::from_file("tests/data/v2_0/minimal.city.json")?;
let from_slice = CityModel::from_slice(
    br#"{"type":"CityJSON","version":"2.0","CityObjects":{},"vertices":[]}"#,
)?;

let borrowed = from_file.as_inner();
let _owned = from_slice.into_inner();
let _ = borrowed;
# Ok::<(), cjlib::Error>(())
```

## JSON Boundary

`cjlib::json` is the only implemented explicit format module.

It owns:

- probing
- document parsing
- feature parsing
- feature-stream reading
- document serialization
- feature serialization
- feature-stream writing

```rust
use cjlib::{json, CityJSONVersion};

let bytes = std::fs::read("tests/data/v2_0/minimal.city.json")?;
let probe = json::probe(&bytes)?;
assert_eq!(probe.kind(), json::RootKind::CityJSON);
assert_eq!(probe.version(), Some(CityJSONVersion::V2_0));

let model = json::from_slice(&bytes)?;
let text = json::to_string(&model)?;
assert!(!text.is_empty());
# Ok::<(), cjlib::Error>(())
```

`json::from_file` is document-oriented.
`CityJSONFeature` streams must go through `json::read_feature_stream`.
The rewrite no longer treats stream aggregation as part of the stable default
surface.

## Illustrative Modules

These modules exist to show intended responsibility boundaries, not to provide
working behavior yet:

- `cjlib::ops::merge`
- `cjlib::arrow::to_file`
- `cjlib::parquet::to_file`

Each of those functions is intentionally `todo!()` so unfinished work is
visible in both the code and the test suite.

That way `cjfake` can generate `cityjson-rs` data and then emit any supported
format by calling the explicit `cjlib` format modules.
`cjlib` stays focused on facade and format integration instead of absorbing
test-data generation concerns.

## Alternative Format Modules

Arrow and Parquet integration should be feature-gated and explicit.

```rust
#[cfg(feature = "arrow")]
let model = cjlib::CityModel::from_file("tests/data/v2_0/minimal.city.json")?;

#[cfg(feature = "arrow")]
cjlib::arrow::to_file("tiles-out.cjarrow", &model)?;

#[cfg(feature = "parquet")]
let model = cjlib::CityModel::from_file("tests/data/v2_0/minimal.city.json")?;

#[cfg(feature = "parquet")]
cjlib::parquet::to_file("tiles-out.cjparquet", &model)?;
# Ok::<(), cjlib::Error>(())
```

Those modules are part of the intended public shape even if their
implementation lands later than the JSON path. The current code only exposes
explicit `to_file` placeholders for them.
Where those backends expose stream-oriented APIs later, the item type should
still be `cjlib::CityModel`.
