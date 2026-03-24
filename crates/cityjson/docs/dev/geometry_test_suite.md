# Geometry Test Suite Design

This document defines the smallest test suite that still gives strong confidence
in the geometry model described in
`docs/dev/geometry_mappings.md`.

It is a test design document, not an implementation plan.

## Goal

The suite should be:

- complete enough to cover every correctness rule in the mapping note
- small enough to maintain without drift
- written against the current geometry model only

The geometry model assumed here is:

- one flat boundary model per geometry kind
- dense semantic and material assignment arrays
- dense, boundary-anchored texture maps
- no local shell or solid regrouping inside semantic, material, or texture maps
- explicit separation between regular geometry, template geometry, and `GeometryInstance`

## Test Design Rules

- Use a small set of reusable canonical fixtures.
- Cover each invariant at least once with either a positive case or a targeted negative mutation.
- Prefer mutation tests over many one-off fixtures.
- If a bad state becomes unrepresentable by types, drop the runtime rejection test and keep only the construction or API-level test that proves the state cannot be created.
- Do not keep compatibility tests for legacy sparse or regrouped internal forms.

## Fixture Construction API

The canonical fixtures should be built with the current `v2_0` authoring API:

- use direct validated insertion (`CityModel::add_geometry(...)` and
  `CityModel::add_geometry_template(...)`) when a test needs precise control
  over already-flattened stored geometry
- use the draft layer (`GeometryDraft`, `PointDraft`, `LineStringDraft`,
  `RingDraft`, `SurfaceDraft`, `ShellDraft`, `SolidDraft`) for authoring from
  raw coordinates or nested topology
- insert draft-authored regular geometry with `GeometryDraft::insert_into(...)`
- insert draft-authored template geometry with
  `GeometryDraft::insert_template_into(...)`
- create semantic, material, texture, and UV resources on `CityModel` first;
  drafts store handles only
- author `GeometryInstance` through `GeometryDraft::instance(...)`

Do not add tests for removed builder behavior such as implicit current-item
assignment, stateful surface/solid assembly, or draft-local resource
deduplication policy.

## Canonical Fixtures

These fixtures are enough to drive the full suite.

| ID | Declared kind(s) | Required content | Main coverage |
| --- | --- | --- | --- |
| `P1` | `MultiPoint` | 3 points; dense `points` semantic bucket; at least one `null` assignment | point-order assignment rules |
| `L1` | `MultiLineString` | 2 linestrings with different lengths; dense `linestrings` semantic bucket; at least one `null` assignment | linestring-order assignment rules |
| `S1` | `MultiSurface`, `CompositeSurface` | 2 surfaces; surface 0 has outer ring + inner ring; surface 1 has outer ring only; dense `surfaces` semantic and material buckets; 3 boundary rings total; dense texture map over all 3 rings | surface order, inner rings, dense textures, untextured ring placeholders |
| `D1` | `Solid` | 1 solid with outer shell + inner shell; enough surfaces to observe shell grouping; dense `surfaces` semantic and material buckets | shell rules, volumetric surface ordering |
| `MS1` | `MultiSolid`, `CompositeSolid` | 2 solids; at least one shell per solid; enough surfaces to observe solid-to-solid ordering; dense `surfaces` semantic and material buckets | solid rules, multi-solid ordering |
| `T1` | template geometry | template geometry using surface topology similar to `S1`; stored in template pools, not regular pools | template geometry validation |
| `I1` | `GeometryInstance` | valid template reference to `T1`; valid reference point in regular vertex pool; valid 4x4 affine transform | instance separation |

`S1` must also include this texture case:

- ring 0 textured
- ring 1 untextured
- ring 2 textured
- at least one geometric vertex reused in both textured rings
- the reused geometric vertex gets different UVs in the two ring occurrences

That one fixture covers the most failure-prone texture rule: UVs attach to
boundary-vertex occurrences, not to unique geometric vertices.

## Required Test Families

Creation-path tests:

- canonical fixture acceptance
- boundary round-trip
- dense semantic/material acceptance
- dense texture acceptance
- cross-layer ordering

Final-geometry validation tests:

- invalid boundary offsets
- invalid geometry-kind shape
- wrong or non-dense semantic/material maps
- invalid texture topology or UV payload
- resource reference validity
- template geometry and `GeometryInstance` separation

### 1. Accept Canonical Fixtures

Create one positive acceptance test family that runs all valid fixtures:

- `P1`
- `L1`
- `S1` as `MultiSurface`
- `S1` as `CompositeSurface`
- `D1`
- `MS1` as `MultiSolid`
- `MS1` as `CompositeSolid`
- `T1`
- `I1`

This test family proves the suite is testing realistic valid shapes, not only
rejection logic.

### 2. Reject Invalid Boundary Offsets

Start from `S1` or `D1` and mutate one offset array at a time:

- first offset is not `0`
- one offset decreases
- one offset exceeds child array length

Required assertion:

- validation fails before any deeper mapping checks depend on those offsets

One parameterized test family is enough.

### 3. Reject Invalid Geometry-Kind Shape

Use the smallest fixture that exposes each bad shape:

