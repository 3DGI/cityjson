# Model Boundary API

This document pins down the boundary between `cityjson_lib::CityModel` and
`cityjson_lib::cityjson`.

`CityModel` should stay small and explicit.
It is a wrapper, not a shadow copy of the whole `cityjson-rs` API.

## Stable Shape

The stable contract is:

- `CityModel` is owned by default
- document-oriented constructors live on the type itself
- the underlying model is available through explicit accessors
- stream APIs do not appear as inherent methods

The current Rust shape is intentionally simple:

```rust
impl CityModel {
    pub fn from_slice(bytes: &[u8]) -> crate::Result<Self>;
    pub fn from_file(path: impl AsRef<std::path::Path>) -> crate::Result<Self>;

    pub fn as_inner(&self) -> &crate::cityjson::v2_0::OwnedCityModel;
    pub fn as_inner_mut(&mut self) -> &mut crate::cityjson::v2_0::OwnedCityModel;
    pub fn into_inner(self) -> crate::cityjson::v2_0::OwnedCityModel;
}
```

The exact `cityjson-rs` instantiation behind the wrapper is an implementation
choice. The durable boundary is the owned wrapper plus the explicit conversion
points.

`CityModel` is also the same wrapper type for:

- a full document
- a grouped subset
- a feature-sized self-contained model

## Why No `Deref`

`Deref` and `DerefMut` blur the boundary in the wrong direction:

- they make `cityjson_lib` look larger than it is
- they encourage root-level method sprawl
- they make later wrapper changes harder to reason about

Explicit access is clearer:

```rust
let mut model = cityjson_lib::CityModel::from_file("amsterdam.city.json")?;
let inner = model.as_inner();
let inner_mut = model.as_inner_mut();
# let _ = (inner, inner_mut);
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
