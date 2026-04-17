# cityjson-arrow Package Schema

This document summarizes the shared canonical transport contract used by the
live `cityjson-arrow` stream and the persistent `cityjson-parquet` package.

## Summary

- package schema id: `cityjson-arrow.package.v3alpha2`
- semantic boundary: `cityjson::v2_0::OwnedCityModel`
- public transport APIs: `write_stream` / `read_stream`,
  `export_reader` / `ModelBatchDecoder`, `PackageWriter` / `PackageReader`
- canonical tables: internal and doc-hidden
- reconstruction target: full-fidelity `OwnedCityModel`

## Canonical Tables

The canonical table set covers:

- metadata, transform, and extensions
- vertices, template vertices, and texture vertices
- semantics and semantic child relations
- materials and textures
- template geometry boundaries and template appearance sidecars
- geometry boundaries and geometry appearance sidecars
- geometry instances and boundary-backed geometries
- cityobjects and cityobject child relations

Each table is schema-locked in `src/schema.rs` and keyed with explicit ids and
ordinals rather than implicit row position.

## Required Tables

The transport contract always requires:

- `metadata`
- `vertices`
- `geometry_boundaries`
- `geometries`
- `cityobjects`

All other tables are optional and appear only when the model uses the
corresponding feature.

## Shared Header And Projection

Both live and persistent transport carry:

- `CityArrowHeader`
- `ProjectionLayout`

The header identifies the package version, `citymodel_id`, and CityJSON
version. The projection layout records typed recursive attribute layouts so the
decoder can validate and reconstruct nested dynamic attributes consistently.

## Persistent Manifest

`PackageManifest` records:

- `package_schema`
- `cityjson_version`
- `citymodel_id`
- `projection`
- ordered `tables`

Each table entry contains:

- canonical table name
- byte offset
- byte length
- row count

The manifest is authoritative for the persistent single-file package. The live
stream carries header and projection in its prelude instead of a persistent
manifest.
