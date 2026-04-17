# Format Module API

This document pins down how `cityjson_lib` should expose sibling format crates.

## Core Rule

The facade should stay explicit:

- `json::from_*` is reserved for the default single-document CityJSON path
- explicit modules own explicit formats
- every format boundary speaks in terms of `CityModel` or streams of
  `CityModel`
- `cityjson_lib` does not grow a generic format registry

## Intended Modules

The public layout is:

```rust
pub mod json;
```

Those modules delegate to backend crates such as `serde_cityjson`,
`cityjson_json`, and the semantic model crate.

## JSON Is Richer

`cityjson_lib::json` is the richest boundary module because it has to cover:

- probing
- document parsing
- feature parsing
- feature-stream reading and writing
- document and feature serialization
- future raw or staged JSON access

The `cityjson_lib` facade should expose operations on `CityModel`, not transport
parts or package internals.

## One Semantic Unit Across Formats

The public facade still trades in one `CityModel` or streams of `CityModel`
values.

## No Generic `read` / `write`

Avoid APIs such as:

- `cityjson_lib::read(path)`
- `cityjson_lib::write(path, &model)`
- `cityjson_lib::Format`
- `cityjson_lib::Codec`

Those compact interfaces push format detection and backend-specific policy into
one place. The explicit-module rule is clearer:

- if you mean JSON, write `cityjson_lib::json`

## Relationship To `cjfake`

`cjfake` should remain above `cityjson_lib`.

```text
cjfake -> cityjson_lib -> { serde_cityjson, cityjson-rs }
```

That lets `cjfake` reuse every format that `cityjson_lib` exposes without making fake
data generation part of the facade itself.
