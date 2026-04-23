# Use Trusted Stored-Geometry Import Instead of `GeometryDraft`

## Status

Accepted

## Related Commits

- `086067a` Deserialize geometry through raw stored parts

## Context

The first large hotspot after `v0.5.0-refactor1` was geometry insertion.

The parser was already flattening most geometry information into a shape close
to the backend storage model, but it still fed every geometry through
`GeometryDraft::insert_into(...)`. That path was designed for checked authoring,
not trusted deserialization, so it repeated work the parser had effectively
already done:

- validate draft-local invariants
- analyze the draft
- preflight allocations
- rebuild stored geometry
- validate the stored geometry again before insertion

In practice, the deserializer was paying a full authoring pipeline even though
it already had flat, validated geometry parts.

## Decision

Deserialization now uses the backend's trusted stored-geometry insertion path.

The parser builds final stored geometry directly and inserts it through the raw
API exposed by `cityjson-rs`, instead of rebuilding it through `GeometryDraft`.

The change in strategy is:

```rust
// Before
let draft = GeometryDraft::new(type_geometry)
    .with_boundaries(boundaries)
    .with_semantics(semantics)
    .with_materials(materials)
    .with_textures(textures);

let handle = draft.insert_into(model)?;
```

```rust
// After
let parts = StoredGeometryParts {
    type_geometry,
    lod,
    boundaries: Some(boundary),
    semantics,
    materials,
    textures,
    instance,
};

let geometry = Geometry::from_stored_parts(parts);
let handle = unsafe { model.add_geometry_unchecked(geometry)? };
```

The trust boundary is explicit:

- the deserializer is responsible for constructing valid stored geometry
- the backend raw insertion API is responsible only for efficient insertion

## Consequences

Good:

- the deserializer no longer pays for redundant draft-to-stored rebuilds
- stored-geometry validation is not repeated on freshly built stored geometry
- the parser and backend now agree on one flat geometry representation

Trade-offs:

- correctness depends on the deserializer upholding stored-geometry invariants
- the raw insertion API is intentionally lower-level and less ergonomic than the
  checked draft API

Representative effect:

- the first round of release probes dropped into roughly `69.6 ms` for
  `10-356-724.city.json` and `2.72 s` for `30gz1_04.city.json`
- this confirmed that geometry insertion, not only parsing, was a major part of
  the regression
