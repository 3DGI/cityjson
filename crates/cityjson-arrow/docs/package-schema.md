# cityarrow Package Schema Draft

This document defines the first concrete schema draft for the canonical
`cityarrow` package format.

Status:

- package schema id: `cityarrow.package.v1alpha1`
- semantic target: `cityjson::v2_0::CityModel`
- storage target: Parquet tables with Arrow-compatible schemas
- interoperability target: derived GeoArrow and GeoParquet views

The package is designed for full-fidelity `CityModel` reconstruction first.
Generic GIS compatibility is provided through derived views, not by collapsing
the canonical model into a single simple-features table.

## Scope

This schema draft defines:

- the package layout on disk
- the canonical Parquet tables
- the Arrow field definitions for each table
- the draft Rust transport structs for `CityModelArrowParts`
- the reconstruction and projection rules

This schema draft does not define:

- a generic multi-format registry
- a canonical WKB encoding for CityJSON solids
- a second semantic model beside `CityModel`

## Package Layout

One package stores one `CityModel`.

```text
citymodel_package/
  manifest.json
  metadata.parquet
  transform.parquet
  extensions.parquet
  vertices.parquet
  cityobjects.parquet
  cityobject_children.parquet
  geometries.parquet
  geometry_boundaries.parquet
  geometry_instances.parquet
  template_vertices.parquet
  template_geometries.parquet
  template_geometry_boundaries.parquet
  semantics.parquet
  semantic_children.parquet
  geometry_surface_semantics.parquet
  materials.parquet
  geometry_surface_materials.parquet
  textures.parquet
  texture_vertices.parquet
  geometry_ring_textures.parquet
  views/
    surfaces.geoparquet
    footprints.geoparquet
    centroids.geoparquet
```

All optional tables may be omitted if the corresponding component is absent.

## Manifest

`manifest.json` is the package entry point. It identifies the schema version and
the table set present in the package.

Example:

```json
{
  "package_schema": "cityarrow.package.v1alpha1",
  "cityjson_version": "2.0",
  "citymodel_id": "rotterdam-sample",
  "tables": {
    "metadata": "metadata.parquet",
    "vertices": "vertices.parquet",
    "cityobjects": "cityobjects.parquet",
    "geometries": "geometries.parquet",
    "geometry_boundaries": "geometry_boundaries.parquet"
  },
  "views": {
    "surfaces": "views/surfaces.geoparquet"
  }
}
```

## General Conventions

- One package contains one logical `CityModel`.
- Every table includes `citymodel_id: LargeUtf8`, even though it is constant
  within the package.
- Canonical object ids stay as strings: `cityobject_id: LargeUtf8`.
- Dense transport ids use `UInt64`.
- Ordering is represented explicitly with `*_ordinal` columns.
- The canonical package uses normalized topology sidecars, not nested
  list-of-list-of-list boundary columns.
- The canonical package does not use Arrow `Union` or `Map` types.
- Projected attributes use flat columns on the owning table.
- Nested attribute values fall back to JSON text columns.

## Canonical Id Space

The package uses these ids:

- `citymodel_id: LargeUtf8`
- `cityobject_id: LargeUtf8`
- `cityobject_ix: UInt64`
- `vertex_id: UInt64`
- `geometry_id: UInt64`
- `template_geometry_id: UInt64`
- `template_vertex_id: UInt64`
- `semantic_id: UInt64`
- `material_id: UInt64`
- `texture_id: UInt64`
- `uv_id: UInt64`

`cityobject_id` is the semantic external identifier.

The numeric ids are transport identifiers used for joins, ordering, and compact
Parquet storage.

## Arrow Type Notation

This document uses these shorthand conventions:

- `Utf8` means Arrow `Utf8`
- `LargeUtf8` means Arrow `LargeUtf8`
- `Binary` means Arrow `Binary`
- `List<T!>` means Arrow `List<T>` with non-null child items
- `FixedSizeList<T!, N>` means Arrow `FixedSizeList<T, N>` with non-null child
  items

## Package Header Types

The transport layer should expose a small explicit header plus the full set of
component batches.

