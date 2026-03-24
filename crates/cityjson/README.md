# cityjson-rs

`cityjson-rs` implements the [CityJSON 2.0](https://www.cityjson.org/specs/2.0.1/) data model in
Rust. The types map directly to the spec's object hierarchy: `CityModel` is the root object,
`CityObject` is each entry in the `CityObjects` map, and `Geometry` covers all eight geometry
types.

JSON encoding and decoding, and upgrades from older `CityJSON` versions, are handled in the
separate `serde_cityjson` crate.

## Installation

```shell
cargo add cityjson
```

## Example

A `Building` at `LoD2` with two attributes, constructed from scratch:

```rust
use cityjson::CityModelType;
use cityjson::v2_0::{
    CityObject, CityObjectIdentifier, CityObjectType, GeometryDraft, LoD,
    OwnedAttributeValue, OwnedCityModel, RingDraft, SurfaceDraft,
};

let mut model = OwnedCityModel::new(CityModelType::CityJSON);

// Build a CompositeSurface: three planar faces of a building shell.
let wall_a = SurfaceDraft::new(
    RingDraft::new([
        [0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 0.0, 3.0], [0.0, 0.0, 3.0],
    ]),
    [],
);
let wall_b = SurfaceDraft::new(
    RingDraft::new([
        [10.0, 0.0, 0.0], [10.0, 10.0, 0.0], [10.0, 10.0, 3.0], [10.0, 0.0, 3.0],
    ]),
    [],
);
let roof = SurfaceDraft::new(
    RingDraft::new([
        [0.0, 0.0, 3.0], [10.0, 0.0, 3.0], [10.0, 10.0, 3.0], [0.0, 10.0, 3.0],
    ]),
    [],
);
let geom = GeometryDraft::composite_surface(Some(LoD::LoD2), [wall_a, wall_b, roof])
    .insert_into(&mut model)
    .unwrap();

// Create the city object and attach the geometry.
let mut building = CityObject::new(
    CityObjectIdentifier::new("building-1".to_string()),
    CityObjectType::Building,
);
building
    .attributes_mut()
    .insert("measuredHeight".to_string(), OwnedAttributeValue::Float(3.0));
building
    .attributes_mut()
    .insert("yearOfConstruction".to_string(), OwnedAttributeValue::Integer(2024));
building.add_geometry(geom);

model.cityobjects_mut().add(building).unwrap();
```

## Modules

| Module      | Contents                                                                                                                            |
|-------------|-------------------------------------------------------------------------------------------------------------------------------------|
| `v2_0`      | Domain types: `CityModel`, `CityObject`, `Geometry`, `GeometryDraft`, `Metadata`, `Transform`, `Semantic`, `Material`, `Texture`, … |
| `resources` | Typed handles, resource pools, and string storage strategies                                                                        |
| `raw`       | Zero-copy read views for use in downstream serializers                                                                              |

## Imports

```rust
use cityjson::v2_0::*;     // all domain types
use cityjson::prelude::*;  // handles, storage strategies, error types
```

The `prelude` re-exports crate-wide types (handles, errors, storage strategies) but not the domain types from `v2_0`.

## API Stability

This crate follows semantic versioning (`MAJOR.MINOR.PATCH`):

- `MAJOR`: incompatible API changes
- `MINOR`: backwards-compatible feature additions
- `PATCH`: backwards-compatible fixes

## Minimum Rust Version

The minimum supported rustc version is `1.93.0`.

## License

Licensed under either:

- Apache License, Version 2.0 (`LICENSE-APACHE`)
- MIT license (`LICENSE-MIT`)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
cityjson-rs by you, as defined in the Apache-2.0 license, shall be dual licensed as above,
without additional terms or conditions.
