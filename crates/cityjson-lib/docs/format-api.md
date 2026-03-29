# Format Module API

This document pins down how `cjlib` should expose sibling format crates.

## Core Rule

The facade should stay explicit:

- `CityModel::from_*` is reserved for the default single-document CityJSON path
- explicit modules own explicit formats
- every format boundary speaks in terms of `CityModel` or streams of
  `CityModel`
- `cjlib` does not grow a generic format registry

## Intended Modules

The public layout is:

```rust
pub mod json;

#[cfg(feature = "arrow")]
pub mod arrow;

#[cfg(feature = "parquet")]
pub mod parquet;
```

Those modules delegate to backend crates such as `serde_cityjson`, `cityarrow`,
and `cityparquet`.

## JSON Is Richer

`cjlib::json` is the richest boundary module because it has to cover:

- probing
- document parsing
- feature parsing
- feature-stream reading and writing
- document and feature serialization
- future raw or staged JSON access

Sibling transport modules can stay smaller as long as they follow the same
semantic rule.

## One Semantic Unit Across Formats

Arrow and Parquet must not invent separate semantic units at the `cjlib`
boundary.
Even if the backend format uses batches, row groups, or other transport-native
chunks internally, the public facade still trades in:

- one `CityModel`
- or streams of `CityModel` values

File-oriented helpers are fine as conveniences:

```rust
#[cfg(feature = "arrow")]
pub mod arrow {
    pub fn from_file(path: impl AsRef<std::path::Path>) -> crate::Result<crate::CityModel>;
    pub fn to_file(
        path: impl AsRef<std::path::Path>,
        model: &crate::CityModel,
    ) -> crate::Result<()>;
}
```

The same idea applies to Parquet and future backends.

## No Generic `read` / `write`

Avoid APIs such as:

- `cjlib::read(path)`
- `cjlib::write(path, &model)`
- `cjlib::Format`
- `cjlib::Codec`

Those compact interfaces push format detection and backend-specific policy into
one place. The explicit-module rule is clearer:

- if you mean JSON, write `cjlib::json`
- if you mean Arrow, write `cjlib::arrow`
- if you mean Parquet, write `cjlib::parquet`

## Relationship To `cjfake`

`cjfake` should remain above `cjlib`.

```text
cjfake -> cjlib -> { serde_cityjson, cityarrow, cityparquet, cityjson-rs }
```

That lets `cjfake` reuse every format that `cjlib` exposes without making fake
data generation part of the facade itself.
