# Remove the Old Geometry Builder and Use Draft-Based Creation

## Status

Accepted

## Context

The crate used to have a large geometry builder API.

That builder started as a convenience layer, but over time it became a second
geometry subsystem:

- it had its own state machine
- it had its own internal bookkeeping and remapping logic
- it mixed authoring convenience with resource policy
- it carried backend-generic plumbing that the public `v2_0` API did not need
- it made it harder to see what the final stored geometry would look like

This caused a few practical problems:

- the code was bigger than it needed to be
- geometry creation was harder to debug
- validation responsibility was split across too many places
- the model could be mutated while geometry was still being assembled
- the builder started to feel like the "real" write path even though stored
  geometry should be the source of truth

We wanted to keep geometry authoring convenient, but remove the extra
architecture.

## Decision

The old backend geometry builder was removed.

Geometry creation now has two layers:

1. direct validated insertion is the real write path
2. an optional `GeometryDraft` layer exists for convenience

The important rule is:

- final stored `Geometry` is authoritative
- draft geometry is only a temporary authoring format

## New Geometry Creation Process

There are now two ways to create geometry.

### 1. Direct insertion

Use direct insertion when the caller already knows the final stored geometry
layout.

In that flow:

- construct final `Geometry`
- call `add_geometry(...)` or `add_geometry_template(...)`
- the model validates the stored geometry before inserting it

This is the lowest-level and most explicit path.

### 2. Draft-based creation

Use `GeometryDraft` when authoring from raw coordinates or nested topology is
easier to read than hand-building flat boundary arrays.

In that flow:

- create any needed semantics, materials, textures, and UVs on `CityModel`
  first
- build a nested draft using `PointDraft`, `LineStringDraft`, `RingDraft`,
  `SurfaceDraft`, `ShellDraft`, `SolidDraft`, and `GeometryDraft`
- call `insert_into(...)` or `insert_template_into(...)`

The draft layer then does one straightforward conversion:

1. validate draft-local rules
2. count what needs to be added
3. reserve capacity before mutation
4. resolve existing and new vertices and UVs
5. flatten directly into stored boundary and dense mapping arrays
6. call the normal validated insertion path

`GeometryInstance` stays as a small special case. It is authored through
`GeometryDraft::instance(...)`.

## Why This Is Better

Good:

- one clear source of truth for geometry storage
- less code and less internal machinery
- no hidden incremental mutation during authoring
- no builder-owned deduplication policy
- easier to reason about what gets stored
- direct insertion and convenience authoring can evolve separately

Trade-offs:

- callers that want convenience must build draft values instead of using an
  imperative builder API
- callers must create or deduplicate semantic, material, and texture resources
  explicitly before draft insertion
- direct insertion is still the better choice when a caller needs exact control
  over the final flat representation

## Consequences

From now on:

- there is no backend geometry builder subsystem
- there is no `GeometryBuildContext` or `GeometryConstructor`
- direct validated insertion remains the core API
- `GeometryDraft` is the small convenience layer for authoring

This keeps geometry creation simple:

- draft values for writing geometry in a natural shape
- stored `Geometry` for the real model representation
