# Transport Attribute-Referenced Geometries Through Arrow and Parquet

## Summary

The Arrow exporter currently emits only geometries attached through
`CityObject.geometry`. Address locations exist solely in the geometry pool and
are referenced from `CityObject.extra`, so their projected geometry IDs cannot
be resolved during import.

Cut the canonical schema to `cityjson-arrow.package.v3alpha4`. Represent
geometry-pool entries without CityObject attachment by making `cityobject_ix`
and `geometry_ordinal` nullable.

## CityJSON Arrow Changes

- Replace the public `V3Alpha3` package/schema variants and schema ID with
  `V3Alpha4`; keep the existing hard-cut policy with no legacy reader.
- Make `cityobject_ix` and `geometry_ordinal` nullable together in both
  boundary-geometry and geometry-instance tables.
- During export:
  - Collect attachment metadata from every CityObject.
  - Iterate the complete geometry pool in handle/ID order.
  - Write attached geometries with both ownership fields populated.
  - Write attribute-only geometries with both fields null.
  - Reject a geometry attached to multiple CityObjects because the canonical
    row permits one owner.
- During import:
  - Insert every geometry row into the geometry pool.
  - Add a pending CityObject attachment only when both ownership fields are
    present.
  - Leave rows with both fields null unattached.
  - Reject rows where only one ownership field is null.
- Keep projected `GeometryRef` values unchanged; their IDs will now resolve
  because all referenced pool geometries are transported.
- Update Arrow schema/spec documentation and the changelog for `v3alpha4`.

## CityJSON Parquet Changes

- Update package and native-dataset manifests to record `V3Alpha4`; container
  magic remains unchanged.
- Inherit the nullable geometry ownership columns from `cityjson-arrow` for
  both Arrow IPC package payloads and native Parquet tables.
- Update the spatial fallback calculation to skip geometry rows with null
  `cityobject_ix`; address locations must not be treated as directly attached
  CityObject geometry.
- Update Parquet format documentation and schema examples to `v3alpha4`,
  documenting null ownership as “pooled but referenced outside
  `CityObject.geometry`.”

## Test Plan

- Reuse the existing Arrow stream/batch and Parquet package/dataset conformance
  cases `cityobject_building_address` and `cityjson_fake_complete`; they must
  round-trip without adding address locations to `CityObject.geometry`.
- Update existing schema/version assertions for `v3alpha4`.
- Add only one focused spatial test confirming an unowned geometry row is
  ignored safely.
- Run the Arrow and Parquet crate suites, then the required workspace `just ci`.

## Assumptions

- `v3alpha3` files are intentionally incompatible after this hard alpha schema
  cut, consistent with the existing schema ADRs.
- A geometry-pool entry may be unattached, but an attached geometry has at most
  one CityObject owner.
- Attribute-only geometries remain available through their attribute handles
  and do not contribute to CityObject fallback bounds.
