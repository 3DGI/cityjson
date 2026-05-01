# Geometry Boundary, Semantics, and Appearance Representation in CityJSON Specification and cityjson-rs

This note describes the crate's flat in-memory representation for:

- geometry boundaries
- semantic mappings
- material mappings
- texture mappings
- global semantic/material/texture/UV resource storage

It is written for code contributors. The focus is the physical data layout and
the accounting rules for offsets, pointers, and resource references.

It is also useful as a language-agnostic description of the flat representation.

## CityJSON Specifications: Geometry Boundary, Semantics, and Appearance Mapping

This note summarizes how CityJSON maps geometry structure, semantics, materials, and textures.

It is based on these specification pages:

- https://www.cityjson.org/dev/geom-arrays/
- https://www.cityjson.org/specs/2.0.1/#semantics-of-geometric-primitives
- https://www.cityjson.org/specs/2.0.1/#appearance-object

### Boundary Representation

CityJSON stores geometry topology in a nested `boundaries` array. The meaning comes from the array depth:

- `MultiPoint`: depth 1
- `MultiLineString`: depth 2
- `MultiSurface`: depth 3
- `CompositeSurface`: depth 3
- `Solid`: depth 4
- `MultiSolid`: depth 5
- `CompositeSolid`: depth 5

The nesting pattern is:

`point -> line -> surface -> shell -> solid`

Examples:

- `MultiSurface` is a list of surfaces, where each surface is a list of rings, and each ring is a list of vertex indices.
- `Solid` adds one more level for shells.
- `MultiSolid` and `CompositeSolid` add one more level for solids.

Interior boundaries are represented by extra rings within a surface, or extra shells within a solid, depending on the geometry type.

### Semantics Mapping

If a geometry has semantics, it has:

- `semantics.surfaces`: an array of semantic objects
- `semantics.values`: an index structure mapping geometry primitives to entries in `surfaces`

Rules:

- Indices in `semantics.values` are 0-based references into `semantics.surfaces`.
- `null` means that the corresponding primitive has no semantics.
- For `MultiPoint` and `MultiLineString`, `values` is a simple array.
- For every other geometry type, the depth of `semantics.values` is `boundaries` depth minus 2.

In practice this means semantics are assigned at the primitive level:

- `MultiPoint`: one semantic value per point
- `MultiLineString`: one semantic value per linestring
- `MultiSurface` / `CompositeSurface`: one semantic value per surface
- `Solid`: one semantic value per surface inside each shell
- `MultiSolid` / `CompositeSolid`: one semantic value per surface inside each shell inside each solid

Semantics do not go down to ring or vertex level. They stop at the surface level for areal and volumetric geometries.

### Appearance Mapping

Appearance data is defined at the CityJSON root in `appearance`, and geometry primitives refer to it by index.

#### Materials

A geometry may have:

- `material[theme].value`
- `material[theme].values`

Rules:

- `value` assigns one material to the whole geometry.
- `values` assigns materials per surface.
- Material indices are 0-based references into `appearance.materials`.
- `null` means that the corresponding surface has no material.
- The depth of `material[theme].values` is `boundaries` depth minus 2.

This makes material mapping structurally parallel to semantics mapping: materials are attached per surface.

#### Textures

A geometry may have:

- `texture[theme].values`

Rules:

- Texture mapping follows the boundary structure down to the ring level.
- The depth of `texture[theme].values` is equal to the depth of `boundaries`.
- For each ring, the first integer is a 0-based reference into `appearance.textures`, containing the texture definitions.
- The remaining integers are 0-based UV vertex references into `appearance.vertices-texture`, containing an array of the (u, v) coordinates of the vertices used for texturing the surfaces.
- The UV vertex references must follow the same order as the boundary ring vertices.
- Each texture ring array therefore has one more entry than the number of vertices in the corresponding boundary ring.
- `null` is used when a surface or ring has no texture.

So the distinction is:

- semantics: per primitive, usually per surface
- materials: per surface
- textures: per ring, with per-vertex UV references

