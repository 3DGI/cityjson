# Format Module API

This document pins down how `cjlib` should expose sibling format crates.

The design goal is to keep the facade explicit and unsurprising:

- `CityModel::from_*` stays reserved for the default CityJSON JSON path
- explicit modules own explicit formats
- `cjlib` does not grow a generic format registry

## Intended Modules

The intended module layout is:

```rust
pub mod json;

#[cfg(feature = "arrow")]
pub mod arrow;

#[cfg(feature = "parquet")]
pub mod parquet;
```

Those names are the public `cjlib` facade.
Their implementations should delegate to sibling backend crates such as `serde_cityjson`, `cityarrow`, and `cityparquet`.

## JSON Is Richer Than The Others

`cjlib::json` should be the richest explicit format module because:

- it is the default and most common path
- it needs probing
- it needs stream aggregation
- it needs text serialization helpers

That is why `cjlib::json` owns:

- `probe`
- `from_slice`
- `from_file`
- `from_stream`
- `to_vec`
- `to_string`
- `to_writer`

The other transport modules should start smaller.

## Arrow And Parquet Should Start File-oriented

The initial contract for Arrow and Parquet should be deliberately narrow:

```rust
#[cfg(feature = "arrow")]
pub mod arrow {
    pub fn from_file(path: impl AsRef<std::path::Path>) -> crate::Result<crate::CityModel>;
    pub fn to_file(
        path: impl AsRef<std::path::Path>,
        model: &crate::CityModel,
    ) -> crate::Result<()>;
}

#[cfg(feature = "parquet")]
pub mod parquet {
    pub fn from_file(path: impl AsRef<std::path::Path>) -> crate::Result<crate::CityModel>;
    pub fn to_file(
        path: impl AsRef<std::path::Path>,
        model: &crate::CityModel,
    ) -> crate::Result<()>;
}
```

This is enough for the intended user stories:

- explicit file-based import
- explicit file-based export
- straightforward backend reuse from tools such as `cjfake`

If stream-oriented APIs become necessary later, they can be added once the backend crates stabilize.

## No Generic `read` / `write`

The public surface should avoid APIs like:

- `cjlib::read(path)`
- `cjlib::write(path, &model)`
- `cjlib::Format`
- `cjlib::Codec`

Those look compact at first, but they push format detection, extension policy, feature interactions, and backend-specific options into one place.
That is exactly the kind of convenience layer that becomes hard to keep elegant.

The explicit-module rule is cleaner:

- if you mean JSON, write `cjlib::json`
- if you mean Arrow, write `cjlib::arrow`
- if you mean Parquet, write `cjlib::parquet`

## Relationship To `cjfake`

`cjfake` should sit above `cjlib`, not inside it.

The preferred dependency direction is:

```text
cjfake -> cjlib -> { serde_cityjson, cityarrow, cityparquet, cityjson-rs }
```

That gives `cjfake` automatic access to every output format that `cjlib` exposes, without making `cjlib` responsible for fake-data generation.

A `cjfake`-style workflow should look like this:

```rust
let model = cjfake::small_city()?;
cjlib::json::to_writer(&mut std::io::stdout(), &model)?;

#[cfg(feature = "arrow")]
cjlib::arrow::to_file("small-city.cjarrow", &model)?;

#[cfg(feature = "parquet")]
cjlib::parquet::to_file("small-city.cjparquet", &model)?;
# Ok::<(), cjlib::Error>(())
```

The important design point is that `cjfake` uses `cjlib`.
`cjlib` should not absorb `cjfake`.
