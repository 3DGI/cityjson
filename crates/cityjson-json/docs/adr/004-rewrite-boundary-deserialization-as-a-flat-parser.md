# Rewrite Boundary Deserialization as a Flat Parser

## Status

Accepted

## Related Commits

- `5943112` Rewrite streaming geometry boundary parsing

## Context

Once `CityObjects` were streamed directly, the dominant remaining geometry cost
was boundary parsing.

Profiling showed that the old boundary path still paid for:

- storing `boundaries` as `&RawValue`
- reparsing each boundary payload separately
- recursive `DeserializeSeed` and layered `Extend*` visitors
- a final copy from an intermediate builder into `Boundary<u32>`

On `30gz1_04.city.json`, the profiling probe attributed about `356 ms` to
`geometry.parse_boundary` alone.

## Decision

Boundary parsing was rewritten as one flat parser with one direct ownership
handoff into the backend boundary type.

The new geometry path has three parts:

1. `StreamingGeometryVisitor` parses geometry objects manually
2. `BoundaryParser` scans the raw boundary bytes directly into flat offset
   vectors
3. `Boundary::from_parts(...)` takes ownership of those vectors without a final
   builder copy

The core idea is:

```rust
match kind.boundary_kind() {
    Some(boundary_kind) => {
        let boundary = BoundaryParser::new(raw.get().as_bytes()).parse(boundary_kind)?;
        boundaries = Some(boundary);
    }
    None => {
        instance_boundaries = Some(serde_json::from_str(raw.get())?);
    }
}
```

The flat parser writes directly into the final offset vectors:

```rust
fn parse_surfaces_array(&mut self) -> Result<()> {
    self.parse_array(|this| {
        this.parts.surfaces.push(boundary_offset(this.parts.rings.len(), "surface")?);
        this.parse_rings_array()
    })
}

fn parse_rings_array(&mut self) -> Result<()> {
    self.parse_array(|this| {
        this.parts.rings.push(boundary_offset(this.parts.vertices.len(), "ring")?);
        this.parse_vertices_array()
    })
}
```

This keeps the deserializer aligned with the backend's flat boundary storage
instead of temporarily rebuilding nested JSON-shaped structures.

## Consequences

Good:

- boundary parsing matches the final model layout directly
- recursive visitor overhead is replaced by one specialized parser
- the final builder-to-boundary copy is removed

Trade-offs:

- the parser code is lower-level and must be tested carefully
- the fast path is specialized to CityJSON boundary structure rather than being
  a generic serde-based abstraction

Representative effect:

- `geometry.parse_boundary` dropped from about `356 ms` to about `214 ms` on
  `30gz1_04.city.json`
- the same hotspot dropped from about `5.79 ms` to about `3.55 ms` on
  `10-356-724.city.json`
- the large-file release probe moved from about `1.264 s` to about `1.150 s`
