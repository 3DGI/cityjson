# Operations API

`cityjson_lib::ops` is the home for reusable workflows above the semantic
model.

The current release line ships four helpers:

- `cleanup(&CityModel) -> Result<CityModel>`
- `subset(&CityModel, ids, exclude) -> Result<CityModel>`
- `append(&mut CityModel, &CityModel) -> Result<()>`
- `merge(models) -> Result<CityModel>`

## Examples

```rust
use cityjson_lib::{json, ops};

let first = json::from_feature_file("tests/data/v2_0/feature-1.city.json")?;
let second = json::from_feature_file("tests/data/v2_0/feature-2.city.json")?;

let mut merged = ops::merge([first, second])?;
let subset = ops::subset(&merged, ["building-1"], false)?;
ops::append(&mut merged, &subset)?;
let cleaned = ops::cleanup(&merged)?;
# let _ = cleaned;
# Ok::<(), cityjson_lib::Error>(())
```

## Design Rule

`ops` stays as free functions instead of turning `CityModel` into a large
method bag.

## Implementation Rule

These helpers are part of the stable `cityjson-lib` facade, with semantic
workflows owned by `cityjson-lib` and JSON-aware implementation details
delegated to `cityjson-json`.
