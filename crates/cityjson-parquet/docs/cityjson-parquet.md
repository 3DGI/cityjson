# cityjson-parquet

`cityjson-parquet` stores a `cityjson-rs` city model as either a seekable
single-file package or a native Parquet canonical-table dataset.

## Public API

### PackageWriter

```rust
let manifest = PackageWriter.write_file("model.cityjson-parquet", &model)?;
```

Encodes an `OwnedCityModel` into a package file. Returns the `PackageManifest`
describing the written tables.

### PackageReader

```rust
let model = PackageReader.read_file("model.cityjson-parquet")?;
let manifest = PackageReader.read_manifest("model.cityjson-parquet")?;
```

`read_file` decodes the full model. `read_manifest` reads only the footer and
manifest JSON; it does not load any table payload and is fast for inspection.

### ParquetDatasetWriter

```rust
let manifest = ParquetDatasetWriter.write_dir("model.parquet-dataset", &model)?;
```

Encodes an `OwnedCityModel` into a dataset directory containing `manifest.json`
and one native Parquet file per canonical table.

### ParquetDatasetReader

```rust
let model = ParquetDatasetReader.read_dir("model.parquet-dataset")?;
let manifest = ParquetDatasetReader.read_manifest("model.parquet-dataset")?;
```

`read_dir` decodes the full model from native Parquet table files. This is the
interop target for PyArrow, DuckDB, Polars, and other Parquet-native tools.

### spatial::SpatialIndex

A Hilbert-curve sorted index over city object bounding boxes. `query` returns
all entries whose bounding boxes overlap a given 2D rectangle. Objects without
a stored `geographical_extent` get a bounding box derived from their geometry
vertices.

`SpatialIndex` is in the `spatial` module and is not re-exported at the crate
root. Use it as `cityjson_parquet::spatial::SpatialIndex`.

!!! note
    `SpatialIndex::build` currently takes an internal parts type. A public
    constructor is planned for a future release.

## How it works

`PackageWriter` serialises each canonical Arrow table as an Arrow IPC file
payload and appends them to the output file in order. The manifest is written
last, so the writer never seeks back. `PackageReader` maps the file into memory
and decodes only the byte slices referenced by the manifest.

`ParquetDatasetWriter` writes the same canonical tables as native Parquet files
under `tables/` and writes a JSON manifest at the dataset root. `ParquetDatasetReader`
validates table order, row counts, and schemas before reconstructing the model.
For native Parquet interoperability, nullable canonical `FixedSizeList` columns
such as `geographical_extent` are written as nullable Parquet lists and validated
back to the fixed logical length on read.

## Re-exported types

The following types from `cityjson_arrow::schema` are available at the crate root:

- `CityArrowHeader`
- `CityArrowPackageVersion`
- `PackageManifest`
- `PackageTableRef`
- `ProjectedFieldSpec`, `ProjectedStructSpec`, `ProjectedValueSpec`
- `ProjectionLayout`
- `canonical_schema_set`

The dataset API also exposes `ParquetDatasetManifest` and `ParquetDatasetTableRef`
from this crate.

## Related documents

- [Package layout](cityjson-parquet-spec.md): binary layout, magic bytes, and manifest contract
- [Native Parquet dataset](native-parquet-dataset.md): directory layout and interop target
- [Package schema](package-schema.md): canonical table contract shared with `cityjson-arrow`
- [Transport design](design.md): why persistent package I/O is a separate crate