### Mental Model

When traversing a geometry:

1. Read `boundaries` to understand the geometric nesting.
2. Read `semantics.values` with the same structure, reduced to the semantic primitive level.
3. Read `material[*].values` with the same structure, also reduced to the surface level.
4. Read `texture[*].values` with the full boundary depth, because textures are attached per ring and use UV vertex indices.

## cityjson-rs: Geometry and Resource Layout

### Design Summary

The crate keeps two representations in mind:

- spec-shaped nested arrays, which are natural for serialization
- flat columnar arrays, which are natural for storage and traversal

The flat form uses sibling arrays plus offset arrays.

At each hierarchy level:

- one array stores the child items
- one sibling array stores the start offset of each parent into that child array
- the end of parent `i` is:
  - the start of parent `i + 1`, or
  - the child array length for the last parent

Generic range rule:

```text
child_range(offsets, i, child_len):
    start = offsets[i]
    end = offsets[i + 1] if i + 1 < len(offsets) else child_len
    return [start, end)
```

### Draft Authoring API

The current geometry-construction API is split into:

- direct validated insertion for already-flattened stored geometry
- a thin `v2_0` draft layer for authoring from raw coordinates and nested
  topology

The draft layer mirrors geometry meaning rather than stored offsets:

- regular geometry is authored with `GeometryDraft::{multi_point,
  multi_line_string, multi_surface, composite_surface, solid, multi_solid,
  composite_solid}`
- template geometry uses the same draft values and inserts through
  `insert_template_into(...)`
- `GeometryInstance` uses `GeometryDraft::instance(...)`
- draft vertices and UVs can reference either existing pool indices or new raw
  coordinates
- surfaces, shells, and solids are nested directly as draft values
- semantic, material, and texture assignments use explicit handles created on
  `CityModel` before insertion

`insert_into(...)` / `insert_template_into(...)` then validate draft-local
invariants, preflight capacity, flatten directly into stored `Boundary` and
dense mapping arrays, and finally call the authoritative direct validated
insertion path.

### Global Resource Pools

Geometry-local maps do not store spec-local dense integer indices.

Instead, the model owns global resource pools for:

- semantics
- materials
- textures
- UV coordinates

Geometry-local mapping arrays store references into those pools.

Conceptually:

```text
model:
    semantic_pool = [Semantic0, Semantic1, ...]
    material_pool = [Material0, Material1, ...]
    texture_pool  = [Texture0, Texture1, ...]
    uv_pool       = [UV0, UV1, UV2, ...]

geometry:
    semantic_map entries -> semantic_pool
    material_map entries -> material_pool
    texture_map.ring_textures entries -> texture_pool
    texture_map.vertices entries -> uv_pool
```

This differs from CityJSON on the wire, where indices are dense array positions
inside the JSON document. Export to spec form therefore needs a remapping pass
from pool references to dense output indices.

The crate exposes [`DenseIndexRemap`](../../src/raw/views.rs) for that export
step when serializers need to translate sparse stored slot indices into dense
document-local indices.

### Boundary Layout

#### Physical Arrays

The boundary layout is:

```text
vertices : [vertex_ref, ...]
rings    : [vertex_start, ...]
surfaces : [ring_start, ...]
shells   : [surface_start, ...]
solids   : [shell_start, ...]
```

Meaning:

- `vertices` stores references into the model's global vertex pool
- `rings[i]` points to the first vertex of ring `i`
- `surfaces[i]` points to the first ring of surface `i`
- `shells[i]` points to the first surface of shell `i`
- `solids[i]` points to the first shell of solid `i`

Hierarchy:

```text
solid -> shell -> surface -> ring -> vertex
```

#### Example: MultiSurface

Spec-shaped boundary:

```text
[
    [[0, 1, 4]],
    [[1, 2, 5, 6], [0, 1, 2]],
    [[2, 3, 4, 8, 7]]
]
```

Flat form:

