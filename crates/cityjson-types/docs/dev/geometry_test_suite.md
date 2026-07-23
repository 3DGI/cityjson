# Geometry Test Suite

This document describes the geometry test suite as it is implemented today in
`cityjson-types`. It replaces the older design, cleanup-plan, and error-case
notes with one current source of truth.

The suite covers the flat boundary model described in
`docs/dev/geometry_mappings.md`, the v2.0 authoring and storage APIs, and the
boundary WKB/EWKB serializers. JSON adapter behavior is intentionally outside
this document because it belongs to `cityjson-json`.

## Scope

Implemented coverage lives in four places:

- `tests/v2_0/geometry/`: canonical fixture acceptance, dense map checks,
  boundary round trips, template/instance behavior, and editing APIs.
- `src/backend/default/boundary/tests.rs`: flat boundary invariants, nested
  conversion behavior, index-width limits, and property tests.
- `src/backend/default/boundary/wkb.rs`: ISO WKB and PostGIS EWKB unit tests.
- `tools/gis-integration/tests/`: GEOS and PostGIS smoke tests for generated
  WKB/EWKB bytes.

The tests are written against the current model only: one flat boundary per
geometry, dense semantic/material assignment arrays, dense texture maps aligned
to boundary rings and vertex occurrences, and explicit separation between
regular geometry, template geometry, and `GeometryInstance`.

## Canonical Fixtures

The reusable fixtures are defined in `tests/v2_0/geometry/fixtures.rs`.

| Fixture | Geometry | Implemented content | Main coverage |
| --- | --- | --- | --- |
| `P1` | `MultiPoint` | 3 point occurrences, two semantic assignments and one `None` | point semantic bucket density and vertex insertion |
| `L1` | `MultiLineString` | 2 linestrings with different lengths, one semantic assignment and one `None` | linestring bucket density and ring offsets |
| `S1` | `MultiSurface` or `CompositeSurface` | 2 surfaces, 3 rings, one inner ring, surface semantics/materials, texture theme with one untextured ring | surface topology, dense surface maps, dense texture maps, occurrence-level UVs |
| `D1` | `Solid` | 1 solid, outer and inner shells, 4 surfaces, dense surface semantics | shell ordering and solid boundary shape |
| `MS1` | `MultiSolid` or `CompositeSolid` | 2 solids, one shell per solid, 4 surfaces, dense surface semantics | solid ordering and multi-solid boundary shape |
| `T1` | template `MultiSurface` | 2 template surfaces in the template vertex pool | template storage and validation |
| `I1` | `GeometryInstance` | valid template reference, regular reference point, identity transform | instance indirection and no local geometry payload |

`S1` deliberately reuses geometric coordinates across textured ring occurrences
while assigning different UV handles. This protects the rule that texture UVs
belong to boundary-vertex occurrences, not unique geometric vertices.

## Geometry Acceptance

`tests/v2_0/geometry/acceptance.rs` contains the high-level positive suite.

`canonical_fixture_acceptance` checks that every canonical fixture is accepted
with the expected `GeometryType`, boundary hierarchy, and primitive counts. It
also verifies that `CompositeSurface`/`MultiSurface` share the same boundary
shape, and `CompositeSolid`/`MultiSolid` share the same boundary shape.

`dense_semantic_and_material_maps_are_accepted` verifies that semantic and
material maps populate exactly the bucket that matches the primitive family:
points for `MultiPoint`, linestrings for `MultiLineString`, and surfaces for
surface- and solid-backed geometry. Dense `None` placeholders are preserved.

`resource_references_resolve_from_geometry_maps` verifies that non-null
semantic, material, texture, and UV handles stored in geometry maps resolve in
the model resource pools.

`dense_texture_maps_are_accepted` verifies that texture ring offsets match the
boundary ring offsets, ring texture assignments are dense, UV assignment length
matches boundary vertex occurrences, untextured rings carry `None` UVs, and
reused geometry vertices may have different UV handles in different ring
occurrences.

## Boundary Round Trips

`tests/v2_0/geometry/roundtrip.rs` covers round trips at the public nested/flat
boundary API level.

`flat_fixture_boundaries_roundtrip_by_type` converts each canonical fixture
boundary from flat storage to the appropriate nested shape and back, preserving
all populated flat arrays.

