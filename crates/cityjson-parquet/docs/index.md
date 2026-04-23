# cityjson-parquet documentation

`cityjson-parquet` stores `cityjson-rs` city models as seekable single-file packages
and native Parquet canonical-table datasets.

## Start here

- [cityjson-parquet](cityjson-parquet.md): public API — `PackageWriter`, `PackageReader`,
  `ParquetDatasetWriter`, `ParquetDatasetReader`, and `SpatialIndex`
- [Package layout](cityjson-parquet-spec.md): binary layout, magic bytes, manifest contract,
  and reader rules
- [Native Parquet dataset](native-parquet-dataset.md): directory layout for Parquet-native tools
- [Canonical table schema](package-schema.md): canonical table contract shared with `cityjson-arrow`
- [Design](design.md): why durable package and dataset I/O is separate from the live stream

## Scope

This site covers the persistent package and native Parquet dataset surfaces.

- `cityjson-parquet` owns `PackageWriter`, `PackageReader`, `ParquetDatasetWriter`,
  `ParquetDatasetReader`, and `spatial::SpatialIndex`
- The canonical table schema and manifest types are owned by `cityjson-arrow`
- The live Arrow IPC stream surface is not part of this crate

## Format surfaces

| Format | API | Layout | Primary use |
|---|---|---|---|
| `.cityjson-parquet` package | `PackageWriter`, `PackageReader` | Single seekable file with Arrow IPC table payloads | Compact persistent package IO |
| Native Parquet dataset | `ParquetDatasetWriter`, `ParquetDatasetReader` | `manifest.json` plus one `.parquet` file per canonical table | Interoperability with PyArrow, DuckDB, Polars, and other Parquet-native tools |

Both formats represent the same semantic CityJSON model through the same
canonical table schema. They are different physical encodings, so conformance
tests compare decoded CityJSON semantics rather than binary equality.

## Package format

- Format version: `cityjson-arrow.package.v3alpha3`
- File extension: `.cityjson-parquet` by convention
- The format is a seekable single-file container backed by Arrow IPC payloads.
  Despite the name, it is not a Parquet columnar file.

## Native Parquet dataset

- Dataset manifest: `manifest.json`
- Table files: `tables/{canonical_table}.parquet`
- This is the format used for PyArrow, DuckDB, and Polars interoperability tests.
