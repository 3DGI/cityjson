# cityjson-parquet Persistent Package Specification

This document describes the current persistent single-file package format used
by `cityjson_parquet::PackageWriter` and `cityjson_parquet::PackageReader`.

The historical filename is retained, but the current package is a seekable
single-file container rather than a directory tree.

## Version

- package magic: `CITYJSON_ARROW_PKG_V3\0`
- footer magic: `CITYJSON_ARROW_PKG_V3IDX\0`
- package schema id: `cityjson-arrow.package.v3alpha2`

## Layout

```text
PACKAGE_MAGIC
table_payload_0
table_payload_1
...
manifest_json
manifest_offset: u64 little-endian
manifest_length: u64 little-endian
PACKAGE_FOOTER_MAGIC
```

## Table Payloads

- payloads are written in canonical table order
- each payload is one Arrow IPC file payload written with Arrow `FileWriter`
- the manifest records the byte offset, byte length, and row count for every
  payload

The writer writes payloads directly to the destination file and derives
manifest offsets from file positions.

## Manifest

`manifest_json` is UTF-8 JSON with:

- `package_schema`
- `cityjson_version`
- `citymodel_id`
- `projection`
- ordered `tables`

Each table entry contains:

- canonical table name
- payload byte offset
- payload byte length
- decoded row count

## Reader Rules

- the reader must validate `PACKAGE_MAGIC`
- the reader must read the footer first to locate the manifest
- the manifest range must stay within the file and before the footer
- manifest tables must appear in canonical order
- each decoded payload schema must match the canonical schema for that table
- each decoded payload row count must match the manifest entry

The package reader maps the file and decodes only the payload slices referenced
by the manifest.
