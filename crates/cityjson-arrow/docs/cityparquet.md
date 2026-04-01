# cityparquet

`cityparquet` is the Parquet package crate for `cityjson-rs`.

It uses the same canonical `CityModelArrowParts` shape as `cityarrow`, but
stores the package tables as Parquet files instead of Arrow IPC files.

## What It Provides

- package write/read support for Parquet packages
- the same canonical table layout as `cityarrow`
- the same reconstruction rules and manifest contract
- round-trip compatibility with the Arrow IPC package schema

## Related Documents

- [Parquet package layout specification](cityjson-parquet-spec.md)
- [Shared package schema](package-schema.md)
- [Transport design](design.md)

## Public Surface

The crate exposes:

- `write_package_dir` and `read_package_dir`
- the shared package manifest and schema types
