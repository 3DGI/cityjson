# cityjson-arrow documentation

`cityjson-arrow` and `cityjson-parquet` are the Arrow transport layer for `cityjson-rs`.

This site documents the live stream API, the persistent package API, the shared
table schema, and the binary format layouts.

## Start Here

- [cityjson-arrow](cityjson-arrow.md): live Arrow IPC stream and batch codec
- [cityjson-parquet](cityjson-parquet.md): persistent single-file package
- [Package schema](package-schema.md): shared canonical table contract
- [Arrow IPC layout](cityjson-arrow-ipc-spec.md): binary layout of the live stream format
- [Package layout](cityjson-parquet-spec.md): binary layout of the persistent package format

## Design decisions

- [Transport design](design.md): why the transport boundary is where it is
- [ADR 1](adr/001-canonical-transport-boundary.md): canonical transport boundary
- [ADR 2](adr/002-address-transport-performance-bottlenecks.md): address transport performance
- [ADR 3](adr/003-separate-live-arrow-ipc-from-persistent-package-io.md): separate live stream from persistent package
- [ADR 4](adr/004-reduce-conversion-cost-with-ordinal-canonical-relations.md): reduce conversion cost with ordinal relations
- [ADR 5](adr/005-cut-v3-schema-for-arrow-native-projection-and-batch-native-conversion.md): v3 schema for Arrow-native projection
- [ADR 6](adr/006-cut-public-surface-to-thin-batch-and-stream-codec.md): thin batch and stream codec surface
- [ADR 7](adr/007-json-fallback-for-heterogeneous-attribute-values.md): JSON fallback for heterogeneous attributes
- [ADR 8](adr/008-benchmark-performance-profile.md): benchmark performance profile

## Scope

This site covers the transport layer only, not the CityJSON data model itself.

- `cityjson-arrow` owns the live Arrow stream and batch codec surface.
- `cityjson-parquet` owns the persistent package I/O surface.
- Both crates share the same canonical table schema and reconstruction rules.
- The current package format version is `cityjson-arrow.package.v3alpha2`.
