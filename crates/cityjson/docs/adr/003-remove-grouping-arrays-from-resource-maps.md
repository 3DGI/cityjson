# Remove Grouping Arrays from Resource Maps

## Status

Accepted

## Context

Semantic, material, and texture maps used to carry some of the same grouping
information as the boundary:

- semantic/material maps carried `shells` and `solids`
- texture maps carried `surfaces`, `shells`, and `solids`

Those arrays did not add new meaning. They copied topology that was already
present in the boundary.

That made the model harder to reason about:

- the same topology could be stored in two places
- the two copies could disagree
- validation had to answer which one was the real source of truth
- code could start inferring geometry kind from resource maps instead of from
  the boundary

## Decision

Boundary topology is the only source of grouping.

Semantic and material maps store only dense primitive assignment arrays aligned
to primitive order:

- `points`
- `linestrings`
- `surfaces`

Texture maps store only dense, boundary-anchored ring-level assignment arrays:

- `vertices`
- `rings`
- `ring_textures`

These arrays are interpreted only relative to the matching boundary:

- semantic/material assignments follow primitive order
- texture `rings` match boundary ring order
- texture `vertices` follow boundary-vertex occurrence order within those rings

When serializing back to CityJSON, nested semantic, material, and texture
`values` arrays are rebuilt from the boundary plus these dense assignment
arrays.

## Consequences

Good:

- one clear source of truth for topology
- fewer invalid states
- simpler validation rules
- simpler import/export logic
- dense assignment arrays with explicit null placeholders are easier to validate

Trade-offs:

- serializers must rebuild nested grouping from the boundary instead of copying
  it from the maps
- resource maps cannot be interpreted correctly without the matching boundary
- callers must preserve boundary/map alignment when mutating stored geometry
