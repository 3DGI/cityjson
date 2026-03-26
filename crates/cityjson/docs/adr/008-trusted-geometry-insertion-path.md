# Trusted Geometry Insertion Path

## Status

Accepted

## Related Commits

- `558451e` Add trusted geometry insertion path

## Context

`cityjson-rs` already stores geometry in a flat, serializer-friendly form, but
the public write path made trusted ingestion awkward:

- stored geometry could only be constructed through a crate-private raw
  constructor
- `CityModel::add_geometry(...)` and `CityModel::add_geometry_template(...)`
  always validated stored geometry on insertion
- `GeometryDraft` rebuilt stored geometry through a multi-step pipeline and then
  revalidated the result before insertion
- bulk importers had no obvious public reservation helper for the full import
  workload

That meant callers with already-validated flat geometry had no direct, explicit
path to insert it efficiently.

## Decision

The crate now exposes a public raw write layer for trusted geometry ingestion.

The new model is:

1. construct final stored geometry directly with `Geometry::from_stored_parts(...)`
2. reserve capacities up front with `CityModel::reserve_import(...)`
3. insert trusted geometry with `CityModel::add_geometry_unchecked(...)`
   or `CityModel::add_geometry_template_unchecked(...)`
4. use `GeometryDraft` as the checked convenience layer, not the only write
   path

`GeometryDraft` still validates draft-local invariants and performs one
preflight pass, but it now inserts through the unchecked raw model API instead
of validating the stored geometry again.

## Consequences

Good:

- deserializers and importers can insert already-validated geometry directly
- draft geometry no longer pays for redundant stored-geometry validation
- bulk import callers can reserve all relevant pools through one public method

Trade-offs:

- trusted callers must uphold stored-geometry invariants themselves
- the raw insertion API is more explicit and less ergonomic than the checked
  draft path
- the unchecked API relies on naming and documentation rather than the type
  system to mark the trust boundary

From this point on, geometry creation is split into two layers:

- `GeometryDraft` for checked authoring
- `StoredGeometryParts` plus unchecked insertion for trusted flat input
