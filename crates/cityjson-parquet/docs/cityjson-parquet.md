# cityjson-parquet

`cityjson-parquet` is the persistent package crate for `cityjson-rs`.

It owns the durable storage boundary in the ADR 3 architecture and uses the
same canonical transport tables as `cityjson-arrow`.

## Public Surface

### PackageWriter

```rust
let manifest = PackageWriter.write_file("model.cityjson-parquet", &model)?;
```

Encodes an `OwnedCityModel` into a seekable single-file package. Returns the
`PackageManifest` describing the written tables.

### PackageReader

```rust
let model = PackageReader.read_file("model.cityjson-parquet")?;
let manifest = PackageReader.read_manifest("model.cityjson-parquet")?;
```

`read_file` decodes the full model. `read_manifest` reads only the footer and
manifest JSON — it does not load any table payload.

### read_package_manifest

```rust
let manifest = read_package_manifest("model.cityjson-parquet")?;
```

Standalone function equivalent to `PackageReader::read_manifest`. Use this for
inspection or fast extent queries without paying the cost of decoding geometry.

### spatial::SpatialIndex

```rust
let parts = cityjson_parquet::read_package_parts_file("model.cityjson-parquet")?;
let index = SpatialIndex::build(&parts);
let visible = index.query(&BBox2D::new(80_000.0, 440_000.0, 81_000.0, 441_000.0));
```

Builds a Hilbert-curve sorted index over `CityObject` bounding boxes. The
`query` method returns all entries whose bounding boxes intersect the supplied
2D rectangle. Objects without a stored `geographical_extent` get a fallback
bounding box computed from their geometry vertices.

`SpatialIndex` lives in the `spatial` module and is not re-exported at the
crate root. Reference it as `cityjson_parquet::spatial::SpatialIndex`.

## Execution Model

- `PackageSink` implements `CanonicalTableSink` from `cityjson-arrow`
- `emit_tables` drives the sink with canonical table batches derived from the
  model
- each batch is serialised as an Arrow IPC file payload written sequentially
  to the output file
- the manifest is written after all payloads; its byte offset and length are
  written as the footer alongside `PACKAGE_FOOTER_MAGIC`
- `PackageReader` memory-maps the file and decodes only the byte slices
  referenced by the manifest

## Re-exported Schema Types

The following types are re-exported from `cityjson_arrow::schema`:

- `CityArrowHeader`
- `CityArrowPackageVersion`
- `PackageManifest`
- `PackageTableRef`
- `ProjectedFieldSpec`, `ProjectedStructSpec`, `ProjectedValueSpec`
- `ProjectionLayout`
- `canonical_schema_set`

## Related Documents

- [Package layout specification](cityjson-parquet-spec.md)
- [Package schema](package-schema.md)
- [Transport design](design.md)