```text
vertices = [0, 1, 4,  1, 2, 5, 6,  0, 1, 2,  2, 3, 4, 8, 7]
rings    = [0, 3, 7, 10]
surfaces = [0, 1, 3]
shells   = []
solids   = []
```

Diagram:

```text
surfaces[0] = 0  -> rings[0..1] -> ring 0 -> vertices[0..3]
surfaces[1] = 1  -> rings[1..3] -> ring 1 -> vertices[3..7]
                                 -> ring 2 -> vertices[7..10]
surfaces[2] = 3  -> rings[3..4] -> ring 3 -> vertices[10..15]
```

#### Example: Solid

A solid adds one more offset layer:

```text
vertices = [...]
rings    = [0, 5, 10, 15, 20, 25]
surfaces = [0, 1, 2, 3, 4, 5]
shells   = [0]
solids   = []
```

Meaning:

- shell `0` starts at surface `0`
- because there is no next shell, shell `0` owns `surfaces[0..6]`
- each surface then owns a range in `rings`
- each ring owns a range in `vertices`

#### Flattening Rule

To flatten a nested structure, push the current child length before appending the
children.

Pseudocode:

```text
flatten_multi_surface(nested):
    boundary.vertices = []
    boundary.rings = []
    boundary.surfaces = []

    for surface in nested:
        boundary.surfaces.push(len(boundary.rings))
        for ring in surface:
            boundary.rings.push(len(boundary.vertices))
            for vertex_ref in ring:
                boundary.vertices.push(vertex_ref)
```

The same rule repeats upward:

- push `len(rings)` into `surfaces`
- push `len(surfaces)` into `shells`
- push `len(shells)` into `solids`

#### Expansion Rule

To recover a nested structure, slice from a start offset to the next sibling start
or the child length.

Pseudocode:

```text
expand_surface(boundary, surface_i):
    ring_start = boundary.surfaces[surface_i]
    ring_end = boundary.surfaces[surface_i + 1] or len(boundary.rings)

    rings = []
    for ring_i in range(ring_start, ring_end):
        vertex_start = boundary.rings[ring_i]
        vertex_end = boundary.rings[ring_i + 1] or len(boundary.vertices)
        rings.push(boundary.vertices[vertex_start:vertex_end])
    return rings
```

#### Boundary Invariants

Every non-empty offset array must:

- start at `0`
- be monotone non-decreasing
- contain only offsets within the child array length

Equal adjacent offsets are allowed if empty children are allowed by the caller,
although most real geometry should not contain empty rings, surfaces, or shells.

### Semantic and Material Layout

#### Physical Arrays

Semantics and per-surface materials use the same logical shape:

```text
points      : [resource_ref_or_null, ...]
linestrings : [resource_ref_or_null, ...]
surfaces    : [resource_ref_or_null, ...]
```

Interpretation:

- `points[i]` is the semantic or material of point `i`
- `linestrings[i]` is the semantic or material of linestring `i`
- `surfaces[i]` is the semantic or material of surface `i`

Boundary topology remains the source of truth. Robust validation should treat the
geometry boundary as authoritative and the semantic/material maps as assignment
arrays over that topology. Semantic and material maps do not restate shell or
solid grouping locally.

Semantic and material maps are dense over their primitive kind:

- one slot per primitive in traversal order
- `null` placeholders for unassigned primitives
- no sparse internal form

Only one of `points`, `linestrings`, or `surfaces` is required for a given
geometry kind:

- `MultiPoint` uses `points`
- `MultiLineString` uses `linestrings`
- `MultiSurface`, `CompositeSurface`, `Solid`, `MultiSolid`, and `CompositeSolid` use `surfaces`

Nested CityJSON semantic and material arrays are always reconstructed from the
boundary topology plus the primitive assignment bucket. No extra shell or solid
regrouping arrays are stored in the internal semantic/material maps.

#### Relation to the Spec

CityJSON stores:

- semantic objects in `semantics.surfaces`
- semantic assignments in nested `semantics.values`
- material objects in `appearance.materials`
- material assignments in `material[theme].values`

The crate instead stores:

