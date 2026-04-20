# cityjson-parquet documentation

`cityjson-parquet` stores `cityjson-rs` city models as seekable single-file packages.

## Start here

- [cityjson-parquet](cityjson-parquet.md): public API — `PackageWriter`, `PackageReader`,
  and `SpatialIndex`
- [Package layout](cityjson-parquet-spec.md): binary layout, magic bytes, manifest contract,
  and reader rules
- [Package schema](package-schema.md): canonical table contract shared with `cityjson-arrow`
- [Design](design.md): why persistent package I/O is a separate crate from the live stream

## Scope

This site covers the persistent package surface only.

- `cityjson-parquet` owns `PackageWriter`, `PackageReader`, and `spatial::SpatialIndex`
- The canonical table schema and manifest types are owned by `cityjson-arrow`
- The live Arrow IPC stream surface is not part of this crate

## Package format

- Format version: `cityjson-arrow.package.v3alpha2`
- File extension: `.cityjson-parquet` by convention
- The format is a seekable single-file container backed by Arrow IPC payloads.
  Despite the name, it is not a Parquet columnar file.
