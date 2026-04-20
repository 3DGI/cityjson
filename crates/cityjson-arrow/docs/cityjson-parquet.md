# cityjson-parquet

`cityjson-parquet` stores a `cityjson-rs` city model as a seekable single-file package.

It uses the same canonical table schema as `cityjson-arrow` and produces files that
are compatible with the `cityjson-arrow.package.v3alpha2` format version.

## Public API

| Type | Purpose |
|---|---|
| `PackageWriter` | Encode a model into a `.cityjson-parquet` file |
| `PackageReader` | Decode a file back into a model or read its manifest |
| `spatial::SpatialIndex` | Viewport query index built from city object bounding boxes |

The input and output type is always `cityjson::v2_0::OwnedCityModel`.

## Related documents

- [Package layout](cityjson-parquet-spec.md): binary layout, magic bytes, and manifest contract
- [Package schema](package-schema.md): canonical table contract shared with `cityjson-arrow`
- [Transport design](design.md): why persistent package I/O is a separate crate