- semantic objects in the global semantic pool
- material objects in the global material pool
- primitive assignments as flat arrays of pool references

So the crate flattens the nested assignment structure and lifts the payload
objects into global pools.

#### Example: Surface Semantics

Spec-shaped semantic assignment for a `MultiSurface`:

```text
values = [null, 4, null]
```

Flat form:

```text
surfaces = [null, SemRef(roof), null]
```

#### Example: Surface Materials by Theme

Per theme:

```text
theme "material-theme":
    surfaces = [null, null, MatRef(wall)]
```

Each theme owns its own flat map. A geometry therefore stores:

```text
materials = [
    ("theme-a", material_map_a),
    ("theme-b", material_map_b),
]
```

#### Primitive-Order Rule

The assignment arrays must use the same primitive order as the flattened
boundary traversal:

- `points` follow point order
- `linestrings` follow ring order for `MultiLineString`
- `surfaces` follow surface order in the flattened boundary

If that order changes, every semantic and material assignment changes with it.

### Texture Layout

#### Why Texture Layout Is Different

Textures are ring-local, not surface-local.

Each textured ring needs:

- one texture resource reference
- one UV coordinate reference per boundary vertex occurrence in that ring

The UV references must appear in the same order as the ring's boundary vertices.

#### Physical Arrays

The flat texture layout is:

```text
vertices      : [uv_ref_or_null, ...]
rings         : [vertex_start, ...]
ring_textures : [texture_ref_or_null, ...]
```

Meaning:

- `vertices` stores references into the global UV pool
- `rings[i]` points to the first UV entry for boundary ring `i`
- `ring_textures[i]` is the texture resource used by boundary ring `i`
- boundary topology is authoritative; texture maps store ring-local assignments only and do not restate surface, shell, or solid grouping locally

Texture maps use one internal form only: dense and boundary-anchored.

That means:

- there is one texture-map entry per boundary ring, in boundary ring order
- untextured rings carry `null` in `ring_textures`
- `vertices` has one slot per boundary-vertex occurrence, in boundary order
- untextured rings use `null` UV references in the matching vertex slice

The interpretation is:

```text
ring i:
    texture = ring_textures[i]
    uv_refs = vertices[rings[i] .. rings[i + 1] or len(vertices)]
```

#### Example: Two Textured Rings

```text
texture_pool = [Tex0, Tex1]
uv_pool      = [UV0, UV1, UV2, UV3, UV4, UV5, UV6]

vertices      = [0, 1, 2, 3,  4, 5, 6]
rings         = [0, 4]
ring_textures = [0, 1]
```

Diagram:

```text
ring 0:
    texture = Tex0
    uv_refs = vertices[0..4] = [UV0, UV1, UV2, UV3]

ring 1:
    texture = Tex1
    uv_refs = vertices[4..7] = [UV4, UV5, UV6]
```

If the corresponding boundary rings are:

```text
boundary ring 0 = [10, 11, 12, 13]
boundary ring 1 = [20, 21, 22]
```

then the texture application is:

```text
(10 -> UV0), (11 -> UV1), (12 -> UV2), (13 -> UV3)
(20 -> UV4), (21 -> UV5), (22 -> UV6)
```

#### Texture Storage Invariant

The key invariant is:

- every texture-map entry refers to exactly one boundary ring with the same index
- the UV slice length for that ring matches the boundary vertex count of that ring
- the UV order matches the boundary vertex order of that ring
- higher topology always comes from the boundary, not from texture-map-local regrouping arrays

#### Relation to the Spec

CityJSON texture arrays are nested to ring depth and encode:

```text
[texture_index, uv_index_0, uv_index_1, ...]
```

The crate instead separates that payload into sibling arrays:

- `ring_textures[i]` stores `texture_index`
- `vertices[rings[i]..rings[i+1]]` stores the UV indices

This is the same information, just stored column-wise instead of nested per
ring. Nested texture arrays are reconstructed from boundary topology plus these
per-ring assignments.

### Resource Mapping Rules

#### Import-Like Direction

