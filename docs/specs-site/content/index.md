# CityJSON Arrow and Parquet Specifications

CityJSON Arrow and CityJSON Parquet are experimental open format
specifications authored and maintained by **Balázs Dukai at 3DGI** as part of
the [`cityjson-rs`](https://github.com/3DGI/cityjson-rs) project.

!!! warning "Experimental specification"
    The current format version is `cityjson-arrow.package.v3alpha3`. It can
    change incompatibly while the version remains an alpha version.

## Specifications

### [CityJSON Arrow](https://specs.citymodel.3dgi.nl/arrow/)

The live Arrow IPC stream, canonical table schema, and seekable Arrow-backed
package layout.

### [CityJSON Parquet](https://specs.citymodel.3dgi.nl/parquet/)

The persistent package API and native Parquet dataset layout for use with
PyArrow, DuckDB, Polars, and other Parquet-native tools.

Both formats use the same canonical table schema. The `.cityjson-parquet`
single-file package contains Arrow IPC payloads; the native dataset writes one
actual Parquet file per canonical table.

## Document information

| Field | Value |
| --- | --- |
| Status | Experimental |
| Format version | `cityjson-arrow.package.v3alpha3` |
| Specification author and editor | Balázs Dukai |
| Affiliation | 3DGI |
| License | [Creative Commons Attribution 4.0 International](license.md) |
| Source | [`3DGI/cityjson-rs`](https://github.com/3DGI/cityjson-rs) |

See [Citation](citation.md) for the preferred citation.
