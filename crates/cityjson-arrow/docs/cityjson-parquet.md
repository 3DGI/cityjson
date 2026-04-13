# cityjson-parquet

`cityjson-parquet` is the persistent package crate for `cityjson-rs`.

It owns the durable package/container boundary in the ADR 3 architecture and
uses the same canonical transport tables as `cityjson-arrow`.

## What It Provides

- package write/read support for seekable single-file packages
- the same canonical table layout as `cityjson-arrow`
- the same reconstruction rules and manifest contract
- round-trip compatibility with the shared `cityjson-arrow.package.v3alpha1` schema

## Related Documents

- [Package layout specification](package-schema.md)
- [Shared package schema](package-schema.md)
- [Transport design](design.md)

## Public Surface

The crate exposes:

- `PackageWriter` and `PackageReader`
- the shared package manifest and schema types