When moving from spec form to the crate's internal form:

1. insert semantic objects into the global semantic pool
2. insert material objects into the global material pool
3. insert texture objects into the global texture pool
4. insert UV coordinates into the global UV pool
5. replace spec-local dense indices with pool references in the flat maps

#### Export-Like Direction

When moving from the crate's internal form to spec form:

1. collect the referenced pool objects for the geometry or model
2. assign dense output indices to them
3. rebuild nested `values` arrays using boundary topology
4. replace pool references with dense output indices

This remapping is required because global pool references are not the same thing as
spec-local dense array positions.

### Correctness Verification Guide

When changing any conversion or builder code, verify all of the following.

These checks apply to regular geometry and template geometry that carry explicit
boundaries. `GeometryInstance` is validated separately because it stores a
template handle, a reference point, and a transformation rather than explicit
boundary and mapping arrays.

#### 1. Boundary Offsets

Check for every offset array:

- first offset is `0` when the array is non-empty
- offsets are monotone
- no offset exceeds the child array length
- expanding and re-flattening produces the same arrays

Minimal pseudocode:

```text
check_offsets(offsets, child_len):
    if len(offsets) == 0:
        return true
    assert offsets[0] == 0
    for i in 0..len(offsets)-2:
        assert offsets[i] <= offsets[i + 1]
    assert offsets[-1] <= child_len
```

#### 2. Geometry-Kind Shape Invariants

Do not validate only offset monotonicity. Also validate that the populated
hierarchy matches the declared geometry kind.

Expected non-empty arrays:

- `MultiPoint`: `vertices`
- `MultiLineString`: `vertices`, `rings`
- `MultiSurface` / `CompositeSurface`: `vertices`, `rings`, `surfaces`
- `Solid`: `vertices`, `rings`, `surfaces`, `shells`
- `MultiSolid` / `CompositeSolid`: `vertices`, `rings`, `surfaces`, `shells`, `solids`

Expected empty arrays:

- `MultiPoint`: `rings`, `surfaces`, `shells`, `solids`
- `MultiLineString`: `surfaces`, `shells`, `solids`
- `MultiSurface` / `CompositeSurface`: `shells`, `solids`
- `Solid`: `solids`

Also check:

- every surface has an outer ring
- every solid has an outer shell
- template geometry follows the same shape rules as its declared geometry kind

Add explicit negative tests for:

- `MultiSurface` with non-empty `shells` or `solids`
- `Solid` with non-empty `solids`
- `MultiSolid` with missing `solids`, `shells`, `surfaces`, `rings`, or `vertices`
- a surface without an outer ring
- a solid without an outer shell

#### 3. Boundary Round-Trip

For each geometry kind:

- nested -> flat -> nested must preserve exact topology
- flat -> nested -> flat must preserve exact arrays

Pay special attention to:

- surfaces with inner rings
- solids with inner shells
- multi-solids with more than one solid

#### 4. Semantic and Material Map Shape

Validate map shape before validating counts.

For every semantic map or per-theme material map:

- exactly one of `points`, `linestrings`, or `surfaces` is populated
- unused buckets are empty
- `MultiPoint` uses `points`
- `MultiLineString` uses `linestrings`
- every surface-based geometry kind uses `surfaces`
- do not infer the full geometry kind from semantic/material map-local topology alone
- boundary topology is authoritative
- semantic/material maps store only primitive assignments, not shell or solid regrouping

This is important because a semantic/material map can be correct at the
primitive-assignment level even when it does not restate solid or multi-solid
grouping locally.

After shape checks, validate counts:

Check that mapping array lengths match primitive counts:

- semantic `points.len` equals point count
- semantic `linestrings.len` equals linestring count
- semantic `surfaces.len` equals surface count for every surface-based geometry kind
- per-theme material `surfaces.len` equals surface count for every surface-based geometry kind
- when a map exists, its populated bucket has one slot per primitive, including `null` placeholders

Semantic and material maps use one internal form only: dense primitive-order
arrays with `null` placeholders. Do not accept sparse assignment arrays in flat
form.

