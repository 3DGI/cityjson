# Model Boundary

`CityModel` is a direct re-export of the owned semantic model type from
`cityjson-rs`.

```rust
pub use cityjson::v2_0::OwnedCityModel as CityModel;
```

## What That Means

- `CityModel` is the semantic unit at the Rust boundary
- document parsing does not become an inherent `CityModel` method
- feature parsing does not become a separate semantic type
- workflow helpers stay in `json`, `ops`, and `query`

## What Lives Outside `CityModel`

These concerns are explicit modules, not inherent model methods:

- `json::from_*`, `json::to_*`, and feature-stream handling
- `ops::{cleanup, extract, append, merge}`
- `query::summary`

## Why This Is Deliberate

The crate stays easier to reason about when:

- `cityjson-rs` owns the deep model surface
- `cityjson-lib` owns the small stable facade
- format-aware behavior stays in `cityjson-json`, not on `CityModel`
