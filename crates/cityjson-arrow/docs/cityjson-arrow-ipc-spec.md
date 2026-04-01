# CityJSON Arrow IPC Package Layout Specification

## Status

Draft

## Version

`cityarrow.package.v1alpha1`

## Abstract

This specification defines the language-agnostic on-disk layout for CityJSON
packages encoded as Arrow IPC files.

The logical package schema is shared with the Parquet layout specification. The
physical encoding changes only the table file format and file extension.

## Scope

This specification covers:

- the package manifest
- the canonical table set
- table-level column layouts
- projection and ordering rules
- the invariants required for round-trip reconstruction

This specification does not define:

- the CityJSON semantic model itself
- a generic multi-model container format
- implementation-specific in-memory data structures

## Conformance

A conforming package MUST satisfy all requirements in this specification.

- A package MUST contain a `manifest.json` file.
- The manifest `package_schema` MUST be `cityarrow.package.v1alpha1`.
- The manifest `table_encoding` MUST be `arrow_ipc_file`.
- Every canonical table referenced by the manifest MUST be stored as an Arrow
  IPC file.
- Readers MUST reject packages whose manifest declares a different encoding.
- Writers MUST preserve the column order defined by the canonical schema.
- Consumers MUST use ids and ordinals for reconstruction, not physical row
  order.

## Package Layout

One package stores one logical CityJSON model.

```text
citymodel_package/
  manifest.json
  metadata.arrow
  transform.arrow
  extensions.arrow
  vertices.arrow
  cityobjects.arrow
  cityobject_children.arrow
  geometries.arrow
  geometry_boundaries.arrow
  geometry_instances.arrow
  template_vertices.arrow
  template_geometries.arrow
  template_geometry_boundaries.arrow
  semantics.arrow
  semantic_children.arrow
  geometry_surface_semantics.arrow
  geometry_point_semantics.arrow
  geometry_linestring_semantics.arrow
  template_geometry_semantics.arrow
  materials.arrow
  geometry_surface_materials.arrow
  template_geometry_materials.arrow
  textures.arrow
  texture_vertices.arrow
  geometry_ring_textures.arrow
  template_geometry_ring_textures.arrow
```

Files listed in the manifest are the canonical package surface. Additional files
MAY be present, but they are non-canonical.

## Manifest

`manifest.json` is the package entry point.

The manifest contains:

- `package_schema`: the package schema identifier
- `table_encoding`: the physical table encoding
- `cityjson_version`: the semantic CityJSON version
- `citymodel_id`: the logical model identifier
- `tables`: the canonical table file locations
- `views`: optional non-canonical derived artifacts

Paths in `tables` and `views` MAY be relative to the manifest directory or
absolute. Relative paths are resolved from the manifest location.

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

## Common Conventions

- Field names are case-sensitive and MUST match exactly.
- Every table includes `citymodel_id: LargeUtf8`.
- Semantic external ids remain strings.
- Dense transport ids use `UInt64`.
- Ordinals use `UInt32` and are zero-based.
- Fixed-width vectors use `FixedSizeList`.
- Boundary and index sequences use `List`.
- Column order is canonical columns first, then projection columns.
- Row order is not a semantic contract.

## Canonical Tables

### Model Header Tables

| Table | Required | Layout |
| --- | --- | --- |
| `metadata` | Yes | `citymodel_id`, `cityjson_version`, `citymodel_kind`, `identifier`, `title`, `reference_system`, `geographical_extent`, plus projected metadata columns |
| `transform` | No | `citymodel_id`, `scale`, `translate` |
| `extensions` | No | `citymodel_id`, `extension_name`, `uri`, `version` |

`metadata` MUST contain exactly one row.
`transform` MUST contain at most one row.
`scale` and `translate` are fixed-size lists of 3 `Float64` values.
`geographical_extent` is an optional fixed-size list of 6 `Float64` values.

### Geometry and Topology Tables

| Table | Required | Layout |
| --- | --- | --- |
| `vertices` | Yes | `citymodel_id`, `vertex_id`, `x`, `y`, `z` |
| `cityobjects` | Yes | `citymodel_id`, `cityobject_id`, `cityobject_ix`, `object_type`, `geographical_extent`, plus projected cityobject columns |
| `cityobject_children` | No | `citymodel_id`, `parent_cityobject_id`, `child_ordinal`, `child_cityobject_id` |
| `geometries` | Yes | `citymodel_id`, `geometry_id`, `cityobject_id`, `geometry_ordinal`, `geometry_type`, `lod`, plus projected geometry columns |
| `geometry_boundaries` | Yes | `citymodel_id`, `geometry_id`, `vertex_indices`, `line_lengths`, `ring_lengths`, `surface_lengths`, `shell_lengths`, `solid_lengths` |
| `geometry_instances` | No | `citymodel_id`, `geometry_id`, `cityobject_id`, `geometry_ordinal`, `lod`, `template_geometry_id`, `reference_point_vertex_id`, `transform_matrix`, plus projected geometry columns |
| `template_vertices` | No | `citymodel_id`, `template_vertex_id`, `x`, `y`, `z` |
| `template_geometries` | No | `citymodel_id`, `template_geometry_id`, `geometry_type`, `lod`, plus projected geometry columns |
| `template_geometry_boundaries` | No | `citymodel_id`, `template_geometry_id`, `vertex_indices`, `line_lengths`, `ring_lengths`, `surface_lengths`, `shell_lengths`, `solid_lengths` |