#### 5. Resource Reference Validity

For every non-null semantic, material, or texture reference:

- the referenced pool slot exists
- the reference is valid for the current slot generation
- export remapping produces stable dense indices

If resources are deduplicated, verify:

- equal logical resources reuse the same pool reference when expected
- non-deduplicated insertion creates distinct references when expected

#### 6. Texture Topology and UV Mapping Correctness

Texture maps use one storage mode only: dense and boundary-anchored.

Validate:

- `rings.len` equals `boundary.rings.len`
- `ring_textures.len` equals `boundary.rings.len`
- `rings[i]` matches `boundary.rings[i]` for every ring
- `vertices.len` equals `boundary.vertices.len`
- higher topology comes from the boundary only; the texture map stores no local surface, shell, or solid regrouping

Then check UV payload correctness.

For every textured ring:

- the referenced texture exists in the texture pool
- every UV reference exists in the UV pool
- the ring's UV count equals the boundary ring's vertex count
- UV order matches the boundary vertex order exactly
- repeated boundary-vertex occurrences are validated by occurrence, not by unique vertex id

For every untextured ring:

- `ring_textures[i]` is `null`
- every UV reference in that ring's slice is `null`

Add an explicit test where the same geometric vertex appears in multiple textured
rings or surfaces with different UVs. That case must succeed, because texture
coordinates attach to boundary-vertex occurrences, not to unique geometric
vertices.

Minimal pseudocode:

```text
check_textured_ring(boundary, texture_map, ring_i):
    b_start = boundary.rings[ring_i]
    b_end = boundary.rings[ring_i + 1] or len(boundary.vertices)
    t_start = texture_map.rings[ring_i]
    t_end = texture_map.rings[ring_i + 1] or len(texture_map.vertices)

    assert (b_end - b_start) == (t_end - t_start)
    if texture_map.ring_textures[ring_i] is null:
        for uv_ref in texture_map.vertices[t_start:t_end]:
            assert uv_ref is null
    else:
        assert texture_map.ring_textures[ring_i] is valid
        for uv_ref in texture_map.vertices[t_start:t_end]:
            assert uv_ref is valid
```

#### 7. Cross-Layer Alignment

Check that all layers agree on traversal order:

- boundary surface order matches semantic/material surface order
- boundary ring order matches texture ring order exactly
- builder-local indices are translated exactly once into flattened storage order

Typical failure modes:

- using unique vertex order where boundary-vertex occurrence order is required
- changing surface order without updating semantic/material arrays
- changing ring order without updating texture arrays
- forgetting that export needs pool-reference-to-dense-index remapping

#### 8. GeometryInstance and Template Separation

`GeometryInstance` is not validated like regular boundary-carrying geometry.
Validate it separately.

Check:

- `GeometryInstance` stores no boundary, semantic map, material map, or texture map payload
- the referenced template exists in the template-geometry pool
- the reference point exists in the regular vertex pool
- the transformation is present and has the expected 4x4 affine layout
- template geometry uses the template-vertex pool, not the regular root-vertex pool
- the referenced template geometry itself passes the same boundary and mapping checks as its declared geometry kind

Add explicit negative tests for:

- missing template reference
- missing reference point
- missing transformation
- accidentally mixing template geometry and regular geometry storage

### Contributor Checklist

Before merging changes to this area, verify:

- boundary offsets are valid
- geometry-kind shape invariants are enforced, including missing outer ring and missing outer shell cases
- nested and flat boundary forms round-trip
- semantic and material maps use the correct assignment bucket, have the correct primitive count, and do not restate shell/solid grouping
- all resource references resolve
- texture maps are dense in boundary ring order, store only ring-level assignments, and have UV references that match ring vertex counts and order
- repeated boundary-vertex occurrences are tested for independent UV assignment
- template geometry, regular geometry, and `GeometryInstance` stay separated correctly
- export/import code remaps resource references correctly

If a change alters traversal order or storage layout, update this note and add
tests that make the new accounting explicit.
