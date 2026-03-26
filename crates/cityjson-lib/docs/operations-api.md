# Operations API

This document pins down the intended public shape of `cjlib::ops`.

The purpose of `cjlib::ops` is to hold higher-level operations that are useful
to applications, but that should not live in the core `cityjson-rs` model
crate.

## Why `ops` Exists

`cityjson-rs` should stay focused on:

- the normalized data model
- invariants
- validated construction and mutation
- extraction, localization, and merge semantics for self-contained submodels

`cjlib` should be allowed to grow a small layer of reusable workflows above
that model.
Examples include:

- filtering by LoD
- cleaning vertices
- updating texture paths
- upgrading versions
- geometry measurements
- CRS assignment and reprojection

Those do not belong as a large inherent-method surface on `CityModel`.
They should live in a dedicated namespace instead.

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

    pub mod textures {
        pub fn rewrite_prefix(
            model: &mut crate::CityModel,
            from: &str,
            to: &str,
        ) -> crate::Result<()>;
    }

    #[cfg(feature = "crs")]
    pub mod crs {
        pub fn assign(model: &mut crate::CityModel, epsg: u32) -> crate::Result<()>;
        pub fn reproject(model: &mut crate::CityModel, target_epsg: u32) -> crate::Result<()>;
    }
}
```

This keeps the overall facade simple:

- `CityModel` for loading and wrapper access
- `cjlib::json` / `arrow` / `parquet` for explicit formats
- `cjlib::ops` for reusable higher-level workflows and thin convenience
  wrappers

## Why Free Functions, Not Inherent Methods

The operations namespace should prefer free functions over a long list of
inherent methods on `CityModel`.

That keeps the boundary clearer:

- loading stays on `CityModel`
- model internals stay in `cityjson-rs`
- higher-level workflows stay grouped under one explicit namespace

This is easier to teach and lower-maintenance than growing dozens of
`CityModel::*` methods over time.

## Keep `Selection` Small

Subsetting needs a structured selector type, but the selector should start
small.

Preferred:

```rust
let selection = cjlib::ops::Selection::from_ids(["id-1", "id-2"]);
let subset = cjlib::ops::subset(&model, selection)?;
# Ok::<(), cjlib::Error>(())
```

Not preferred:

- many overloaded `subset_*` entry points
- ad hoc string mini-languages
- a large query DSL at the `cjlib` boundary

If additional selectors become necessary later, they can be added to
`Selection` without fragmenting the top-level API.

## Feature-gate Heavy Dependencies

CRS work may pull in heavier dependencies than the rest of the facade.
For that reason, the CRS operations should be feature-gated rather than always
present.

The intended rule is:

- core operations stay available by default
- heavier integration layers stay behind explicit cargo features

## Relationship To `cityjson-rs`

`cjlib::ops` should use `cityjson-rs` as its foundation.
It should not bypass model invariants or duplicate the underlying storage
model.

The role split stays:

- `cityjson-rs`: authoritative model rules, submodel extraction, and merge
- `cjlib::ops`: reusable higher-level behavior and ergonomic selection wrappers
  on top of that model

When `ops::merge` or `ops::subset` exist, they should delegate to model-owned
capabilities in `cityjson-rs` rather than define competing merge logic in
`cjlib`.
