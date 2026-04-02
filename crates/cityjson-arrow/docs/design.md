# cityarrow Design

This document records the current execution model for `cityarrow` and
`cityparquet`.

## Core Boundary

- the semantic API is `cityjson::v2_0::OwnedCityModel`
- canonical Arrow tables are internal, not public API
- both live and persistent transport paths share the same canonical table
  schemas and projection layout

## Public Surfaces

- `cityarrow::ModelEncoder` and `cityarrow::ModelDecoder` own the live Arrow
  IPC stream boundary
- `cityparquet::PackageWriter` and `cityparquet::PackageReader` own the
  persistent single-file package boundary
- `cityarrow::internal` keeps doc-hidden conversion and transport helpers for
  sibling crates and benchmarks

## Canonical Table Order

The shared transport order is:

1. metadata
2. transform
3. extensions
4. vertices
5. template vertices
6. texture vertices
7. semantics
8. semantic children
9. materials
10. textures
11. template geometry boundaries
12. template geometry semantics
13. template geometry materials
14. template geometry ring textures
15. template geometries
16. geometry boundaries
17. geometry surface semantics
18. geometry point semantics
19. geometry linestring semantics
20. geometry surface materials
21. geometry ring textures
22. geometry instances
23. geometries
24. cityobjects
25. cityobject children

That order lets the incremental decoder reconstruct shared pools and geometry
dependencies before cityobject attachment.

## Live Stream

The live stream format is sequential and does not require pre-buffering every
serialized table payload.

- stream magic: `CITYARROW_STREAM_V3\0`
- prelude: JSON with `{ header, projection }`
- frame header: `table_tag: u8`, `rows: u64`
- frame payload: one Arrow IPC stream payload for that canonical table batch
- stream terminator: tag `255`

The reader validates schemas per table and feeds batches directly into the
incremental decoder. It no longer uses whole-stream `read_to_end`.

## Persistent Package

The persistent package is one seekable file.

- package magic: `CITYARROW_PKG_V3\0`
- table payloads are written directly to the file in canonical order
- manifest entries record table name, file offset, payload length, and row
  count
- the JSON manifest is appended near the end of the file
- the footer stores `manifest_offset`, `manifest_length`, and
  `CITYARROW_PKG_V3IDX\0`

Package reads are footer-first. The manifest is read without loading the whole
file, and the package reader then maps the file and decodes only the referenced
table payloads.

## Decoder Shape

The steady-state read path is incremental.

- metadata initializes the semantic model
- shared pools and sidecars are loaded table by table
- template geometries and geometries are reconstructed from ordered batches
- cityobjects are attached after geometry handles exist

The public read path no longer rebuilds a full canonical parts aggregate before
semantic import.

## Remaining Cost Centers

The current implementation still pays for:

- conversion of `OwnedCityModel` into canonical rows and record batches
- grouped sidecar staging keyed by canonical ids
- eager whole-table batch materialization inside each individual payload

Those are now isolated enough to benchmark separately.
