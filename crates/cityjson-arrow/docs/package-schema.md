# cityarrow Package Schema

This document defines the current canonical schema for the `cityarrow` package
format.

The detailed encoding-specific specifications live in:

- [docs/cityjson-arrow-ipc-spec.md](cityjson-arrow-ipc-spec.md)
- [docs/cityjson-parquet-spec.md](cityjson-parquet-spec.md)

## Summary

- package schema id: `cityarrow.package.v1alpha1`
- semantic target: `cityjson::v2_0::OwnedCityModel`
- supported table encodings: Parquet and Arrow IPC file
- reconstruction target: full-fidelity `OwnedCityModel`
- canonical transport type: `CityModelArrowParts`

## Scope

This document defines:

- the package layout on disk
- the canonical tables and their storage encodings
- the transport structs exposed by `CityModelArrowParts`
- the reconstruction and projection rules

This document does not define:

- a generic multi-format registry
- a canonical WKB encoding for every CityJSON volumetric geometry
- a second semantic model beside `OwnedCityModel`

## Package Layout

One package stores one logical CityJSON model.

The manifest selects the table encoding. Parquet packages use `.parquet`
filenames. Arrow IPC packages use `.arrow` filenames.

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
  geometry_point_semantics.parquet
  geometry_linestring_semantics.parquet
  template_geometry_semantics.parquet
  materials.parquet
  geometry_surface_materials.parquet
  template_geometry_materials.parquet
  textures.parquet
  texture_vertices.parquet
  geometry_ring_textures.parquet
  template_geometry_ring_textures.parquet
```

All optional tables may be omitted when the corresponding component is absent.
Arrow IPC packages use the same layout with `.arrow` file extensions.

`PackageManifest` also has an optional `views` map for non-canonical derived
artifacts, but the current package readers and writers only require the
canonical tables listed above.

## Manifest

`manifest.json` is the package entry point. It identifies the package schema,
the selected table encoding, and the canonical tables present in the package.

Example:

```json
{
  "package_schema": "cityarrow.package.v1alpha1",
  "table_encoding": "arrow_ipc_file",
  "cityjson_version": "2.0",
  "citymodel_id": "rotterdam-sample",
  "tables": {
    "metadata": "metadata.arrow",
    "vertices": "vertices.arrow",
    "cityobjects": "cityobjects.arrow",
    "geometries": "geometries.arrow",
    "geometry_boundaries": "geometry_boundaries.arrow"
  }
}
```

`table_encoding` defaults to `parquet` when omitted so older Parquet manifests
remain readable.

## General Conventions

- one package contains one logical CityJSON model
- every table includes `citymodel_id: LargeUtf8`
- semantic external ids stay as strings
- dense transport ids use `UInt64`
- ordering is represented explicitly with `*_ordinal` columns
- canonical topology uses normalized sidecars instead of deeply nested boundary
  columns
- canonical schemas do not use Arrow `Union` or `Map`
- projected attributes live on the owning table as flat columns
- nested attribute values fall back to JSON text columns

## Canonical Id Space

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

`cityobject_id` is the semantic external identifier. The numeric ids are
transport identifiers used for joins, ordering, and compact storage.

## Transport Structs

The transport layer exposes an explicit package header and the canonical
component batches through `CityModelArrowParts`.

```rust
use arrow::record_batch::RecordBatch;

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
    pub geometry_point_semantics: Option<RecordBatch>,
    pub geometry_linestring_semantics: Option<RecordBatch>,
    pub template_geometry_semantics: Option<RecordBatch>,

    pub materials: Option<RecordBatch>,
    pub geometry_surface_materials: Option<RecordBatch>,
    pub template_geometry_materials: Option<RecordBatch>,

    pub textures: Option<RecordBatch>,
    pub texture_vertices: Option<RecordBatch>,
    pub geometry_ring_textures: Option<RecordBatch>,
    pub template_geometry_ring_textures: Option<RecordBatch>,
}
```

The manifest surface is:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifest {
    pub package_schema: CityArrowPackageVersion,
    pub table_encoding: PackageTableEncoding,
    pub cityjson_version: String,
    pub citymodel_id: String,
    pub tables: PackageTables,
    pub views: BTreeMap<String, PathBuf>,
}
```

`views` is optional metadata for non-canonical artifacts. The current package
helpers round-trip the canonical tables and do not depend on any particular
view names.

## Canonical Tables

The canonical package uses these tables:

- `metadata`
- `transform`
- `extensions`
- `vertices`
- `cityobjects`
- `cityobject_children`
- `geometries`
- `geometry_boundaries`
- `geometry_instances`
- `template_vertices`
- `template_geometries`
- `template_geometry_boundaries`
- `semantics`
- `semantic_children`
- `geometry_surface_semantics`
- `geometry_point_semantics`
- `geometry_linestring_semantics`
- `template_geometry_semantics`
- `materials`
- `geometry_surface_materials`
- `template_geometry_materials`
- `textures`
- `texture_vertices`
- `geometry_ring_textures`
- `template_geometry_ring_textures`

The exact Arrow field sets are schema-locked in the Rust test suite. The
canonical source of truth for those field definitions is `src/schema.rs`.

## Reconstruction Rules

- geometry topology is reconstructed from normalized boundary sidecars
- semantics, materials, and textures are reconstructed through explicit ids and
  ordinals
- parent/child relations are reconstructed from `cityobject_children`
- projected attributes are reconstructed from owning-table columns and field
  metadata
- missing optional tables mean the corresponding component is absent, not empty
  by convention

## Projection Rules

- primitive attribute values map to native scalar columns when projected
- nested arrays and maps fall back to JSON text columns
- mixed-type logical keys fall back to lossless text columns
- projection remains conservative by default so reconstruction stays lossless

## Derived Views

Non-canonical artifacts may be attached through manifest `views`, but they are
outside the canonical schema contract and may omit information that the
canonical package preserves.
