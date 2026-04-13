# Format Module API

This document pins down how `cityjson_lib` should expose sibling format crates.

## Core Rule

The facade should stay explicit:

- `CityModel::from_*` is reserved for the default single-document CityJSON path
- explicit modules own explicit formats
- every format boundary speaks in terms of `CityModel` or streams of
  `CityModel`
- `cityjson_lib` does not grow a generic format registry

## Intended Modules

The public layout is:

```rust
pub mod json;
pub mod arrow;
pub mod parquet;
```

Those modules delegate to backend crates such as `serde_cityjson`,
`cityarrow`, and `cityparquet`.

## JSON Is Richer

`cityjson_lib::json` is the richest boundary module because it has to cover:

- probing
- document parsing
- feature parsing
- feature-stream reading and writing
- document and feature serialization
- future raw or staged JSON access

Sibling transport modules can stay smaller as long as they follow the same
semantic rule.

Within that family:

- `cityjson_lib::arrow` owns live Arrow IPC stream read and write helpers around
  `CityModel`
- `cityjson_lib::parquet` owns persistent package-file read and write helpers around
  `CityModel`

Transport-native schema and package details stay in the backend crates. The
`cityjson_lib` facade should expose operations on `CityModel`, not transport parts or
package internals.

## One Semantic Unit Across Formats

Arrow and Parquet must not invent separate semantic units at the `cityjson_lib`
boundary.
Even if the backend format uses batches, row groups, or other transport-native
chunks internally, the public facade still trades in:

- one `CityModel`
- or streams of `CityModel` values

File-oriented helpers are fine as conveniences:

```rust
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

- `cityjson_lib::read(path)`
- `cityjson_lib::write(path, &model)`
- `cityjson_lib::Format`
- `cityjson_lib::Codec`

Those compact interfaces push format detection and backend-specific policy into
one place. The explicit-module rule is clearer:

- if you mean JSON, write `cityjson_lib::json`
- if you mean Arrow, write `cityjson_lib::arrow`
- if you mean Parquet, write `cityjson_lib::parquet`

## Relationship To `cjfake`

`cjfake` should remain above `cityjson_lib`.

```text
cjfake -> cityjson_lib -> { serde_cityjson, cityarrow, cityjson-rs }
```

That lets `cjfake` reuse every format that `cityjson_lib` exposes without making fake
data generation part of the facade itself.
