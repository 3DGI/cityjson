# Model Boundary API

This document pins down the intended boundary between `cjlib::CityModel` and `cjlib::cityjson`.

The goal is to keep `cjlib` small and explicit.
`CityModel` should be a thin owned wrapper, not a shadow copy of the full `cityjson-rs` API.

## Intended Surface

The intended wrapper surface is:

```rust,ignore
impl CityModel {
    pub fn from_slice(bytes: &[u8]) -> crate::Result<Self>;
    pub fn from_file(path: impl AsRef<std::path::Path>) -> crate::Result<Self>;
    pub fn from_stream(reader: impl std::io::BufRead) -> crate::Result<Self>;

    pub fn as_inner(&self) -> &crate::cityjson::v2_0::OwnedCityModel;
    pub fn as_inner_mut(&mut self) -> &mut crate::cityjson::v2_0::OwnedCityModel;
    pub fn into_inner(self) -> crate::cityjson::v2_0::OwnedCityModel;
}

impl AsRef<crate::cityjson::v2_0::OwnedCityModel> for CityModel;
impl AsMut<crate::cityjson::v2_0::OwnedCityModel> for CityModel;
impl From<crate::cityjson::v2_0::OwnedCityModel> for CityModel;
```

That is enough for the facade.

## Why No `Deref`

The wrapper should not rely on `Deref` or `DerefMut`.

`Deref` makes the boundary blurry:

- it makes `cjlib` look like it owns far more of the model API than it really should
- it encourages root-level API sprawl
- it makes it harder to evolve the wrapper without surprising users

Explicit access is clearer:

```rust,ignore
let mut model = cjlib::CityModel::from_file("amsterdam.city.json")?;
let inner = model.as_inner();
let _ = inner;
let inner_mut = model.as_inner_mut();
# let _ = inner_mut;
```

## Why Re-export The Crate, Not Lots Of Items

The advanced access path should be:

```rust,ignore
use cjlib::cityjson;
```

That is cleaner than re-exporting many `cityjson-rs` items individually at the `cjlib` root.
It keeps the `cjlib` namespace small and avoids a long-term maintenance burden around selective re-exports.
