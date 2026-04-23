# Operations API

`cityjson_lib::ops` is the home for reusable workflows above the semantic
model.

The `subset` and `merge` behavior was ported from `cjio` and is now owned
natively by `cityjson-lib`.

The current release line ships these helpers:

- `cleanup(&CityModel) -> Result<CityModel>`
- `subset(&CityModel, ids, exclude) -> Result<CityModel>`
- `select_cityobjects(&CityModel, predicate) -> Result<ModelSelection>`
- `select_geometries(&CityModel, predicate) -> Result<ModelSelection>`
- `extract(&CityModel, &ModelSelection) -> Result<CityModel>`
- `append(&mut CityModel, &CityModel) -> Result<()>`
- `merge(models) -> Result<CityModel>`

## Examples

```rust
use cityjson_lib::{json, ops};

let first = json::from_feature_file("tests/data/v2_0/feature-1.city.json")?;
let second = json::from_feature_file("tests/data/v2_0/feature-2.city.json")?;

let mut merged = ops::merge([first, second])?;
let selection = ops::select_cityobjects(&merged, |ctx| ctx.id() == "building-1")?;
let subset = ops::extract(&merged, &selection)?;
ops::append(&mut merged, &subset)?;
let cleaned = ops::cleanup(&merged)?;
# let _ = cleaned;
# Ok::<(), cityjson_lib::Error>(())
```

```rust
use cityjson_lib::{json, ops};

let model = json::from_file("tests/data/v2_0/ops/merge_left.city.json")?;
let selection = ops::select_geometries(&model, |ctx| {
    ctx.cityobject_id() == "shared-furniture" && ctx.geometry_index() == 0
})?;
let filtered = ops::extract(&model, &selection)?;
# let _ = filtered;
# Ok::<(), cityjson_lib::Error>(())
```

## Design Rule

`ops` stays as free functions instead of turning `CityModel` into a large
method bag.

## Implementation Rule

These helpers are part of the stable `cityjson-lib` facade, with semantic
workflows owned by `cityjson-lib` and JSON-aware implementation details
delegated to `cityjson-json`.