`nested_boundaries_roundtrip_and_preserve_grouping` starts from explicit nested
line, surface, solid, and multi-solid values and verifies that flattening and
re-expansion preserve line grouping, inner-ring attachment, shell contents, and
solid ordering.

## Draft Authoring

`src/v2_0/geometry_draft.rs` contains construction-level tests for the draft
API, which is the main authoring path used by the canonical fixtures.

Implemented checks include:

- empty required parts are rejected for points, lines, rings, surfaces, shells,
  solids, and multi-solid inputs;
- duplicate material themes on one surface and duplicate texture themes on one
  ring are rejected;
- missing semantic, material, texture, and UV handles are rejected;
- inserted coordinates and UV coordinates are deduplicated where the draft API
  promises deduplication;
- existing resource handles are accepted and preserved in emitted maps.

## Template, Instance, and Editing APIs

`tests/v2_0/geometry/instances.rs` covers positive template and instance
behavior. Template geometry is stored in the template pool, uses template
vertices, and validates with the same shape rules as regular geometry.
`GeometryInstance` stores no boundaries, semantics, materials, or textures; it
references an existing template and a regular-pool reference point, and resolves
through the model.

`tests/v2_0/geometry/editing.rs` covers the low-level editing escape hatches:
cloning stored parts, rebuilding a geometry from stored parts, replacing a
geometry while preserving handles visible through city objects, and rejecting
invalid replacement handles or replacement payloads. It also covers
semantic/material map builders for matching geometry families, mismatched
families, and missing resource handles.

## CityModel Geometry Validation

Additional geometry validation tests live in `src/v2_0/citymodel.rs` because
they exercise checked insertion and model-level reference resolution.

Implemented checks include:

- regular geometry boundary vertices must resolve in the regular vertex pool,
  even if matching template vertices exist;
- template geometry boundary vertices must resolve in the template vertex pool,
  even if matching regular vertices exist;
- `GeometryInstance` cannot be inserted into the template geometry pool;
- unchecked regular/template insertion accepts valid stored geometry while
  keeping the validation bypass explicit;
- geographical extent calculation resolves instance transforms and reports
  missing city object, geometry, vertex, and template references;
- default material and texture themes are stored independently and can be
  validated against themes that actually occur in geometry maps.

## Boundary Invariants

`src/backend/default/boundary/tests.rs` covers the flat boundary type directly.

Implemented checks include:

- `check_type()` reports the highest populated hierarchy level;
- `is_consistent()` accepts sorted offsets within child-array bounds and rejects
  out-of-bounds or decreasing offsets;
- singleton out-of-bounds offsets are rejected for rings, surfaces, shells, and
  solids;
- direct nested conversion works for `MultiPoint`, `MultiLineString`,
  `MultiSurface`/`CompositeSurface`, `Solid`, and
  `MultiSolid`/`CompositeSolid`;
- incompatible nested conversions fail instead of silently reinterpreting a
  boundary as another geometry family;
- coordinate iterators preserve boundary order, while unique-coordinate helpers
  deduplicate repeated vertex references;
- checked `Boundary::from_parts` rejects invalid offsets, while unsafe
  `from_parts_unchecked` preserves the caller-provided arrays;
- empty nested inputs collapse to `BoundaryType::None`;
- empty child segments in nested lines, surfaces, shells, and solids are
  represented consistently by offsets;
- index-width edge cases around `u16::MAX` either round trip or return index
  conversion/overflow errors without panics;
- property tests generate valid nested boundaries across index widths and
  malformed flattened boundaries for rejection.

The boundary tests intentionally distinguish boundary consistency from geometry
validity. A flat boundary can represent empty child segments consistently even
when a higher-level geometry authoring path would reject that shape.

## ISO WKB

`src/backend/default/boundary/wkb.rs` implements and tests little-endian ISO
SQL/MM WKB with XYZ coordinates.

Supported output and input types are:

- `MultiPointZ`
- `MultiLineStringZ`
- `MultiPolygonZ`

Surface-backed `CityJSON` boundaries, including `Solid`, `MultiSolid`, and
`CompositeSolid`, are flattened to `MultiPolygonZ` because ISO WKB has no solid
geometry type.

Implemented positive coverage includes:

- each supported boundary kind emits the expected top-level WKB type;
- `Boundary -> WKB -> Boundary -> WKB` is byte-stable for preservable shapes;
- supported WKB inputs parse into flat boundaries and vertex pools;
- repeated point references emit repeated point children;
- floating-point coordinate bits are preserved;
- open CityJSON polygon rings are closed on write by repeating the first
  coordinate;
