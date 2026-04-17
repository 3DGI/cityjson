# Format Modules

The current release line exposes one format module:

```rust
pub mod json;
```

## Why Only `json`

This branch is the publishable core line.
That means:

- `json` is the only release-facing format module
- CityJSONSeq is handled explicitly inside `json`
- transport experiments are not part of the published crate surface

## Design Rule

`cityjson-lib` does not provide a generic registry such as:

- `read(path)`
- `write(path, &model)`
- `Format`
- `Codec`

If the caller means JSON, the API should say `json`.

## Semantic Rule

Format modules operate on:

- one `CityModel`
- or streams of `CityModel`

They do not turn wire-format parts into first-class semantic types in the
public facade.