`geometry_boundaries` and `geometries` MUST have the same row count and MUST be
aligned by `geometry_id`.
If `template_geometries` is present, `template_geometry_boundaries` MUST also be
present, and the two tables MUST be aligned by `template_geometry_id`.

`vertex_indices` is a required list of `UInt64` values.
`line_lengths`, `ring_lengths`, `surface_lengths`, `shell_lengths`, and
`solid_lengths` are optional lists of `UInt32` values.
`transform_matrix`, when present, is a fixed-size list of 16 `Float64` values.

### Semantics Tables

| Table | Required | Layout |
| --- | --- | --- |
| `semantics` | No | `citymodel_id`, `semantic_id`, `semantic_type`, plus projected semantic columns |
| `semantic_children` | No | `citymodel_id`, `parent_semantic_id`, `child_ordinal`, `child_semantic_id` |
| `geometry_surface_semantics` | No | `citymodel_id`, `geometry_id`, `surface_ordinal`, `semantic_id` |
| `geometry_point_semantics` | No | `citymodel_id`, `geometry_id`, `point_ordinal`, `semantic_id` |
| `geometry_linestring_semantics` | No | `citymodel_id`, `geometry_id`, `linestring_ordinal`, `semantic_id` |
| `template_geometry_semantics` | No | `citymodel_id`, `template_geometry_id`, `primitive_type`, `primitive_ordinal`, `semantic_id` |

`semantic_id` MAY be null in the assignment tables to represent the absence of a
semantic assignment.
`primitive_type` values are `point`, `linestring`, or `surface`.

### Appearance Tables

| Table | Required | Layout |
| --- | --- | --- |
| `materials` | No | `citymodel_id`, `material_id`, plus projected material payload columns |
| `geometry_surface_materials` | No | `citymodel_id`, `geometry_id`, `surface_ordinal`, `theme`, `material_id` |
| `template_geometry_materials` | No | `citymodel_id`, `template_geometry_id`, `primitive_type`, `primitive_ordinal`, `theme`, `material_id` |
| `textures` | No | `citymodel_id`, `texture_id`, `image_uri`, plus projected texture payload columns |
| `texture_vertices` | No | `citymodel_id`, `uv_id`, `u`, `v` |
| `geometry_ring_textures` | No | `citymodel_id`, `geometry_id`, `surface_ordinal`, `ring_ordinal`, `theme`, `texture_id`, `uv_indices` |
| `template_geometry_ring_textures` | No | `citymodel_id`, `template_geometry_id`, `surface_ordinal`, `ring_ordinal`, `theme`, `texture_id`, `uv_indices` |

`uv_indices` is a required list of `UInt64` values.

## Projection Layout

The projection layout is not stored as a separate manifest field. It is inferred
from the physical schema of the tables.

The canonical projection families are:

- `metadata_extra`
- `cityobject_attributes`
- `cityobject_extra`
- `geometry_extra`
- `semantic_attributes`
- `material_payload`
- `texture_payload`

The projected columns MUST appear in the order defined by the package schema.
Tables that share a projection family MUST use the same projected column layout.

## Arrow IPC File Encoding

Each canonical table is stored as a standalone Arrow IPC file.

Implementations MAY write a table as one or more record batches within the file.
Readers MUST treat the file contents as a table with the schema recorded in the
IPC metadata.

The Arrow IPC encoding changes the file format only. It does not change the
logical schema, the manifest contract, or the reconstruction rules.

## Reconstruction Rules

Consumers MUST reconstruct models from ids and ordinals.

Ordinal columns are zero-based positions within the owning collection.

- `cityobject_children` uses `parent_cityobject_id` and `child_ordinal`.
- Geometry assignment tables use `geometry_id` plus the relevant ordinal
  columns.
- Template geometry assignment tables use `template_geometry_id`, `primitive_type`
  where applicable, and the relevant ordinal columns.
- Material and texture assignment tables use `theme` to separate parallel
  appearance maps.

The package format does not require physical row order to be semantically
meaningful.
