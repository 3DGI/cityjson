# cityjson-parquet documentation

`cityjson-parquet` defines the persistent package boundary for `cityjson-rs`.

This site documents the single-file package format, the shared canonical table
contract, and the transport design that motivated the package layer.

## Start Here

- [cityjson-parquet](cityjson-parquet.md): public API and execution model
- [Package layout specification](cityjson-parquet-spec.md): binary layout,
  magic bytes, manifest contract, and reader rules
- [Package schema](package-schema.md): canonical table contract shared with
  `cityjson-arrow`
- [Transport design](design.md): the ADR 3 architecture and the reasoning
  behind separating live stream IO from persistent package IO

## Scope

This site documents the persistent package surface only.

- `cityjson-parquet` owns `PackageWriter`, `PackageReader`, and
  `spatial::SpatialIndex`
- the canonical table schema and manifest types are owned by `cityjson-arrow`
  and documented in the `cityjson-arrow` documentation site
- the live Arrow IPC stream surface is not part of this crate

## Package Format

The current package format is `cityjson-arrow.package.v3alpha2`.

Files use the extension `.cityjson-parquet` by convention. The format is a
seekable single-file container backed by Arrow IPC table payloads; despite the
name it is not a Parquet columnar file.
