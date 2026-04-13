# Operations API

This document pins down the intended public shape of `cityjson_lib::ops`.

`cityjson_lib::ops` is the place for higher-level reusable workflows that are useful
to applications but do not belong in the semantic model crate itself.

## Why `ops` Exists

`cityjson-rs` should stay focused on:

- the normalized data model
- invariants and validated mutation
- extraction, localization, remapping, and merge semantics

`cityjson_lib::ops` can then provide reusable workflows above that model, for example:

- filtering by LoD
- cleaning vertices
- updating texture paths
- upgrading versions
- geometry measurements
- CRS assignment and reprojection

## Intended Shape

The intended public shape is:

```rust
pub mod ops {
    pub struct Selection<'a> {
        /* private fields */
    }

    impl<'a> Selection<'a> {
        pub fn from_ids(ids: impl IntoIterator<Item = &'a str>) -> Self;
    }

    pub fn merge(models: impl IntoIterator<Item = crate::CityModel>) -> crate::Result<crate::CityModel>;
    pub fn subset(model: &crate::CityModel, selection: Selection<'_>) -> crate::Result<crate::CityModel>;
    pub fn upgrade(model: crate::CityModel) -> crate::Result<crate::CityModel>;

    pub mod lod {
        pub fn filter(model: &crate::CityModel, lod: &str) -> crate::Result<crate::CityModel>;
    }

    pub mod geometry {
        pub fn surface_area(model: &crate::CityModel, object_id: &str) -> crate::Result<f64>;
        pub fn volume(model: &crate::CityModel, object_id: &str) -> crate::Result<f64>;
    }

    pub mod vertices {
        pub struct CleanReport {
            pub duplicates_removed: usize,
            pub orphans_removed: usize,
        }

        pub fn clean(model: &mut crate::CityModel) -> crate::Result<CleanReport>;
    }
}
```

This keeps the facade split clean:

- `CityModel` for loading and wrapper access
- explicit modules for explicit formats
- `cityjson_lib::ops` for reusable workflows

## Prefer Free Functions

The operations namespace should prefer free functions over a large set of
inherent `CityModel` methods.
That keeps loading, model semantics, and higher-level workflows clearly
separated.

## Keep Selectors Small

Subsetting needs a structured selector type, but the selector should start
small:

```rust
let selection = cityjson_lib::ops::Selection::from_ids(["id-1", "id-2"]);
let subset = cityjson_lib::ops::subset(&model, selection)?;
# Ok::<(), cityjson_lib::Error>(())
```

Avoid:

- many overlapping `subset_*` entry points
- ad hoc string mini-languages
- a large query DSL at the `cityjson_lib` boundary

## Relationship To `cityjson-rs`

`cityjson_lib::ops` should build on `cityjson-rs`, not compete with it.
When `merge`, `subset`, cleanup, or upgrade helpers exist, they should delegate
to semantic-model capabilities where correctness depends on model invariants.