- `MultiSurface` with non-empty `shells`
- `MultiSurface` with non-empty `solids`
- `Solid` with non-empty `solids`
- `MultiSolid` missing any required populated level
- surface-based geometry with a surface that has no outer ring
- solid-based geometry with a solid that has no outer shell
- template geometry with the same invalid shape as its declared kind

Required assertion:

- validation fails because the populated hierarchy does not match the declared kind

One parameterized test family is enough.

### 4. Boundary Round-Trip

Run both directions:

- nested -> flat -> nested
- flat -> nested -> flat

Required fixtures:

- `P1`
- `L1`
- `S1`
- `D1`
- `MS1`
- `T1`

Required assertions:

- exact topology is preserved
- exact flat arrays are preserved
- inner rings remain attached to the correct surface
- inner shells remain attached to the correct solid
- multi-solid ordering is preserved

This should be two parameterized test families, one per direction.

### 5. Accept Dense Semantic and Material Maps

Use `P1`, `L1`, `S1`, `D1`, and `MS1`.

Required assertions:

- exactly one assignment bucket is populated
- the populated bucket matches the geometry kind
- unused buckets are empty
- populated bucket length equals primitive count
- `null` placeholders are preserved in primitive order
- no shell or solid regrouping is needed to reconstruct nested spec form
- point and linestring fixtures exercise semantic buckets only; material coverage
  starts at the surface level

One parameterized test family is enough.

### 6. Reject Wrong or Non-Dense Semantic and Material Maps

Start from `P1`, `L1`, `S1`, `D1`, or `MS1` and mutate one rule at a time:

- more than one assignment bucket is populated
- the wrong bucket is populated for the geometry kind
- populated bucket length does not equal primitive count
- a dense assignment array is shortened by dropping `null` placeholders

Required assertion:

- validation fails on map shape or count mismatch

One parameterized test family is enough.

### 7. Validate Resource References and Export Remapping

Use `S1` and `D1`.

Required positive assertions:

- every non-null semantic, material, texture, and UV reference resolves
- export remapping to dense output indices is stable
- the same logical input produces the same exported dense mapping

Required negative mutations:

- invalid semantic reference
- invalid material reference
- invalid texture reference
- invalid UV reference

If the crate exposes deduplication as part of the contract, add two more checks:

- equal logical resources reuse one pool entry when deduplication is expected
- distinct insertions remain distinct when deduplication is not expected

### 8. Accept Dense Texture Maps

Use `S1`.

Required assertions:

- `texture.rings.len == boundary.rings.len`
- `texture.ring_textures.len == boundary.rings.len`
- `texture.rings[i] == boundary.rings[i]` for every ring
- `texture.vertices.len == boundary.vertices.len`
- textured rings have one UV per boundary-vertex occurrence in the same order
- untextured rings have `null` texture entries and all-`null` UV slices
- reused geometric vertices can have different UVs in different ring occurrences

One positive test family is enough.

### 9. Reject Invalid Texture Topology or UV Payload

Start from `S1` and mutate one rule at a time:

- `rings.len` differs from `boundary.rings.len`
- `ring_textures.len` differs from `boundary.rings.len`
- `rings[i]` does not match `boundary.rings[i]`
- `vertices.len` differs from `boundary.vertices.len`
- a textured ring has the wrong UV count
- an untextured ring has a non-null UV entry
- a textured ring contains an invalid texture or UV reference

Required assertion:

- validation fails on the first violated dense texture invariant

One parameterized test family is enough.

### 10. Validate Cross-Layer Ordering

Use `S1` and `D1`.

Required assertions:

- reordering surfaces in the boundary traversal changes semantic and material
  surface order with the boundary
- reordering rings in the boundary traversal changes texture order with the
  boundary
- unique-vertex UV reconstruction is not available through the creation API;
  ring-local boundary-occurrence mapping is the only in-tree authoring path

This family lives next to the draft-to-stored creation implementation because
the invariant is about emitted storage order, not post-build rejection.

### 11. Validate Template Geometry and `GeometryInstance` Separation

Positive cases:

- `T1` passes the same boundary and mapping checks as regular geometry of the same kind
- `I1` stores no boundary or mapping payload of its own
- `I1` references an existing template
- `I1` references an existing regular vertex as its reference point
- `I1` has a valid 4x4 affine transform

Negative mutations:

- missing template reference
- missing reference point
- missing transform
- template geometry accidentally stored in regular geometry pools
- regular geometry accidentally stored in template pools
- `GeometryInstance` carries boundary or mapping payload directly

One positive and one parameterized negative test family are enough.

## Minimal Suite Summary

This is the smallest complete suite I would recommend:

- 7 canonical fixtures
- 13 test families total
- most families are parameterized mutations or parameterized fixture runs

In practice that means a small number of source fixtures and a moderate number
of assertions, while still covering:

- all geometry kinds
- all flat boundary layers
- dense semantic and material rules
- dense ring-anchored texture rules
- resource validity and export remapping
- template geometry rules
- `GeometryInstance` separation

## Non-Goals

This suite does not try to test:

- geometric validity in the computational-geometry sense, such as planarity, self-intersection, or manifoldness
- performance characteristics
- backward compatibility with legacy internal sparse or regrouped representations

Those should be separate test suites if needed.
