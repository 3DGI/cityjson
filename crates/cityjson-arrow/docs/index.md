# cityarrow documentation

`cityarrow` and `cityparquet` define the canonical transport boundary for
`cityjson-rs`.

This site documents the user-facing package surface, the shared package schema,
and the language-agnostic on-disk layouts for the supported encodings.

## Start Here

- [cityarrow](cityarrow.md): Arrow IPC package I/O and model conversion
- [cityparquet](cityparquet.md): Parquet package I/O
- [Package schema](package-schema.md): shared canonical table contract
- [Arrow IPC spec](cityjson-arrow-ipc-spec.md): Arrow IPC layout specification
- [Parquet spec](cityjson-parquet-spec.md): Parquet layout specification

## Scope

The documentation focuses on the package boundary, not the CityJSON semantic
model itself.

- `cityarrow` owns the Arrow-side transport and conversion surface.
- `cityparquet` owns the Parquet-side package I/O surface.
- Both crates share the same canonical schema and reconstruction rules.

## Implementation Notes

The package format is currently `cityarrow.package.v1alpha1`.
It is intentionally schema-locked and reconstructible from ids and ordinals.
