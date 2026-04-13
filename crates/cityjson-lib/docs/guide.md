# Guide to Using cityjson-lib

This guide describes how the Rust-facing `cityjson_lib` surface is meant to be used.

## Start With `CityModel`

The default entry point is `cityjson_lib::CityModel`.

```rust
use cityjson_lib::CityModel;

let model = CityModel::from_file("amsterdam.city.json")?;
println!("loaded {} CityObjects", model.as_inner().cityobjects().len());
# Ok::<(), cityjson_lib::Error>(())
```

Use the root constructor path for the common case:

- `from_slice` for bytes already in memory
- `from_file` for file-based import

`CityModel` may represent a whole document, a subset, or a feature-sized
package. The type stays the same; only the scope changes.

## Use `cityjson_lib::json` For Boundary Control

When callers need probing, feature handling, or explicit stream APIs, move to
`cityjson_lib::json`.

```rust
use cityjson_lib::{json, CityJSONVersion};

let bytes = std::fs::read("amsterdam.city.json")?;
let probe = json::probe(&bytes)?;
assert_eq!(probe.kind(), json::RootKind::CityJSON);
assert_eq!(probe.version(), Some(CityJSONVersion::V2_0));

let model = json::from_slice(&bytes)?;
# Ok::<(), cityjson_lib::Error>(())
```

The JSON module is also where document and feature serialization live.

## Read And Write Feature Streams Explicitly

Feature streams are an explicit boundary concern.

```rust
use std::fs::File;
use std::io::BufReader;

let reader = BufReader::new(File::open("tiles.city.jsonl")?);
for model in cityjson_lib::json::read_feature_stream(reader)? {
    let model = model?;
    let _ = model;
}
# Ok::<(), cityjson_lib::Error>(())
```

Writing follows the same pattern:

```rust
use std::fs::File;
use std::io::BufReader;

let reader = BufReader::new(File::open("tiles.city.jsonl")?);
let models = cityjson_lib::json::read_feature_stream(reader)?
    .collect::<cityjson_lib::Result<Vec<_>>>()?;

let mut writer = Vec::new();
cityjson_lib::json::write_feature_stream(&mut writer, models)?;
# let _ = writer;
# Ok::<(), cityjson_lib::Error>(())
```

The point is to keep JSONL handling explicit instead of hiding it behind a
document-oriented constructor.

## Use Explicit Format Modules

The same rule applies to non-JSON backends:

```rust
# fn main() -> cityjson_lib::Result<()> {
let model = cityjson_lib::CityModel::from_file("tests/data/v2_0/minimal.city.json")?;

cityjson_lib::arrow::to_file("tiles-out.cjarrow", &model)?;

cityjson_lib::parquet::to_file("tiles-out.cjparquet", &model)?;
# Ok(())
# }
```

Format choice stays explicit at the call site. The Arrow path writes one live
Arrow IPC stream file. The Parquet path writes one persistent package file.

## Drop Down To `cityjson_lib::cityjson` For Model Work

`cityjson_lib` does not try to proxy the whole model API.
Once the model is loaded, advanced work should happen through the re-exported
`cityjson-rs` crate.

```rust
let inner =
    cityjson_lib::cityjson::v2_0::OwnedCityModel::new(cityjson_lib::cityjson::CityModelType::CityJSON);
let mut model = cityjson_lib::CityModel::from(inner);

let borrowed = model.as_inner();
let borrowed_mut = model.as_inner_mut();
let owned = model.into_inner();
# let _ = (borrowed, borrowed_mut, owned);
```

That boundary stays explicit on purpose:

- `cityjson_lib` owns entry points and boundary modules
- `cityjson-rs` owns the semantic model
- `cityjson_lib::cityjson` is the advanced path

## Use `ops` For Reusable Workflows

Operations that sit above the semantic model, but are still worth sharing,
belong in `cityjson_lib::ops`.

```rust
use cityjson_lib::{ops, CityModel};

let model = CityModel::from_file("amsterdam.city.json")?;
let selection = ops::Selection::from_ids(["building-1"]);
let subset = ops::subset(&model, selection)?;
let _surface_area = ops::geometry::surface_area(&subset, "building-1")?;
# Ok::<(), cityjson_lib::Error>(())
```

This keeps `CityModel` from turning into a catch-all method bag while still
leaving room for reusable workflows such as filtering, cleanup, measurement,
and upgrade helpers.

## Error Handling

Callers should branch on error categories, not on display text.

```rust
use cityjson_lib::{json, ErrorKind};

let error = json::from_slice(br#"{"type":"CityJSON","CityObjects":{},"vertices":[]}"#).unwrap_err();
assert_eq!(error.kind(), ErrorKind::Version);
```

The stable contract is the category:

- `ErrorKind::Io`
- `ErrorKind::Syntax`
- `ErrorKind::Version`
- `ErrorKind::Shape`
- `ErrorKind::Unsupported`
- `ErrorKind::Model`