```rust
use arrow_array::RecordBatch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CityArrowPackageVersion {
    V1Alpha1,
}

#[derive(Debug, Clone)]
pub struct CityArrowHeader {
    pub package_version: CityArrowPackageVersion,
    pub citymodel_id: String,
    pub cityjson_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectedValueType {
    Boolean,
    UInt64,
    Int64,
    Float64,
    LargeUtf8,
    GeometryId,
    WkbBinary,
}

#[derive(Debug, Clone)]
pub struct ProjectedFieldSpec {
    pub name: String,
    pub data_type: ProjectedValueType,
    pub nullable: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectionLayout {
    pub metadata_extra: Vec<ProjectedFieldSpec>,
    pub cityobject_attributes: Vec<ProjectedFieldSpec>,
    pub cityobject_extra: Vec<ProjectedFieldSpec>,
    pub geometry_extra: Vec<ProjectedFieldSpec>,
    pub semantic_attributes: Vec<ProjectedFieldSpec>,
    pub material_payload: Vec<ProjectedFieldSpec>,
    pub texture_payload: Vec<ProjectedFieldSpec>,
}

#[derive(Debug, Clone)]
pub struct CityModelArrowParts {
    pub header: CityArrowHeader,
    pub projection: ProjectionLayout,

    pub metadata: RecordBatch,
    pub transform: Option<RecordBatch>,
    pub extensions: Option<RecordBatch>,

    pub vertices: RecordBatch,
    pub cityobjects: RecordBatch,
    pub cityobject_children: Option<RecordBatch>,

    pub geometries: RecordBatch,
    pub geometry_boundaries: RecordBatch,
    pub geometry_instances: Option<RecordBatch>,

    pub template_vertices: Option<RecordBatch>,
    pub template_geometries: Option<RecordBatch>,
    pub template_geometry_boundaries: Option<RecordBatch>,

    pub semantics: Option<RecordBatch>,
    pub semantic_children: Option<RecordBatch>,
    pub geometry_surface_semantics: Option<RecordBatch>,

    pub materials: Option<RecordBatch>,
    pub geometry_surface_materials: Option<RecordBatch>,

    pub textures: Option<RecordBatch>,
    pub texture_vertices: Option<RecordBatch>,
    pub geometry_ring_textures: Option<RecordBatch>,
}
```

## Canonical Table Schemas

### metadata

Rows: exactly 1

```text
citymodel_id: LargeUtf8!
cityjson_version: Utf8!
citymodel_kind: Utf8!
identifier: LargeUtf8?
title: LargeUtf8?
reference_system: LargeUtf8?
geographical_extent: FixedSizeList<Float64!, 6>?
extra.*: projected
```

### transform

Rows: 0 or 1

```text
citymodel_id: LargeUtf8!
scale: FixedSizeList<Float64!, 3>!
translate: FixedSizeList<Float64!, 3>!
```

### extensions

Rows: 0..n

```text
citymodel_id: LargeUtf8!
extension_name: Utf8!
uri: LargeUtf8!
version: Utf8?
```

### vertices

Rows: 0..n

```text
citymodel_id: LargeUtf8!
vertex_id: UInt64!
x: Float64!
y: Float64!
z: Float64!
```

### cityobjects

Rows: 0..n

```text
citymodel_id: LargeUtf8!
cityobject_id: LargeUtf8!
cityobject_ix: UInt64!
object_type: Utf8!
geographical_extent: FixedSizeList<Float64!, 6>?
attributes.*: projected
extra.*: projected
```

### cityobject_children

Rows: 0..n

```text
citymodel_id: LargeUtf8!
parent_cityobject_id: LargeUtf8!
child_ordinal: UInt32!
child_cityobject_id: LargeUtf8!
```

This table carries the canonical parent-to-children ordering. Reverse parent
lookups may be derived from this relation and do not need a second canonical
table.

### geometries

Rows: 0..n

This table stores boundary-carrying geometries such as `MultiSurface`, `Solid`,
and `MultiSolid`.