- already closed legacy rings are not double-closed;
- holes stay attached to the same polygon;
- solid shell order and multi-solid solid/shell/surface order are preserved when
  flattened.

Implemented rejection coverage includes:

- empty boundaries;
- inconsistent offsets;
- missing vertex references;
- surface-backed boundaries with no reachable polygons;
- polygons with no rings;
- polygon rings with too few vertices;
- big-endian input;
- EWKB flags in ISO WKB input;
- unsupported or non-Z ISO type codes;
- top-level singular `PointZ`, `LineStringZ`, and `PolygonZ`;
- wrong child geometry types;
- empty top-level multi-geometries;
- zero-ring polygons;
- unclosed polygon rings;
- truncated payloads and trailing bytes.

## PostGIS EWKB

`src/backend/default/boundary/wkb.rs` also implements and tests little-endian
PostGIS EWKB with XYZ coordinates.

Supported top-level EWKB output/input types are represented by `EwkbType`:

- `MultiPoint`
- `MultiLineString`
- `MultiPolygon`
- `PolyhedralSurface`
- `Tin`

The writer takes an explicit `EwkbType` because the same surface-backed
boundary can be represented as `MultiPolygonZ`, `PolyhedralSurfaceZ`, or
`TINZ`. Optional SRID metadata is written only on the top-level geometry.

`TINZ` is supported as a collection of `TriangleZ` children. A top-level
`TriangleZ` is not supported because a single triangle is not a meaningful
CityJSON boundary target by itself.

Implemented positive coverage includes:

- writing and reading `MultiPointZ` EWKB with an optional SRID;
- writing and reading `PolyhedralSurfaceZ`;
- reading surface EWKB into a caller-selected target boundary shape, including
  wrapping as `Solid`;
- writing `TINZ` as `TriangleZ` children while preserving SRID metadata;
- preserving parsed EWKB type and optional SRID in `EwkbBoundary`.

Implemented rejection coverage includes:

- invalid TIN topology on write, including surfaces with holes or non-triangle
  rings;
- ISO WKB passed to `from_ewkb`, because EWKB input must carry the Z flag;
- EWKB with `M` coordinates;
- child geometries carrying SRID metadata;
- top-level `TriangleZ`;
- incompatible requested target boundary types.

The EWKB reader also rejects the same structural hazards as the WKB reader:
unsupported geometry types, wrong child types, empty child collections,
malformed rings, truncation, and trailing bytes.

## GIS Integration

`tools/gis-integration/tests` verifies generated bytes against external GIS
libraries in Docker.

Implemented checks include:

- GEOS loads the three ISO cases (`MultiPointZ`, `MultiLineStringZ`, and `MultiPolygonZ`) and round-trips them through its SRID-bearing EWKB writer;
- GEOS WKT tests stay ISO-only because GEOS does not preserve PostGIS extended surface dialects;
- PostGIS loads ISO WKB/WKT for those three cases and EWKB/EWKT with SRID 7415 for the ISO cases plus `PolyhedralSurfaceZ` and `TINZ`, reports the expected
  geometry type, dimensionality, and child count, and returns parseable WKB from
  `ST_AsBinary`;
- PostGIS loads generated EWKB through `ST_GeomFromEWKB`, preserves SRID 7415,
  reports the expected geometry metadata, and returns parseable little-endian
  EWKB from `ST_AsEWKB(..., 'NDR')`.

## Intentional Boundaries

The current suite does not treat these as `cityjson-types` geometry-suite
coverage:

- JSON parsing/serialization errors such as missing `type`, malformed boundary
  arrays, unsupported imported appearance for point/line geometry, or dense
  template index emission. Those belong in `cityjson-json`.
- Runtime rejection for states that the public draft API cannot construct. When
  the type system or builder API makes a state unrepresentable, coverage should
  stay at the construction/API boundary.
- A public EWKB `Triangle` type. Triangle children exist internally for `TINZ`,
  but top-level triangle EWKB is rejected.

## Running The Suite

Use the root `justfile` recipes:

- `just test -p cityjson-types` for the crate tests, including the geometry and
  WKB/EWKB unit tests.
- `just test-gis` for GEOS/PostGIS integration.
- `just ci` before claiming a PR is ready.
