# Guide to Using cjlib

This guide describes the intended public API of `cjlib`.
It is a contract for the rewrite, not a description of temporary
implementation details.

## Start With `CityModel`

The normal entry point should be `cjlib::CityModel`.

```rust
use cjlib::CityModel;

let model = CityModel::from_file("amsterdam.city.json")?;
println!("loaded {} CityObjects", model.as_inner().cityobjects().len());
# Ok::<(), cjlib::Error>(())
```

`CityModel` should remain the ergonomic default for loading one CityJSON
document:

- `from_slice` for bytes already in memory
- `from_file` for file-based import

The same wrapper type may represent:

- a whole document
- a grouped subset
- a single-feature-sized self-contained model

That is a semantic scope difference, not a type difference.

## Use The Explicit JSON Module For Boundary Control

The top-level `CityModel::from_*` methods should stay reserved for the common
single-document path.
When the caller wants probing, feature-level handling, or model streams, the
API should move into `cjlib::json`.

```rust
use cjlib::{json, CityJSONVersion};

let bytes = std::fs::read("amsterdam.city.json")?;
let probe = json::probe(&bytes)?;
assert_eq!(probe.kind(), json::RootKind::CityJSON);
assert_eq!(probe.version(), Some(CityJSONVersion::V2_0));

let model = json::from_slice(&bytes)?;
# Ok::<(), cjlib::Error>(())
```

## Read Or Merge A `CityJSONFeature` Stream

The JSON boundary should support both streaming and aggregation.

When the caller wants one semantic submodel at a time:

```rust
use std::fs::File;
use std::io::BufReader;

let reader = BufReader::new(File::open("tiles.city.jsonl")?);
let models = cjlib::json::read_feature_stream(reader)?;
for model in models {
    let model = model?;
    let _ = model;
}
# Ok::<(), cjlib::Error>(())
```

When the caller wants to rebuild one larger model from a strict stream:

```rust
use std::fs::File;
use std::io::BufReader;

let reader = BufReader::new(File::open("tiles.city.jsonl")?);
let model = cjlib::json::merge_feature_stream(reader)?;
# let _ = model;
# Ok::<(), cjlib::Error>(())
```

The strict aggregation rules should stay:

- the first non-empty value must be `CityJSON`
- remaining values must be `CityJSONFeature`
- all versions must agree
- conflicting IDs or incompatible root state are errors

## Use Explicit Format Modules When You Need Boundary Control

The same pattern should scale to sibling transport crates:

- `cjlib::arrow`
- `cjlib::parquet`

File-oriented helpers are fine, but the semantic rule should remain the same:

```rust
# fn main() -> cjlib::Result<()> {
#[cfg(feature = "arrow")]
{
    let model = cjlib::arrow::from_file("tiles.cjarrow")?;
    cjlib::arrow::to_file("tiles-out.cjarrow", &model)?;
}

#[cfg(feature = "parquet")]
{
    let model = cjlib::parquet::from_file("tiles.cjparquet")?;
    cjlib::parquet::to_file("tiles-out.cjparquet", &model)?;
}
# Ok(())
# }
```

Where the backend format naturally supports streams, the item type should still
be `cjlib::CityModel`, not a format-specific semantic object.

The explicit JSON module should also own serialization:

```rust
use cjlib::{json, CityModel};

let model = CityModel::from_file("amsterdam.city.json")?;
let bytes = json::to_vec(&model)?;
let text = json::to_string(&model)?;

let mut writer = Vec::new();
json::to_writer(&mut writer, &model)?;
let feature_text = json::to_feature_string(&model)?;
# let _ = (bytes, text, writer, feature_text);
# Ok::<(), cjlib::Error>(())
```

If lower-level JSON access becomes important later, it should be added here as
an explicit raw or staged API rather than folded into `CityModel`.

## Drop Down To `cityjson-rs` For Model Work

`cjlib` should not proxy the entire model API.
Once the model is loaded, most interaction should happen through the
re-exported `cityjson-rs` crate.
That boundary should stay explicit rather than relying on `Deref`.

```rust
let inner =
    cjlib::cityjson::v2_0::OwnedCityModel::new(cjlib::cityjson::CityModelType::CityJSON);
let mut model = cjlib::CityModel::from(inner);

let borrowed: &cjlib::cityjson::v2_0::OwnedCityModel = model.as_inner();
let _ = borrowed;
let borrowed_mut: &mut cjlib::cityjson::v2_0::OwnedCityModel = model.as_inner_mut();
let _ = borrowed_mut;
let owned: cjlib::cityjson::v2_0::OwnedCityModel = model.into_inner();
# let _ = owned;
```

This is the key design point:

- `cjlib` owns the entry points
- `cityjson-rs` owns the model
- `cjlib::cityjson` is the advanced access path

## Error Handling

The public error API should be structured enough for callers to branch on
categories without matching strings.

```rust
use cjlib::{json, ErrorKind};

let error = json::from_slice(br#"{"type":"CityJSON","CityObjects":{},"vertices":[]}"#).unwrap_err();
assert_eq!(error.kind(), ErrorKind::Version);
```

The exact display text may evolve, but the category-level contract should be
stable.
The intended stable categories are:

- `ErrorKind::Io`
- `ErrorKind::Syntax`
- `ErrorKind::Version`
- `ErrorKind::Shape`
- `ErrorKind::Unsupported`
- `ErrorKind::Model`

## Alternative Formats

The intended API for non-JSON formats is explicit and feature-gated.

```rust
# fn main() -> cjlib::Result<()> {
#[cfg(feature = "arrow")]
let arrow_model = cjlib::arrow::from_file("buildings.cjarrow")?;

#[cfg(feature = "parquet")]
let parquet_model = cjlib::parquet::from_file("buildings.cjparquet")?;
# Ok(())
# }
```

That keeps the crate easy to teach:

- CityJSON JSON: `CityModel::from_*`
- explicit JSON boundary work: `cjlib::json::*`
- alternate encodings: dedicated format modules

## Use `ops` For Higher-level Workflows

Operations that are useful to applications, but do not belong in the
`cityjson-rs` core model, should live under `cjlib::ops`.

```rust
use cjlib::{ops, CityModel};

let mut model = CityModel::from_file("amsterdam.city.json")?;
let selection = ops::Selection::from_ids(["building-1"]);
let subset = ops::subset(&model, selection)?;
let merged = ops::merge([model, subset])?;
let _surface_area = ops::geometry::surface_area(&merged, "building-1")?;
# Ok::<(), cjlib::Error>(())
```

The intended split is:

- `cityjson-rs` for authoritative extraction and merge semantics
- `ops::merge`, `ops::subset`, `ops::upgrade` as optional ergonomic wrappers
  over those model capabilities
- `ops::lod`, `ops::vertices`, `ops::textures` for maintenance-style operations
- `ops::geometry` for measurements
- feature-gated `ops::crs` for CRS assignment and reprojection

This keeps `CityModel` from turning into a catch-all method bag.

## Keep `cjfake` Above The Facade

`cjfake` should not become `cjlib::fake`.

The cleaner ecosystem layering is:

- `cjfake` generates `cityjson-rs`-compatible model data
- `cjfake` uses `cjlib` to write JSON, Arrow, Parquet, or future formats
- `cjlib` stays focused on format integration and reusable operations

That way new formats can become available to `cjfake` simply by landing in
`cjlib`, without making fake-data generation part of the facade itself.
