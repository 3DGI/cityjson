# Stream Write Serialization and Shared Write Context

## Status

Accepted

## Related Commits

- `e812be0` optimize write serialization path

## Context

The previous write path paid for two layers of work before emitting JSON:

- it converted the typed city model into an intermediate `serde_json::Value`
  tree
- it rebuilt lookup structures such as city object ids and dense material,
  texture, and template indices repeatedly during serialization

The benchmark split made that cost visible. Write benchmarks were several times
slower than the `serde_json::to_string` baseline, especially on large and
attribute-heavy datasets.

## Decision

The serializer now writes the `CityModel` directly through `serde::Serialize`
instead of first materializing a JSON DOM.

To support that, the write path introduces a shared `WriteContext` that is
constructed once per serialization and reused across nested serializers. The
context precomputes:

- city object handle to id mappings
- dense geometry template indices
- dense material indices
- dense texture indices

The public API keeps `to_string` and `to_string_validated`, and adds
`to_writer`, `to_writer_validated`, `to_vec`, and `to_vec_validated` so callers
can avoid unnecessary string allocation when they do not need it.

## Consequences

Good:

- removes the largest avoidable allocation layer from the write path
- makes serializer cost visible as direct structured emission instead of DOM
  construction plus encoding
- avoids recomputing global lookup maps across nested serialization steps
- enables writer-based serialization APIs

Trade-offs:

- serializer code is more explicit and less compact than routing everything
  through `serde_json::Value`
- more of the JSON layout is now maintained manually in `src/ser`

## Notes

This change improves the write baseline substantially, but it does not remove
all remaining hotspots. Cases dominated by boundary walking and some relation or
vertex-heavy workloads still trail `serde_json::to_string`, so further work is
likely to focus on geometry and attribute serialization details rather than on
top-level API shape.
