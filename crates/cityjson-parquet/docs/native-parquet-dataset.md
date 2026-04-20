# Native Parquet dataset

The native Parquet dataset format stores each canonical CityJSON Arrow table as a
separate Parquet file. It is the interoperability target for PyArrow, DuckDB,
Polars, and other Parquet-native tools.

## Layout

```text
dataset/
  manifest.json
  tables/
    metadata.parquet
    vertices.parquet
    geometry_boundaries.parquet
    geometries.parquet
    cityobjects.parquet
    ...
```

`manifest.json` contains:

| Field | Type | Description |
|---|---|---|
| `package_schema` | string | Always `cityjson-arrow.package.v3alpha3` |
| `cityjson_version` | string | CityJSON version of the source data |
| `citymodel_id` | string | Identifier for the source city model |
| `projection` | object | Attribute projection layout |
| `tables` | array | Ordered canonical table entries |

Each table entry contains:

| Field | Type | Description |
|---|---|---|
| `name` | string | Canonical table name |
| `path` | string | Safe relative path to the Parquet file |
| `rows` | uint64 | Declared row count |

## Reader rules

- Required canonical tables MUST be present.
- Table entries MUST appear in canonical order.
- Table paths MUST be relative paths inside the dataset directory.
- Each Parquet file schema MUST match the canonical schema for that table after
  applying the native Parquet physical mappings below.
- Canonical `fixed_size_list<float64>[N]` fields are encoded in native Parquet
  as nullable `list<float64>` fields with the same child nullability. This avoids
  nullable fixed-size list interoperability failures in PyArrow while preserving
  the CityJSON Arrow logical constraint.
- Readers MUST validate every non-null value in those Parquet list fields has
  exactly `N` items and MUST normalize the decoded batches back to the canonical
  Arrow `FixedSizeList` type before CityJSON decoding.
- Parquet list child field names are treated as physical metadata; readers validate
  child type/nullability and normalize batches to the canonical Arrow schema before decoding.
- Decoded row counts MUST match the manifest row counts.

## Relationship to `.cityjson-parquet`

The existing `.cityjson-parquet` file is a seekable single-file package backed by
Arrow IPC payloads. It remains the compact package API.

The native Parquet dataset is a separate API for ecosystem interoperability and
columnar query engines.
