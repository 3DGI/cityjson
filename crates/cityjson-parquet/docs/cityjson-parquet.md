# cityjson-parquet

`cityjson-parquet` stores a `cityjson-rs` city model as a seekable single-file package.

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

## Re-exported types

The following types from `cityjson_arrow::schema` are available at the crate root:

- `CityArrowHeader`
- `CityArrowPackageVersion`
- `PackageManifest`
- `PackageTableRef`
- `ProjectedFieldSpec`, `ProjectedStructSpec`, `ProjectedValueSpec`
- `ProjectionLayout`
- `canonical_schema_set`

## Related documents

- [Package layout](cityjson-parquet-spec.md): binary layout, magic bytes, and manifest contract
- [Package schema](package-schema.md): canonical table contract shared with `cityjson-arrow`
- [Transport design](design.md): why persistent package I/O is a separate crate
