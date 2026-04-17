# Model Boundary API

This document pins down the boundary between `cityjson_lib::CityModel` and
`cityjson_lib::cityjson`.

`CityModel` should stay small and explicit.
It is the owned semantic model re-exported by this crate, not a second wrapper
layer.

## Stable Shape

The stable contract is:

- `CityModel` is the owned `cityjson-rs` model type used by this facade
- document-oriented constructors live in explicit modules such as
  `cityjson_lib::json`
- stream APIs do not appear as inherent methods
- format-specific helpers stay in explicit sibling modules

The current Rust shape is intentionally simple:

```rust
pub use cityjson::v2_0::OwnedCityModel as CityModel;
```

The durable boundary is the direct model re-export plus the explicit module
boundaries for JSON and operations.

`CityModel` is also the same owned type for:

- a full document
- a grouped subset
- a feature-sized self-contained model

## Why No Wrapper Layer

An extra wrapper struct did not add durable semantics. It only introduced
wrapper-specific constructors and conversion methods that obscured the actual
model boundary.

Keeping `CityModel` as a direct re-export is clearer:

```rust
let model = cityjson_lib::json::from_file("amsterdam.city.json")?;
let cityobject_count = model.cityobjects().len();
# let _ = cityobject_count;
```

## Why Re-export `cityjson-rs`

The advanced path should be:

```rust
use cityjson_lib::cityjson;
```

That is cleaner than re-exporting a long list of model items at the crate root.

## What Does Not Belong Here

The following do not belong as inherent `CityModel` methods:

- stream-oriented loaders
- raw or staged JSON readers
- backend-specific transport helpers
- large workflow method bags

Those belong in explicit modules such as `cityjson_lib::json`, sibling format modules,
or `cityjson_lib::ops`.
