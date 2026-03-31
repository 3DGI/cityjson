# Add a Streaming Package Layer for Large-Model and Bounded-Memory Operation

## Status

Proposed

## Related Commits

- `3dbdbfa` Add ADR for current transport architecture
- `a534911` Reduce real-data test memory pressure

## Context

`cityarrow` currently provides a coherent canonical transport boundary around
`CityModelArrowParts`, but the implemented conversion and package read paths are
eager and fully in-memory.

That behavior is acceptable for small and medium models, but it becomes a
scalability problem for large real-world datasets:

- `convert::to_parts` first materializes full row vectors and then builds one
  full `RecordBatch` per canonical table
- package readers load every table into memory and concatenate all input
  batches into one batch per table
- exact test roundtrips show that large datasets already push multi-gigabyte
  resident memory before semantic equality checks

The current public API is still valuable and should remain available:

1. `cityjson::v2_0::OwnedCityModel` is the semantic source and sink
2. `CityModelArrowParts` is the canonical fully materialized transport shape
3. package helpers read and write that fully materialized transport shape

The problem is not the canonical schema. The problem is that the crate has only
one operational mode: fully materialize everything, then serialize or
deserialize it.

Bounded-memory claims also need to be stated precisely. As long as the semantic
unit is a fully materialized `OwnedCityModel`, end-to-end constant-memory
roundtrip is not achievable. The crate can, however, support bounded-memory
package I/O and bounded-memory conversion into package tables.

## Decision

`cityarrow` will keep `CityModelArrowParts` as the canonical in-memory
transport type and add an additive streaming package layer beside it.

The architecture direction is:

1. keep the current `to_parts`, `from_parts`, and package helper APIs as the
   convenience layer
2. add a new streaming API surface for package read and write
3. refactor conversion so that canonical tables can be emitted incrementally
   into the streaming writer
4. treat fully materialized `OwnedCityModel` reconstruction as a reduced-peak
   goal, not as a bounded-memory promise

The streaming layer will preserve the current canonical schema and manifest
contract. It changes execution strategy, not the logical package model.

### API Surface Sketch

The new API should be explicit and concrete. It should not introduce a generic
plugin framework or hide the canonical tables behind opaque dynamic dispatch.

An initial sketch:

```rust
use std::path::Path;

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;

use crate::error::Result;
use crate::schema::{
    CityArrowHeader, PackageManifest, PackageTableEncoding, ProjectionLayout,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackageTableKind {
    Metadata,
    Transform,
    Extensions,
    Vertices,
    CityObjects,
    CityObjectChildren,
    Geometries,
    GeometryBoundaries,
    GeometryInstances,
    TemplateVertices,
    TemplateGeometries,
    TemplateGeometryBoundaries,
    Semantics,
    SemanticChildren,
    GeometrySurfaceSemantics,
    GeometryPointSemantics,
    GeometryLinestringSemantics,
    TemplateGeometrySemantics,
    Materials,
    GeometrySurfaceMaterials,
    GeometryPointMaterials,
    GeometryLinestringMaterials,
    TemplateGeometryMaterials,
    Textures,
    TextureVertices,
    GeometryRingTextures,
    TemplateGeometryRingTextures,
}

#[derive(Debug, Clone)]
pub struct StreamingWriteOptions {
    pub encoding: PackageTableEncoding,
    pub target_rows_per_batch: usize,
}

impl Default for StreamingWriteOptions {
    fn default() -> Self {
        Self {
            encoding: PackageTableEncoding::Parquet,
            target_rows_per_batch: 16_384,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StreamingReadOptions {
    pub source_batch_size: usize,
}

impl Default for StreamingReadOptions {
    fn default() -> Self {
        Self {
            source_batch_size: 16_384,
        }
    }
}

pub trait TableBatchSink {
    fn write_required(&mut self, table: PackageTableKind, batch: RecordBatch) -> Result<()>;
    fn write_optional(&mut self, table: PackageTableKind, batch: RecordBatch) -> Result<()>;
    fn finish(self) -> Result<PackageManifest>;
}

pub trait TableBatchSource {
    fn header(&self) -> &CityArrowHeader;
    fn projection(&self) -> &ProjectionLayout;
    fn table_schema(&self, table: PackageTableKind) -> SchemaRef;
    fn next_batch(&mut self, table: PackageTableKind) -> Result<Option<RecordBatch>>;
}

pub fn write_package_streaming(
    dir: impl AsRef<Path>,
    header: &CityArrowHeader,
    projection: &ProjectionLayout,
    source: &mut impl TableBatchSource,
    options: &StreamingWriteOptions,
) -> Result<PackageManifest>;

pub fn read_package_streaming(
    dir: impl AsRef<Path>,
    options: &StreamingReadOptions,
) -> Result<PackageTableReader>;

pub struct PackageTableReader {
    pub header: CityArrowHeader,
    pub projection: ProjectionLayout,
    // per-table readers hidden behind concrete helper methods
}

impl PackageTableReader {
    pub fn schema(&self, table: PackageTableKind) -> SchemaRef;
    pub fn next_batch(&mut self, table: PackageTableKind) -> Result<Option<RecordBatch>>;
}

pub fn write_package_from_model_streaming(
    dir: impl AsRef<Path>,
    model: &cityjson::v2_0::OwnedCityModel,
    options: &StreamingWriteOptions,
) -> Result<PackageManifest>;
```

The public layering should be:

- `to_parts` remains the collector that produces full `CityModelArrowParts`
- `write_package_dir` and `write_package_ipc_dir` remain convenience wrappers
- new streaming package helpers become the preferred path for large models
- future table-wise consumer APIs may read directly from `PackageTableReader`
  without reconstructing `OwnedCityModel`

### Execution Rules

The streaming layer must obey these rules:

- canonical schemas remain identical to the current package contract
- ids and ordinals remain the reconstruction key; row order is not a semantic
  contract
- required one-row tables such as `metadata` must still be validated as such
- paired tables such as `geometries` and `geometry_boundaries` must maintain
  consistency across chunk boundaries
- package readers must validate per-batch schema and whole-table invariants
  without concatenating all source batches into one destination batch
- current in-memory APIs may be implemented as collectors over the streaming
  layer

### Implementation Phases

The intended order is:

1. add the streaming package read and write layer around existing canonical
   schemas
2. make current package helpers thin collectors over that layer
3. refactor conversion to emit canonical rows into chunked per-table builders
   instead of materializing full row vectors
4. add table-wise consumer APIs for large-model scans that do not require full
   `OwnedCityModel` reconstruction
5. later reduce reconstruction peak memory where possible, while documenting
   that a full `OwnedCityModel` remains model-sized in memory

## Consequences

Good:

- package I/O can become bounded by active batch size instead of whole-table
  size
- the current canonical schema and manifest contract stay intact
- existing in-memory APIs remain available for convenience and exact tests
- large-model consumers gain a path that does not require full canonical table
  concatenation
- the crate can improve scalability without inventing a second semantic model

Trade-offs:

- the crate will carry two operational layers: a convenience materialized API
  and a streaming API
- implementation complexity will increase around per-table flushing rules,
  validation, and cross-table synchronization
- not every workflow can be truly bounded-memory while `OwnedCityModel`
  remains the semantic unit
- tests will need to cover both semantic equivalence and chunk-boundary
  invariants
- some current helper functions that assume single-batch tables will need to be
  rewritten or narrowed to the materialized compatibility layer
