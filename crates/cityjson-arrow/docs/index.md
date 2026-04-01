# cityarrow documentation

`cityarrow` and `cityparquet` define the ADR 3 transport boundary for
`cityjson-rs`.

This site documents the live stream surface, the persistent package surface,
the shared package schema, and the language-agnostic layouts.

## Start Here

- [cityarrow](cityarrow.md): live Arrow IPC stream transport and model conversion
- [cityparquet](cityparquet.md): persistent package I/O
- [Package schema](package-schema.md): shared canonical table contract
- [Arrow IPC spec](cityjson-arrow-ipc-spec.md): Arrow IPC layout specification
- [Parquet spec](cityjson-parquet-spec.md): Parquet layout specification

## Scope

The documentation focuses on the package boundary, not the CityJSON semantic
model itself.

- `cityarrow` owns the live Arrow stream transport and conversion surface.
- `cityparquet` owns the persistent package I/O surface.
- Both crates share the same canonical schema and reconstruction rules.

## Implementation Notes

The package format is currently `cityarrow.package.v2alpha1`.
It is intentionally schema-locked and reconstructible from ids and ordinals.