```text
citymodel_id: LargeUtf8!
geometry_id: UInt64!
cityobject_id: LargeUtf8!
geometry_ordinal: UInt32!
geometry_type: Utf8!
lod: Utf8?
extra.*: projected
```

### geometry_boundaries

Rows: 0..n

One row per row in `geometries`.

```text
citymodel_id: LargeUtf8!
geometry_id: UInt64!
vertex_indices: List<UInt64!>!
line_lengths: List<UInt32!>?
ring_lengths: List<UInt32!>?
surface_lengths: List<UInt32!>?
shell_lengths: List<UInt32!>?
solid_lengths: List<UInt32!>?
```

These arrays are the canonical normalized boundary payload.

They are derived from the offset-based `Boundary` layout in `cityjson-rs`, but
use count arrays because counts are easier to serialize cleanly in Arrow and
Parquet and easier to adapt into GeoArrow-compatible polygon layouts.

### geometry_instances

Rows: 0..n

This table stores `GeometryInstance` rows and is disjoint from
`geometry_boundaries`.

```text
citymodel_id: LargeUtf8!
geometry_id: UInt64!
cityobject_id: LargeUtf8!
geometry_ordinal: UInt32!
lod: Utf8?
template_geometry_id: UInt64!
reference_point_vertex_id: UInt64!
transform_matrix: FixedSizeList<Float64!, 16>?
extra.*: projected
```

### template_vertices

Rows: 0..n

```text
citymodel_id: LargeUtf8!
template_vertex_id: UInt64!
x: Float64!
y: Float64!
z: Float64!
```

### template_geometries

Rows: 0..n

```text
citymodel_id: LargeUtf8!
template_geometry_id: UInt64!
geometry_type: Utf8!
lod: Utf8?
extra.*: projected
```

### template_geometry_boundaries

Rows: 0..n

```text
citymodel_id: LargeUtf8!
template_geometry_id: UInt64!
vertex_indices: List<UInt64!>!
line_lengths: List<UInt32!>?
ring_lengths: List<UInt32!>?
surface_lengths: List<UInt32!>?
shell_lengths: List<UInt32!>?
solid_lengths: List<UInt32!>?
```

### semantics

Rows: 0..n

```text
citymodel_id: LargeUtf8!
semantic_id: UInt64!
semantic_type: Utf8!
attributes.*: projected
```

### semantic_children

Rows: 0..n

```text
citymodel_id: LargeUtf8!
parent_semantic_id: UInt64!
child_ordinal: UInt32!
child_semantic_id: UInt64!
```

### geometry_surface_semantics

Rows: 0..n

```text
citymodel_id: LargeUtf8!
geometry_id: UInt64!
surface_ordinal: UInt32!
semantic_id: UInt64?
```

This table aligns surface semantics by canonical exported surface order. Surface
meaning is therefore explicit relational data rather than implicit geometry
payload.

### materials

Rows: 0..n

```text
citymodel_id: LargeUtf8!
material_id: UInt64!
payload.*: projected
```

### geometry_surface_materials

Rows: 0..n

```text
citymodel_id: LargeUtf8!
geometry_id: UInt64!
surface_ordinal: UInt32!
theme: Utf8!
material_id: UInt64!
```

### textures

Rows: 0..n

```text
citymodel_id: LargeUtf8!
texture_id: UInt64!
image_uri: LargeUtf8!
payload.*: projected
```

### texture_vertices

Rows: 0..n

```text
citymodel_id: LargeUtf8!
uv_id: UInt64!
u: Float64!
v: Float64!
```

### geometry_ring_textures

Rows: 0..n

```text
citymodel_id: LargeUtf8!
geometry_id: UInt64!
surface_ordinal: UInt32!
ring_ordinal: UInt32!
theme: Utf8!
texture_id: UInt64!
uv_indices: List<UInt64!>!
```

This table stores ring-level texture assignment plus UV index mapping. Texture
references alone are not enough to reconstruct textured CityJSON geometries.

## Boundary Contract By Geometry Type

The canonical boundary payload depends on geometry type:

- `MultiPoint`: `vertex_indices`
- `MultiLineString`: `vertex_indices`, `line_lengths`
- `MultiSurface`: `vertex_indices`, `ring_lengths`, `surface_lengths`
- `CompositeSurface`: `vertex_indices`, `ring_lengths`, `surface_lengths`
- `Solid`: `vertex_indices`, `ring_lengths`, `surface_lengths`, `shell_lengths`
- `MultiSolid`: `vertex_indices`, `ring_lengths`, `surface_lengths`,
  `shell_lengths`, `solid_lengths`
- `CompositeSolid`: `vertex_indices`, `ring_lengths`, `surface_lengths`,
  `shell_lengths`, `solid_lengths`
- `GeometryInstance`: no `geometry_boundaries` row

## Projected Attribute Encoding

The canonical package must remain Parquet-safe.

Primitive attribute projection rules:

- `Null -> null`
- `Bool -> Boolean`
- `Unsigned -> UInt64`
- `Integer -> Int64`
- `Float -> Float64`
- `String -> LargeUtf8`

Nested attribute projection rules:

- `Vec -> LargeUtf8` containing JSON
- `Map -> LargeUtf8` containing JSON

Geometry-valued attribute projection rules:

- canonical field: `*_geometry_id: UInt64?`
- optional shadow field: `*_wkb: Binary?`

If one logical key appears with mixed primitive value types across rows, the
canonical projected column should be widened to `LargeUtf8` rather than trying
to preserve Arrow unions in Parquet.

Examples:

- `attributes.height: Float64?`
- `attributes.name: LargeUtf8?`
- `attributes.address_json: LargeUtf8?`
- `attributes.related_part_geometry_id: UInt64?`
- `attributes.related_part_wkb: Binary?`
- `extra.source: LargeUtf8?`

## Reconstruction Rules

Lossless reconstruction into `CityModel` is defined as follows:

- `metadata`, `transform`, and `extensions` rebuild model-level state
- `vertices` rebuild the shared vertex pool
- `cityobjects` and `cityobject_children` rebuild object identity and hierarchy
- `geometries` and `geometry_boundaries` rebuild boundary-carrying geometries
- `geometry_instances` rebuild `GeometryInstance` rows
- `semantics` and `geometry_surface_semantics` rebuild semantic surface
  assignment
- `materials`, `textures`, `texture_vertices`,
  `geometry_surface_materials`, and `geometry_ring_textures` rebuild appearance
  sidecars
- template tables rebuild template resources and template-based instances

The round-trip requirement is:

```text
CityModel -> CityModelArrowParts -> CityModel
```

for the supported component set, without requiring derived GIS views.

## Derived Interoperability Views

GeoArrow and GeoParquet outputs are derived products of the canonical package,
not the package itself.

Recommended derived views:

- `surfaces.geoparquet`
  - one row per exported surface polygon
  - includes `cityobject_id`, `geometry_id`, `surface_ordinal`,
    `semantic_type`
- `footprints.geoparquet`
  - one row per city object footprint
- `centroids.geoparquet`
  - one row per city object centroid
- optional `solids_wkb.parquet`
  - one row per volumetric geometry using an explicitly lossy or
    extension-defined binary geometry view

The derived views may be incomplete or lossy. The canonical package is the
source of truth.

## Implementation Notes For The Rewrite

The current `cityarrow` codebase should be rewritten to this target shape rather
than migrated incrementally from its older pool-based and union-oriented
assumptions.

The first implementation pass should:

- replace the current `CityModelArrowParts` with the batch set defined here
- split geometry metadata from boundary payload
- remove Arrow `Map` and `Union` assumptions from canonical Parquet schemas
- stop assuming quantized integer vertex storage
- implement projected attribute discovery before writing Parquet
- keep GeoArrow and GeoParquet generation as explicit secondary export paths

## Non-goals

This schema draft does not attempt to:

- make `cityjson-rs` adopt the package layout as its in-memory model
- encode CityJSON solids as if they were native simple-feature geometries
- use WKB as the canonical representation of CityJSON topology or semantics
- hide the difference between canonical package storage and derived GIS views
