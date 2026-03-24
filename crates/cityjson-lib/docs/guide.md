# Guide to Using cjlib

This guide describes the intended public API of `cjlib`.
It is a contract for the rewrite, not a description of temporary implementation details.

## Start With `CityModel`

The normal entry point should be `cjlib::CityModel`.

```rust
use cjlib::CityModel;

let model = CityModel::from_file("amsterdam.city.json")?;
println!("loaded {} CityObjects", model.cityobjects().len());
# Ok::<(), cjlib::Error>(())
```

`CityModel` should remain the ergonomic default for the CityJSON JSON path:

- `from_slice` for bytes already in memory
- `from_file` for file-based import
- `from_stream` for `CityJSON` plus `CityJSONFeature` streams

## Read a `CityJSONFeature` Stream

`from_stream` is the high-level convenience path for strict JSONL aggregation.

```rust
use std::fs::File;
use std::io::BufReader;

use cjlib::CityModel;

let reader = BufReader::new(File::open("tiles.city.jsonl")?);
let model = CityModel::from_stream(reader)?;
# Ok::<(), cjlib::Error>(())
```

The intended semantics stay strict:

- the first non-empty value must be `CityJSON`
- remaining values must be `CityJSONFeature`
- all versions must agree
- conflicting IDs or incompatible root state are errors

## Use Explicit Format Modules When You Need Boundary Control

The top-level `CityModel::from_*` methods should stay reserved for the common CityJSON JSON path.

When the caller wants explicit format handling, the API should move into modules:

```rust
use cjlib::{json, CityJSONVersion};

let bytes = std::fs::read("amsterdam.city.json")?;
let probe = json::probe(&bytes)?;
assert_eq!(probe.kind(), json::RootKind::CityJSON);
assert_eq!(probe.version(), Some(CityJSONVersion::V2_0));

let model = json::from_slice(&bytes)?;
# Ok::<(), cjlib::Error>(())
```

The same pattern should scale to sibling transport crates:

- `cjlib::arrow`
- `cjlib::parquet`

The explicit JSON module should also own serialization:

```rust
use cjlib::{json, CityModel};

let model = CityModel::from_file("amsterdam.city.json")?;
let bytes = json::to_vec(&model)?;
let text = json::to_string(&model)?;

let mut writer = Vec::new();
json::to_writer(&mut writer, &model)?;
# let _ = (bytes, text, writer);
# Ok::<(), cjlib::Error>(())
```

## Drop Down To `cityjson-rs` For Model Work

`cjlib` should not proxy the entire model API.
Once the model is loaded, most interaction should happen through the re-exported `cityjson-rs` types.

```rust
use cjlib::{CityModel, CityModelType};

let model = CityModel::new(CityModelType::CityJSON);
let inner: &cjlib::cityjson::v2_0::OwnedCityModel = model.as_inner();
let _ = inner;
```

This is the key design point:

- `cjlib` owns the entry points
- `cityjson-rs` owns the model

## Error Handling

The public error API should be structured enough for callers to branch on categories without matching strings.

```rust
use cjlib::{json, ErrorKind};

let error = json::from_slice(br#"{"type":"CityJSON","CityObjects":{},"vertices":[]}"#).unwrap_err();
assert_eq!(error.kind(), ErrorKind::Version);
```

The exact display text may evolve, but the category-level contract should be stable.

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
